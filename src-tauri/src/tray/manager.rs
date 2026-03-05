// System Tray Manager implementation
// Handles system tray/menubar UI, menu updates, and notifications

use tauri::{
    AppHandle, Manager, Emitter,
    menu::{Menu, MenuBuilder, MenuItemBuilder, CheckMenuItemBuilder, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};
use anyhow::{Result, Context};
use crate::account::AccountBrief;
use std::path::PathBuf;

// Embed the icon data as a fallback
const EMBEDDED_ICON: &[u8] = include_bytes!("../../icons/32x32.png");

/// Manages the system tray/menubar interface.
pub struct SystemTrayManager {
    app_handle: AppHandle,
}

impl SystemTrayManager {
    /// Initializes the system tray manager with icon and initial menu.
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        let manager = Self { app_handle };
        manager.setup_tray()?;
        Ok(manager)
    }

    /// Sets up the system tray icon and initial menu.
    fn setup_tray(&self) -> Result<()> {
        // Load tray icon.
        let icon = self.load_tray_icon()?;
        
        // Build initial menu.
        let menu = self.build_menu(vec![])?;
        
        // Create tray icon.
        let _tray = TrayIconBuilder::with_id("main")
            .icon(icon)
            .menu(&menu)
            .tooltip("Trae Account Switcher")
            .on_menu_event(|app, event| {
                // Handle menu item clicks.
                match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "refresh" => {
                        // Emit refresh event to frontend.
                        if let Err(e) = app.emit("refresh-accounts", ()) {
                            log::error!("Failed to emit refresh event: {}", e);
                        }
                    }
                    id if id.starts_with("account_") => {
                        // Extract account ID and emit switch event.
                        let account_id = id.strip_prefix("account_").unwrap_or("");
                        if let Err(e) = app.emit("switch-account", account_id) {
                            log::error!("Failed to emit switch event: {}", e);
                        }
                    }
                    _ => {}
                }
            })
            .on_tray_icon_event(|_tray, event| {
                // Handle tray icon events (e.g., click).
                if let TrayIconEvent::Click { .. } = event {
                    // Menu will show automatically on click.
                }
            })
            .build(&self.app_handle)
            .context("Failed to build tray icon")?;
        
        Ok(())
    }

    /// Loads the tray icon image.
    fn load_tray_icon(&self) -> Result<tauri::image::Image<'static>> {
        // Strategy 1: Try resource directory (production builds).
        if let Ok(resource_dir) = self.app_handle.path().resource_dir() {
            let icon_path = resource_dir.join("icons/32x32.png");
            if let Ok(image) = self.try_load_from_path(icon_path.clone()) {
                log::info!("Loaded tray icon from resource directory");
                return Ok(image);
            } else {
                log::info!("Failed to load icon from resource directory, trying fallback strategies");
            }
        } else {
            log::info!("Resource directory not available, trying fallback strategies");
        }

        // Strategy 2: Try source directory (development mode).
        // Use app_config_dir as a base and navigate to the project root.
        if let Ok(config_dir) = self.app_handle.path().app_config_dir() {
            // Navigate up from config dir to find project root with src-tauri.
            let mut current = config_dir.clone();
            for _ in 0..5 {
                let source_icon_path = current.join("src-tauri/icons/32x32.png");
                if let Ok(image) = self.try_load_from_path(source_icon_path.clone()) {
                    log::info!("Loaded tray icon from source directory (development mode)");
                    return Ok(image);
                }
                if let Some(parent) = current.parent() {
                    current = parent.to_path_buf();
                } else {
                    break;
                }
            }
            log::info!("Failed to load icon from source directory, using embedded icon");
        }

        // Strategy 3: Use embedded icon as last resort.
        match self.load_from_embedded() {
            Ok(image) => {
                log::info!("Loaded tray icon from embedded data");
                Ok(image)
            }
            Err(e) => {
                anyhow::bail!("Failed to load tray icon from all strategies: {}", e)
            }
        }
    }

    /// Tries to load icon from a specific path.
    fn try_load_from_path(&self, path: PathBuf) -> Result<tauri::image::Image<'static>> {
        // Read the icon file.
        let icon_bytes = std::fs::read(&path)
            .context(format!("Failed to read icon from {:?}", path))?;
        
        // Load as PNG and convert to RGBA.
        let img = image::load_from_memory(&icon_bytes)
            .context("Failed to decode icon image")?
            .to_rgba8();
        
        let (width, height) = img.dimensions();
        let rgba = img.into_raw();
        
        Ok(tauri::image::Image::new_owned(rgba, width, height))
    }

    /// Loads icon from embedded data.
    fn load_from_embedded(&self) -> Result<tauri::image::Image<'static>> {
        // Load as PNG and convert to RGBA.
        let img = image::load_from_memory(EMBEDDED_ICON)
            .context("Failed to decode embedded icon image")?
            .to_rgba8();
        
        let (width, height) = img.dimensions();
        let rgba = img.into_raw();
        
        Ok(tauri::image::Image::new_owned(rgba, width, height))
    }

    /// Builds the menu with the account list.
    fn build_menu(&self, accounts: Vec<AccountBrief>) -> Result<Menu<tauri::Wry>> {
        let menu = MenuBuilder::new(&self.app_handle);
        
        if accounts.is_empty() {
            // Show "No accounts available" message.
            let menu = menu
                .item(&MenuItemBuilder::with_id("no_accounts", "No accounts available").enabled(false).build(&self.app_handle)?)
                .separator()
                .item(&PredefinedMenuItem::separator(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("refresh", "Refresh").build(&self.app_handle)?)
                .item(&MenuItemBuilder::with_id("quit", "Quit").build(&self.app_handle)?)
                .build()?;
            
            return Ok(menu);
        }
        
        // Add account items.
        let mut menu_builder = menu;
        
        for account in accounts {
            // Create menu item for each account.
            let item_id = format!("account_{}", account.id);
            let display_name = if account.name.is_empty() {
                account.email.clone()
            } else {
                format!("{} ({})", account.name, account.email)
            };
            
            // Use check menu item to mark current account.
            if account.is_current {
                let item = CheckMenuItemBuilder::with_id(&item_id, &display_name)
                    .checked(true)
                    .build(&self.app_handle)?;
                menu_builder = menu_builder.item(&item);
            } else {
                let item = MenuItemBuilder::with_id(&item_id, &display_name)
                    .build(&self.app_handle)?;
                menu_builder = menu_builder.item(&item);
            }
        }
        
        // Add separator and action items.
        let menu = menu_builder
            .separator()
            .item(&PredefinedMenuItem::separator(&self.app_handle)?)
            .item(&MenuItemBuilder::with_id("refresh", "Refresh").build(&self.app_handle)?)
            .item(&MenuItemBuilder::with_id("quit", "Quit").build(&self.app_handle)?)
            .build()?;
        
        Ok(menu)
    }

    /// Updates the account menu with the current account list.
    pub fn update_account_menu(&mut self, accounts: Vec<AccountBrief>) -> Result<()> {
        // Build new menu.
        let menu = self.build_menu(accounts)?;
        
        // Get the tray icon and update its menu.
        if let Some(tray) = self.app_handle.tray_by_id("main") {
            tray.set_menu(Some(menu))
                .context("Failed to update tray menu")?;
        } else {
            // If tray doesn't exist yet, create it.
            self.setup_tray()?;
        }
        
        Ok(())
    }

    /// Shows a progress indicator.
    pub fn show_progress(&self, step: u8, total: u8, message: &str) {
        // Emit progress event to frontend.
        let progress = serde_json::json!({
            "step": step,
            "total": total,
            "message": message,
        });
        
        if let Err(e) = self.app_handle.emit("switch-progress", progress) {
            log::warn!("Failed to emit progress event: {}", e);
        }
    }

    /// Shows a success notification.
    pub fn show_success(&self, message: &str) {
        // Emit success event to frontend.
        if let Err(e) = self.app_handle.emit("switch-success", message) {
            log::warn!("Failed to emit success event: {}", e);
        }
    }

    /// Shows an error notification.
    pub fn show_error(&self, message: &str) {
        // Emit error event to frontend.
        if let Err(e) = self.app_handle.emit("switch-error", message) {
            log::warn!("Failed to emit error event: {}", e);
        }
    }
}
