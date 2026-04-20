//! PR management commands

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum PrCmd {
    /// Create a new pull request
    Create {
        /// Base branch to create PR against
        #[arg(short, long, default_value = "main")]
        base: String,
    },
    /// Check PR status and readiness
    Check {
        /// PR number (optional, checks current branch PR if not specified)
        pr: Option<String>,
    },
    /// Merge a pull request
    Merge {
        /// PR number to merge
        pr: String,
    },
}

pub fn handle(cmd: PrCmd) -> Result<()> {
    match cmd {
        PrCmd::Create { base } => create_pr(&base),
        PrCmd::Check { pr } => check_pr(pr.as_deref()),
        PrCmd::Merge { pr } => merge_pr(&pr),
    }
}

fn create_pr(base: &str) -> Result<()> {
    println!("Creating PR against '{}'...", base);
    println!("Note: This requires GitHub CLI (gh) to be installed and authenticated.");
    println!(
        "Run: gh pr create --base {} --title 'Your Title' --body 'Description'",
        base
    );
    Ok(())
}

fn check_pr(pr: Option<&str>) -> Result<()> {
    match pr {
        Some(pr_num) => println!("Checking PR #{}...", pr_num),
        None => println!("Checking current branch PR status..."),
    }
    println!("Note: This requires GitHub CLI (gh) to be installed.");
    Ok(())
}

fn merge_pr(pr: &str) -> Result<()> {
    println!("Merging PR #{}...", pr);
    println!("Note: This requires GitHub CLI (gh) to be installed and authenticated.");
    Ok(())
}
