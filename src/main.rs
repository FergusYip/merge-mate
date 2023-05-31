use clap::{Parser, Subcommand};
use std::process::{Command, exit};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create pull requests for all branches in the current stack
    Push {},
    /// Update all pull requests in the current stack
    Update {},
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
        Commands::Push {} => {
            panic!("Push not implemented")
        }
        Commands::Update {} => {
            panic!("Update not implemented")
        }
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
    let output = Command::new("git branchless")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    output.status.success()
}
