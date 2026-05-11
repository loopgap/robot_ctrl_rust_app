// ═══════════════════════════════════════════════════════════════
// MCP 服务器实现 - 使用官方 rmcp SDK
// ═══════════════════════════════════════════════════════════════

use rmcp::{
    handler::server::wrapper::Parameters, model::*, tool, tool_handler, tool_router,
    transport::stdio, ErrorData as McpError, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::models::{ParsedPacket, RobotState};

// ========== 共享状态（保持与 AppState 的兼容性）==========

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
    pub runtime_mode: String,
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
            runtime_mode: "embedded".into(),
            request_count: 0,
            unauthorized_count: 0,
        }
    }
}

// ========== 参数定义 ==========

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PidParams {
    #[schemars(description = "比例系数")]
    pub kp: f64,
    #[schemars(description = "积分系数")]
    pub ki: f64,
    #[schemars(description = "微分系数")]
    pub kd: f64,
    #[schemars(description = "目标设定值")]
    pub setpoint: f64,
}

// ========== MCP 服务器结构体 ==========

#[derive(Clone)]
/// MCP (Model Context Protocol) 服务器，使用官方 rmcp SDK 实现
///
/// 提供低风险 MCP 工具方法：
/// - get_pid_params: 获取 PID 参数
/// - set_pid_params: 设置 MCP 内存态 PID 参数
/// - get_robot_state: 获取机器人状态
/// - get_state_history: 获取状态历史
/// - get_parsed_packets: 获取解析的数据包
/// - suggest_params: 获取 AI 建议的参数
/// - get_server_status: 获取服务器状态与安全边界
pub struct RobotMcpServer {
    state: Arc<Mutex<McpSharedState>>,
}

// ========== 工具实现 ==========

#[tool_router]
impl RobotMcpServer {
    pub fn new(state: Arc<Mutex<McpSharedState>>) -> Self {
        Self { state }
    }

    #[tool(description = "获取当前 PID 控制器参数")]
    async fn get_pid_params(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "kp": s.kp,
                "ki": s.ki,
                "kd": s.kd,
                "setpoint": s.setpoint
            })
            .to_string(),
        )]))
    }

    #[tool(
        description = "Set PID controller parameters in the in-memory MCP state only; this does not write to serial, CAN, Modbus, USB, or hardware outputs."
    )]
    async fn set_pid_params(
        &self,
        Parameters(PidParams {
            kp,
            ki,
            kd,
            setpoint,
        }): Parameters<PidParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut s = self.state.lock().await;
        s.kp = kp;
        s.ki = ki;
        s.kd = kd;
        s.setpoint = setpoint;
        s.status = format!("MCP set params kp={:.4} ki={:.4} kd={:.4}", kp, ki, kd);
        s.request_count = s.request_count.saturating_add(1);
        Ok(CallToolResult::success(vec![Content::text("ok")]))
    }

    #[tool(description = "Get MCP server version, mode, counters, and safety status.")]
    async fn get_server_status(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "name": "robot-control-mcp",
                "version": env!("CARGO_PKG_VERSION"),
                "runtime_mode": s.runtime_mode,
                "status": s.status,
                "request_count": s.request_count,
                "unauthorized_count": s.unauthorized_count,
                "state_history_len": s.state_history.len(),
                "parsed_packets_len": s.parsed_packets.len(),
                "hardware_write_enabled": false
            })
            .to_string(),
        )]))
    }

    #[tool(description = "获取机器人当前状态")]
    async fn get_robot_state(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&s.current_state).unwrap_or_default(),
        )]))
    }

    #[tool(description = "获取历史状态记录（最近 500 条）")]
    async fn get_state_history(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&s.state_history).unwrap_or_default(),
        )]))
    }

    #[tool(description = "获取已解析的数据包（最近 200 条）")]
    async fn get_parsed_packets(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string(&s.parsed_packets).unwrap_or_default(),
        )]))
    }

    #[tool(description = "获取 AI 建议的 PID 参数")]
    async fn suggest_params(&self) -> Result<CallToolResult, McpError> {
        let s = self.state.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::json!({
                "kp": s.suggested_kp,
                "ki": s.suggested_ki,
                "kd": s.suggested_kd,
                "status": s.status
            })
            .to_string(),
        )]))
    }
}

