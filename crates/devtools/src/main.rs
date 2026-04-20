//! Developer Tools - Build, Release and CI/CD Utilities
//!
//! A unified Rust CLI for managing the robot_ctrl workspace.

mod audit;
mod build;
mod clean;
mod pr;
mod release;
mod validate;
mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "devtools")]
#[command(about = "Developer Tools for robot_ctrl workspace", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// GitHub PR management
    Pr {
        #[command(subcommand)]
        cmd: pr::PrCmd,
    },
    /// Version management
    Version {
        #[command(subcommand)]
        cmd: version::VersionCmd,
    },
    /// Code audit (formatting, clippy, security)
    Audit {
        #[arg(long)]
        full: bool,
    },
    /// Build operations
    Build {
        #[command(subcommand)]
        cmd: build::BuildCmd,
    },
    /// Release operations
    Release {
        #[command(subcommand)]
        cmd: release::ReleaseCmd,
    },
    /// Clean build artifacts
    Clean {
        #[arg(long)]
        all: bool,
    },
    /// Validate workspace structure
    Validate {
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Pr { cmd }) => pr::handle(cmd),
        Some(Commands::Version { cmd }) => version::handle(cmd),
        Some(Commands::Audit { full }) => audit::handle(full),
        Some(Commands::Build { cmd }) => build::handle(cmd),
        Some(Commands::Release { cmd }) => release::handle(cmd),
        Some(Commands::Clean { all }) => clean::handle(all),
        Some(Commands::Validate { strict }) => validate::handle(strict),
        None => {
            println!("No command specified. Use --help for usage information.");
            Ok(())
        }
    }
}
