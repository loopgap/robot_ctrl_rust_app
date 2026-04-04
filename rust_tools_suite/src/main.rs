#![windows_subsystem = "windows"]

use clap::Parser;

mod app;
mod cli;
mod file_ops;
mod guide;
mod i18n;
mod settings;
mod theme;
mod tools;
mod workflow;

use app::ToolSuiteApp;
use eframe::egui;
use theme::{apply_theme, install_font_fallback};

#[derive(Parser, Debug)]
#[command(name = "rust_tools_suite")]
#[command(about = "Rust Tools Suite", long_about = None)]
struct Args {
    #[arg(long, short = 'g')]
    gui: bool,
    #[arg(short, long)]
    port: Option<String>,
    #[arg(short, long)]
    baud: Option<u32>,
    #[arg(long)]
    doctor: bool,
}

fn main() -> eframe::Result<()> {
    let args = Args::parse();

    if args.gui || args.port.is_none() {
        run_gui()
    } else {
        let cli = cli::Cli {
            command: if args.doctor {
                Some(cli::Commands::Doctor)
            } else if args.port.is_some() {
                Some(cli::Commands::Connect {
                    port: args.port,
                    baud: args.baud,
                })
            } else {
                None
            },
        };
        cli::run_cli(cli);
        Ok(())
    }
}

fn run_gui() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 860.0])
            .with_min_inner_size([980.0, 640.0])
            .with_title("Rust Tools Suite"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Tools Suite",
        options,
        Box::new(|cc| {
            install_font_fallback(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx, true);
            Ok(Box::new(ToolSuiteApp::default()))
        }),
    )
}
