// Account Switcher core implementation

use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::error::{AuthError, DataError, NetworkError, SwitcherError};
use super::types::{SwitchProgress, SwitchResult, SwitchState, SwitchTransaction};
use crate::account::{Account, AccountManager};
use crate::storage::Credential;
use crate::websocket::{SessionEvent, WebSocketServer};

/// Timeout duration for switch operations (5 seconds).
const SWITCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Core account switcher logic.
pub struct AccountSwitcher {
    account_manager: Arc<Mutex<AccountManager>>,
    ws_server: Arc<WebSocketServer>,
    state: Arc<Mutex<SwitchState>>,
    last_transaction: Arc<Mutex<Option<SwitchTransaction>>>,
}

impl AccountSwitcher {
    /// Creates a new account switcher instance.
    pub fn new(
        account_manager: Arc<Mutex<AccountManager>>,
        ws_server: Arc<WebSocketServer>,
    ) -> Result<Self> {
        Ok(Self {
            account_manager,
            ws_server,
            state: Arc::new(Mutex::new(SwitchState::default())),
            last_transaction: Arc::new(Mutex::new(None)),
        })
    }

    /// Switches to a different account.
    pub async fn switch_account<F>(
        &self,
        account_id: &str,
        progress_callback: F,
    ) -> Result<SwitchResult, SwitcherError>
    where
        F: Fn(SwitchProgress),
    {
        let start_time = Instant::now();

        // Check if a switch is already in progress.
        {
            let state = self.state.lock().await;
            if state.switch_in_progress {
                return Err(SwitcherError::Other(anyhow!(
                    "Another switch operation is already in progress"
                )));
            }
        }

        // Mark switch as in progress.
        {
            let mut state = self.state.lock().await;
            state.switch_in_progress = true;
        }

        // Execute the switch with timeout and error handling.
        let result = match timeout(
            SWITCH_TIMEOUT,
            self.execute_switch(account_id, progress_callback, start_time),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                // Timeout occurred.
                let error = SwitcherError::Network(NetworkError::Timeout(
                    "Switch operation timed out after 5 seconds".to_string(),
                ));

                // Attempt rollback on timeout.
                if let Err(rollback_err) = self.rollback().await {
                    log::error!("Rollback after timeout failed: {}", rollback_err);
                }

                Err(error)
            }
        };

        // Mark switch as complete.
        {
            let mut state = self.state.lock().await;
            state.switch_in_progress = false;
        }

