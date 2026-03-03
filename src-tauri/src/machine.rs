use anyhow::{anyhow, Result};
use uuid::Uuid;
use std::fs;
use std::path::PathBuf;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::Command;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// Path to the MachineGuid in the Windows registry.
#[cfg(target_os = "windows")]
const MACHINE_GUID_PATH: &str = r"SOFTWARE\Microsoft\Cryptography";
#[cfg(target_os = "windows")]
const MACHINE_GUID_KEY: &str = "MachineGuid";

/// Retrieves the current system's MachineGuid from the Windows registry.
#[cfg(target_os = "windows")]
pub fn get_machine_guid() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey(MACHINE_GUID_PATH)
        .map_err(|e| anyhow!("Failed to open registry: {}", e))?;

    let guid: String = key.get_value(MACHINE_GUID_KEY)
        .map_err(|e| anyhow!("Failed to read MachineGuid: {}", e))?;

    Ok(guid)
}

/// Sets the system's MachineGuid in the Windows registry (requires administrator privileges).
#[cfg(target_os = "windows")]
pub fn set_machine_guid(new_guid: &str) -> Result<()> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(MACHINE_GUID_PATH, KEY_SET_VALUE)
        .map_err(|e| anyhow!("Failed to open registry (requires administrator privileges): {}", e))?;

    key.set_value(MACHINE_GUID_KEY, &new_guid)
        .map_err(|e| anyhow!("Failed to set MachineGuid: {}", e))?;

    Ok(())
}

/// Generates a new random MachineGuid.
pub fn generate_machine_guid() -> String {
    Uuid::new_v4().to_string()
}

/// Resets the MachineGuid to a new random value.
#[cfg(target_os = "windows")]
pub fn reset_machine_guid() -> Result<String> {
    let new_guid = generate_machine_guid();
    set_machine_guid(&new_guid)?;
    Ok(new_guid)
}

/// Retrieves the Trae IDE data directory path.
#[cfg(target_os = "windows")]
fn get_trae_data_path() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| anyhow!("Failed to get APPDATA environment variable"))?;
    Ok(PathBuf::from(appdata).join("Trae"))
}

#[cfg(target_os = "macos")]
fn get_trae_data_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow!("Failed to get HOME environment variable"))?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Trae"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_trae_data_path() -> Result<PathBuf> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

/// Retrieves the machine ID from the Trae IDE.
pub fn get_trae_machine_id() -> Result<String> {
    let trae_path = get_trae_data_path()?;
    let machine_id_path = trae_path.join("machineid");

    if !machine_id_path.exists() {
        return Err(anyhow!("Trae IDE machine ID file does not exist"));
    }

    let content = fs::read_to_string(&machine_id_path)
        .map_err(|e| anyhow!("Failed to read Trae machine ID: {}", e))?;

    Ok(content.trim().to_string())
}

/// Sets the machine ID for the Trae IDE.
pub fn set_trae_machine_id(new_id: &str) -> Result<()> {
    let trae_path = get_trae_data_path()?;
    let machine_id_path = trae_path.join("machineid");

    fs::write(&machine_id_path, new_id)
        .map_err(|e| anyhow!("Failed to write Trae machine ID: {}", e))?;

    Ok(())
}

