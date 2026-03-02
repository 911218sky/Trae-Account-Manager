// Secure Storage Manager implementation

use anyhow::Result;
use super::types::Credential;

#[cfg(target_os = "windows")]
use super::windows::windows_impl;

#[cfg(target_os = "macos")]
use super::macos::macos_impl;

#[cfg(target_os = "linux")]
use super::linux::linux_impl;

/// Cross-platform secure storage manager
pub struct SecureStorageManager;

impl SecureStorageManager {
    /// Create a new secure storage manager
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Store a credential
    pub fn store_credential(&self, account_id: &str, credential: &Credential) -> Result<()> {
        #[cfg(target_os = "windows")]
        return windows_impl::store_credential(account_id, credential);

        #[cfg(target_os = "macos")]
        return macos_impl::store_credential(account_id, credential);

        #[cfg(target_os = "linux")]
        return linux_impl::store_credential(account_id, credential);

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(anyhow::anyhow!("Unsupported platform"))
    }

    /// Get a credential
    pub fn get_credential(&self, account_id: &str) -> Result<Credential> {
        #[cfg(target_os = "windows")]
        return windows_impl::get_credential(account_id);

        #[cfg(target_os = "macos")]
        return macos_impl::get_credential(account_id);

        #[cfg(target_os = "linux")]
        return linux_impl::get_credential(account_id);

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(anyhow::anyhow!("Unsupported platform"))
    }

    /// Delete a credential
    pub fn delete_credential(&self, account_id: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        return windows_impl::delete_credential(account_id);

        #[cfg(target_os = "macos")]
        return macos_impl::delete_credential(account_id);

        #[cfg(target_os = "linux")]
        return linux_impl::delete_credential(account_id);

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(anyhow::anyhow!("Unsupported platform"))
    }

    /// List all stored credentials
    pub fn list_credentials(&self) -> Result<Vec<String>> {
        #[cfg(target_os = "windows")]
        return windows_impl::list_credentials();

        #[cfg(target_os = "macos")]
        return macos_impl::list_credentials();

        #[cfg(target_os = "linux")]
        return linux_impl::list_credentials();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        Err(anyhow::anyhow!("Unsupported platform"))
    }
}

impl Default for SecureStorageManager {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
#[path = "manager.test.rs"]
mod tests;
