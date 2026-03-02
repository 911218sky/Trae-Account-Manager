// Account Switcher data types

use serde::{Deserialize, Serialize};

/// Progress information during account switch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchProgress {
    pub step: u8,
    pub total: u8,
    pub message: String,
    pub elapsed_ms: u64,
}

/// Result of an account switch operation
#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchResult {
    pub success: bool,
    pub account_id: String,
    pub duration_ms: u64,
    pub notified_instances: usize,
}

/// Current switch state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchState {
    pub current_account_id: Option<String>,
    pub previous_account_id: Option<String>,
    pub switch_in_progress: bool,
    pub last_switch_time: Option<i64>,
    pub last_switch_duration_ms: Option<u64>,
}

impl Default for SwitchState {
    fn default() -> Self {
        Self {
            current_account_id: None,
            previous_account_id: None,
            switch_in_progress: false,
            last_switch_time: None,
            last_switch_duration_ms: None,
        }
    }
}

/// Transaction data for rollback support
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SwitchTransaction {
    pub previous_account_id: String,
    pub previous_token: String,
    pub previous_cookies: String,
    pub previous_token_expired_at: Option<String>,
    pub timestamp: i64,
}

impl SwitchTransaction {
    /// Create a new transaction from account data
    pub fn new(
        account_id: String,
        token: String,
        cookies: String,
        token_expired_at: Option<String>,
    ) -> Self {
        Self {
            previous_account_id: account_id,
            previous_token: token,
            previous_cookies: cookies,
            previous_token_expired_at: token_expired_at,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}
