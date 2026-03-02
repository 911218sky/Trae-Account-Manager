// CLI module
// Command-line interface for account switching

pub mod commands;
pub mod types;

pub use commands::handle_cli;
pub use types::Cli;
