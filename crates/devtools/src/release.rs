//! Release commands

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ReleaseCmd {
    /// Create a new release
    Create {
        /// Version to release (e.g., 0.1.0)
        version: String,
    },
    /// Rollback a release
    Rollback {
        /// Version to rollback
        version: String,
    },
    /// Publish to crates.io
    Publish {
        /// Crate to publish (optional, publishes all if not specified)
        #[arg(short, long)]
        crate_name: Option<String>,
    },
}

pub fn handle(cmd: ReleaseCmd) -> Result<()> {
    match cmd {
        ReleaseCmd::Create { version } => create_release(&version),
        ReleaseCmd::Rollback { version } => rollback_release(&version),
        ReleaseCmd::Publish { crate_name } => publish(crate_name.as_deref()),
    }
}

fn create_release(version: &str) -> Result<()> {
    println!("Creating release v{}...", version);
    println!("\nThis will:");
    println!("  1. Update version in Cargo.toml files");
    println!("  2. Create git tag v{}", version);
    println!("  3. Build release artifacts");
    println!("  4. Create GitHub release");
    println!("\nNote: Manual execution required for safety.");
    Ok(())
}

fn rollback_release(version: &str) -> Result<()> {
    println!("Rolling back release v{}...", version);
    println!("\nThis will:");
    println!("  1. Delete GitHub release");
    println!("  2. Delete git tag v{}", version);
    println!("  3. Revert version changes");
    println!("\nWarning: This is destructive!");
    Ok(())
}

fn publish(crate_name: Option<&str>) -> Result<()> {
    if let Some(name) = crate_name {
        println!("Publishing '{}' to crates.io...", name);
    } else {
        println!("Publishing all crates to crates.io...");
    }
    println!("\nNote: Ensure you have publish access and are logged in with 'cargo login'");
    Ok(())
}
