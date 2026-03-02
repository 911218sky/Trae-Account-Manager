use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::auth::TokenManager;
use crate::ApiError;

type Result<T> = std::result::Result<T, ApiError>;

/// Refreshes the token for the currently active account.
/// 
/// This command uses the TokenManager to refresh the access token and refresh token
/// for the currently active account. A mutex lock ensures only one refresh operation
/// occurs at a time.
/// 
/// # Returns
/// * `Ok(())` - Token refresh succeeded
/// * `Err(ApiError)` - Token refresh failed with error message
#[tauri::command]
pub async fn refresh_active_token(
    state: State<'_, Arc<Mutex<TokenManager>>>
) -> Result<()> {
    let token_manager = state.lock().await;
    token_manager
        .refresh_active_account_token()
        .await
        .map_err(|e| ApiError { message: e })
}

/// Forces re-authentication by clearing the current account's token.
/// 
/// This command clears the access token and refresh token for the currently active account,
/// forcing the user to re-authenticate. The frontend should redirect to the login page
/// after calling this command.
/// 
/// # Returns
/// * `Ok(())` - Token clear succeeded
/// * `Err(ApiError)` - Token clear failed with error message
#[tauri::command]
pub async fn force_reauth(
    state: State<'_, Arc<Mutex<TokenManager>>>
) -> Result<()> {
    let token_manager = state.lock().await;
    token_manager
        .clear_active_account_token()
        .await
        .map_err(|e| ApiError { message: e })
}

/// Checks if the current account's token is expiring soon.
/// 
/// This command checks if the currently active account's token will expire within 30 minutes.
/// The frontend can use this command to decide whether to proactively refresh the token.
/// 
/// # Returns
/// * `Ok(true)` - Token is expiring or has expired (within 30 minutes)
/// * `Ok(false)` - Token is still valid
/// * `Err(ApiError)` - Check failed with error message
#[tauri::command]
pub async fn check_token_expiry(
    state: State<'_, Arc<Mutex<TokenManager>>>
) -> Result<bool> {
    let token_manager = state.lock().await;
    token_manager
        .is_token_expiring_soon()
        .await
        .map_err(|e| ApiError { message: e })
}
