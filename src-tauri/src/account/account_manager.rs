use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

use super::types::*;
use crate::api::{TraeApiClient, UsageSummary, UsageQueryResponse};
use crate::storage::{SecureStorageManager, Credential};

/// Manages user accounts and authentication state.
pub struct AccountManager {
    store: AccountStore,
    data_path: PathBuf,
    secure_storage: SecureStorageManager,
}

impl AccountManager {
    /// Creates a new account manager instance.
    pub fn new() -> Result<Self> {
        let data_path = Self::get_data_path()?;
        let store = Self::load_store(&data_path)?;
        let secure_storage = SecureStorageManager::new()?;

        Ok(Self { 
            store, 
            data_path,
            secure_storage,
        })
    }

    /// Returns the data storage path for account data.
    fn get_data_path() -> Result<PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("com", "sauce", "trae-auto")
            .ok_or_else(|| anyhow!("Failed to get application data directory"))?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;

        Ok(data_dir.join("accounts.json"))
    }

    /// Loads the account store from the specified path.
    fn load_store(path: &PathBuf) -> Result<AccountStore> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let store: AccountStore = serde_json::from_str(&content)?;
            Ok(store)
        } else {
            Ok(AccountStore::default())
        }
    }

    /// Saves the account store to disk.
    fn save_store(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.store)?;
        fs::write(&self.data_path, content)?;
        Ok(())
    }

    /// Adds a new account using cookies.
    pub async fn add_account(&mut self, cookies: String) -> Result<Account> {
        let mut client = TraeApiClient::new(&cookies)?;

        let token_result = client.get_user_token().await?;
        let user_info = client.get_user_info().await?;

        if self
            .store
            .accounts
            .iter()
            .any(|a| a.user_id == token_result.user_id)
        {
            return Err(anyhow!("Account already exists"));
        }

        let mut account = Account::new(
            user_info.screen_name.clone(),
            user_info.non_plain_text_email.unwrap_or_default(),
            cookies,
            token_result.user_id,
            token_result.tenant_id,
        );

        account.avatar_url = user_info.avatar_url;
        account.region = user_info.region;
        account.jwt_token = Some(token_result.token);
        account.token_expired_at = Some(token_result.expired_at);

        self.store.accounts.push(account.clone());

        if self.store.active_account_id.is_none() {
            self.store.active_account_id = Some(account.id.clone());
        }

        self.save_store()?;
        Ok(account)
    }

    /// Adds a new account using a token and optional cookies.
    pub async fn add_account_by_token(&mut self, token: String, cookies: Option<String>) -> Result<Account> {
        let client = TraeApiClient::new_with_token(&token)?;
        let user_info = client.get_user_info_by_token().await?;

        if self
            .store
            .accounts
            .iter()
            .any(|a| a.user_id == user_info.user_id)
        {
            return Err(anyhow!("Account already exists"));
        }

        let (name, email, avatar_url) = if let Some(ref cookies_str) = cookies {
            match self.get_user_info_with_cookies(cookies_str).await {
                Ok(info) => (
                    info.screen_name,
                    info.non_plain_text_email.unwrap_or_default(),
                    info.avatar_url,
                ),
                Err(_) => (
                    user_info.screen_name.unwrap_or_else(|| format!("User_{}", &user_info.user_id[..8.min(user_info.user_id.len())])),
                    user_info.email.unwrap_or_default(),
                    user_info.avatar_url.unwrap_or_default(),
                ),
            }
        } else {
            (
                user_info.screen_name.unwrap_or_else(|| format!("User_{}", &user_info.user_id[..8.min(user_info.user_id.len())])),
                user_info.email.unwrap_or_default(),
                user_info.avatar_url.unwrap_or_default(),
            )
        };

        let mut account = Account::new(
            name,
            email,
            cookies.unwrap_or_default(),
            user_info.user_id.clone(),
            user_info.tenant_id.clone(),
        );

        account.avatar_url = avatar_url;
        account.jwt_token = Some(token);
        account.token_expired_at = None;

        self.store.accounts.push(account.clone());

        if self.store.active_account_id.is_none() {
            self.store.active_account_id = Some(account.id.clone());
        }

        self.save_store()?;
        Ok(account)
    }

    /// Retrieves user information using cookies.
    async fn get_user_info_with_cookies(&self, cookies: &str) -> Result<crate::api::UserInfoResult> {
        let client = TraeApiClient::new(cookies)?;
        client.get_user_info().await
    }

    /// Removes an account by ID.
    pub fn remove_account(&mut self, account_id: &str) -> Result<()> {
        let index = self
            .store
            .accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("Account not found"))?;

        self.store.accounts.remove(index);

        if self.store.active_account_id.as_deref() == Some(account_id) {
            self.store.active_account_id = self.store.accounts.first().map(|a| a.id.clone());
        }

        self.save_store()?;
        Ok(())
    }

    /// Sets the active account by ID.
    pub fn set_active_account(&mut self, account_id: &str) -> Result<()> {
        if !self.store.accounts.iter().any(|a| a.id == account_id) {
            return Err(anyhow!("Account not found"));
        }

        self.store.active_account_id = Some(account_id.to_string());
        self.save_store()?;
        Ok(())
    }

    /// Switches to the specified account and updates Trae IDE login information.
    pub fn switch_account(&mut self, account_id: &str) -> Result<()> {
        if self.store.current_account_id.as_deref() == Some(account_id) {
            return Err(anyhow!("Account is already active"));
        }

        let account = self.store.accounts.iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("Account not found"))?
            .clone();

        let token = account.jwt_token.as_ref()
            .ok_or_else(|| anyhow!("Account has no valid token"))?;

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

        crate::machine::switch_trae_account(&login_info, account.machine_id.as_deref())?;

        if let Some(machine_id) = &account.machine_id {
            match crate::machine::set_machine_guid(machine_id) {
                Ok(_) => log::info!("System machine ID switched: {}", machine_id),
                Err(e) => log::warn!("Failed to switch system machine ID (may require admin privileges): {}", e),
            }
        }

        for acc in &mut self.store.accounts {
            acc.is_active = false;
        }

        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
            acc.is_active = true;
        }

        self.store.active_account_id = Some(account_id.to_string());
        self.store.current_account_id = Some(account_id.to_string());
        self.save_store()?;

        println!("[INFO] Switched to account: {}", account.email);
        Ok(())
    }

    /// Binds the current system machine ID to the account.
    pub fn bind_machine_id(&mut self, account_id: &str) -> Result<String> {
        let current_machine_id = crate::machine::get_machine_guid()?;

        let account = self.store.accounts.iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("Account not found"))?;

        account.machine_id = Some(current_machine_id.clone());
        account.updated_at = chrono::Utc::now().timestamp();
        let email = account.email.clone();

        self.save_store()?;
        println!("[INFO] Machine ID {} bound to account {}", current_machine_id, email);

        Ok(current_machine_id)
    }

    /// Returns a list of all accounts.
    pub fn get_accounts(&self) -> Vec<AccountBrief> {
        let current_id = self.store.current_account_id.as_deref();
        log::debug!("get_accounts: current_account_id = {:?}", current_id);
        self.store.accounts.iter().map(|account| {
            let is_current = current_id == Some(account.id.as_str());
            log::debug!("Account {} ({}): is_active={}, is_current={}", 
                account.email, account.id, account.is_active, is_current);
            AccountBrief::from_account(account, is_current)
        }).collect()
    }

    /// Returns the active account or None if no account is selected.
    pub fn get_active_account(&self) -> Option<&Account> {
        self.store
            .active_account_id
            .as_ref()
            .and_then(|id| self.store.accounts.iter().find(|a| &a.id == id))
    }

    /// Returns the account with the specified ID.
    pub fn get_account(&self, account_id: &str) -> Result<Account> {
        self.store
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| anyhow!("Account not found"))
    }

    /// Returns a reference to the secure storage manager.
    pub fn get_secure_storage(&self) -> &SecureStorageManager {
        &self.secure_storage
    }

    /// Stores account credentials in secure storage.
    pub fn store_account_credential(&self, account_id: &str) -> Result<()> {
        let account = self.get_account(account_id)?;
        
        let credential = Credential {
            token: account.jwt_token.unwrap_or_default(),
            refresh_token: None,
            cookies: account.cookies,
            expires_at: account.token_expired_at,
        };
        
        self.secure_storage.store_credential(account_id, &credential)?;
        Ok(())
    }

    /// Retrieves account credentials from secure storage.
    pub fn get_account_credential(&self, account_id: &str) -> Result<Credential> {
        self.secure_storage.get_credential(account_id)
    }

    /// Deletes account credentials from secure storage.
    pub fn delete_account_credential(&self, account_id: &str) -> Result<()> {
        self.secure_storage.delete_credential(account_id)
    }

    /// Returns the usage summary for the specified account.
    pub async fn get_account_usage(&mut self, account_id: &str) -> Result<UsageSummary> {
        let account = self
            .store
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("账号不存在"))?
            .clone();

        let summary = if let Some(token) = &account.jwt_token {
            let client = TraeApiClient::new_with_token(token)?;
            match client.get_usage_summary_by_token().await {
                Ok(summary) => summary,
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") && !account.cookies.is_empty() {
                        log::info!("Token expired, attempting to refresh using cookies...");
                        let mut cookie_client = TraeApiClient::new(&account.cookies)?;
                        let token_result = cookie_client.get_user_token().await?;

                        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
                            acc.jwt_token = Some(token_result.token.clone());
                            acc.token_expired_at = Some(token_result.expired_at.clone());
                        }
                        self.save_store()?;

                        let new_client = TraeApiClient::new_with_token(&token_result.token)?;
                        new_client.get_usage_summary_by_token().await?
                    } else if error_msg.contains("401") {
                        return Err(anyhow!("Token expired, please update token or cookies"));
                    } else {
                        return Err(e);
                    }
                }
            }
        } else if !account.cookies.is_empty() {
            let mut client = TraeApiClient::new(&account.cookies)?;
            client.get_usage_summary().await?
        } else {
            return Err(anyhow!("Account has no valid token or cookies"));
        };

        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
            acc.plan_type = summary.plan_type.clone();
            acc.updated_at = chrono::Utc::now().timestamp();
        }
        self.save_store()?;

        Ok(summary)
    }

    /// Refreshes the token for the specified account.
    pub async fn refresh_token(&mut self, account_id: &str) -> Result<()> {
        let account = self
            .store
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("账号不存在"))?
            .clone();

        let mut client = TraeApiClient::new(&account.cookies)?;
        let token_result = client.get_user_token().await?;

        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
            acc.jwt_token = Some(token_result.token);
            acc.token_expired_at = Some(token_result.expired_at);
            acc.updated_at = chrono::Utc::now().timestamp();
        }

        self.save_store()?;
        Ok(())
    }

    /// Updates the token for the specified account and returns the usage summary.
    pub async fn update_account_token(&mut self, account_id: &str, token: String) -> Result<UsageSummary> {
        let client = TraeApiClient::new_with_token(&token)?;

        let user_info = client.get_user_info_by_token().await?;

        let acc = self.store.accounts.iter_mut()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("Account not found"))?;

        if acc.user_id != user_info.user_id {
            return Err(anyhow!("Token user does not match account"));
        }

        acc.jwt_token = Some(token.clone());
        acc.updated_at = chrono::Utc::now().timestamp();

        let summary = client.get_usage_summary_by_token().await?;
        acc.plan_type = summary.plan_type.clone();

        self.save_store()?;
        Ok(summary)
    }

    /// Updates the cookies for the specified account.
    pub async fn update_cookies(&mut self, account_id: &str, cookies: String) -> Result<()> {
        let mut client = TraeApiClient::new(&cookies)?;
        let token_result = client.get_user_token().await?;

        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
            if acc.user_id != token_result.user_id {
                return Err(anyhow!("Cookies user does not match account"));
            }

            acc.cookies = cookies;
            acc.jwt_token = Some(token_result.token);
            acc.token_expired_at = Some(token_result.expired_at);
            acc.updated_at = chrono::Utc::now().timestamp();
        } else {
            return Err(anyhow!("Account not found"));
        }

        self.save_store()?;
        Ok(())
    }

    /// Exports all accounts as JSON.
    pub fn export_accounts(&self) -> Result<String> {
        let export_data: Vec<serde_json::Value> = self.store.accounts.iter().map(|acc| {
            serde_json::json!({
                "name": acc.name,
                "email": acc.email,
                "cookies": acc.cookies,
                "user_id": acc.user_id,
                "tenant_id": acc.tenant_id,
                "region": acc.region,
                "plan_type": acc.plan_type,
                "avatar_url": acc.avatar_url,
                "jwt_token": acc.jwt_token,
                "machine_id": acc.machine_id,
            })
        }).collect();

        serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow!("Export failed: {}", e))
    }

    /// Imports accounts from JSON data.
    pub async fn import_accounts(&mut self, data: &str) -> Result<usize> {
        let import_data: Vec<serde_json::Value> = serde_json::from_str(data)
            .map_err(|e| anyhow!("JSON parse failed: {}", e))?;

        let total = import_data.len();
        log::info!("Starting import of {} accounts", total);

        let cookies_list: Vec<String> = import_data
            .iter()
            .filter_map(|item| {
                item.get("cookies")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
            })
            .collect();

        if cookies_list.is_empty() {
            log::info!("No valid cookies found for import");
            return Ok(0);
        }

        log::info!("Found {} valid cookies, starting parallel processing...", cookies_list.len());

        let batch_size = 5;
        let mut all_results = Vec::new();
        
        for (batch_idx, chunk) in cookies_list.chunks(batch_size).enumerate() {
            let batch_start = batch_idx * batch_size;
            log::info!("Processing batch {}/{} ({} accounts)...", 
                batch_idx + 1, 
                (cookies_list.len() + batch_size - 1) / batch_size,
                chunk.len()
            );

            let tasks: Vec<_> = chunk
                .iter()
                .enumerate()
                .map(|(idx, cookies)| {
                    let cookies = cookies.clone();
                    let account_idx = batch_start + idx + 1;
                    async move {
                        let mut client = match TraeApiClient::new(&cookies) {
                            Ok(c) => c,
                            Err(e) => {
                                log::warn!("Progress: {}/{} - Failed to create API client: {}", 
                                    account_idx, total, e);
                                return None;
                            }
                        };

                        let token_result = match client.get_user_token().await {
                            Ok(t) => t,
                            Err(e) => {
                                log::warn!("Progress: {}/{} - Failed to get token: {}", 
                                    account_idx, total, e);
                                return None;
                            }
                        };

                        let user_info = match client.get_user_info().await {
                            Ok(info) => info,
                            Err(e) => {
                                log::warn!("Progress: {}/{} - Failed to get user info: {}", 
                                    account_idx, total, e);
                                return None;
                            }
                        };

                        log::info!("Progress: {}/{} - Successfully retrieved account info", account_idx, total);
                        Some((cookies, token_result, user_info))
                    }
                })
                .collect();

            let batch_results = futures::future::join_all(tasks).await;
            all_results.extend(batch_results.into_iter().flatten());
        }

        log::info!("Parallel processing complete, successfully retrieved {} account info", all_results.len());
        log::info!("Starting save to storage...");

        let mut imported_count = 0;
        for (idx, (cookies, token_result, user_info)) in all_results.iter().enumerate() {
            let email = user_info.non_plain_text_email.clone()
                .unwrap_or_else(|| format!("{}@unknown", user_info.user_id));
            if self.store.accounts.iter().any(|a| a.email == email) {
                log::info!("Save progress: {}/{} - Account {} already exists, skipping", 
                    idx + 1, all_results.len(), email);
                continue;
            }

            let account = Account {
                id: uuid::Uuid::new_v4().to_string(),
                name: user_info.screen_name.clone(),
                email: email.clone(),
                avatar_url: user_info.avatar_url.clone(),
                user_id: token_result.user_id.clone(),
                tenant_id: token_result.tenant_id.clone(),
                region: user_info.region.clone(),
                cookies: cookies.clone(),
                jwt_token: Some(token_result.token.clone()),
                token_expired_at: Some(token_result.expired_at.clone()),
                plan_type: String::new(),
                is_active: false,
                machine_id: None,
                created_at: chrono::Utc::now().timestamp(),
                updated_at: chrono::Utc::now().timestamp(),
            };

            self.store.accounts.push(account);
            imported_count += 1;
            log::info!("Save progress: {}/{} - Successfully saved account {}", 
                idx + 1, all_results.len(), email);
        }

        self.save_store()?;
        log::info!("Import complete: {} successful, {} total", imported_count, total);
        
        Ok(imported_count)
    }

    /// Retrieves usage events for the specified account within a time range.
    pub async fn get_usage_events(
        &mut self,
        account_id: &str,
        start_time: i64,
        end_time: i64,
        page_num: i32,
        page_size: i32,
    ) -> Result<UsageQueryResponse> {
        let account = self
            .store
            .accounts
            .iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("账号不存在"))?
            .clone();

        if let Some(token) = &account.jwt_token {
            let client = TraeApiClient::new_with_token(token)?;
            match client.query_usage(start_time, end_time, page_size, page_num).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") && !account.cookies.is_empty() {
                        println!("[INFO] Token expired, attempting to refresh using cookies...");
                        let mut cookie_client = TraeApiClient::new(&account.cookies)?;
                        let token_result = cookie_client.get_user_token().await?;

                        if let Some(acc) = self.store.accounts.iter_mut().find(|a| a.id == account_id) {
                            acc.jwt_token = Some(token_result.token.clone());
                            acc.token_expired_at = Some(token_result.expired_at.clone());
                        }
                        self.save_store()?;

                        let new_client = TraeApiClient::new_with_token(&token_result.token)?;
                        new_client.query_usage(start_time, end_time, page_size, page_num).await
                    } else if error_msg.contains("401") {
                        Err(anyhow!("Token expired, please update token or cookies"))
                    } else {
                        Err(e)
                    }
                }
            }
        } else if !account.cookies.is_empty() {
            let mut client = TraeApiClient::new(&account.cookies)?;
            client.get_user_token().await?;
            client.query_usage(start_time, end_time, page_size, page_num).await
        } else {
            Err(anyhow!("Account has no valid token or cookies"))
        }
    }

    /// Reads the currently logged-in account from Trae IDE configuration.
    pub async fn read_trae_ide_account(&mut self) -> Result<Option<Account>> {
        #[cfg(target_os = "windows")]
        let trae_data_path = {
            let appdata = std::env::var("APPDATA")
                .map_err(|_| anyhow!("Failed to get APPDATA environment variable"))?;
            PathBuf::from(appdata).join("Trae")
        };
        
        #[cfg(target_os = "macos")]
        let trae_data_path = {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow!("Failed to get HOME environment variable"))?;
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Trae")
        };
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let trae_data_path: PathBuf = {
            return Err(anyhow!("This feature is only supported on Windows and macOS"));
        };

        let storage_path = trae_data_path
            .join("User")
            .join("globalStorage")
            .join("storage.json");

        if !storage_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("Failed to read Trae IDE config file: {}", e))?;

        let storage: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse Trae IDE config file: {}", e))?;

        let auth_info_str = storage
            .get("iCubeAuthInfo://icube.cloudide")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Trae IDE login info not found"))?;

        let auth_info: serde_json::Value = serde_json::from_str(auth_info_str)
            .map_err(|e| anyhow!("Failed to parse Trae IDE auth info: {}", e))?;

        let token = auth_info
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Token not found"))?
            .to_string();

        let user_id = auth_info
            .get("userId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("User ID not found"))?
            .to_string();

        let email = auth_info
            .get("account")
            .and_then(|acc| acc.get("email"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let avatar_url = auth_info
            .get("account")
            .and_then(|acc| acc.get("avatar_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let username = auth_info
            .get("account")
            .and_then(|acc| acc.get("username"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if self.store.accounts.iter().any(|a| a.user_id == user_id) {
            log::info!("Trae IDE account already exists in account manager");
            return Ok(None);
        }

        let client = TraeApiClient::new_with_token(&token)?;
        let user_info = client.get_user_info_by_token().await?;

        let mut account = Account::new(
            if username.is_empty() {
                user_info.screen_name.unwrap_or_else(|| format!("User_{}", &user_id[..8.min(user_id.len())]))
            } else {
                username
            },
            if email.is_empty() {
                user_info.email.unwrap_or_default()
            } else {
                email
            },
            String::new(),
            user_id,
            user_info.tenant_id,
        );

        account.avatar_url = if avatar_url.is_empty() {
            user_info.avatar_url.unwrap_or_default()
        } else {
            avatar_url
        };
        account.jwt_token = Some(token);

        self.store.accounts.push(account.clone());

        if self.store.active_account_id.is_none() {
            self.store.active_account_id = Some(account.id.clone());
        }

        self.save_store()?;

        println!("[INFO] Successfully read and added account from Trae IDE: {}", account.email);
        Ok(Some(account))
    }

    /// Checks if the account token is expiring soon (< 1 hour) or already expired.
    fn is_token_expiring_soon(account: &Account) -> bool {
        match &account.token_expired_at {
            None => true, // 无过期时间信息，需要刷新
            Some(expired_at) => {
                match chrono::DateTime::parse_from_rfc3339(expired_at) {
                    Ok(expiry) => {
                        let now = chrono::Utc::now();
                        let one_hour = chrono::Duration::hours(1);
                        expiry.with_timezone(&chrono::Utc) < now + one_hour
                    }
                    Err(_) => {
                        // 尝试解析为时间戳（秒）
                        if let Ok(ts) = expired_at.parse::<i64>() {
                            let now = chrono::Utc::now().timestamp();
                            ts < now + 3600
                        } else {
                            true // 无法解析，需要刷新
                        }
                    }
                }
            }
        }
    }

    /// Refreshes all tokens that are expiring soon.
    pub async fn refresh_all_tokens(&mut self) -> Result<Vec<String>> {
        let mut refreshed = Vec::new();
        let account_ids: Vec<String> = self.store.accounts.iter()
            .filter(|a| !a.cookies.is_empty())
            .filter(|a| Self::is_token_expiring_soon(a))
            .map(|a| a.id.clone())
            .collect();

        for id in account_ids {
            match self.refresh_token(&id).await {
                Ok(_) => {
                    log::info!("Token auto-refresh successful: {}", id);
                    refreshed.push(id);
                }
                Err(e) => {
                    log::warn!("Token auto-refresh failed {}: {}", id, e);
                }
            }
        }
        Ok(refreshed)
    }

    /// Claims the birthday bonus for the specified account.
    pub async fn claim_birthday_bonus(&mut self, account_id: &str) -> Result<()> {
        let account = self.store.accounts.iter()
            .find(|a| a.id == account_id)
            .ok_or_else(|| anyhow!("Account not found"))?;

        let token = account.jwt_token.as_ref()
            .ok_or_else(|| anyhow!("Account has no token"))?;

        let client = TraeApiClient::new_with_token(token)?;

        let claimed = client.query_birthday_bonus().await?;
        if claimed {
            return Err(anyhow!("Account has already claimed the bonus"));
        }

        client.claim_birthday_bonus().await?;

        println!("[INFO] Successfully claimed birthday bonus: {}", account.email);
        Ok(())
    }

    /// Returns a paginated list of accounts.
    pub fn get_accounts_paginated(
        &self,
        params: PaginationParams,
    ) -> Result<PaginatedResult<AccountBrief>> {
        let all_accounts = self.get_accounts();
        let total = all_accounts.len();
        
        let start = (params.page.saturating_sub(1)) * params.page_size;
        let end = std::cmp::min(start + params.page_size, total);
        
        let data = if start < total {
            all_accounts[start..end].to_vec()
        } else {
            vec![]
        };
        
        Ok(PaginatedResult {
            data,
            total,
            page: params.page,
            page_size: params.page_size,
            has_more: end < total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_status_determination() {
        // Test Normal status (token expires in 2 hours)
        let future_time = chrono::Utc::now() + chrono::Duration::hours(2);
        let token_expired_at = Some(future_time.to_rfc3339());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Normal);

        // Test Expiring status (token expires in 30 minutes)
        let expiring_time = chrono::Utc::now() + chrono::Duration::minutes(30);
        let token_expired_at = Some(expiring_time.to_rfc3339());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Expiring);

        // Test Expired status (token expired 1 hour ago)
        let expired_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let token_expired_at = Some(expired_time.to_rfc3339());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Expired);

        // Test Unknown status (no expiration time)
        let status = AccountBrief::determine_token_status(&None);
        assert_eq!(status, TokenStatus::Unknown);

        // Test with timestamp format (Normal)
        let future_ts = (chrono::Utc::now() + chrono::Duration::hours(2)).timestamp();
        let token_expired_at = Some(future_ts.to_string());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Normal);

        // Test with timestamp format (Expiring)
        let expiring_ts = (chrono::Utc::now() + chrono::Duration::minutes(30)).timestamp();
        let token_expired_at = Some(expiring_ts.to_string());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Expiring);

        // Test with timestamp format (Expired)
        let expired_ts = (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp();
        let token_expired_at = Some(expired_ts.to_string());
        let status = AccountBrief::determine_token_status(&token_expired_at);
        assert_eq!(status, TokenStatus::Expired);
    }

    #[test]
    fn test_account_manager_has_secure_storage() {
        let manager = AccountManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let _storage = manager.get_secure_storage();
        // If we get here without panic, the secure storage is properly initialized
    }

    #[test]
    fn test_get_accounts_includes_token_status() {
        let manager = AccountManager::new();
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let accounts = manager.get_accounts();
        
        // All accounts should have a token_status field
        for account in accounts {
            // token_status should be one of the valid enum values
            match account.token_status {
                TokenStatus::Normal | TokenStatus::Expiring | TokenStatus::Expired | TokenStatus::Unknown => {
                    // Valid status
                }
            }
        }
    }
}