// ========== 服务器处理器 ==========

#[tool_handler]
impl ServerHandler for RobotMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "robot-control-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions("机器人控制 MCP 服务器，提供 PID 参数读写和状态查询".to_string())
    }
}

// ========== 启动函数 ==========

pub async fn start_mcp_server(state: Arc<Mutex<McpSharedState>>) -> Result<(), anyhow::Error> {
    info!("Starting MCP server...");
    {
        let mut s = state.lock().await;
        s.runtime_mode = "stdio".into();
        s.status = "MCP stdio server ready".into();
    }
    let server = RobotMcpServer::new(state);
    let service = server.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP server error: {:?}", e);
    })?;
    info!("MCP server started successfully");
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_shared_state_default() {
        let state = McpSharedState::default();
        assert_eq!(state.kp, 1.0);
        assert_eq!(state.ki, 0.1);
        assert_eq!(state.kd, 0.01);
        assert_eq!(state.setpoint, 0.0);
        assert_eq!(state.status, "Ready");
        assert_eq!(state.runtime_mode, "embedded");
        assert_eq!(state.request_count, 0);
    }

    #[tokio::test]
    async fn test_pid_params_deserialize() {
        let json = r#"{"kp":2.5,"ki":0.3,"kd":0.05,"setpoint":100.0}"#;
        let params: PidParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.kp, 2.5);
        assert_eq!(params.ki, 0.3);
        assert_eq!(params.kd, 0.05);
        assert_eq!(params.setpoint, 100.0);
    }

    #[tokio::test]
    async fn test_robot_mcp_server_new() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);
        let info = server.get_info();
        assert_eq!(info.server_info.name, "robot-control-mcp");
    }

    #[tokio::test]
    async fn test_get_pid_params() {
        let state = Arc::new(Mutex::new(McpSharedState {
            kp: 2.0,
            ki: 0.2,
            kd: 0.05,
            setpoint: 100.0,
            ..Default::default()
        }));

        let server = RobotMcpServer::new(state);
        let result = server.get_pid_params().await.unwrap();

        // 验证结果
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_set_pid_params() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state.clone());

        let params = PidParams {
            kp: 3.0,
            ki: 0.3,
            kd: 0.07,
            setpoint: 100.0,
        };
        let result = server.set_pid_params(Parameters(params)).await.unwrap();

        // 验证结果
        assert!(!result.is_error.unwrap_or(true));

        // 验证状态已更新
        let s = state.lock().await;
        assert_eq!(s.kp, 3.0);
        assert_eq!(s.setpoint, 100.0);
    }

    #[tokio::test]
    async fn test_get_robot_state() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);

        let result = server.get_robot_state().await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_suggest_params() {
        let state = Arc::new(Mutex::new(McpSharedState {
            suggested_kp: 1.5,
            suggested_ki: 0.15,
            suggested_kd: 0.015,
            status: "AI suggested".into(),
            ..Default::default()
        }));

        let server = RobotMcpServer::new(state);
        let result = server.suggest_params().await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_get_server_status_reports_safety_boundary() {
        let state = Arc::new(Mutex::new(McpSharedState {
            runtime_mode: "stdio".into(),
            ..Default::default()
        }));
        let server = RobotMcpServer::new(state);
        let result = server.get_server_status().await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
        let text = result.content[0].as_text().unwrap().text.as_str();
        let status: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(status["version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
        assert_eq!(status["runtime_mode"].as_str(), Some("stdio"));
        assert_eq!(status["hardware_write_enabled"].as_bool(), Some(false));
    }

    #[tokio::test]
    async fn test_get_state_history_empty() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);
        let result = server.get_state_history().await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_get_parsed_packets_empty() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);
        let result = server.get_parsed_packets().await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_set_pid_params_updates_status() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state.clone());
        let params = PidParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            setpoint: 50.0,
        };
        server.set_pid_params(Parameters(params)).await.unwrap();
        let s = state.lock().await;
        assert!(s.status.contains("MCP set params"));
    }

    #[tokio::test]
    async fn test_server_info_version() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);
        let info = server.get_info();
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_server_capabilities() {
        let state = Arc::new(Mutex::new(McpSharedState::default()));
        let server = RobotMcpServer::new(state);
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
    }
}
