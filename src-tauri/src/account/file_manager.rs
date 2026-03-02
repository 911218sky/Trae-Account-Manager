use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;

use super::types::ExportFormat;

/// Manages the lifecycle of exported files.
pub struct FileManager {
    export_dir: PathBuf,
}

impl FileManager {
    /// Creates a new FileManager instance.
    pub fn new() -> Result<Self> {
        let export_dir = Self::get_export_directory()?;
        
        if !export_dir.exists() {
            fs::create_dir_all(&export_dir)
                .context("Failed to create export directory")?;
        }
        
        Ok(Self { export_dir })
    }
    
    /// Returns the export directory path.
    pub fn get_export_directory() -> Result<PathBuf> {
        let app_dir = dirs::data_local_dir()
            .context("Failed to get local data directory")?;
        
        Ok(app_dir.join("trae-account-manager").join("exports"))
    }
    
    /// Returns the full path for the export file.
    pub fn get_export_path(&self, filename: &str) -> PathBuf {
        self.export_dir.join(filename)
    }
    
    /// Generates an export filename with format: accounts_export_YYYYMMDD_HHMMSS.{csv|json}
    pub fn generate_filename(&self, format: &ExportFormat) -> String {
        let now = chrono::Local::now();
        let extension = match format {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
        };
        format!(
            "accounts_export_{}_{}.{}",
            now.format("%Y%m%d"),
            now.format("%H%M%S"),
            extension
        )
    }
    
    /// Returns a reference to the export directory path.
    #[allow(dead_code)]
    pub fn export_dir(&self) -> &Path {
        &self.export_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filename_format() {
        let manager = FileManager::new().unwrap();
        
        let csv_filename = manager.generate_filename(&ExportFormat::Csv);
        assert!(csv_filename.starts_with("accounts_export_"));
        assert!(csv_filename.ends_with(".csv"));
        
        let json_filename = manager.generate_filename(&ExportFormat::Json);
        assert!(json_filename.starts_with("accounts_export_"));
        assert!(json_filename.ends_with(".json"));
    }
    
    #[test]
    fn test_export_directory_creation() {
        let manager = FileManager::new().unwrap();
        assert!(manager.export_dir().exists());
        assert!(manager.export_dir().is_dir());
    }
}
