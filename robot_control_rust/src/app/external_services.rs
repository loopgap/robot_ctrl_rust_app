use crate::services::llm_service::SuggestedParams;
use crate::services::mcp_server::McpSharedState;
use std::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ExternalServices {
    pub mcp_server_handle: Option<tokio::task::JoinHandle<()>>,
    pub mcp_shared_state: Arc<Mutex<McpSharedState>>,
    pub llm_result_rx: Option<mpsc::Receiver<Result<SuggestedParams, String>>>,
}

impl ExternalServices {
    pub fn new() -> Self {
        Self {
            mcp_server_handle: None,
            mcp_shared_state: Arc::new(Mutex::new(McpSharedState::default())),
            llm_result_rx: None,
        }
    }

    pub fn is_mcp_running(&self) -> bool {
        self.mcp_server_handle
            .as_ref()
            .is_some_and(|h| !h.is_finished())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_not_running_by_default() {
        let es = ExternalServices::new();
        assert!(!es.is_mcp_running());
    }

    #[test]
    fn test_llm_rx_none_by_default() {
        let es = ExternalServices::new();
        assert!(es.llm_result_rx.is_none());
    }

    #[test]
    fn test_mcp_shared_state_default() {
        let es = ExternalServices::new();
        // Verify the shared state is accessible and in default state
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let state = es.mcp_shared_state.lock().await;
            // Default state should be initialized without panic
            drop(state);
        });
    }

    #[test]
    fn test_mcp_server_handle_none_by_default() {
        let es = ExternalServices::new();
        assert!(es.mcp_server_handle.is_none());
    }

    #[test]
    fn test_llm_result_rx_can_be_set() {
        let mut es = ExternalServices::new();
        let (tx, rx) = std::sync::mpsc::channel();
        es.llm_result_rx = Some(rx);
        assert!(es.llm_result_rx.is_some());

        // Send a result through the channel
        tx.send(Ok(crate::services::llm_service::SuggestedParams {
            kp: 1.0,
            ki: 0.1,
            kd: 0.01,
            reasoning: "test".into(),
        }))
        .unwrap();

        let received = es.llm_result_rx.as_ref().unwrap().try_recv();
        assert!(received.is_ok());
        let result = received.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kp, 1.0);
    }

    #[test]
    fn test_llm_result_rx_error_path() {
        let mut es = ExternalServices::new();
        let (tx, rx) = std::sync::mpsc::channel();
        es.llm_result_rx = Some(rx);

        tx.send(Err("LLM timeout".into())).unwrap();

        let received = es.llm_result_rx.as_ref().unwrap().try_recv();
        assert!(received.is_ok());
        let result = received.unwrap();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "LLM timeout");
    }
}
