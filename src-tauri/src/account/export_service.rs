use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tauri::Emitter;

use super::{AccountManager, AccountExportData, ExportFormat, ExportResponse, ExportProgress, FileManager, StreamWriter};

/// Coordinates account data export and file generation.
pub struct ExportService {
    account_manager: Arc<Mutex<AccountManager>>,
    file_manager: FileManager,
}

impl ExportService {
    /// Creates a new export service instance.
    pub fn new(account_manager: Arc<Mutex<AccountManager>>) -> Result<Self> {
        let file_manager = FileManager::new()?;
        
        Ok(Self {
            account_manager,
            file_manager,
        })
    }
    
    /// Exports accounts in the specified format.
    pub async fn export_accounts(
        &self,
        account_ids: Vec<String>,
        format: ExportFormat,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<ExportResponse> {
        let total = account_ids.len();
        let start_time = Instant::now();
        
        let filename = self.file_manager.generate_filename(&format);
        let file_path = self.file_manager.get_export_path(&filename);
        
        let mut writer = StreamWriter::new(&file_path, format.clone())?;
        
        const BATCH_SIZE: usize = 500;
        let mut processed = 0;
        
        for (_batch_idx, chunk) in account_ids.chunks(BATCH_SIZE).enumerate() {
            let accounts = self.fetch_accounts(chunk).await?;
            
            writer.write_batch(&accounts)
                .context("Failed to write batch")?;
            
            processed += accounts.len();
            
            if let Some(ref handle) = app_handle {
                self.emit_progress(handle, processed, total, start_time)?;
            }
        }
        
        let records_written = writer.records_written();
        if records_written != total {
            writer.cleanup(&file_path)?;
            return Err(anyhow!(
                "Data integrity check failed: expected {} accounts, got {}",
                total,
                records_written
            ));
        }
        
        writer.finalize()
            .context("Failed to finalize export file")?;
        
        let download_url = format!("/exports/{}", filename);
        
        Ok(ExportResponse {
            download_url,
            filename,
            record_count: total,
        })
    }
    
    /// Fetches account data for the specified IDs.
    async fn fetch_accounts(&self, ids: &[String]) -> Result<Vec<AccountExportData>> {
        let manager = self.account_manager.lock().await;
        let mut accounts = Vec::with_capacity(ids.len());
        
        for id in ids {
            match manager.get_account(id) {
                Ok(account) => {
                    accounts.push(AccountExportData::from(&account));
                }
                Err(e) => {
                    log::warn!("Failed to fetch account {}: {}", id, e);
                }
            }
        }
        
        Ok(accounts)
    }
    
    /// Emits a progress update event to the frontend.
    fn emit_progress(
        &self,
        app_handle: &tauri::AppHandle,
        current: usize,
        total: usize,
        start_time: Instant,
    ) -> Result<()> {
        let elapsed = start_time.elapsed().as_secs_f64();
        let percentage = ((current as f64 / total as f64) * 100.0) as u32;
        let estimated_remaining = if current > 0 {
            ((elapsed / current as f64) * (total - current) as f64) as u32
        } else {
            0
        };
        
        let progress = ExportProgress {
            current,
            total,
            percentage,
            estimated_time_remaining: estimated_remaining,
        };
        
        app_handle.emit("export-progress", progress)
            .context("Failed to emit progress event")?;
        
        Ok(())
    }
}
