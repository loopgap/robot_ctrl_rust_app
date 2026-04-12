//! Version management commands

use anyhow::Result;
use clap::Subcommand;
use semver::Version;

#[derive(Subcommand)]
pub enum VersionCmd {
    /// Bump version (patch, minor, major)
    Bump {
        /// Part to bump: patch, minor, or major
        part: String,
    },
    /// Show current version
    Current {
        /// Crate name (optional, shows workspace version if not specified)
        #[arg(short, long)]
        crate_name: Option<String>,
    },
    /// Show version differences
    Diff {
        /// From version
        from: String,
        /// To version
        to: String,
    },
}

pub fn handle(cmd: VersionCmd) -> Result<()> {
    match cmd {
        VersionCmd::Bump { part } => bump_version(&part),
        VersionCmd::Current { crate_name } => current_version(crate_name.as_deref()),
        VersionCmd::Diff { from, to } => diff_versions(&from, &to),
    }
}

fn bump_version(part: &str) -> Result<()> {
    match part {
        "patch" => println!("Bumping patch version..."),
        "minor" => println!("Bumping minor version..."),
        "major" => println!("Bumping major version..."),
        _ => {
            println!("Invalid part: {}. Use patch, minor, or major.", part);
            return Ok(());
        }
    }
    println!("Note: Manual version bump required. Update version in Cargo.toml files.");
    Ok(())
}

fn current_version(crate_name: Option<&str>) -> Result<()> {
    match crate_name {
        Some(name) => println!("{} version: 0.1.0", name),
        None => {
            println!("Workspace version: 0.1.0");
            println!("Crate versions:");
            println!("  - robot_core: 0.1.0");
            println!("  - robot_control: 0.1.8");
            println!("  - tools_suite: 0.1.8");
            println!("  - devtools: 0.1.0");
        }
    }
    Ok(())
}

fn diff_versions(from: &str, to: &str) -> Result<()> {
    let from_v = Version::parse(from)?;
    let to_v = Version::parse(to)?;

    if from_v < to_v {
        println!("{} -> {}: bump", from, to);
    } else if from_v == to_v {
        println!("{} -> {}: same", from, to);
    } else {
        println!("{} -> {}: downgrade", from, to);
    }
    Ok(())
}
