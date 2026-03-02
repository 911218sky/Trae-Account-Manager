mod api;
mod account;
mod auth;
mod commands;
mod machine;
mod login;

// External Account Switcher modules
mod tray;
mod switcher;
mod websocket;
mod storage;
mod hotkey;
pub mod cli;
mod config;

// Re-export public types
pub use config::AppConfig;
pub use switcher::AccountSwitcher;
pub use websocket::{WebSocketServer, SessionEvent};
pub use account::AccountManager;

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{State, Emitter};

use account::{AccountBrief, AccountManager as InternalAccountManager, Account, PaginationParams, PaginatedResult, ExportFormat, ExportResponse, ExportService};
use api::{UsageSummary, UsageQueryResponse};
use auth::TokenManager;
use commands::{refresh_active_token, force_reauth, check_token_expiry};
use storage::{SecureStorageManager, migrate_to_secure_storage};
use hotkey::{HotkeyManager, HotkeyAction};

/// Application state containing all major components.
pub struct AppState {
    pub account_manager: Arc<Mutex<InternalAccountManager>>,
    pub token_manager: Arc<Mutex<TokenManager>>,
    pub tray_manager: Arc<Mutex<Option<tray::SystemTrayManager>>>,
    pub account_switcher: Arc<switcher::AccountSwitcher>,
    pub ws_server: Arc<websocket::WebSocketServer>,
    pub app_config: Arc<Mutex<AppConfig>>,
}

/// Error type for API responses.
#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    pub message: String,
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, ApiError>;

// ============ Tauri Commands ============

/// Adds an account using a token and optional cookies.
#[tauri::command]
async fn add_account_by_token(token: String, cookies: Option<String>, state: State<'_, AppState>) -> Result<Account> {
    let mut manager = state.account_manager.lock().await;
    manager.add_account_by_token(token, cookies).await.map_err(Into::into)
}

/// Removes an account.
#[tauri::command]
async fn remove_account(account_id: String, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state.account_manager.lock().await;
    manager.remove_account(&account_id).map_err(Into::into)
}

/// Retrieves all accounts.
#[tauri::command]
async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountBrief>> {
    let manager = state.account_manager.lock().await;
    Ok(manager.get_accounts())
}

/// Retrieves a paginated list of accounts.
#[tauri::command]
async fn get_accounts_paginated(
    page: usize,
    page_size: usize,
    state: State<'_, AppState>
) -> Result<PaginatedResult<AccountBrief>> {
    let manager = state.account_manager.lock().await;
    manager.get_accounts_paginated(PaginationParams { page, page_size })
        .map_err(Into::into)
}

/// Retrieves details for a single account.
#[tauri::command]
async fn get_account(account_id: String, state: State<'_, AppState>) -> Result<Account> {
    let manager = state.account_manager.lock().await;
    manager.get_account(&account_id).map_err(Into::into)
}

/// Switches to an account (sets it as active and updates machine ID).
#[tauri::command]
async fn switch_account(account_id: String, state: State<'_, AppState>) -> Result<()> {
    // Use the AccountSwitcher for proper integration
    let result = state.account_switcher
        .switch_account(&account_id, |progress| {
            log::debug!(
                "Switch progress: [{}/{}] {}",
                progress.step,
                progress.total,
                progress.message
            );
        })
        .await;
    
    match result {
        Ok(switch_result) => {
            log::info!(
                "Account switched successfully: {} ({}ms, {} IDE instances notified)",
                account_id,
                switch_result.duration_ms,
                switch_result.notified_instances
            );
            
            // Update tray menu to reflect the change
            let manager = state.account_manager.lock().await;
            let accounts = manager.get_accounts();
            drop(manager);
            
            let mut tray_manager = state.tray_manager.lock().await;
            if let Some(tray) = tray_manager.as_mut() {
                if let Err(e) = tray.update_account_menu(accounts) {
                    log::warn!("Failed to update tray menu: {}", e);
                }
            }
            
            Ok(())
        }
        Err(e) => {
            let error_msg = switcher::AccountSwitcher::get_error_message(&e);
            log::error!("Failed to switch account: {}", error_msg);
            Err(ApiError {
                message: error_msg,
            })
        }
    }
}

