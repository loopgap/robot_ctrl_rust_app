//! Code audit commands

use anyhow::Result;

pub fn handle(full: bool) -> Result<()> {
    println!("Running code audit...");

    if full {
        println!("Running full audit (fmt + clippy + security)...");
    } else {
        println!("Running quick audit (fmt only)...");
    }

    println!("\n=== Formatting Check ===");
    let _ = std::process::Command::new("cargo")
        .args(["fmt", "--check"])
        .status();

    if full {
        println!("\n=== Clippy Lints ===");
        let _ = std::process::Command::new("cargo")
            .args(["clippy", "--", "-D", "warnings"])
            .status();

        println!("\n=== Security Audit ===");
        let _ = std::process::Command::new("cargo").args(["audit"]).status();
    }

    Ok(())
}