        result
    }

    /// Executes the actual switch logic.
    async fn execute_switch<F>(
        &self,
        account_id: &str,
        progress_callback: F,
        start_time: Instant,
    ) -> Result<SwitchResult, SwitcherError>
    where
        F: Fn(SwitchProgress),
    {
        // Step 1: Validation Phase (~100ms).
        progress_callback(SwitchProgress {
            step: 1,
            total: 4,
            message: "Validating account...".to_string(),
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        });

        let (account, transaction) = self.validate_and_prepare(account_id).await?;

        // Save transaction for potential rollback.
        {
            let mut last_tx = self.last_transaction.lock().await;
            *last_tx = transaction.clone();
        }

        // Step 2: API Call Phase (~500ms).
        // Note: In the current implementation, we don't have a separate backend API switch endpoint.
        // The switch is handled locally by updating the account manager state.
        progress_callback(SwitchProgress {
            step: 2,
            total: 4,
            message: "Updating session...".to_string(),
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        });

        // Update the account manager to switch accounts.
        {
            let mut manager = self.account_manager.lock().await;
            manager
                .switch_account(account_id)
                .map_err(|e| {
                    SwitcherError::Data(DataError::AccountNotFound(format!(
                        "Failed to switch account: {}",
                        e
                    )))
                })?;
        }

        // Step 3: Local Update Phase (~200ms).
        progress_callback(SwitchProgress {
            step: 3,
            total: 4,
            message: "Updating local storage...".to_string(),
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        });

        // Update secure storage with credentials (non-blocking, log warning on failure).
        if let Err(e) = self.update_secure_storage(&account).await {
            log::warn!("Failed to update secure storage (non-critical): {}", e);
        } else {
            log::info!("Secure storage updated successfully");
        }

        // Update state.
        {
            let mut state = self.state.lock().await;
            state.previous_account_id = transaction.as_ref().map(|t| t.previous_account_id.clone());
            state.current_account_id = Some(account_id.to_string());
            state.last_switch_time = Some(chrono::Utc::now().timestamp());
        }

        // Step 4: Notification Phase (~300ms).
        progress_callback(SwitchProgress {
            step: 4,
            total: 4,
            message: "Notifying IDE instances...".to_string(),
            elapsed_ms: start_time.elapsed().as_millis() as u64,
        });

        let notified_count = self.broadcast_session_change(&account).await?;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        // Update state with duration.
        {
            let mut state = self.state.lock().await;
            state.last_switch_duration_ms = Some(duration_ms);
        }

        Ok(SwitchResult {
            success: true,
            account_id: account_id.to_string(),
            duration_ms,
            notified_instances: notified_count,
        })
    }

    /// Validates account and prepares for switch.
    async fn validate_and_prepare(
        &self,
        account_id: &str,
    ) -> Result<(Account, Option<SwitchTransaction>), SwitcherError> {
        let manager = self.account_manager.lock().await;

        // Check if account exists.
        let account = manager.get_account(account_id).map_err(|_| {
            SwitcherError::Data(DataError::AccountNotFound(account_id.to_string()))
        })?;

        // Validate token exists.
        if account.jwt_token.is_none() {
            return Err(SwitcherError::Auth(AuthError::InvalidToken));
        }

        // Check token expiration status.
        let token_status = crate::account::types::AccountBrief::determine_token_status(
            &account.token_expired_at,
        );
        if token_status == crate::account::types::TokenStatus::Expired {
            return Err(SwitcherError::Auth(AuthError::TokenExpired));
        }

        // Get current account for transaction/rollback.
        let transaction = manager.get_active_account().and_then(|current_account| {
            current_account.jwt_token.as_ref().map(|token| {
                SwitchTransaction::new(
                    current_account.id.clone(),
                    token.clone(),
                    current_account.cookies.clone(),
                    current_account.token_expired_at.clone(),
                )
            })
        });

        Ok((account, transaction))
    }

    /// Updates secure storage with account credentials.
    async fn update_secure_storage(&self, account: &Account) -> Result<(), SwitcherError> {
        let manager = self.account_manager.lock().await;
        let secure_storage = manager.get_secure_storage();

        let credential = Credential {
            token: account.jwt_token.clone().unwrap_or_default(),
            refresh_token: None,
            cookies: account.cookies.clone(),
            expires_at: account.token_expired_at.clone(),
        };

        secure_storage
            .store_credential(&account.id, &credential)
            .map_err(|e| {
                SwitcherError::System(crate::switcher::error::SystemError::SecureStorageAccessDenied(
                    format!("Failed to update secure storage: {}", e),
                ))
            })?;

        Ok(())
    }

    /// Broadcasts session change to all IDE instances.
    async fn broadcast_session_change(&self, account: &Account) -> Result<usize, SwitcherError> {
        let event = SessionEvent::SessionChanged {
            account_id: account.id.clone(),
            user_id: account.user_id.clone(),
            email: account.email.clone(),
            token: account.jwt_token.clone().unwrap_or_default(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        self.ws_server
            .broadcast(event)
            .await
            .map_err(|_e| SwitcherError::Network(NetworkError::WebSocketDisconnected))
    }

    /// Rolls back to the previous account using saved transaction.
    pub async fn rollback(&self) -> Result<(), SwitcherError> {
        // Get the saved transaction.
        let transaction = {
            let last_tx = self.last_transaction.lock().await;
            last_tx
                .clone()
                .ok_or_else(|| {
                    SwitcherError::Other(anyhow!("No previous account to rollback to"))
                })?
        };

        log::info!(
            "Rolling back to account: {}",
            transaction.previous_account_id
        );

        // Restore previous session state.
        {
            let mut manager = self.account_manager.lock().await;
            manager
                .switch_account(&transaction.previous_account_id)
                .map_err(|e| {
                    SwitcherError::Data(DataError::AccountNotFound(format!(
                        "Failed to rollback: {}",
                        e
                    )))
                })?;
        }

        // Restore previous credentials in secure storage.
        {
            let manager = self.account_manager.lock().await;
            let secure_storage = manager.get_secure_storage();

            let credential = Credential {
                token: transaction.previous_token.clone(),
                refresh_token: None,
                cookies: transaction.previous_cookies.clone(),
                expires_at: transaction.previous_token_expired_at.clone(),
            };

            secure_storage
                .store_credential(&transaction.previous_account_id, &credential)
                .map_err(|e| {
                    SwitcherError::System(
                        crate::switcher::error::SystemError::SecureStorageAccessDenied(format!(
                            "Failed to restore credentials: {}",
                            e
                        )),
                    )
                })?;
        }

        // Notify IDE instances of rollback.
        let manager = self.account_manager.lock().await;
        if let Ok(account) = manager.get_account(&transaction.previous_account_id) {
            let event = SessionEvent::SessionChanged {
                account_id: account.id.clone(),
                user_id: account.user_id.clone(),
                email: account.email.clone(),
                token: transaction.previous_token.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            };

            self.ws_server.broadcast(event).await.map_err(|_| {
                SwitcherError::Network(NetworkError::WebSocketDisconnected)
            })?;
        }

        // Update UI to reflect rollback.
        {
            let mut state = self.state.lock().await;
            state.current_account_id = Some(transaction.previous_account_id.clone());
            state.previous_account_id = None;
        }

        log::info!("Rollback completed successfully");

        Ok(())
    }

    /// Validates an account.
    pub async fn validate_account(&self, account_id: &str) -> Result<bool, SwitcherError> {
        let manager = self.account_manager.lock().await;

        // Check if account exists.
        let account = match manager.get_account(account_id) {
            Ok(acc) => acc,
            Err(_) => return Ok(false),
        };

        // Check if token exists.
        if account.jwt_token.is_none() {
            return Ok(false);
        }

        // Check token status.
        let token_status = crate::account::types::AccountBrief::determine_token_status(
            &account.token_expired_at,
        );

        // Account is valid if token is not expired.
        Ok(token_status != crate::account::types::TokenStatus::Expired)
    }

    /// Gets the current switch state.
    pub async fn get_state(&self) -> SwitchState {
        self.state.lock().await.clone()
    }

    /// Gets a user-friendly error message.
    pub fn get_error_message(error: &SwitcherError) -> String {
        error.user_message()
    }

    /// Checks if an error is retryable.
    pub fn is_error_retryable(error: &SwitcherError) -> bool {
        error.is_retryable()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::AccountManager;
    use crate::websocket::WebSocketServer;
    use std::sync::atomic::{AtomicU16, Ordering};

    // Global port counter to ensure each test uses a unique port
    static PORT_COUNTER: AtomicU16 = AtomicU16::new(9600);

    /// Helper function to get a unique port for testing.
    fn get_unique_port() -> u16 {
        PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Helper function to create a test account manager.
    async fn create_test_account_manager() -> Arc<Mutex<AccountManager>> {
        let manager = AccountManager::new().expect("Failed to create account manager");
        Arc::new(Mutex::new(manager))
    }

    /// Helper function to create a test WebSocket server.
    async fn create_test_ws_server() -> Arc<WebSocketServer> {
        let port = get_unique_port();
        let server = WebSocketServer::start(port).await.expect("Failed to start WebSocket server");
        Arc::new(server)
    }

    #[tokio::test]
    async fn test_account_switcher_creation() {
        // Test that AccountSwitcher can be created successfully.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;

        let switcher = AccountSwitcher::new(manager, ws_server);
        assert!(switcher.is_ok());
    }

    #[tokio::test]
    async fn test_validate_account_nonexistent() {
        // Test validating a non-existent account returns false.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let result = switcher.validate_account("nonexistent_id").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test]
    async fn test_get_initial_state() {
        // Test that initial state is correct.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let state = switcher.get_state().await;
        assert_eq!(state.current_account_id, None);
        assert_eq!(state.previous_account_id, None);
        assert_eq!(state.switch_in_progress, false);
        assert_eq!(state.last_switch_time, None);
        assert_eq!(state.last_switch_duration_ms, None);
    }

    #[tokio::test]
    async fn test_switch_account_nonexistent() {
        // Test switching to a non-existent account fails.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let result = switcher.switch_account("nonexistent_id", |_| {}).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Account") || error_msg.contains("帳號"),
            "Error message should mention account: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_rollback_without_previous_account() {
        // Test rollback fails when there's no previous account.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let result = switcher.rollback().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No previous account"));
    }

    #[tokio::test]
    async fn test_progress_callback_invoked() {
        // Test that progress callback is invoked during switch.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let progress_count = Arc::new(Mutex::new(0));
        let progress_count_clone = progress_count.clone();

        let callback = move |_progress: SwitchProgress| {
            let count = progress_count_clone.clone();
            tokio::spawn(async move {
                let mut c = count.lock().await;
                *c += 1;
            });
        };

        // This will fail because account doesn't exist, but callback should still be invoked.
        let _ = switcher.switch_account("test_id", callback).await;

        // Give callbacks time to execute.
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let count = progress_count.lock().await;
        // At least the first progress callback should have been invoked.
        assert!(*count >= 1);
    }

    #[tokio::test]
    async fn test_error_classification_account_not_found() {
        // Test that non-existent account returns DataError.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        let result = switcher.switch_account("nonexistent", |_| {}).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert!(matches!(error, SwitcherError::Data(_)));
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        // Test that timeout is enforced (this test verifies the timeout mechanism exists).
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        // Try to switch to non-existent account - should fail quickly, not timeout.
        let start = std::time::Instant::now();
        let result = switcher.switch_account("nonexistent", |_| {}).await;
        let duration = start.elapsed();

        assert!(result.is_err());
        // Should fail quickly (< 1 second), not wait for timeout.
        assert!(duration.as_secs() < 1);
    }

    #[tokio::test]
    async fn test_switch_in_progress_prevents_concurrent_switch() {
        // Test that concurrent switches are prevented.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = Arc::new(AccountSwitcher::new(manager, ws_server).unwrap());

        // Manually set switch_in_progress to true.
        {
            let mut state = switcher.state.lock().await;
            state.switch_in_progress = true;
        }

        // Try to switch - should fail immediately.
        let result = switcher.switch_account("test", |_| {}).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("in progress"));

        // Clean up.
        {
            let mut state = switcher.state.lock().await;
            state.switch_in_progress = false;
        }
    }

    #[tokio::test]
    async fn test_transaction_saved_for_rollback() {
        // Test that transaction is saved when attempting a switch.
        let manager = create_test_account_manager().await;
        let ws_server = create_test_ws_server().await;
        let switcher = AccountSwitcher::new(manager, ws_server).unwrap();

        // Attempt a switch (will fail, but transaction should be saved if there was a previous account).
        let _ = switcher.switch_account("nonexistent", |_| {}).await;

        // Check that last_transaction is accessible (even if None).
        let last_tx = switcher.last_transaction.lock().await;
        // Transaction should be None since there was no previous account.
        assert!(last_tx.is_none());
    }

    #[tokio::test]
    async fn test_error_user_message() {
        // Test that user-friendly error messages are generated.
        use super::super::error::{AuthError, DataError, NetworkError};

        let timeout_error = SwitcherError::Network(NetworkError::Timeout("test".to_string()));
        let msg = AccountSwitcher::get_error_message(&timeout_error);
        assert!(msg.contains("timed out") || msg.contains("timeout"));

        let auth_error = SwitcherError::Auth(AuthError::TokenExpired);
        let msg = AccountSwitcher::get_error_message(&auth_error);
        assert!(msg.contains("Token") || msg.contains("expired"));

        let data_error = SwitcherError::Data(DataError::AccountNotFound("test".to_string()));
        let msg = AccountSwitcher::get_error_message(&data_error);
        assert!(msg.contains("account") || msg.contains("Account"));
    }

    #[tokio::test]
    async fn test_error_retryable_classification() {
        // Test that errors are correctly classified as retryable or not.
        use super::super::error::{AuthError, NetworkError};

        let timeout_error = SwitcherError::Network(NetworkError::Timeout("test".to_string()));
        assert!(AccountSwitcher::is_error_retryable(&timeout_error));

        let connection_error = SwitcherError::Network(NetworkError::ConnectionFailed("test".to_string()));
        assert!(AccountSwitcher::is_error_retryable(&connection_error));

        let auth_error = SwitcherError::Auth(AuthError::TokenExpired);
        assert!(!AccountSwitcher::is_error_retryable(&auth_error));
    }
}
