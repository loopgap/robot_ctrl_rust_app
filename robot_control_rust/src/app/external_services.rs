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
}
