#[cfg(test)]
mod tests {
    use super::super::{SecureStorageManager, Credential};

    fn create_test_credential() -> Credential {
        Credential {
            token: "test_token_12345".to_string(),
            refresh_token: Some("refresh_token_67890".to_string()),
            cookies: "session=abc123; path=/".to_string(),
            expires_at: Some("2024-12-31T23:59:59Z".to_string()),
        }
    }

    #[test]
    fn test_store_and_retrieve_credential() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "test_account_001";
        let credential = create_test_credential();

        // Store credential
        manager
            .store_credential(account_id, &credential)
            .expect("Failed to store credential");

        // Retrieve credential
        let retrieved = manager
            .get_credential(account_id)
            .expect("Failed to retrieve credential");

        // Verify
        assert_eq!(retrieved.token, credential.token);
        assert_eq!(retrieved.refresh_token, credential.refresh_token);
        assert_eq!(retrieved.cookies, credential.cookies);
        assert_eq!(retrieved.expires_at, credential.expires_at);

        // Cleanup
        let _ = manager.delete_credential(account_id);
    }

    #[test]
    fn test_update_credential() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "test_account_002";
        let credential1 = create_test_credential();

        // Store initial credential
        manager
            .store_credential(account_id, &credential1)
            .expect("Failed to store initial credential");

        // Update with new credential
        let credential2 = Credential {
            token: "new_token_99999".to_string(),
            refresh_token: None,
            cookies: "session=xyz789; path=/".to_string(),
            expires_at: None,
        };

        manager
            .store_credential(account_id, &credential2)
            .expect("Failed to update credential");

        // Retrieve and verify updated credential
        let retrieved = manager
            .get_credential(account_id)
            .expect("Failed to retrieve updated credential");

        assert_eq!(retrieved.token, credential2.token);
        assert_eq!(retrieved.refresh_token, credential2.refresh_token);
        assert_eq!(retrieved.cookies, credential2.cookies);

        // Cleanup
        let _ = manager.delete_credential(account_id);
    }

    #[test]
    fn test_delete_credential() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "test_account_003";
        let credential = create_test_credential();

        // Store credential
        manager
            .store_credential(account_id, &credential)
            .expect("Failed to store credential");

        // Delete credential
        manager
            .delete_credential(account_id)
            .expect("Failed to delete credential");

        // Verify it's deleted
        let result = manager.get_credential(account_id);
        assert!(result.is_err(), "Credential should not exist after deletion");
    }

    #[test]
    fn test_get_nonexistent_credential() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "nonexistent_account";

        let result = manager.get_credential(account_id);
        assert!(result.is_err(), "Should return error for nonexistent credential");
    }

    #[test]
    fn test_delete_nonexistent_credential() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "nonexistent_account";

        // Should not error when deleting nonexistent credential
        let result = manager.delete_credential(account_id);
        assert!(result.is_ok(), "Deleting nonexistent credential should succeed");
    }

    #[test]
    fn test_multiple_accounts() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        
        let accounts = vec![
            ("account_1", "token_1"),
            ("account_2", "token_2"),
            ("account_3", "token_3"),
        ];

        // Store multiple credentials
        for (account_id, token) in &accounts {
            let credential = Credential {
                token: token.to_string(),
                refresh_token: None,
                cookies: "test_cookies".to_string(),
                expires_at: None,
            };
            manager
                .store_credential(account_id, &credential)
                .expect("Failed to store credential");
        }

        // Verify each credential
        for (account_id, token) in &accounts {
            let retrieved = manager
                .get_credential(account_id)
                .expect("Failed to retrieve credential");
            assert_eq!(&retrieved.token, token);
        }

        // Cleanup
        for (account_id, _) in &accounts {
            let _ = manager.delete_credential(account_id);
        }
    }

    #[test]
    fn test_credential_with_special_characters() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "test_account_special";
        
        let credential = Credential {
            token: "token_with_!@#$%^&*()_+-={}[]|:;<>?,./".to_string(),
            refresh_token: Some("refresh_with_特殊字符_🎉".to_string()),
            cookies: "session=abc; path=/; secure; httponly".to_string(),
            expires_at: Some("2024-12-31T23:59:59.999Z".to_string()),
        };

        // Store credential
        manager
            .store_credential(account_id, &credential)
            .expect("Failed to store credential with special characters");

        // Retrieve and verify
        let retrieved = manager
            .get_credential(account_id)
            .expect("Failed to retrieve credential with special characters");

        assert_eq!(retrieved.token, credential.token);
        assert_eq!(retrieved.refresh_token, credential.refresh_token);

        // Cleanup
        let _ = manager.delete_credential(account_id);
    }

    #[test]
    fn test_empty_credential_fields() {
        let manager = SecureStorageManager::new().expect("Failed to create manager");
        let account_id = "test_account_empty";
        
        let credential = Credential {
            token: "".to_string(),
            refresh_token: None,
            cookies: "".to_string(),
            expires_at: None,
        };

        // Store credential with empty fields
        manager
            .store_credential(account_id, &credential)
            .expect("Failed to store credential with empty fields");

        // Retrieve and verify
        let retrieved = manager
            .get_credential(account_id)
            .expect("Failed to retrieve credential with empty fields");

        assert_eq!(retrieved.token, "");
        assert_eq!(retrieved.cookies, "");

        // Cleanup
        let _ = manager.delete_credential(account_id);
    }
}
