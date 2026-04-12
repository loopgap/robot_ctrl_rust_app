//! Clean commands

use anyhow::Result;
use std::path::Path;

pub fn handle(all: bool) -> Result<()> {
    if all {
        println!("Cleaning all build artifacts and cache...");
    } else {
        println!("Cleaning build artifacts...");
    }

    let targets = vec!["target"];

    if all {
        println!("Also removing cache directories...");
    }

    for target in targets {
        let path = Path::new(target);
        if path.exists() {
            println!("Removing {}...", target);
            std::fs::remove_dir_all(target)?;
        }
    }

    println!("\nClean complete!");
    Ok(())
}
