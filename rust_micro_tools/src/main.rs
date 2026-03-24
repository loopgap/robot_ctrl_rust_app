#![windows_subsystem = "windows"]

mod app;
mod guide;
mod i18n;
mod settings;
mod theme;
mod tools;
mod workflow;
mod cli; // 添加新智能互动 CLI 模块

use app::ToolSuiteApp;
use eframe::egui;
use theme::apply_theme;
use clap::Parser;

fn main() -> eframe::Result<()> {
    // 检查是否带有控制台启动参数
    // 如果带参数，或者通过专门控制台环境启动，可拦截执行
    let args = std::env::args().collect::<Vec<_>>();
    
    // 如果存在超过1个参数，我们判断为意图使用 CLI/TUI 特性
    // 对于 --help 等标准解析，如果用 GUI 需要静默，这里做了简化：一旦有参就交管给 CLI
    if args.len() > 1 {
        let cli_app = cli::Cli::parse();
        cli::run_cli(cli_app);
        
        // 按照参数决定是否继续执行 GUI
        // 这里默认提供 CLI 功能之后就退出，除非我们特别拦截
        return Ok(());
    }

    // fallback 到高性能原生 GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([980.0, 640.0])
            .with_title("Rust Micro Tools Suite (Smart Acceleration)"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Micro Tools Suite",
        options,
        Box::new(|cc| {
            apply_theme(&cc.egui_ctx);
            // 这里可以进行高频零拷贝解析器或 FFI 管道初始化...
            Ok(Box::new(ToolSuiteApp::default()))
        }),
    )
}
