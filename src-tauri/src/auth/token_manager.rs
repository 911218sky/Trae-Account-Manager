use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::account::AccountManager;

/// Manages the lifecycle of access tokens and refresh tokens.
pub struct TokenManager {
    account_manager: Arc<Mutex<AccountManager>>,
    refresh_lock: Arc<Mutex<()>>,
}

impl TokenManager {
    /// Creates a new token manager instance.
    /// 
    /// # Arguments
    /// * `account_manager` - Reference to the account manager wrapped in Arc<Mutex>
    /// 
    /// # Returns
    /// A new TokenManager instance.
    pub fn new(account_manager: Arc<Mutex<AccountManager>>) -> Self {
        Self {
            account_manager,
            refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Refreshes the token for the currently active account.
    /// Uses a mutex lock to ensure only one refresh operation occurs at a time.
    /// 
    /// # Returns
    /// * `Ok(())` - Token refresh succeeded
    /// * `Err(String)` - Token refresh failed with error message
    pub async fn refresh_active_account_token(&self) -> Result<(), String> {
        // Acquire lock to prevent concurrent refresh operations.
        let _lock = self.refresh_lock.lock().await;

        // Get the account manager.
        let mut account_manager = self.account_manager.lock().await;

        // Get the currently active account.
        let active_account = account_manager
            .get_active_account()
            .ok_or_else(|| "No active account".to_string())?;

        let account_id = active_account.id.clone();

        // Check if account has cookies (required for token refresh).
        if active_account.cookies.is_empty() {
            return Err("Account has no cookies, cannot refresh token".to_string());
        }

        // Call the account manager's refresh_token method.
        account_manager
            .refresh_token(&account_id)
            .await
            .map_err(|e| format!("Token refresh failed: {}", e))?;

        Ok(())
    }

    /// Checks if the token is expiring soon (within 30 minutes).
    /// 
    /// # Returns
    /// * `Ok(true)` - Token is expiring or has expired
    /// * `Ok(false)` - Token is still valid
    /// * `Err(String)` - Check failed with error message
    pub async fn is_token_expiring_soon(&self) -> Result<bool, String> {
        let account_manager = self.account_manager.lock().await;

        // Get the currently active account.
        let active_account = account_manager
            .get_active_account()
            .ok_or_else(|| "No active account".to_string())?;

        // Check if token expiration time is available.
        match &active_account.token_expired_at {
            None => Ok(true), // No expiration info, treat as needing refresh.
            Some(expired_at) => {
                // Try parsing as RFC3339 format.
                match chrono::DateTime::parse_from_rfc3339(expired_at) {
                    Ok(expiry) => {
                        let now = chrono::Utc::now();
                        let thirty_minutes = chrono::Duration::minutes(30);
                        Ok(expiry.with_timezone(&chrono::Utc) < now + thirty_minutes)
                    }
                    Err(_) => {
                        // Try parsing as Unix timestamp (seconds).
                        if let Ok(ts) = expired_at.parse::<i64>() {
                            let now = chrono::Utc::now().timestamp();
                            Ok(ts < now + 1800) // 30 minutes = 1800 seconds
                        } else {
                            Ok(true) // Cannot parse, treat as needing refresh.
                        }
                    }
                }
            }
        }
    }

    /// Clears the token for the current account (used for forced re-login).
    /// 
    /// # Returns
    /// * `Ok(())` - Token clear succeeded
    /// * `Err(String)` - Token clear failed with error message
    /// 
    /// # Note
    /// This method requires the AccountManager to provide a clear_token function.
    /// Currently, the AccountManager does not provide a direct token clearing method.
    /// This will be implemented in subsequent integration tasks.
    pub async fn clear_active_account_token(&self) -> Result<(), String> {
        // Note: This functionality requires adding a clear_token method to AccountManager.
        // Currently, AccountManager does not provide a direct token clearing method.
        // This will be implemented in subsequent integration tasks.
        
        Err("Token clearing functionality will be implemented in subsequent tasks".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be implemented in subsequent tasks.
}
