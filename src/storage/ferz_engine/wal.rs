//! Write-Ahead Logging (WAL)
//! 
//! WAL ensures atomicity and durability by logging every change to a separate
//! log file before they are applied to the data pages.

use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

use crate::util::error::Result;
use crate::util::types::{PageId, PAGE_SIZE};

/// Log record type
#[repr(u8)]
pub enum LogType {
    UpdatePage = 1,
    AllocatePage = 2,
    Commit = 3,
    Rollback = 4,
}

/// WAL Writer
pub struct WalWriter {
    file: Mutex<File>,
}

impl WalWriter {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
            
        Ok(Self {
            file: Mutex::new(file),
        })
    }

    /// Log a page update
    pub fn log_update(&self, page_id: PageId, data: &[u8; PAGE_SIZE]) -> Result<u64> {
        let mut file = self.file.lock().unwrap();
        
        // Header: Type(1) + PageId(4) + Data(PAGE_SIZE)
        file.write_all(&[LogType::UpdatePage as u8])?;
        file.write_all(&page_id.as_u32().to_be_bytes())?;
        file.write_all(data)?;
        
        let pos = file.seek(SeekFrom::Current(0))?;
        file.sync_data()?;
        
        Ok(pos)
    }

    /// Log a transaction commit
    pub fn log_commit(&self, tx_id: u64) -> Result<()> {
        let mut file = self.file.lock().unwrap();
        file.write_all(&[LogType::Commit as u8])?;
        file.write_all(&tx_id.to_be_bytes())?;
        file.sync_data()?;
        Ok(())
    }
}

pub struct WAL {
    // Will be implemented in next phase
}
