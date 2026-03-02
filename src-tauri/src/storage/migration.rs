// Credential migration functionality
// Migrates plaintext tokens from accounts.json to platform-native secure storage

use anyhow::Result;
use crate::account::AccountManager;
use super::{SecureStorageManager, Credential};

/// Migrate credentials from AccountManager to SecureStorageManager
/// 
/// This function reads all accounts from the AccountManager, extracts their
/// tokens and credentials, and stores them in the platform-native secure storage.
/// 
/// # Arguments
/// * `account_manager` - Mutable reference to the AccountManager
/// * `secure_storage` - Reference to the SecureStorageManager
/// 
/// # Returns
/// * `Result<usize>` - Number of accounts successfully migrated
/// 
/// # Example
/// ```
/// let mut account_manager = AccountManager::new()?;
/// let secure_storage = SecureStorageManager::new()?;
/// let migrated_count = migrate_to_secure_storage(&mut account_manager, &secure_storage).await?;
/// println!("Migrated {} accounts", migrated_count);
/// ```
pub async fn migrate_to_secure_storage(
    account_manager: &mut AccountManager,
    secure_storage: &SecureStorageManager,
) -> Result<usize> {
    let accounts = account_manager.get_accounts();
    let mut migrated = 0;
    
    for account_brief in accounts {
        // Get the full account details
        if let Ok(full_account) = account_manager.get_account(&account_brief.id) {
            // Only migrate if the account has a token
            if let Some(token) = full_account.jwt_token {
                let credential = Credential {
                    token,
                    refresh_token: None,
                    cookies: full_account.cookies,
                    expires_at: full_account.token_expired_at,
                };
                
                // Store the credential in secure storage
                match secure_storage.store_credential(&account_brief.id, &credential) {
                    Ok(_) => {
                        migrated += 1;
                        log::info!("Migrated credentials for account: {}", account_brief.id);
                    }
                    Err(e) => {
                        log::error!("Failed to migrate credentials for account {}: {}", account_brief.id, e);
                        // Continue with other accounts even if one fails
                    }
                }
            }
        }
    }
    
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_migrate_with_existing_accounts() {
        // Test migration with existing accounts
        let mut account_manager = AccountManager::new().unwrap();
        let secure_storage = SecureStorageManager::new().unwrap();
        
        let result = migrate_to_secure_storage(&mut account_manager, &secure_storage).await;
        assert!(result.is_ok());
        
        // The result should be a valid count
        let migrated_count = result.unwrap();
        println!("Successfully migrated {} accounts", migrated_count);
    }
}
