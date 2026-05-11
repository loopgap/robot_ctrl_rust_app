use zed_extension_api as zed;

struct RobotControlMcpExtension;

impl zed::Extension for RobotControlMcpExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &zed::ContextServerId,
        _project: &zed::Project,
    ) -> zed::Result<zed::Command> {
        let command = std::env::var("ROBOT_CONTROL_MCP_PATH")
            .unwrap_or_else(|_| "robot_control_mcp".to_string());

        Ok(zed::Command {
            command,
            args: vec!["--stdio".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(RobotControlMcpExtension);
