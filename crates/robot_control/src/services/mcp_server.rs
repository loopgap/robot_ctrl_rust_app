// ═══════════════════════════════════════════════════════════════
// MCP 服务器实现 - JSON-RPC 2.0 协议
// ═══════════════════════════════════════════════════════════════

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::{ParsedPacket, RobotState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
    #[serde(default)]
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSharedState {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub current_state: RobotState,
    pub state_history: Vec<RobotState>,
    pub parsed_packets: Vec<ParsedPacket>,
    pub suggested_kp: f64,
    pub suggested_ki: f64,
    pub suggested_kd: f64,
    pub status: String,
    pub request_count: u64,
    pub unauthorized_count: u64,
}

impl Default for McpSharedState {
    fn default() -> Self {
        Self {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 0.0,
            current_state: RobotState::default(),
            state_history: Vec::new(),
            parsed_packets: Vec::new(),
            suggested_kp: 1.0,
            suggested_ki: 0.1,
            suggested_kd: 0.01,
            status: "Ready".into(),
            request_count: 0,
            unauthorized_count: 0,
        }
    }
}

pub struct McpServer {
    pub port: u16,
    pub running: Arc<AtomicBool>,
}

impl McpServer {
    pub fn start(
        shared: Arc<Mutex<McpSharedState>>,
        port: u16,
        auth_token: Option<String>,
        running: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|e| format!("Failed to bind MCP port {}: {}", port, e))?;
        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            for stream in listener.incoming() {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
                if let Ok(stream) = stream {
                    let shared = Arc::clone(&shared);
                    let auth_token = auth_token.clone();
                    thread::spawn(move || {
                        handle_client(stream, shared, auth_token);
                    });
                }
            }
        });

        Ok(())
    }

    pub fn stop(running: Arc<AtomicBool>) {
        running.store(false, Ordering::SeqCst);
    }
}

fn handle_client(
    mut stream: TcpStream,
    shared: Arc<Mutex<McpSharedState>>,
    auth_token: Option<String>,
) {
    let mut buf = [0u8; 4096];
    if let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            return;
        }
        let req_str = String::from_utf8_lossy(&buf[..n]);
        if let Ok(req) = serde_json::from_str::<McpRequest>(&req_str) {
            if let Some(expected) = &auth_token {
                if req.auth_token.as_deref() != Some(expected.as_str()) {
                    if let Ok(mut s) = shared.lock() {
                        s.unauthorized_count = s.unauthorized_count.saturating_add(1);
                    }
                    let resp = McpResponse {
                        jsonrpc: "2.0".into(),
                        result: None,
                        error: Some(json!({"code": -32001, "message": "Unauthorized"})),
                        id: req.id.clone(),
                    };
                    let resp_str = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into());
                    let _ = stream.write_all(resp_str.as_bytes());
                    return;
                }
            }
            if let Ok(mut s) = shared.lock() {
                s.request_count = s.request_count.saturating_add(1);
            }
            let resp = handle_request(req, &shared);
            let resp_str = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".into());
            let _ = stream.write_all(resp_str.as_bytes());
        }
    }
}

fn handle_request(req: McpRequest, shared: &Arc<Mutex<McpSharedState>>) -> McpResponse {
    let mut result = None;
    let mut error = None;
    let jsonrpc = "2.0".to_string();
    let id = req.id.clone();
    let method = req.method.as_str();
    let params = req.params.clone();

    match method {
        "get_pid_params" => {
            let s = shared.lock().unwrap();
            result = Some(json!({
                "kp": s.kp,
                "ki": s.ki,
                "kd": s.kd,
                "setpoint": s.setpoint,
            }));
        }
        "set_pid_params" => {
            if let Some(p) = params {
                if let (Some(kp), Some(ki), Some(kd), Some(sp)) = (
                    p.get("kp").and_then(|v| v.as_f64()),
                    p.get("ki").and_then(|v| v.as_f64()),
                    p.get("kd").and_then(|v| v.as_f64()),
                    p.get("setpoint").and_then(|v| v.as_f64()),
                ) {
                    let mut s = shared.lock().unwrap();
                    s.kp = kp;
                    s.ki = ki;
                    s.kd = kd;
                    s.setpoint = sp;
                    s.status = format!("MCP set params kp={:.4} ki={:.4} kd={:.4}", kp, ki, kd);
                    result = Some(json!({"ok": true}));
                } else {
                    error = Some(json!({"code": -32602, "message": "Invalid params"}));
                }
            } else {
                error = Some(json!({"code": -32602, "message": "Missing params"}));
            }
        }
        "get_robot_state" => {
            let s = shared.lock().unwrap();
            result = Some(json!(&s.current_state));
        }
        "get_state_history" => {
            let s = shared.lock().unwrap();
            result = Some(json!(&s.state_history));
        }
        "get_parsed_packets" => {
            let s = shared.lock().unwrap();
            result = Some(json!(&s.parsed_packets));
        }
        "suggest_params" => {
            let s = shared.lock().unwrap();
            result = Some(json!({
                "kp": s.suggested_kp,
                "ki": s.suggested_ki,
                "kd": s.suggested_kd,
                "status": s.status,
            }));
        }
        "tools" | "tools/list" => {
            result = Some(json!([
                "get_pid_params",
                "set_pid_params",
                "get_robot_state",
                "get_state_history",
                "get_parsed_packets",
                "suggest_params"
            ]));
        }
        "initialize" => {
            result = Some(json!({
                "protocolVersion": "2025-11-05",
                "serverInfo": {"name": "robot-control-mcp", "version": "0.1.1"}
            }));
        }
        _ => {
            error = Some(json!({"code": -32601, "message": "Method not found"}));
        }
    }
    McpResponse {
        jsonrpc,
        result,
        error,
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_get_pid_params() {
        let shared = Arc::new(Mutex::new(McpSharedState {
            kp: 2.0,
            ki: 0.2,
            kd: 0.05,
            ..Default::default()
        }));

        let req = McpRequest {
            jsonrpc: "2.0".into(),
            method: "get_pid_params".into(),
            params: None,
            id: Some(json!(1)),
            auth_token: None,
        };
        let resp = handle_request(req, &shared);
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["kp"].as_f64().unwrap(), 2.0);
    }

    #[test]
    fn test_handle_set_pid_params() {
        let shared = Arc::new(Mutex::new(McpSharedState::default()));
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            method: "set_pid_params".into(),
            params: Some(json!({"kp": 3.0, "ki": 0.3, "kd": 0.07, "setpoint": 100.0})),
            id: Some(json!(2)),
            auth_token: None,
        };
        let resp = handle_request(req, &shared);
        assert!(resp.error.is_none());

        let s = shared.lock().unwrap();
        assert_eq!(s.kp, 3.0);
        assert_eq!(s.setpoint, 100.0);
    }

    #[test]
    fn test_unknown_method() {
        let shared = Arc::new(Mutex::new(McpSharedState::default()));
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            method: "unknown".into(),
            params: None,
            id: Some(json!(3)),
            auth_token: None,
        };
        let resp = handle_request(req, &shared);
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap()["code"].as_i64().unwrap(), -32601);
    }

    #[test]
    fn test_auth_token_field_deserialize() {
        let raw = r#"{"jsonrpc":"2.0","method":"get_pid_params","id":1,"auth_token":"secret"}"#;
        let req: McpRequest = serde_json::from_str(raw).unwrap();
        assert_eq!(req.auth_token.as_deref(), Some("secret"));
    }
}
