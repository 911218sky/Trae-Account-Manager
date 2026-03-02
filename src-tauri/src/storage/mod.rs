// Secure Storage module
// Cross-platform secure credential storage

pub mod manager;
pub mod types;
pub mod migration;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

pub use manager::SecureStorageManager;
pub use types::Credential;
pub use migration::migrate_to_secure_storage;
