// Application configuration

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub websocket_port: u16,
    pub hotkey: String,
    pub auto_refresh_tokens: bool,
    pub notification_enabled: bool,
    pub switch_timeout_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            websocket_port: 9527,
            hotkey: if cfg!(target_os = "macos") {
                "Cmd+Shift+A".to_string()
            } else {
                "Ctrl+Shift+A".to_string()
            },
            auto_refresh_tokens: true,
            notification_enabled: true,
            switch_timeout_ms: 5000,
        }
    }
}

#[cfg(test)]
#[path = "config.test.rs"]
mod tests;

impl AppConfig {
    /// Load configuration from file or create default
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Return default config if file doesn't exist
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_file_path()?;
        
        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }
        
        // Serialize config to JSON with pretty formatting
        let content = serde_json::to_string_pretty(self)?;
        
        // Write to file
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    /// Get platform-specific config directory
    pub fn config_dir() -> anyhow::Result<std::path::PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        
        #[cfg(target_os = "windows")]
        let config_path = home.join("AppData").join("Roaming").join("Trae Auto");
        
        #[cfg(target_os = "macos")]
        let config_path = home.join("Library").join("Application Support").join("Trae Auto");
        
        #[cfg(target_os = "linux")]
        let config_path = home.join(".config").join("trae-auto");
        
        Ok(config_path)
    }
    
    /// Get full path to config file
    pub fn config_file_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }
}
