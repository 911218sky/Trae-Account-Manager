// Hotkey data types

use serde::{Deserialize, Serialize};

/// Actions that can be triggered by hotkeys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotkeyAction {
    /// Open the account switcher menu
    OpenMenu,
    /// Switch to a specific account
    SwitchToAccount(String),
}
