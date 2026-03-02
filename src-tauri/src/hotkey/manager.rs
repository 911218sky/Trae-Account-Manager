// Hotkey Manager implementation

use anyhow::{anyhow, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::collections::HashMap;
use std::sync::Arc;

use super::types::HotkeyAction;
use crate::switcher::error::SystemError;

/// Manages global hotkeys
///
/// # Example
///
/// ```no_run
/// use trae_auto_lib::hotkey::{HotkeyManager, HotkeyAction};
///
/// # fn main() -> anyhow::Result<()> {
/// let mut manager = HotkeyManager::new()?;
///
/// // Register the default hotkey to open menu
/// let default_key = HotkeyManager::default_hotkey();
/// manager.register(&default_key, HotkeyAction::OpenMenu)?;
///
/// // Register a custom hotkey to switch to a specific account
/// manager.register("Ctrl+Shift+1", HotkeyAction::SwitchToAccount("acc_123".to_string()))?;
///
/// // Check for conflicts
/// if manager.check_conflict("Ctrl+Shift+1") {
///     println!("Hotkey is already registered");
/// }
///
/// // Unregister a hotkey
/// manager.unregister("Ctrl+Shift+1")?;
/// # Ok(())
/// # }
/// ```
pub struct HotkeyManager {
    manager: Arc<GlobalHotKeyManager>,
    hotkeys: HashMap<String, (HotKey, HotkeyAction)>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| anyhow!("Failed to create GlobalHotKeyManager: {}", e))?;

        Ok(Self {
            manager: Arc::new(manager),
            hotkeys: HashMap::new(),
        })
    }

    /// Register a global hotkey
    pub fn register(&mut self, key_combo: &str, action: HotkeyAction) -> Result<()> {
        // Check for conflicts first
        if self.check_conflict(key_combo) {
            return Err(SystemError::HotkeyRegistrationFailed(format!(
                "Hotkey '{}' is already registered",
                key_combo
            ))
            .into());
        }

        // Parse the key combination
        let hotkey = self.parse_key_combo(key_combo)?;

        // Register with the global hotkey manager
        self.manager
            .register(hotkey)
            .map_err(|e| {
                SystemError::HotkeyRegistrationFailed(format!(
                    "Failed to register hotkey '{}': {}",
                    key_combo, e
                ))
            })?;

        // Store the hotkey and action
        self.hotkeys
            .insert(key_combo.to_string(), (hotkey, action));

        Ok(())
    }

    /// Unregister a hotkey
    #[allow(dead_code)]
    pub fn unregister(&mut self, key_combo: &str) -> Result<()> {
        if let Some((hotkey, _)) = self.hotkeys.remove(key_combo) {
            self.manager.unregister(hotkey).map_err(|e| {
                anyhow!(
                    "Failed to unregister hotkey '{}': {}",
                    key_combo,
                    e
                )
            })?;
        }
        Ok(())
    }

    /// Check if a hotkey conflicts with existing hotkeys
    pub fn check_conflict(&self, key_combo: &str) -> bool {
        self.hotkeys.contains_key(key_combo)
    }

    /// Get the default hotkey for the current platform
    pub fn default_hotkey() -> String {
        #[cfg(target_os = "macos")]
        {
            "Cmd+Shift+A".to_string()
        }
        #[cfg(not(target_os = "macos"))]
        {
            "Ctrl+Shift+A".to_string()
        }
    }

    /// Parse a key combination string into a HotKey
    fn parse_key_combo(&self, key_combo: &str) -> Result<HotKey> {
        let parts: Vec<&str> = key_combo.split('+').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty key combination"));
        }

        let mut modifiers = Modifiers::empty();
        let mut key_code: Option<Code> = None;

        for part in parts {
            match part.to_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
                "shift" => modifiers |= Modifiers::SHIFT,
                "alt" | "option" => modifiers |= Modifiers::ALT,
                "cmd" | "command" | "super" | "meta" => modifiers |= Modifiers::SUPER,
                // Single letter keys
                "a" => key_code = Some(Code::KeyA),
                "b" => key_code = Some(Code::KeyB),
                "c" => key_code = Some(Code::KeyC),
                "d" => key_code = Some(Code::KeyD),
                "e" => key_code = Some(Code::KeyE),
                "f" => key_code = Some(Code::KeyF),
                "g" => key_code = Some(Code::KeyG),
                "h" => key_code = Some(Code::KeyH),
                "i" => key_code = Some(Code::KeyI),
                "j" => key_code = Some(Code::KeyJ),
                "k" => key_code = Some(Code::KeyK),
                "l" => key_code = Some(Code::KeyL),
                "m" => key_code = Some(Code::KeyM),
                "n" => key_code = Some(Code::KeyN),
                "o" => key_code = Some(Code::KeyO),
                "p" => key_code = Some(Code::KeyP),
                "q" => key_code = Some(Code::KeyQ),
                "r" => key_code = Some(Code::KeyR),
                "s" => key_code = Some(Code::KeyS),
                "t" => key_code = Some(Code::KeyT),
                "u" => key_code = Some(Code::KeyU),
                "v" => key_code = Some(Code::KeyV),
                "w" => key_code = Some(Code::KeyW),
                "x" => key_code = Some(Code::KeyX),
                "y" => key_code = Some(Code::KeyY),
                "z" => key_code = Some(Code::KeyZ),
                // Number keys
                "0" => key_code = Some(Code::Digit0),
                "1" => key_code = Some(Code::Digit1),
                "2" => key_code = Some(Code::Digit2),
                "3" => key_code = Some(Code::Digit3),
                "4" => key_code = Some(Code::Digit4),
                "5" => key_code = Some(Code::Digit5),
                "6" => key_code = Some(Code::Digit6),
                "7" => key_code = Some(Code::Digit7),
                "8" => key_code = Some(Code::Digit8),
                "9" => key_code = Some(Code::Digit9),
                // Function keys
                "f1" => key_code = Some(Code::F1),
                "f2" => key_code = Some(Code::F2),
                "f3" => key_code = Some(Code::F3),
                "f4" => key_code = Some(Code::F4),
                "f5" => key_code = Some(Code::F5),
                "f6" => key_code = Some(Code::F6),
                "f7" => key_code = Some(Code::F7),
                "f8" => key_code = Some(Code::F8),
                "f9" => key_code = Some(Code::F9),
                "f10" => key_code = Some(Code::F10),
                "f11" => key_code = Some(Code::F11),
                "f12" => key_code = Some(Code::F12),
                _ => {
                    return Err(anyhow!("Unknown key: {}", part));
                }
            }
        }

        let key_code = key_code.ok_or_else(|| anyhow!("No key code specified"))?;

        Ok(HotKey::new(Some(modifiers), key_code))
    }

    /// Get the action for a hotkey event
    #[allow(dead_code)]
    pub fn get_action(&self, event: &GlobalHotKeyEvent) -> Option<HotkeyAction> {
        for (hotkey, action) in self.hotkeys.values() {
            if hotkey.id() == event.id {
                return Some(action.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hotkey_manager() {
        let manager = HotkeyManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_default_hotkey() {
        let default = HotkeyManager::default_hotkey();
        #[cfg(target_os = "macos")]
        assert_eq!(default, "Cmd+Shift+A");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(default, "Ctrl+Shift+A");
    }

    #[test]
    fn test_check_conflict_empty() {
        let manager = HotkeyManager::new().unwrap();
        assert!(!manager.check_conflict("Ctrl+Shift+A"));
    }

    #[test]
    fn test_parse_key_combo_ctrl_shift_a() {
        let manager = HotkeyManager::new().unwrap();
        let result = manager.parse_key_combo("Ctrl+Shift+A");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_key_combo_cmd_shift_a() {
        let manager = HotkeyManager::new().unwrap();
        let result = manager.parse_key_combo("Cmd+Shift+A");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_key_combo_invalid() {
        let manager = HotkeyManager::new().unwrap();
        let result = manager.parse_key_combo("InvalidKey");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_key_combo_empty() {
        let manager = HotkeyManager::new().unwrap();
        let result = manager.parse_key_combo("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_key_combo_function_keys() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("Ctrl+F1").is_ok());
        assert!(manager.parse_key_combo("Shift+F12").is_ok());
    }

    #[test]
    fn test_parse_key_combo_numbers() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("Ctrl+1").is_ok());
        assert!(manager.parse_key_combo("Alt+9").is_ok());
    }

    #[test]
    fn test_register_and_check_conflict() {
        let mut manager = HotkeyManager::new().unwrap();
        let key_combo = "Ctrl+Shift+T";
        let action = HotkeyAction::OpenMenu;

        // Initially no conflict
        assert!(!manager.check_conflict(key_combo));

        // Register the hotkey
        let result = manager.register(key_combo, action);
        // Note: This may fail on some systems if the hotkey is already in use
        // by another application, which is expected behavior
        if result.is_ok() {
            // After registration, should detect conflict
            assert!(manager.check_conflict(key_combo));
        }
    }

    #[test]
    fn test_register_duplicate_fails() {
        let mut manager = HotkeyManager::new().unwrap();
        let key_combo = "Ctrl+Shift+B";
        let action = HotkeyAction::OpenMenu;

        // First registration
        let result1 = manager.register(key_combo, action.clone());
        
        // If first registration succeeded, second should fail
        if result1.is_ok() {
            let result2 = manager.register(key_combo, action);
            assert!(result2.is_err());
        }
    }

    #[test]
    fn test_unregister() {
        let mut manager = HotkeyManager::new().unwrap();
        let key_combo = "Ctrl+Shift+U";
        let action = HotkeyAction::OpenMenu;

        // Register
        let result = manager.register(key_combo, action);
        if result.is_ok() {
            assert!(manager.check_conflict(key_combo));

            // Unregister
            let unregister_result = manager.unregister(key_combo);
            assert!(unregister_result.is_ok());

            // Should no longer conflict
            assert!(!manager.check_conflict(key_combo));
        }
    }

    #[test]
    fn test_parse_key_combo_case_insensitive() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("ctrl+shift+a").is_ok());
        assert!(manager.parse_key_combo("CTRL+SHIFT+A").is_ok());
        assert!(manager.parse_key_combo("Ctrl+Shift+A").is_ok());
    }

    #[test]
    fn test_parse_key_combo_with_spaces() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("Ctrl + Shift + A").is_ok());
        assert!(manager.parse_key_combo("  Ctrl  +  Shift  +  A  ").is_ok());
    }

    #[test]
    fn test_parse_key_combo_alt_modifier() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("Alt+A").is_ok());
        assert!(manager.parse_key_combo("Option+A").is_ok());
    }

    #[test]
    fn test_parse_key_combo_multiple_modifiers() {
        let manager = HotkeyManager::new().unwrap();
        assert!(manager.parse_key_combo("Ctrl+Alt+Shift+A").is_ok());
        assert!(manager.parse_key_combo("Cmd+Shift+F1").is_ok());
    }

    #[test]
    fn test_switch_to_account_action() {
        let mut manager = HotkeyManager::new().unwrap();
        let key_combo = "Ctrl+Shift+1";
        let action = HotkeyAction::SwitchToAccount("account_123".to_string());

        let result = manager.register(key_combo, action);
        // May fail if hotkey is in use, which is expected
        if result.is_ok() {
            assert!(manager.check_conflict(key_combo));
        }
    }
}