/// Retrieves usage information for an account.
#[tauri::command]
async fn get_account_usage(account_id: String, state: State<'_, AppState>) -> Result<UsageSummary> {
    let mut manager = state.account_manager.lock().await;
    manager.get_account_usage(&account_id).await.map_err(Into::into)
}

/// Updates the token for an account.
#[tauri::command]
async fn update_account_token(account_id: String, token: String, state: State<'_, AppState>) -> Result<UsageSummary> {
    let mut manager = state.account_manager.lock().await;
    manager.update_account_token(&account_id, token).await.map_err(Into::into)
}

/// Exports all accounts.
#[tauri::command]
async fn export_accounts(state: State<'_, AppState>) -> Result<String> {
    let manager = state.account_manager.lock().await;
    manager.export_accounts().map_err(Into::into)
}

/// Imports accounts from data.
#[tauri::command]
async fn import_accounts(data: String, state: State<'_, AppState>) -> Result<usize> {
    let mut manager = state.account_manager.lock().await;
    manager.import_accounts(&data).await.map_err(Into::into)
}

/// Exports selected accounts in the specified format.
#[tauri::command]
async fn export_selected_accounts(
    account_ids: Vec<String>,
    format: String,
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResponse> {
    // Validate parameters
    if account_ids.is_empty() {
        return Err(ApiError {
            message: "Please select at least one account".to_string(),
        });
    }
    
    if account_ids.len() > 10000 {
        return Err(ApiError {
            message: "Export quantity exceeds limit (maximum 10,000)".to_string(),
        });
    }
    
    let export_format = match format.as_str() {
        "csv" => ExportFormat::Csv,
        "json" => ExportFormat::Json,
        _ => {
            return Err(ApiError {
                message: "Unsupported export format".to_string(),
            });
        }
    };
    
    // Call export service
    let export_service = ExportService::new(state.account_manager.clone())
        .map_err(|e| ApiError { message: e.to_string() })?;
    
    export_service
        .export_accounts(account_ids, export_format, Some(app_handle))
        .await
        .map_err(Into::into)
}

/// Retrieves usage events for an account.
#[tauri::command]
async fn get_usage_events(
    account_id: String,
    start_time: i64,
    end_time: i64,
    page_num: i32,
    page_size: i32,
    state: State<'_, AppState>
) -> Result<UsageQueryResponse> {
    let mut manager = state.account_manager.lock().await;
    manager.get_usage_events(&account_id, start_time, end_time, page_num, page_size)
        .await
        .map_err(Into::into)
}

/// Reads account information from the Trae IDE.
#[tauri::command]
async fn read_trae_account(state: State<'_, AppState>) -> Result<Option<Account>> {
    let mut manager = state.account_manager.lock().await;
    manager.read_trae_ide_account().await.map_err(Into::into)
}

/// Retrieves the current system machine ID.
#[tauri::command]
async fn get_machine_id() -> Result<String> {
    machine::get_machine_guid().map_err(Into::into)
}

/// Resets the system machine ID (generates a new random ID).
#[tauri::command]
async fn reset_machine_id() -> Result<String> {
    machine::reset_machine_guid().map_err(Into::into)
}

/// Sets the system machine ID to a specific value.
#[tauri::command]
async fn set_machine_id(machine_id: String) -> Result<()> {
    machine::set_machine_guid(&machine_id).map_err(Into::into)
}

/// Binds the account machine ID (saves the current system machine ID to the account).
#[tauri::command]
async fn bind_account_machine_id(account_id: String, state: State<'_, AppState>) -> Result<String> {
    let mut manager = state.account_manager.lock().await;
    manager.bind_machine_id(&account_id).map_err(Into::into)
}

/// Retrieves the Trae IDE's machine ID.
#[tauri::command]
async fn get_trae_machine_id() -> Result<String> {
    machine::get_trae_machine_id().map_err(Into::into)
}

/// Sets the Trae IDE's machine ID.
#[tauri::command]
async fn set_trae_machine_id(machine_id: String) -> Result<()> {
    machine::set_trae_machine_id(&machine_id).map_err(Into::into)
}

/// Clears the Trae IDE login state (resets the IDE to a fresh installation state).
#[tauri::command]
async fn clear_trae_login_state() -> Result<()> {
    machine::clear_trae_login_state().map_err(Into::into)
}

/// Retrieves the saved Trae IDE path.
#[tauri::command]
async fn get_trae_path() -> Result<String> {
    machine::get_saved_trae_path().map_err(Into::into)
}

/// Sets the Trae IDE path.
#[tauri::command]
async fn set_trae_path(path: String) -> Result<()> {
    machine::save_trae_path(&path).map_err(Into::into)
}

/// Automatically scans for the Trae IDE path.
#[tauri::command]
async fn scan_trae_path() -> Result<String> {
    machine::scan_trae_path().map_err(Into::into)
}

/// Refreshes the token for a single account.
#[tauri::command]
async fn refresh_token(account_id: String, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state.account_manager.lock().await;
    manager.refresh_token(&account_id).await.map_err(Into::into)
}

/// Re-logs in to an account (refreshes token and rewrites to IDE).
#[tauri::command]
async fn relogin_account(account_id: String, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state.account_manager.lock().await;
    
    // Refresh token
    manager.refresh_token(&account_id).await
        .map_err(|e| ApiError {
            message: format!("Failed to refresh token: {}", e),
        })?;
    
    // If this is the currently active account, rewrite to IDE
    if let Some(active_account) = manager.get_active_account() {
        if active_account.id == account_id {
            let account = manager.get_account(&account_id)
                .map_err(|e| ApiError {
                    message: format!("Account not found: {}", e),
                })?;
            
            let token = account.jwt_token.as_ref()
                .ok_or_else(|| ApiError {
                    message: "Account does not have a valid token".to_string(),
                })?;
            
            let login_info = crate::machine::TraeLoginInfo {
                token: token.clone(),
                refresh_token: None,
                user_id: account.user_id.clone(),
                email: account.email.clone(),
                username: account.name.clone(),
                avatar_url: account.avatar_url.clone(),
                host: String::new(),
                region: if account.region.is_empty() { "SG".to_string() } else { account.region.clone() },
            };
            
            crate::machine::write_trae_login_info(&login_info)
                .map_err(|e| ApiError {
                    message: format!("Failed to rewrite to IDE: {}", e),
                })?;
            
            log::info!("Re-logged in to account and wrote to IDE: {}", account.email);
        }
    }
    
    Ok(())
}

/// Batch refreshes all tokens that are about to expire.
#[tauri::command]
async fn refresh_all_tokens(state: State<'_, AppState>) -> Result<Vec<String>> {
    let mut manager = state.account_manager.lock().await;
    manager.refresh_all_tokens().await.map_err(Into::into)
}

/// Claims a gift.
#[tauri::command]
async fn claim_gift(account_id: String, state: State<'_, AppState>) -> Result<()> {
    let mut manager = state.account_manager.lock().await;
    manager.claim_birthday_bonus(&account_id).await.map_err(Into::into)
}

/// Starts browser login flow.
#[tauri::command]
async fn start_browser_login(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<()> {
    let manager = state.account_manager.clone();
    login::start_login_flow(app, manager).await.map_err(|e| ApiError { message: e })?;
    Ok(())
}

/// Migrates credentials to secure storage.
#[tauri::command]
async fn migrate_credentials(state: State<'_, AppState>) -> Result<usize> {
    let mut manager = state.account_manager.lock().await;
    let secure_storage = SecureStorageManager::new().map_err(|e| ApiError { message: e.to_string() })?;
    
    migrate_to_secure_storage(&mut manager, &secure_storage)
        .await
        .map_err(Into::into)
}

/// Updates the system tray menu.
#[tauri::command]
async fn update_tray_menu(state: State<'_, AppState>) -> Result<()> {
    let manager = state.account_manager.lock().await;
    let accounts = manager.get_accounts();
    
    let mut tray_manager = state.tray_manager.lock().await;
    if let Some(tray) = tray_manager.as_mut() {
        tray.update_account_menu(accounts)
            .map_err(|e| ApiError { message: e.to_string() })?;
    }
    
    Ok(())
}

/// Retrieves the default hotkey.
#[tauri::command]
async fn get_default_hotkey() -> Result<String> {
    Ok(HotkeyManager::default_hotkey())
}

/// Retrieves the switch state.
#[tauri::command]
async fn get_switch_state(state: State<'_, AppState>) -> Result<serde_json::Value> {
    let switch_state = state.account_switcher.get_state().await;
    Ok(serde_json::to_value(switch_state).map_err(|e| ApiError {
        message: e.to_string(),
    })?)
}

/// Rolls back to the previous account.
#[tauri::command]
async fn rollback_account(state: State<'_, AppState>) -> Result<()> {
    state.account_switcher
        .rollback()
        .await
        .map_err(|e| ApiError {
            message: switcher::AccountSwitcher::get_error_message(&e),
        })?;
    
    // Update tray menu after rollback
    let manager = state.account_manager.lock().await;
    let accounts = manager.get_accounts();
    drop(manager);
    
    let mut tray_manager = state.tray_manager.lock().await;
    if let Some(tray) = tray_manager.as_mut() {
        if let Err(e) = tray.update_account_menu(accounts) {
            log::warn!("Failed to update tray menu: {}", e);
        }
    }
    
    Ok(())
}

/// Validates an account.
#[tauri::command]
async fn validate_account(account_id: String, state: State<'_, AppState>) -> Result<bool> {
    state.account_switcher
        .validate_account(&account_id)
        .await
        .map_err(|e| ApiError {
            message: switcher::AccountSwitcher::get_error_message(&e),
        })
}

/// Retrieves the number of WebSocket connections.
#[tauri::command]
async fn get_websocket_connections(state: State<'_, AppState>) -> Result<usize> {
    Ok(state.ws_server.connection_count().await)
}

/// Retrieves the application configuration.
#[tauri::command]
async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig> {
    let config = state.app_config.lock().await;
    Ok(config.clone())
}

/// Updates the application configuration.
#[tauri::command]
async fn update_app_config(config: AppConfig, state: State<'_, AppState>) -> Result<()> {
    // Save to file
    config.save().map_err(|e| ApiError {
        message: e.to_string(),
    })?;
    
    // Update in-memory config
    let mut app_config = state.app_config.lock().await;
    *app_config = config;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // Step 1: Load application configuration
    let app_config = AppConfig::load().expect("Failed to load application configuration");
    let app_config_arc = Arc::new(Mutex::new(app_config.clone()));
    
    // Step 2: Initialize account manager
    let account_manager = InternalAccountManager::new().expect("Failed to initialize account manager");
    let account_manager_arc = Arc::new(Mutex::new(account_manager));
    
    // Step 3: Create TokenManager, sharing AccountManager
    let token_manager = TokenManager::new(account_manager_arc.clone());
    
    // Step 4: Initialize WebSocket server with fallback ports
    let ws_port = app_config.websocket_port;
    let fallback_ports = [ws_port, 9528, 9529, 9530, 9531];
    
    let mut ws_server = None;
    let mut last_error = None;
    
    for port in fallback_ports.iter() {
        match websocket::WebSocketServer::start(*port).await {
            Ok(server) => {
                log::info!("WebSocket server started on port: {}", port);
                ws_server = Some(server);
                break;
            }
            Err(e) => {
                log::warn!("Port {} is in use, trying next port...", port);
                last_error = Some(e);
            }
        }
    }
    
    let ws_server = ws_server.unwrap_or_else(|| {
        panic!("Failed to start WebSocket server, all ports are in use: {:?}", last_error);
    });
    let ws_server_arc = Arc::new(ws_server);
    
    // Start WebSocket heartbeat loop
    let ws_server_clone = ws_server_arc.clone();
    tauri::async_runtime::spawn(async move {
        ws_server_clone.heartbeat_loop().await;
    });
    
    log::info!("WebSocket server started on port {}", ws_port);
    
    // Step 5: Initialize AccountSwitcher
    let account_switcher = switcher::AccountSwitcher::new(
        account_manager_arc.clone(),
        ws_server_arc.clone(),
    ).expect("Failed to initialize account switcher");
    let account_switcher_arc = Arc::new(account_switcher);
    
    // Create shared state for tray manager
    let tray_manager_arc = Arc::new(Mutex::new(None));
    let tray_manager_clone = tray_manager_arc.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
            // Step 6: Initialize system tray manager after app is ready
            let app_handle = app.handle().clone();
            let tray_manager = tray::SystemTrayManager::new(app_handle.clone())
                .map_err(|e| format!("Failed to initialize tray manager: {}", e))?;
            
            // Store tray manager in shared state using try_lock (non-blocking)
            let mut tray_lock = tray_manager_clone.try_lock()
                .map_err(|_| "Failed to acquire tray manager lock")?;
            *tray_lock = Some(tray_manager);
            drop(tray_lock); // Explicitly release the lock
            
            log::info!("System tray manager initialized");
            
            // Step 7: Initialize hotkey manager and register default hotkey
            // Note: HotkeyManager must stay in the main thread due to platform limitations
            let mut hotkey_manager = HotkeyManager::new()
                .map_err(|e| format!("Failed to initialize hotkey manager: {}", e))?;
            
            let default_hotkey = HotkeyManager::default_hotkey();
            if let Err(e) = hotkey_manager.register(&default_hotkey, HotkeyAction::OpenMenu) {
                log::warn!("Failed to register default hotkey: {}", e);
            } else {
                log::info!("Registered default hotkey: {}", default_hotkey);
            }
            
            // Step 8: Start hotkey event loop
            // Connect hotkey events to the account switcher and tray
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                // Get the global hotkey event receiver
                let receiver = global_hotkey::GlobalHotKeyEvent::receiver();
                log::info!("Hotkey event loop started");
                
                loop {
                    // Poll for hotkey events
                    if let Ok(event) = receiver.try_recv() {
                        log::debug!("Received hotkey event: {:?}", event);
                        
                        // Emit hotkey event to frontend
                        // The frontend or tray menu will handle opening the menu
                        if let Err(e) = app_handle_clone.emit("hotkey-triggered", event.id) {
                            log::error!("Failed to emit hotkey event: {}", e);
                        }
                    }
                    
                    // Small delay to prevent busy-waiting
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });
            
            log::info!("Application startup complete - all components initialized");
            
            Ok(())
        })
        .manage(AppState {
            account_manager: account_manager_arc,
            token_manager: Arc::new(Mutex::new(token_manager)),
            tray_manager: tray_manager_arc,
            account_switcher: account_switcher_arc,
            ws_server: ws_server_arc,
            app_config: app_config_arc,
        })
        .invoke_handler(tauri::generate_handler![
            add_account_by_token,
            remove_account,
            get_accounts,
            get_accounts_paginated,
            get_account,
            switch_account,
            get_account_usage,
            update_account_token,
            export_accounts,
            import_accounts,
            export_selected_accounts,
            get_usage_events,
            read_trae_account,
            get_machine_id,
            reset_machine_id,
            set_machine_id,
            bind_account_machine_id,
            get_trae_machine_id,
            set_trae_machine_id,
            clear_trae_login_state,
            get_trae_path,
            set_trae_path,
            scan_trae_path,
            claim_gift,
            refresh_token,
            relogin_account,
            refresh_all_tokens,
            start_browser_login,
            refresh_active_token,
            force_reauth,
            check_token_expiry,
            migrate_credentials,
            update_tray_menu,
            get_default_hotkey,
            get_switch_state,
            rollback_account,
            validate_account,
            get_websocket_connections,
            get_app_config,
            update_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
