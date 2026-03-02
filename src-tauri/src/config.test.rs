#[cfg(test)]
mod tests {
    use super::super::AppConfig;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        
        assert_eq!(config.websocket_port, 9527);
        assert_eq!(config.auto_refresh_tokens, true);
        assert_eq!(config.notification_enabled, true);
        assert_eq!(config.switch_timeout_ms, 5000);
        
        // Check platform-specific hotkey
        #[cfg(target_os = "macos")]
        assert_eq!(config.hotkey, "Cmd+Shift+A");
        
        #[cfg(not(target_os = "macos"))]
        assert_eq!(config.hotkey, "Ctrl+Shift+A");
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        
        // Serialize to JSON
        let json = serde_json::to_string(&config).unwrap();
        
        // Deserialize back
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.websocket_port, deserialized.websocket_port);
        assert_eq!(config.hotkey, deserialized.hotkey);
        assert_eq!(config.auto_refresh_tokens, deserialized.auto_refresh_tokens);
        assert_eq!(config.notification_enabled, deserialized.notification_enabled);
        assert_eq!(config.switch_timeout_ms, deserialized.switch_timeout_ms);
    }

    #[test]
    fn test_config_dir_path() {
        let config_dir = AppConfig::config_dir().unwrap();
        
        // Check platform-specific paths
        #[cfg(target_os = "windows")]
        assert!(config_dir.to_str().unwrap().contains("AppData"));
        
        #[cfg(target_os = "macos")]
        assert!(config_dir.to_str().unwrap().contains("Application Support"));
        
        #[cfg(target_os = "linux")]
        assert!(config_dir.to_str().unwrap().contains(".config"));
        
        // All platforms should end with trae-auto or Trae Auto
        let path_str = config_dir.to_str().unwrap();
        assert!(path_str.contains("trae-auto") || path_str.contains("Trae Auto"));
    }

    #[test]
    fn test_config_file_path() {
        let config_file = AppConfig::config_file_path().unwrap();
        
        // Should end with config.json
        assert!(config_file.to_str().unwrap().ends_with("config.json"));
    }

    #[test]
    fn test_load_nonexistent_config_returns_default() {
        // When config file doesn't exist, should return default
        let config = AppConfig::load().unwrap();
        let default = AppConfig::default();
        
        assert_eq!(config.websocket_port, default.websocket_port);
        assert_eq!(config.auto_refresh_tokens, default.auto_refresh_tokens);
    }
}
