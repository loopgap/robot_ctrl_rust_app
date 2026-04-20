//! Validate workspace structure

use anyhow::Result;
use std::path::Path;

pub fn handle(strict: bool) -> Result<()> {
    println!("Validating workspace structure...\n");

    let required_dirs = vec![
        "crates/robot_core/src",
        "crates/robot_control/src",
        "crates/tools_suite/src",
        "crates/devtools/src",
    ];

    let required_files = vec![
        "Cargo.toml",
        "crates/robot_core/Cargo.toml",
        "crates/robot_control/Cargo.toml",
        "crates/tools_suite/Cargo.toml",
        "crates/devtools/Cargo.toml",
    ];

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check directories
    for dir in &required_dirs {
        if Path::new(dir).exists() {
            println!("[OK] {}", dir);
        } else {
            errors.push(format!("Missing directory: {}", dir));
        }
    }

    // Check files
    for file in &required_files {
        if Path::new(file).exists() {
            println!("[OK] {}", file);
        } else {
            errors.push(format!("Missing file: {}", file));
        }
    }

    // Check .cargo/config.toml
    if Path::new(".cargo/config.toml").exists() {
        println!("[OK] .cargo/config.toml");
    } else if strict {
        warnings.push("Missing .cargo/config.toml - linker flags not configured");
    }

    // Summary
    println!("\n=== Validation Summary ===");
    if errors.is_empty() && warnings.is_empty() {
        println!("✓ Workspace structure is valid!");
    } else {
        if !errors.is_empty() {
            println!("\n✗ Errors:");
            for e in &errors {
                println!("  - {}", e);
            }
        }
        if !warnings.is_empty() {
            println!("\n⚠ Warnings:");
            for w in &warnings {
                println!("  - {}", w);
            }
        }
    }

    if strict && (!errors.is_empty() || !warnings.is_empty()) {
        std::process::exit(1);
    }

    Ok(())
}
