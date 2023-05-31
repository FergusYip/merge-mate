extern crate core;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
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

    match &args.command {
        Commands::Push {} => {
            panic!("Push not implemented")
        }
        Commands::Update {} => {
            panic!("Update not implemented")
        }
    }
}
