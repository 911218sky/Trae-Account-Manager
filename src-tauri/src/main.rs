// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[tokio::main]
async fn main() {
    // Check if any CLI arguments are provided
    let args: Vec<String> = std::env::args().collect();
    
    // If we have more than 1 argument (program name + subcommand), we're in CLI mode
    if args.len() > 1 {
        // Try to parse CLI arguments
        match trae_auto_lib::cli::Cli::try_parse() {
            Ok(cli) => {
                // CLI mode
                match trae_auto_lib::cli::handle_cli(cli).await {
                    Ok(_) => std::process::exit(0),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                // Check if this is a help or version request (should exit with 0)
                // or an actual error (should exit with 1)
                use clap::error::ErrorKind;
                match e.kind() {
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                        // Help and version are not errors, output to stdout
                        println!("{}", e);
                        std::process::exit(0);
                    }
                    _ => {
                        // Actual errors go to stderr
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    } else {
        // GUI mode - run the Tauri application
        trae_auto_lib::run().await;
    }
}
