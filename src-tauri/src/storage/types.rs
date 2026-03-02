// Storage data types

use serde::{Deserialize, Serialize};

/// Account credential information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub token: String,
    pub refresh_token: Option<String>,
    pub cookies: String,
    pub expires_at: Option<String>,
}
