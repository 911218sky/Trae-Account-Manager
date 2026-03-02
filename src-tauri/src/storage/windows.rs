// Windows Credential Manager implementation

#[cfg(target_os = "windows")]
pub mod windows_impl {
    use anyhow::{Context, Result};
    use super::super::types::Credential;
    use std::ptr;
    use winapi::um::wincred::{
        CredDeleteW, CredEnumerateW, CredFree, CredReadW, CredWriteW,
        CREDENTIALW, CRED_ENUMERATE_ALL_CREDENTIALS, CRED_TYPE_GENERIC,
        PCREDENTIALW, PCREDENTIALW as PCREDENTIAL_PTR,
    };
    use winapi::shared::winerror::ERROR_NOT_FOUND;

    const SERVICE_NAME: &str = "TraeAuto";

    /// Convert Rust string to wide string (UTF-16)
    fn to_wide_string(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Convert wide string to Rust string
    fn from_wide_string(wide: *const u16) -> String {
        if wide.is_null() {
            return String::new();
        }
        unsafe {
            let len = (0..).take_while(|&i| *wide.offset(i) != 0).count();
            let slice = std::slice::from_raw_parts(wide, len);
            String::from_utf16_lossy(slice)
        }
    }

    /// Build target name for credential
    fn build_target_name(account_id: &str) -> String {
        format!("{}:{}", SERVICE_NAME, account_id)
    }

    /// Store credential using Windows Credential Manager
    pub fn store_credential(account_id: &str, credential: &Credential) -> Result<()> {
        let target_name = build_target_name(account_id);
        let target_name_wide = to_wide_string(&target_name);
        
        // Serialize credential to JSON
        let credential_json = serde_json::to_string(credential)
            .context("Failed to serialize credential")?;
        let credential_bytes = credential_json.as_bytes();

        let mut cred = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target_name_wide.as_ptr() as *mut u16,
            Comment: ptr::null_mut(),
            LastWritten: unsafe { std::mem::zeroed() },
            CredentialBlobSize: credential_bytes.len() as u32,
            CredentialBlob: credential_bytes.as_ptr() as *mut u8,
            Persist: 2, // CRED_PERSIST_LOCAL_MACHINE
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: ptr::null_mut(),
        };

        unsafe {
            if CredWriteW(&mut cred, 0) == 0 {
                return Err(anyhow::anyhow!(
                    "Failed to write credential: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }

        Ok(())
    }

    /// Get credential from Windows Credential Manager
    pub fn get_credential(account_id: &str) -> Result<Credential> {
        let target_name = build_target_name(account_id);
        let target_name_wide = to_wide_string(&target_name);
        let mut cred_ptr: PCREDENTIALW = ptr::null_mut();

        unsafe {
            if CredReadW(
                target_name_wide.as_ptr(),
                CRED_TYPE_GENERIC,
                0,
                &mut cred_ptr,
            ) == 0
            {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
                    return Err(anyhow::anyhow!("Credential not found for account: {}", account_id));
                }
                return Err(anyhow::anyhow!("Failed to read credential: {}", error));
            }

            let cred = &*cred_ptr;
            let blob_slice = std::slice::from_raw_parts(
                cred.CredentialBlob,
                cred.CredentialBlobSize as usize,
            );
            let credential_json = String::from_utf8_lossy(blob_slice);
            let credential: Credential = serde_json::from_str(&credential_json)
                .context("Failed to deserialize credential")?;

            CredFree(cred_ptr as *mut _);

            Ok(credential)
        }
    }

    /// Delete credential from Windows Credential Manager
    pub fn delete_credential(account_id: &str) -> Result<()> {
        let target_name = build_target_name(account_id);
        let target_name_wide = to_wide_string(&target_name);

        unsafe {
            if CredDeleteW(target_name_wide.as_ptr(), CRED_TYPE_GENERIC, 0) == 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
                    return Ok(()); // Already deleted
                }
                return Err(anyhow::anyhow!("Failed to delete credential: {}", error));
            }
        }

        Ok(())
    }

    /// List all credentials from Windows Credential Manager
    pub fn list_credentials() -> Result<Vec<String>> {
        let mut cred_count: u32 = 0;
        let mut creds_ptr: *mut PCREDENTIAL_PTR = ptr::null_mut();
        let mut account_ids = Vec::new();

        unsafe {
            if CredEnumerateW(
                ptr::null(),
                CRED_ENUMERATE_ALL_CREDENTIALS,
                &mut cred_count,
                &mut creds_ptr,
            ) == 0
            {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(ERROR_NOT_FOUND as i32) {
                    return Ok(vec![]); // No credentials found
                }
                return Err(anyhow::anyhow!("Failed to enumerate credentials: {}", error));
            }

            let creds_slice = std::slice::from_raw_parts(creds_ptr, cred_count as usize);
            let prefix = format!("{}:", SERVICE_NAME);

            for &cred_ptr in creds_slice {
                let cred = &*cred_ptr;
                let target_name = from_wide_string(cred.TargetName);
                
                if target_name.starts_with(&prefix) {
                    let account_id = target_name.strip_prefix(&prefix).unwrap().to_string();
                    account_ids.push(account_id);
                }
            }

            CredFree(creds_ptr as *mut _);
        }

        Ok(account_ids)
    }
}
