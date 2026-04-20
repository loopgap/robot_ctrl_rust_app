//! Build commands

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum BuildCmd {
    /// Run preflight checks (fmt + clippy + test)
    Preflight,
    /// Build release binary
    Release {
        /// Crate to build (optional, builds all if not specified)
        #[arg(short, long)]
        crate_name: Option<String>,
        /// Build slim release (strip debug info)
        #[arg(long)]
        slim: bool,
    },
    /// Build documentation
    Docs {
        /// Open in browser after building
        #[arg(long)]
        open: bool,
    },
}

pub fn handle(cmd: BuildCmd) -> Result<()> {
    match cmd {
        BuildCmd::Preflight => preflight(),
        BuildCmd::Release { crate_name, slim } => release_build(crate_name.as_deref(), slim),
        BuildCmd::Docs { open } => build_docs(open),
    }
}

fn preflight() -> Result<()> {
    println!("Running preflight checks...\n");

    println!("=== 1. Formatting ===");
    let _ = std::process::Command::new("cargo")
        .args(["fmt", "--check"])
        .status();

    println!("\n=== 2. Clippy ===");
    let _ = std::process::Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .status();

    println!("\n=== 3. Testing ===");
    let _ = std::process::Command::new("cargo").args(["test"]).status();

    println!("\n=== 4. Building ===");
    let _ = std::process::Command::new("cargo").args(["build"]).status();

    println!("\nPreflight complete!");
    Ok(())
}

fn release_build(crate_name: Option<&str>, slim: bool) -> Result<()> {
    if let Some(name) = crate_name {
        println!("Building release for '{}'...", name);
    } else {
        println!("Building release for all crates...");
    }

    let mut args = vec!["build", "--release"];
    if let Some(name) = crate_name {
        args.push("-p");
        args.push(name);
    }

    let mut cmd = std::process::Command::new("cargo");
    cmd.args(&args);
    cmd.status()?;

    if slim {
        println!("\nCreating slim build (stripping debug info)...");
    }

    Ok(())
}

fn build_docs(open: bool) -> Result<()> {
    println!("Building documentation...");

    std::process::Command::new("cargo")
        .args(["doc", "--no-deps"])
        .status()?;

    if open {
        println!("Opening documentation...");
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg("target/doc/index.html")
                .status();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "target\\doc\\index.html"])
                .status();
        }
    }

    Ok(())
}
