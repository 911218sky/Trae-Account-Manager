// macOS Keychain implementation

#[cfg(target_os = "macos")]
pub mod macos_impl {
    use anyhow::{Context, Result};
    use super::super::types::Credential;
    use security_framework::passwords::{
        delete_generic_password, get_generic_password, set_generic_password,
    };

    const SERVICE_NAME: &str = "TraeAuto";

    /// Store credential using macOS Keychain
    pub fn store_credential(account_id: &str, credential: &Credential) -> Result<()> {
        // Serialize credential to JSON
        let credential_json = serde_json::to_string(credential)
            .context("Failed to serialize credential")?;
        let credential_bytes = credential_json.as_bytes();

        // Delete existing credential if it exists (to update)
        let _ = delete_generic_password(SERVICE_NAME, account_id);

        // Store new credential
        set_generic_password(SERVICE_NAME, account_id, credential_bytes)
            .context("Failed to store credential in Keychain")?;

        Ok(())
    }

    /// Get credential from macOS Keychain
    pub fn get_credential(account_id: &str) -> Result<Credential> {
        let credential_bytes = get_generic_password(SERVICE_NAME, account_id)
            .context("Failed to retrieve credential from Keychain")?;

        let credential_json = String::from_utf8(credential_bytes)
            .context("Failed to decode credential data")?;

        let credential: Credential = serde_json::from_str(&credential_json)
            .context("Failed to deserialize credential")?;

        Ok(credential)
    }

    /// Delete credential from macOS Keychain
    pub fn delete_credential(account_id: &str) -> Result<()> {
        delete_generic_password(SERVICE_NAME, account_id)
            .context("Failed to delete credential from Keychain")?;

        Ok(())
    }

    /// List all credentials from macOS Keychain
    pub fn list_credentials() -> Result<Vec<String>> {
        // Note: security-framework doesn't provide a direct way to enumerate all items
        // This is a limitation of the macOS Keychain API through security-framework
        // In practice, we'll maintain a separate list of account IDs in the account manager
        // For now, return an empty list as this is primarily used for migration
        Ok(vec![])
    }
}