/// Checks if the Trae IDE is currently running.
#[cfg(target_os = "windows")]
pub fn is_trae_running() -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq Trae.exe", "/NH"])
        .output();

    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            result.contains("Trae.exe")
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
pub fn is_trae_running() -> bool {
    // Use pgrep to match processes with "Trae.app" in the path
    Command::new("pgrep")
        .args(["-f", "Trae.app/Contents/MacOS"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Terminates the Trae IDE process.
#[cfg(target_os = "windows")]
pub fn kill_trae() -> Result<()> {
    if !is_trae_running() {
        log::info!("Trae IDE is not running");
        return Ok(());
    }

    log::info!("Closing Trae IDE...");

    // Try graceful shutdown first
    let _ = Command::new("taskkill")
        .args(["/IM", "Trae.exe"])
        .output();

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Force kill if still running
    if is_trae_running() {
        let output = Command::new("taskkill")
            .args(["/F", "/IM", "Trae.exe"])
            .output()
            .map_err(|e| anyhow!("Failed to close Trae IDE: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            if !err.is_empty() {
                return Err(anyhow!("Failed to close Trae IDE: {}", err));
            }
        }
    }

    // Wait for process to fully exit
    std::thread::sleep(std::time::Duration::from_millis(1000));

    log::info!("Trae IDE closed");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn kill_trae() -> Result<()> {
    if !is_trae_running() {
        log::info!("Trae IDE is not running");
        return Ok(());
    }

    log::info!("Closing Trae IDE...");

    // Use osascript to gracefully close the Trae application
    let _ = Command::new("osascript")
        .args(["-e", "tell application \"Trae\" to quit"])
        .output();

    // Wait a moment
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Force kill if still running
    if is_trae_running() {
        log::info!("Graceful shutdown failed, force closing...");
        let _ = Command::new("pkill")
            .args(["-9", "-f", "Trae.app/Contents/MacOS"])
            .output();
        
        // Wait a moment
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    if is_trae_running() {
        return Err(anyhow!("Failed to close Trae IDE, please close it manually and try again"));
    }

    log::info!("Trae IDE closed");
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn is_trae_running() -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn kill_trae() -> Result<()> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

/// Retrieves the Trae IDE configuration file path.
fn get_trae_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("com", "sauce", "trae-auto")
        .ok_or_else(|| anyhow!("Failed to get application data directory"))?;
    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir)?;
    Ok(config_dir.join("trae_path.txt"))
}

/// Retrieves the saved Trae IDE path.
pub fn get_saved_trae_path() -> Result<String> {
    let config_path = get_trae_config_path()?;
    if config_path.exists() {
        let path = fs::read_to_string(&config_path)?;
        let path = path.trim().to_string();
        if !path.is_empty() && PathBuf::from(&path).exists() {
            return Ok(path);
        }
    }
    Err(anyhow!("Trae IDE path not configured"))
}

/// Saves the Trae IDE path.
#[cfg(target_os = "windows")]
pub fn save_trae_path(path: &str) -> Result<()> {
    let exe_path = PathBuf::from(path);
    if !exe_path.exists() {
        return Err(anyhow!("The specified path does not exist"));
    }
    if !path.to_lowercase().ends_with(".exe") {
        return Err(anyhow!("Please select the Trae.exe file"));
    }
    let config_path = get_trae_config_path()?;
    fs::write(&config_path, path)?;
    log::info!("Saved Trae IDE path: {}", path);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn save_trae_path(path: &str) -> Result<()> {
    let app_path = PathBuf::from(path);
    if !app_path.exists() {
        return Err(anyhow!("The specified path does not exist"));
    }
    // macOS applications are .app bundle directories
    if !path.to_lowercase().ends_with(".app") {
        return Err(anyhow!("Please select the Trae.app application"));
    }
    let config_path = get_trae_config_path()?;
    fs::write(&config_path, path)?;
    log::info!("Saved Trae IDE path: {}", path);
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn save_trae_path(_path: &str) -> Result<()> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

/// Automatically scans for the Trae IDE installation path.
#[cfg(target_os = "windows")]
pub fn scan_trae_path() -> Result<String> {
    // Common Windows application installation locations
    let possible_paths = vec![
        // User local installation path
        format!("{}\\AppData\\Local\\Programs\\Trae\\Trae.exe", 
            std::env::var("USERPROFILE").unwrap_or_default()),
        // Program Files
        "C:\\Program Files\\Trae\\Trae.exe".to_string(),
        "C:\\Program Files (x86)\\Trae\\Trae.exe".to_string(),
    ];
    
    for path in possible_paths {
        if PathBuf::from(&path).exists() {
            log::info!("Auto-detected Trae IDE: {}", path);
            return Ok(path);
        }
    }
    
    Err(anyhow!("Trae IDE installation path not found, please configure manually"))
}

#[cfg(target_os = "macos")]
pub fn scan_trae_path() -> Result<String> {
    // Common macOS application installation locations
    let possible_paths = [
        "/Applications/Trae.app",
        &format!("{}/Applications/Trae.app", std::env::var("HOME").unwrap_or_default()),
    ];
    
    for path in possible_paths {
        if PathBuf::from(path).exists() {
            return Ok(path.to_string());
        }
    }
    
    Err(anyhow!("Trae IDE not found, please configure the path manually"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn scan_trae_path() -> Result<String> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

/// Opens the Trae IDE.
#[cfg(target_os = "windows")]
pub fn open_trae() -> Result<()> {
    let trae_exe = match get_saved_trae_path() {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            // Try auto-scan
            match scan_trae_path() {
                Ok(path) => {
                    // Auto-save the scanned path
                    let _ = save_trae_path(&path);
                    PathBuf::from(path)
                },
                Err(_) => return Err(anyhow!("Trae IDE path not configured, please set it in settings")),
            }
        }
    };

    if !trae_exe.exists() {
        return Err(anyhow!("Trae IDE path is invalid, please reconfigure it in settings"));
    }

    println!("[INFO] Starting Trae IDE: {}", trae_exe.display());

    Command::new(&trae_exe)
        .spawn()
        .map_err(|e| anyhow!("Failed to start Trae IDE: {}", e))?;

    log::info!("Trae IDE started");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_trae() -> Result<()> {
    let trae_app = match get_saved_trae_path() {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            // Try auto-scan
            match scan_trae_path() {
                Ok(path) => PathBuf::from(path),
                Err(_) => return Err(anyhow!("Trae IDE path not configured, please set it in settings")),
            }
        }
    };

    if !trae_app.exists() {
        return Err(anyhow!("Trae IDE path is invalid, please reconfigure it in settings"));
    }

    println!("[INFO] Starting Trae IDE: {}", trae_app.display());

    Command::new("open")
        .arg("-a")
        .arg(&trae_app)
        .spawn()
        .map_err(|e| anyhow!("Failed to start Trae IDE: {}", e))?;

    log::info!("Trae IDE started");
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn open_trae() -> Result<()> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

/// Account login information structure for writing to Trae IDE.
#[derive(Debug, Clone)]
pub struct TraeLoginInfo {
    pub token: String,
    pub refresh_token: Option<String>,
    pub user_id: String,
    pub email: String,
    pub username: String,
    pub avatar_url: String,
    pub host: String,
    pub region: String,
}

/// Writes account login information to the Trae IDE.
pub fn write_trae_login_info(info: &TraeLoginInfo) -> Result<()> {
    let trae_path = get_trae_data_path()?;

    // Ensure the directory exists
    let storage_dir = trae_path.join("User").join("globalStorage");
    fs::create_dir_all(&storage_dir)
        .map_err(|e| anyhow!("Failed to create directory: {}", e))?;

    let storage_path = storage_dir.join("storage.json");

    // Read existing configuration or create new one
    let mut json: serde_json::Value = if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("Failed to read storage.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let obj = json.as_object_mut()
        .ok_or_else(|| anyhow!("storage.json format error"))?;

    // Calculate expiration time (14 days from now)
    let now = chrono::Utc::now();
    let expired_at = now + chrono::Duration::days(14);
    let refresh_expired_at = now + chrono::Duration::days(180);

    // Build host URL
    let host = if info.host.is_empty() {
        match info.region.to_uppercase().as_str() {
            "SG" => "https://api-sg-central.trae.ai",
            "CN" => "https://api.trae.com.cn",
            _ => "https://api-sg-central.trae.ai",
        }
    } else {
        &info.host
    };

    // Build iCubeAuthInfo
    let auth_info = serde_json::json!({
        "token": info.token,
        "refreshToken": info.refresh_token.clone().unwrap_or_default(),
        "expiredAt": expired_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "refreshExpiredAt": refresh_expired_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "tokenReleaseAt": now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        "userId": info.user_id,
        "host": host,
        "userRegion": {
            "region": info.region.to_uppercase(),
            "_aiRegion": info.region.to_uppercase()
        },
        "account": {
            "username": info.username,
            "iss": "",
            "iat": 0,
            "organization": "",
            "work_country": "",
            "email": info.email,
            "avatar_url": info.avatar_url,
            "description": "",
            "scope": "marscode",
            "loginScope": "trae",
            "storeCountryCode": "cn",
            "storeCountrySrc": "uid",
            "storeRegion": info.region.to_uppercase(),
            "userTag": "row"
        }
    });

    // Build iCubeEntitlementInfo
    let entitlement_info = serde_json::json!({
        "identityStr": "Free",
        "identity": 0,
        "isPayFreshman": false,
        "isSupportCommercialization": true,
        "hasPackage": false,
        "enableEntitlement": true,
        "detail": {
            "can_gen_solo_code": false,
            "fast_request_per": 1,
            "in_wait": false,
            "permission": 1,
            "toast_read": false,
            "toastRead": false,
            "canGenSoloCode": false,
            "fastRequestPer": 1,
            "inWaitlist": false
        }
    });

    // Write login information
    obj.insert(
        "iCubeAuthInfo://icube.cloudide".to_string(),
        serde_json::Value::String(serde_json::to_string(&auth_info).unwrap())
    );
    obj.insert(
        "iCubeEntitlementInfo://icube.cloudide".to_string(),
        serde_json::Value::String(serde_json::to_string(&entitlement_info).unwrap())
    );

    // Write back to file
    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))?;
    fs::write(&storage_path, new_content)
        .map_err(|e| anyhow!("Failed to write storage.json: {}", e))?;

    log::info!("Wrote Trae IDE login information: {}", info.email);
    Ok(())
}

/// Switches the Trae IDE to the specified account (closes the IDE, clears old login state, and writes new account information).
pub fn switch_trae_account(info: &TraeLoginInfo, machine_id: Option<&str>) -> Result<()> {
    // 0. Close Trae IDE first
    kill_trae()?;

    let trae_path = get_trae_data_path()?;

    // 1. Set machine ID (use provided value or generate new one)
    let new_machine_id = match machine_id {
        Some(mid) => mid.to_string(),
        None => generate_machine_guid(),
    };
    let machine_id_path = trae_path.join("machineid");
    fs::write(&machine_id_path, &new_machine_id)
        .map_err(|e| anyhow!("Failed to write Trae machine ID: {}", e))?;
    log::info!("Set Trae machine ID: {}", new_machine_id);

    // 2. Delete state.vscdb database (clears old login cache)
    let state_db_path = trae_path.join("User").join("globalStorage").join("state.vscdb");
    if state_db_path.exists() {
        let _ = fs::remove_file(&state_db_path);
        log::info!("Deleted state.vscdb");
    }

    // 3. Delete state.vscdb.backup
    let state_db_backup_path = trae_path.join("User").join("globalStorage").join("state.vscdb.backup");
    if state_db_backup_path.exists() {
        let _ = fs::remove_file(&state_db_backup_path);
    }

    // 4. Clear Local State
    let local_state_path = trae_path.join("Local State");
    if local_state_path.exists() {
        let _ = fs::remove_file(&local_state_path);
    }

    // 5. Clear IndexedDB
    let indexed_db_path = trae_path.join("IndexedDB");
    if indexed_db_path.exists() {
        let _ = fs::remove_dir_all(&indexed_db_path);
    }

    // 6. Clear Local Storage
    let local_storage_path = trae_path.join("Local Storage");
    if local_storage_path.exists() {
        let _ = fs::remove_dir_all(&local_storage_path);
    }

    // 7. Clear Session Storage
    let session_storage_path = trae_path.join("Session Storage");
    if session_storage_path.exists() {
        let _ = fs::remove_dir_all(&session_storage_path);
    }

    // 8. Clear Cookies
    let cookies_path = trae_path.join("Network").join("Cookies");
    if cookies_path.exists() {
        let _ = fs::remove_file(&cookies_path);
        log::info!("Cleared Cookies");
    }

    // 9. Clear Cookies-journal
    let cookies_journal_path = trae_path.join("Network").join("Cookies-journal");
    if cookies_journal_path.exists() {
        let _ = fs::remove_file(&cookies_journal_path);
    }

    // 10. Update telemetry ID in storage.json and write login information
    let storage_dir = trae_path.join("User").join("globalStorage");
    fs::create_dir_all(&storage_dir)
        .map_err(|e| anyhow!("Failed to create directory: {}", e))?;
    let storage_path = storage_dir.join("storage.json");

    // Read existing configuration or create new one
    let mut json: serde_json::Value = if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("Failed to read storage.json: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let obj = json.as_object_mut()
        .ok_or_else(|| anyhow!("storage.json format error"))?;

    // Remove old login information
    obj.remove("iCubeAuthInfo://icube.cloudide");
    obj.remove("iCubeEntitlementInfo://icube.cloudide");
    obj.remove("iCubeServerData://icube.cloudide");
    obj.remove("iCubeAuthInfo://usertag");

    // Update telemetry ID
    let new_telemetry_id = format!("{:x}", md5_hash(&new_machine_id));
    obj.insert("telemetry.machineId".to_string(), serde_json::Value::String(new_telemetry_id));
    obj.insert("telemetry.sqmId".to_string(), serde_json::Value::String(format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())));
    obj.insert("telemetry.devDeviceId".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));

    // Write back to file
    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))?;
    fs::write(&storage_path, new_content)
        .map_err(|e| anyhow!("Failed to write storage.json: {}", e))?;

    // 11. Write new login information
    write_trae_login_info(info)?;

    log::info!("Switched Trae IDE to account: {}", info.email);

    // 12. Auto-open Trae IDE
    if let Err(e) = open_trae() {
        log::warn!("Failed to auto-open Trae IDE: {}", e);
    }

    Ok(())
}

/// Clears the Trae IDE login state (resets the IDE to a fresh installation state).
pub fn clear_trae_login_state() -> Result<()> {
    let trae_path = get_trae_data_path()?;

    // 1. Generate new machine ID
    let new_machine_id = generate_machine_guid();
    let machine_id_path = trae_path.join("machineid");
    fs::write(&machine_id_path, &new_machine_id)
        .map_err(|e| anyhow!("Failed to reset Trae machine ID: {}", e))?;
    log::info!("Reset Trae machine ID: {}", new_machine_id);

    // 2. Clear login information from storage.json
    let storage_path = trae_path.join("User").join("globalStorage").join("storage.json");
    if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| anyhow!("Failed to read storage.json: {}", e))?;

        // Parse JSON and remove login-related fields
        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = json.as_object_mut() {
                // Remove login-related fields
                obj.remove("iCubeAuthInfo://icube.cloudide");
                obj.remove("iCubeEntitlementInfo://icube.cloudide");
                obj.remove("iCubeServerData://icube.cloudide");
                obj.remove("iCubeAuthInfo://usertag");

                // Reset telemetry ID
                let new_telemetry_id = format!("{:x}", md5_hash(&new_machine_id));
                obj.insert("telemetry.machineId".to_string(), serde_json::Value::String(new_telemetry_id));
                obj.insert("telemetry.sqmId".to_string(), serde_json::Value::String(format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())));
                obj.insert("telemetry.devDeviceId".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));

                // Write back to file
                let new_content = serde_json::to_string_pretty(&json)
                    .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))?;
                fs::write(&storage_path, new_content)
                    .map_err(|e| anyhow!("Failed to write storage.json: {}", e))?;
                log::info!("Cleared login information from storage.json");
            }
        }
    }

    // 3. Delete state.vscdb database (contains more login state)
    let state_db_path = trae_path.join("User").join("globalStorage").join("state.vscdb");
    if state_db_path.exists() {
        fs::remove_file(&state_db_path)
            .map_err(|e| anyhow!("Failed to delete state.vscdb: {}", e))?;
        log::info!("Deleted state.vscdb");
    }

    // 4. Delete state.vscdb.backup
    let state_db_backup_path = trae_path.join("User").join("globalStorage").join("state.vscdb.backup");
    if state_db_backup_path.exists() {
        let _ = fs::remove_file(&state_db_backup_path);
        log::info!("Deleted state.vscdb.backup");
    }

    // 5. Clear Local State (contains encryption keys)
    let local_state_path = trae_path.join("Local State");
    if local_state_path.exists() {
        let _ = fs::remove_file(&local_state_path);
        log::info!("Deleted Local State");
    }

    // 6. Clear IndexedDB (may contain login cache)
    let indexed_db_path = trae_path.join("IndexedDB");
    if indexed_db_path.exists() {
        let _ = fs::remove_dir_all(&indexed_db_path);
        log::info!("Cleared IndexedDB");
    }

    // 7. Clear Local Storage
    let local_storage_path = trae_path.join("Local Storage");
    if local_storage_path.exists() {
        let _ = fs::remove_dir_all(&local_storage_path);
        log::info!("Cleared Local Storage");
    }

    // 8. Clear Session Storage
    let session_storage_path = trae_path.join("Session Storage");
    if session_storage_path.exists() {
        let _ = fs::remove_dir_all(&session_storage_path);
        log::info!("Cleared Session Storage");
    }

    // 9. Clear Cookies
    let cookies_path = trae_path.join("Network").join("Cookies");
    if cookies_path.exists() {
        let _ = fs::remove_file(&cookies_path);
        log::info!("Cleared Cookies");
    }

    Ok(())
}

