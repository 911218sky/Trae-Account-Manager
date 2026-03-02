// Linux Secret Service implementation

#[cfg(target_os = "linux")]
pub mod linux_impl {
    use anyhow::{Context, Result};
    use super::super::types::Credential;
    use secret_service::blocking::{Collection, SecretService};
    use secret_service::EncryptionType;
    use std::collections::HashMap;

    const SERVICE_NAME: &str = "TraeAuto";

    /// Get the default collection from Secret Service
    fn get_collection() -> Result<Collection<'static>> {
        let service = SecretService::connect(EncryptionType::Dh)
            .context("Failed to connect to Secret Service")?;
        
        let collection = service
            .get_default_collection()
            .context("Failed to get default collection")?;

        // Unlock the collection if it's locked
        if collection.is_locked().context("Failed to check lock status")? {
            collection.unlock().context("Failed to unlock collection")?;
        }

        Ok(collection)
    }

    /// Build attributes for the credential
    fn build_attributes(account_id: &str) -> HashMap<&str, &str> {
        let mut attributes = HashMap::new();
        attributes.insert("service", SERVICE_NAME);
        attributes.insert("account", account_id);
        attributes
    }

    /// Store credential using Linux Secret Service
    pub fn store_credential(account_id: &str, credential: &Credential) -> Result<()> {
        let collection = get_collection()?;

        // Serialize credential to JSON
        let credential_json = serde_json::to_string(credential)
            .context("Failed to serialize credential")?;
        let credential_bytes = credential_json.as_bytes();

        // Build label and attributes
        let label = format!("{} - {}", SERVICE_NAME, account_id);
        let attributes = build_attributes(account_id);

        // Delete existing item if it exists
        let search_items = collection
            .search_items(attributes.clone())
            .context("Failed to search for existing items")?;
        
        for item in search_items {
            item.delete().context("Failed to delete existing item")?;
        }

        // Create new item
        collection
            .create_item(
                &label,
                attributes,
                credential_bytes,
                true, // replace
                "text/plain",
            )
            .context("Failed to create item in Secret Service")?;

        Ok(())
    }

    /// Get credential from Linux Secret Service
    pub fn get_credential(account_id: &str) -> Result<Credential> {
        let collection = get_collection()?;
        let attributes = build_attributes(account_id);

        let search_items = collection
            .search_items(attributes)
            .context("Failed to search for credential")?;

        if search_items.is_empty() {
            return Err(anyhow::anyhow!("Credential not found for account: {}", account_id));
        }

        let item = &search_items[0];
        let secret = item.get_secret().context("Failed to get secret")?;
        let credential_json = String::from_utf8(secret)
            .context("Failed to decode credential data")?;

        let credential: Credential = serde_json::from_str(&credential_json)
            .context("Failed to deserialize credential")?;

        Ok(credential)
    }

    /// Delete credential from Linux Secret Service
    pub fn delete_credential(account_id: &str) -> Result<()> {
        let collection = get_collection()?;
        let attributes = build_attributes(account_id);

        let search_items = collection
            .search_items(attributes)
            .context("Failed to search for credential")?;

        for item in search_items {
            item.delete().context("Failed to delete item")?;
        }

        Ok(())
    }

    /// List all credentials from Linux Secret Service
    pub fn list_credentials() -> Result<Vec<String>> {
        let collection = get_collection()?;
        
        let mut service_attr = HashMap::new();
        service_attr.insert("service", SERVICE_NAME);

        let search_items = collection
            .search_items(service_attr)
            .context("Failed to search for credentials")?;

        let mut account_ids = Vec::new();
        for item in search_items {
            let attributes = item.get_attributes().context("Failed to get attributes")?;
            if let Some(account_id) = attributes.get("account") {
                account_ids.push(account_id.to_string());
            }
        }

        Ok(account_ids)
    }
}
