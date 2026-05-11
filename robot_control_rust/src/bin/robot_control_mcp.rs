use std::sync::Arc;

use robot_control_core::services::mcp_server::{start_mcp_server, McpSharedState};
use tokio::sync::Mutex;

fn print_usage() {
    eprintln!("Usage: robot_control_mcp --stdio");
    eprintln!("       robot_control_mcp --version");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if args.is_empty() || args.iter().any(|arg| arg == "--stdio") {
        let shared = Arc::new(Mutex::new(McpSharedState::default()));
        if let Err(err) = start_mcp_server(shared).await {
            eprintln!("robot_control_mcp failed: {err}");
            std::process::exit(1);
        }
        return;
    }

    print_usage();
    std::process::exit(2);
}