/// Simple MD5 hash (used to generate telemetry.machineId format).
fn md5_hash(input: &str) -> u128 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher2 = DefaultHasher::new();
    format!("{}{}", input, h1).hash(&mut hasher2);
    let h2 = hasher2.finish();

    ((h1 as u128) << 64) | (h2 as u128)
}

// macOS platform implementation
#[cfg(target_os = "macos")]
pub fn get_machine_guid() -> Result<String> {
    // Use ioreg command to read IOPlatformUUID
    let output = Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|e| anyhow!("Failed to execute ioreg: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse IOPlatformUUID
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            // Format: "IOPlatformUUID" = "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"
            if let Some(uuid) = line.split('"').nth(3) {
                return Ok(uuid.to_string());
            }
        }
    }
    
    Err(anyhow!("Failed to get IOPlatformUUID"))
}

#[cfg(target_os = "macos")]
pub fn set_machine_guid(_new_guid: &str) -> Result<()> {
    // macOS does not allow modifying system UUID
    Err(anyhow!("macOS does not support modifying system machine ID"))
}

#[cfg(target_os = "macos")]
pub fn reset_machine_guid() -> Result<String> {
    // macOS does not allow resetting system UUID
    Err(anyhow!("macOS does not support resetting system machine ID"))
}

// Placeholder implementations for non-Windows/macOS platforms
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn get_machine_guid() -> Result<String> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn set_machine_guid(_new_guid: &str) -> Result<()> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn reset_machine_guid() -> Result<String> {
    Err(anyhow!("This feature is only supported on Windows and macOS"))
}
