// Error types for account switcher

use thiserror::Error;

/// Network-related errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection timeout: {0}")]
    Timeout(String),
    
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(String),
    
    #[error("WebSocket connection disconnected")]
    WebSocketDisconnected,
}

/// Authentication-related errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token has expired")]
    TokenExpired,
    
    #[error("Token is invalid")]
    InvalidToken,
    
    #[error("Account has been disabled")]
    AccountDisabled,
}

/// Data-related errors
#[derive(Debug, Error)]
pub enum DataError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Data is corrupted: {0}")]
    CorruptedData(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),
}

/// System-related errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Cannot access secure storage: {0}")]
    SecureStorageAccessDenied(String),
    
    #[error("Hotkey registration failed: {0}")]
    HotkeyRegistrationFailed(String),
    
    #[error("System tray initialization failed: {0}")]
    TrayInitFailed(String),
}

/// Unified error type for account switcher
#[derive(Debug, Error)]
pub enum SwitcherError {
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Data error: {0}")]
    Data(#[from] DataError),
    
    #[error("System error: {0}")]
    System(#[from] SystemError),
    
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

impl SwitcherError {
    /// Returns a user-friendly error message.
    pub fn user_message(&self) -> String {
        match self {
            SwitcherError::Network(NetworkError::Timeout(_)) => {
                "Operation timed out. Please check your network connection and try again.".to_string()
            }
            SwitcherError::Network(NetworkError::ConnectionFailed(_)) => {
                "Failed to connect to server. Please check your network connection.".to_string()
            }
            SwitcherError::Network(NetworkError::WebSocketDisconnected) => {
                "Connection lost. Attempting to reconnect...".to_string()
            }
            SwitcherError::Auth(AuthError::TokenExpired) => {
                "Token has expired. Please update your credentials.".to_string()
            }
            SwitcherError::Auth(AuthError::InvalidToken) => {
                "Token is invalid. Please log in again.".to_string()
            }
            SwitcherError::Auth(AuthError::AccountDisabled) => {
                "Account has been disabled. Please contact the administrator.".to_string()
            }
            SwitcherError::Data(DataError::AccountNotFound(_)) => {
                "The specified account could not be found.".to_string()
            }
            SwitcherError::Data(DataError::CorruptedData(_)) => {
                "Data is corrupted. Please try reloading.".to_string()
            }
            SwitcherError::Data(DataError::InvalidConfig(_)) => {
                "Configuration file format is invalid.".to_string()
            }
            SwitcherError::System(SystemError::SecureStorageAccessDenied(_)) => {
                "Cannot access secure storage. Please check your permission settings.".to_string()
            }
            SwitcherError::System(SystemError::HotkeyRegistrationFailed(_)) => {
                "Hotkey registration failed. It may conflict with another application.".to_string()
            }
            SwitcherError::System(SystemError::TrayInitFailed(_)) => {
                "System tray initialization failed.".to_string()
            }
            SwitcherError::Other(e) => format!("An error occurred: {}", e),
        }
    }

    /// Checks if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            SwitcherError::Network(NetworkError::Timeout(_))
                | SwitcherError::Network(NetworkError::ConnectionFailed(_))
                | SwitcherError::Network(NetworkError::WebSocketDisconnected)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_timeout() {
        let error = NetworkError::Timeout("API call".to_string());
        assert_eq!(error.to_string(), "Connection timeout: API call");
    }

    #[test]
    fn test_auth_error_token_expired() {
        let error = AuthError::TokenExpired;
        assert_eq!(error.to_string(), "Token has expired");
    }

    #[test]
    fn test_data_error_account_not_found() {
        let error = DataError::AccountNotFound("acc_123".to_string());
        assert_eq!(error.to_string(), "Account not found: acc_123");
    }

    #[test]
    fn test_system_error_secure_storage() {
        let error = SystemError::SecureStorageAccessDenied("Permission denied".to_string());
        assert_eq!(error.to_string(), "Cannot access secure storage: Permission denied");
    }

    #[test]
    fn test_switcher_error_user_message() {
        let error = SwitcherError::Network(NetworkError::Timeout("test".to_string()));
        let msg = error.user_message();
        assert!(msg.contains("timed out") || msg.contains("timeout"));
    }

    #[test]
    fn test_error_is_retryable() {
        let timeout_error = SwitcherError::Network(NetworkError::Timeout("test".to_string()));
        assert!(timeout_error.is_retryable());

        let auth_error = SwitcherError::Auth(AuthError::TokenExpired);
        assert!(!auth_error.is_retryable());
    }

    #[test]
    fn test_error_conversion_from_network() {
        let network_error = NetworkError::ConnectionFailed("test".to_string());
        let switcher_error: SwitcherError = network_error.into();
        assert!(matches!(switcher_error, SwitcherError::Network(_)));
    }

    #[test]
    fn test_error_conversion_from_auth() {
        let auth_error = AuthError::InvalidToken;
        let switcher_error: SwitcherError = auth_error.into();
        assert!(matches!(switcher_error, SwitcherError::Auth(_)));
    }
}
