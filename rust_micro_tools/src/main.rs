#![windows_subsystem = "windows"]

use clap::Parser;

mod app;
mod cli;
mod guide;
mod i18n;
mod settings;
mod theme;
mod tools;
mod workflow;

use app::ToolSuiteApp;
use eframe::egui;
use theme::apply_theme;

#[derive(Parser, Debug)]
#[command(name = "rust_micro_tools")]
#[command(about = "Rust Micro Tools Suite", long_about = None)]
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
            Ok(Box::new(ToolSuiteApp::default()))
        }),
    )
}
