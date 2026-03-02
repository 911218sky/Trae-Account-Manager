use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::types::{AccountExportData, ExportFormat};

/// Writes account data in batches to a file in the specified format.
pub struct StreamWriter {
    writer: BufWriter<File>,
    format: ExportFormat,
    records_written: usize,
    is_first_record: bool,
}

impl StreamWriter {
    /// Creates a new stream writer instance.
    pub fn new(path: &Path, format: ExportFormat) -> Result<Self> {
        let file = File::create(path)
            .context("Failed to create export file")?;
        let writer = BufWriter::with_capacity(8192, file);
        
        let mut stream_writer = Self {
            writer,
            format: format.clone(),
            records_written: 0,
            is_first_record: true,
        };
        
        match format {
            ExportFormat::Csv => stream_writer.init_csv()?,
            ExportFormat::Json => stream_writer.init_json()?,
        }
        
        Ok(stream_writer)
    }
    
    /// Initializes CSV format with header row.
    fn init_csv(&mut self) -> Result<()> {
        self.writer.write_all(&[0xEF, 0xBB, 0xBF])?;
        self.writer.write_all(b"account_id,username,email,status,created_date,last_login\n")?;
        Ok(())
    }
    
    /// Initializes JSON format with opening bracket.
    fn init_json(&mut self) -> Result<()> {
        self.writer.write_all(b"[\n")?;
        Ok(())
    }
    
    /// Writes a batch of account records.
    pub fn write_batch(&mut self, accounts: &[AccountExportData]) -> Result<()> {
        match self.format {
            ExportFormat::Csv => self.write_csv_batch(accounts)?,
            ExportFormat::Json => self.write_json_batch(accounts)?,
        }
        
        self.records_written += accounts.len();
        
        if self.records_written % 100 == 0 {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// Writes a batch of records in CSV format.
    fn write_csv_batch(&mut self, accounts: &[AccountExportData]) -> Result<()> {
        for account in accounts {
            let account_id = Self::escape_csv_field(&account.account_id);
            let username = Self::escape_csv_field(&account.username);
            let email = Self::escape_csv_field(&account.email);
            let status = Self::escape_csv_field(&account.status);
            let created_date = Self::escape_csv_field(&account.created_date);
            let last_login = Self::escape_csv_field(&account.last_login);
            
            writeln!(
                self.writer,
                "{},{},{},{},{},{}",
                account_id, username, email, status, created_date, last_login
            )?;
        }
        
        Ok(())
    }
    
    /// Escapes CSV field values by handling commas, quotes, and newlines.
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
    
    /// Writes a batch of records in JSON format.
    fn write_json_batch(&mut self, accounts: &[AccountExportData]) -> Result<()> {
        for account in accounts {
            if !self.is_first_record {
                self.writer.write_all(b",\n")?;
            }
            self.is_first_record = false;
            
            let json = serde_json::to_string_pretty(account)?;
            
            for line in json.lines() {
                self.writer.write_all(b"  ")?;
                self.writer.write_all(line.as_bytes())?;
                self.writer.write_all(b"\n")?;
            }
        }
        
        Ok(())
    }
    
    /// Flushes the write buffer.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
            .context("Failed to flush writer")?;
        Ok(())
    }
    
    /// Finalizes the export file and closes it.
    pub fn finalize(mut self) -> Result<()> {
        match self.format {
            ExportFormat::Csv => {
            }
            ExportFormat::Json => {
                self.writer.write_all(b"\n]")?;
            }
        }
        
        self.flush()?;
        Ok(())
    }
    
    /// Cleans up the partial export file in case of error.
    pub fn cleanup(self, path: &Path) -> Result<()> {
        drop(self);
        
        if path.exists() {
            std::fs::remove_file(path)
                .context("Failed to cleanup partial export file")?;
        }
        
        Ok(())
    }
    
    /// Returns the number of records written.
    pub fn records_written(&self) -> usize {
        self.records_written
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    fn create_test_account(id: &str) -> AccountExportData {
        AccountExportData {
            account_id: id.to_string(),
            username: format!("user_{}", id),
            email: format!("user_{}@example.com", id),
            status: "normal".to_string(),
            created_date: "2024-01-01 00:00:00".to_string(),
            last_login: "2024-01-02 00:00:00".to_string(),
        }
    }
    
    #[test]
    fn test_csv_escape_simple() {
        assert_eq!(StreamWriter::escape_csv_field("simple"), "simple");
    }
    
    #[test]
    fn test_csv_escape_comma() {
        assert_eq!(StreamWriter::escape_csv_field("hello,world"), "\"hello,world\"");
    }
    
    #[test]
    fn test_csv_escape_quote() {
        assert_eq!(StreamWriter::escape_csv_field("say \"hello\""), "\"say \"\"hello\"\"\"");
    }
    
    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(StreamWriter::escape_csv_field("line1\nline2"), "\"line1\nline2\"");
    }
}
