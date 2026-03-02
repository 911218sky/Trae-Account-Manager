use serde::{Deserialize, Serialize};

/// Account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub cookies: String,
    pub jwt_token: Option<String>,
    pub token_expired_at: Option<String>,
    pub user_id: String,
    pub tenant_id: String,
    pub region: String,
    pub plan_type: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub is_active: bool,
    /// Machine ID associated with the account.
    #[serde(default)]
    pub machine_id: Option<String>,
}

impl Account {
    pub fn new(
        name: String,
        email: String,
        cookies: String,
        user_id: String,
        tenant_id: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid_simple(),
            name,
            email,
            avatar_url: String::new(),
            cookies,
            jwt_token: None,
            token_expired_at: None,
            user_id,
            tenant_id,
            region: String::new(),
            plan_type: "Free".to_string(),
            created_at: now,
            updated_at: now,
            is_active: true,
            machine_id: None,
        }
    }
}

/// Account storage structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountStore {
    pub accounts: Vec<Account>,
    pub active_account_id: Option<String>,
    /// Currently active account ID in Trae IDE.
    #[serde(default)]
    pub current_account_id: Option<String>,
}

/// 简单的 UUID 生成
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{:x}{:x}", duration.as_secs(), duration.subsec_nanos())
}

/// Token status enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenStatus {
    Normal,    // Token is valid and not expiring soon
    Expiring,  // Token will expire within 1 hour
    Expired,   // Token has expired
    Unknown,   // Token status cannot be determined
}

/// Brief account information for list display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBrief {
    pub id: String,
    pub name: String,
    pub email: String,
    pub avatar_url: String,
    pub plan_type: String,
    pub is_active: bool,
    pub created_at: i64,
    /// Machine ID associated with the account.
    pub machine_id: Option<String>,
    /// Whether this is the currently active account in Trae IDE.
    pub is_current: bool,
    /// Token expiration time.
    pub token_expired_at: Option<String>,
    /// Token status.
    pub token_status: TokenStatus,
}

impl From<&Account> for AccountBrief {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            name: account.name.clone(),
            email: account.email.clone(),
            avatar_url: account.avatar_url.clone(),
            plan_type: account.plan_type.clone(),
            is_active: account.is_active,
            created_at: account.created_at,
            machine_id: account.machine_id.clone(),
            is_current: false,
            token_expired_at: account.token_expired_at.clone(),
            token_status: Self::determine_token_status(&account.token_expired_at),
        }
    }
}

impl AccountBrief {
    /// Creates an AccountBrief from an Account with the is_current flag set.
    pub fn from_account(account: &Account, is_current: bool) -> Self {
        Self {
            id: account.id.clone(),
            name: account.name.clone(),
            email: account.email.clone(),
            avatar_url: account.avatar_url.clone(),
            plan_type: account.plan_type.clone(),
            is_active: account.is_active,
            created_at: account.created_at,
            machine_id: account.machine_id.clone(),
            is_current,
            token_expired_at: account.token_expired_at.clone(),
            token_status: Self::determine_token_status(&account.token_expired_at),
        }
    }

    /// Determines the token status based on expiration time.
    pub fn determine_token_status(token_expired_at: &Option<String>) -> TokenStatus {
        match token_expired_at {
            None => TokenStatus::Unknown,
            Some(expired_at) => {
                // 尝试解析为 RFC3339 格式
                if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expired_at) {
                    let now = chrono::Utc::now();
                    let expiry_utc = expiry.with_timezone(&chrono::Utc);
                    
                    if expiry_utc < now {
                        return TokenStatus::Expired;
                    }
                    
                    let one_hour = chrono::Duration::hours(1);
                    if expiry_utc < now + one_hour {
                        return TokenStatus::Expiring;
                    }
                    
                    return TokenStatus::Normal;
                }
                
                // 尝试解析为时间戳（秒）
                if let Ok(ts) = expired_at.parse::<i64>() {
                    let now = chrono::Utc::now().timestamp();
                    
                    if ts < now {
                        return TokenStatus::Expired;
                    }
                    
                    if ts < now + 3600 {
                        return TokenStatus::Expiring;
                    }
                    
                    return TokenStatus::Normal;
                }
                
                TokenStatus::Unknown
            }
        }
    }
}

/// Pagination query parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: usize,
    pub page_size: usize,
}

/// Pagination query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub has_more: bool,
}

/// Export format enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Csv,
    Json,
}

/// Account data for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountExportData {
    pub account_id: String,
    pub username: String,
    pub email: String,
    pub status: String,
    pub created_date: String,
    pub last_login: String,
}

impl From<&Account> for AccountExportData {
    fn from(account: &Account) -> Self {
        Self {
            account_id: account.id.clone(),
            username: account.name.clone(),
            email: account.email.clone(),
            status: Self::determine_status(account),
            created_date: Self::format_timestamp(account.created_at),
            last_login: Self::format_timestamp(account.updated_at),
        }
    }
}

impl AccountExportData {
    /// Determines the account status based on token expiration.
    fn determine_status(account: &Account) -> String {
        if let Some(expired_at) = &account.token_expired_at {
            if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(expired_at) {
                let now = chrono::Utc::now();
                if expiry.with_timezone(&chrono::Utc) < now {
                    return "expired".to_string();
                }
                if expiry.with_timezone(&chrono::Utc) < now + chrono::Duration::hours(1) {
                    return "expiring".to_string();
                }
            }
        }
        "normal".to_string()
    }
    
    /// Formats a timestamp as ISO 8601 format.
    fn format_timestamp(timestamp: i64) -> String {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string())
    }
}

/// Export response containing download information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub download_url: String,
    pub filename: String,
    pub record_count: usize,
}

/// Export progress information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub percentage: u32,
    pub estimated_time_remaining: u32, // seconds
}
