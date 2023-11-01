mod commands;

use clap::{Parser, Subcommand};
use std::process::{exit, Command};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create pull requests for all branches in the current stack
    // Push {},
    // Wait for GitHub Pull Request to be in sync with the local branch
    Wait { branch: Option<String> },
    /// Update all pull requests in the current stack
    Update {
        /// Branches to update
        #[arg(default_value_t = String::from("stack()"))]
        revset: String,
    },
}

fn main() {
    let args = Args::parse();

    if !is_github_cli_installed() {
        eprintln!("Github CLI is not installed. Please install it.");
        exit(1)
    }
    if !is_git_branchless_installed() {
        eprintln!("Git Branchless is not installed. Please install it.");
        exit(1)
    }

    match &args.command {
        // Commands::Push {} => {
        //     panic!("Push not implemented")
        // }
        Commands::Wait { branch } => commands::wait(branch.to_owned()),
        Commands::Update { revset } => commands::update(revset),
    }
}

fn is_github_cli_installed() -> bool {
    let output = Command::new("gh")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    output.status.success()
}

fn is_git_branchless_installed() -> bool {
    let output = Command::new("git")
        .args(["branchless", "--version"])
        .output()
        .expect("Failed to execute command");

    output.status.success()
}
