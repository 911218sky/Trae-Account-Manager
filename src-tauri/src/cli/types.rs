// CLI data types

use clap::{Parser, Subcommand};

/// Trae Account Switcher CLI
#[derive(Parser, Debug)]
#[command(name = "trae-switcher")]
#[command(about = "Trae Account Switcher CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all logged-in accounts
    List,
    
    /// Switch to a specific account
    Switch {
        /// Account ID or email
        account: String,
    },
    
    /// Show the current active account
    Current,
    
    /// Start the daemon process
    Daemon {
        /// Run in background
        #[arg(short, long)]
        background: bool,
    },
}
