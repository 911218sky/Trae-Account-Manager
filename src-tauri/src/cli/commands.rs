// CLI command handlers

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::{Cli, Commands};
use crate::account::AccountManager;
use crate::switcher::AccountSwitcher;
use crate::websocket::WebSocketServer;

/// Handle CLI commands
pub async fn handle_cli(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::List => handle_list().await,
        Commands::Switch { account } => handle_switch(&account).await,
        Commands::Current => handle_current().await,
        Commands::Daemon { background } => handle_daemon(background).await,
    }
}

/// List all logged-in accounts
async fn handle_list() -> Result<()> {
    let account_manager = AccountManager::new()?;
    let accounts = account_manager.get_accounts();

    if accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    println!("Logged-in accounts:");
    println!("{:<40} {:<30} {:<20} {:<10}", "ID", "Email", "Name", "Status");
    println!("{}", "-".repeat(100));

    for account in accounts {
        let status = if account.is_current {
            "CURRENT"
        } else {
            match account.token_status {
                crate::account::types::TokenStatus::Normal => "Active",
                crate::account::types::TokenStatus::Expiring => "Expiring",
                crate::account::types::TokenStatus::Expired => "Expired",
                crate::account::types::TokenStatus::Unknown => "Unknown",
            }
        };

        println!(
            "{:<40} {:<30} {:<20} {:<10}",
            account.id, account.email, account.name, status
        );
    }

    Ok(())
}

/// Switch to a specific account by email or account ID
async fn handle_switch(account_identifier: &str) -> Result<()> {
    // Initialize account manager
    let account_manager = AccountManager::new()?;
    let accounts = account_manager.get_accounts();

    // Find account by email or ID
    let target_account = accounts
        .iter()
        .find(|acc| acc.id == account_identifier || acc.email == account_identifier)
        .ok_or_else(|| anyhow!("Account not found: {}", account_identifier))?;

    // Check if already current
    if target_account.is_current {
        println!("Already using account: {} ({})", target_account.email, target_account.name);
        return Ok(());
    }

    // Initialize WebSocket server for notifications
    let ws_server = WebSocketServer::start(9527).await?;
    let ws_server_arc = Arc::new(ws_server);

    // Initialize account switcher
    let account_manager_arc = Arc::new(Mutex::new(account_manager));
    let switcher = AccountSwitcher::new(account_manager_arc, ws_server_arc)?;

    // Perform the switch
    println!("Switching to account: {} ({})...", target_account.email, target_account.name);

    let result = switcher
        .switch_account(&target_account.id, |progress| {
            println!(
                "[{}/{}] {}",
                progress.step, progress.total, progress.message
            );
        })
        .await;

    match result {
        Ok(switch_result) => {
            println!(
                "✓ Successfully switched to account: {} ({})",
                target_account.email, target_account.name
            );
            println!(
                "  Duration: {}ms, Notified {} IDE instance(s)",
                switch_result.duration_ms, switch_result.notified_instances
            );
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Failed to switch account: {}", AccountSwitcher::get_error_message(&e));
            
            // Check if error is retryable
            if AccountSwitcher::is_error_retryable(&e) {
                eprintln!("  This error is retryable. Please try again.");
            }
            
            Err(anyhow!("Switch failed: {}", e))
        }
    }
}

/// Display the current active account
async fn handle_current() -> Result<()> {
    let account_manager = AccountManager::new()?;

    match account_manager.get_active_account() {
        Some(account) => {
            println!("Current account:");
            println!("  ID:     {}", account.id);
            println!("  Name:   {}", account.name);
            println!("  Email:  {}", account.email);
            println!("  User:   {}", account.user_id);
            println!("  Tenant: {}", account.tenant_id);
            println!("  Region: {}", account.region);
            println!("  Plan:   {}", account.plan_type);
            
            // Show token status
            let token_status = crate::account::types::AccountBrief::determine_token_status(
                &account.token_expired_at,
            );
            let status_str = match token_status {
                crate::account::types::TokenStatus::Normal => "Active",
                crate::account::types::TokenStatus::Expiring => "Expiring soon",
                crate::account::types::TokenStatus::Expired => "Expired",
                crate::account::types::TokenStatus::Unknown => "Unknown",
            };
            println!("  Token:  {}", status_str);
            
            if let Some(expires_at) = &account.token_expired_at {
                println!("  Expires: {}", expires_at);
            }
            
            Ok(())
        }
        None => {
            println!("No active account.");
            Ok(())
        }
    }
}

/// Start the daemon process
async fn handle_daemon(background: bool) -> Result<()> {
    if background {
        println!("Starting daemon in background mode...");
        // TODO: Implement background daemon mode
        // This would typically involve:
        // 1. Forking the process (Unix) or creating a service (Windows)
        // 2. Detaching from the terminal
        // 3. Running the Tauri app without a window
        eprintln!("Background mode not yet implemented");
        return Err(anyhow!("Background daemon mode is not yet implemented"));
    } else {
        println!("Starting daemon in foreground mode...");
        // Run the Tauri application normally
        // This will be handled by the main application entry point
        crate::run().await;
        Ok(())
    }
}
