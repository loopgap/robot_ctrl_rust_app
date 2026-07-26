// Keep a console when CLI support is compiled, otherwise Windows hides the
// status/error text emitted by `enter-bootloader` and the other commands.
#![cfg_attr(
    all(target_os = "windows", not(feature = "cli")),
    windows_subsystem = "windows"
)]

#[cfg(not(any(feature = "gui", feature = "cli")))]
compile_error!(
    "rust_tools_suite requires at least one interface feature: \
     enable `gui` and/or `cli` (e.g. --features gui)."
);

#[cfg(feature = "gui")]
mod app;
#[cfg(any(feature = "gui", feature = "cli"))]
mod at32_boot_entry;
#[cfg(feature = "cli")]
mod cli;
mod error;
#[cfg(feature = "gui")]
mod file_ops;
#[cfg(feature = "gui")]
mod guide;
#[cfg(feature = "gui")]
mod i18n;
#[cfg(feature = "gui")]
mod settings;
#[cfg(feature = "gui")]
mod theme;
#[cfg(feature = "gui")]
mod tools;
#[cfg(feature = "gui")]
mod workflow;

#[cfg(feature = "gui")]
use app::ToolSuiteApp;
#[cfg(feature = "gui")]
use eframe::egui;
#[cfg(feature = "gui")]
use theme::{apply_theme, install_font_fallback};

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
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
    #[command(subcommand)]
    command: Option<cli::Commands>,
}

#[cfg(target_os = "linux")]
fn check_linux_env() {
    if std::env::var("WINIT_UNIX_BACKEND").is_err() {
        std::env::set_var("WINIT_UNIX_BACKEND", "wayland,x11");
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    check_linux_env();

    #[cfg(feature = "cli")]
    {
        let args = Args::parse();
        if !args.gui && (args.port.is_some() || args.command.is_some()) {
            let cli = cli::Cli {
                command: if args.command.is_some() {
                    args.command
                } else if args.doctor {
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
            return;
        }
    }

    #[cfg(feature = "gui")]
    {
        let _ = run_gui();
    }

    #[cfg(not(feature = "gui"))]
    {
        eprintln!("GUI feature not enabled. Use --gui with the 'gui' feature, or enable it in Cargo.toml.");
        std::process::exit(1);
    }
}

#[cfg(feature = "gui")]
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
