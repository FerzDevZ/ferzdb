//! This module provides durability by logging all mutations to an append-only file
//! before they are applied to the data pages.

use std::fs::{File, OpenOptions};
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;
use crate::util::error::{FerzError, Result};
use crate::util::types::{PageId, WAL_MAGIC};
use serde::{Serialize, Deserialize};

/// Types of log records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogType {
    Insert,
    Update,
    Delete,
    Begin,
    Commit,
    Abort,
}

/// A single record in the WAL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub lsn: u64,           // Log Sequence Number
    pub xid: u64,           // Transaction ID
    pub log_type: LogType,
    pub page_id: PageId,
    pub data: Vec<u8>,      // Delta or full row
}

pub struct WalManager {
    file: Mutex<File>,
    next_lsn: Mutex<u64>,
}

impl WalManager {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Check or write magic header
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            file.write_all(WAL_MAGIC)?;
            file.sync_all()?;
        } else {
            let mut magic = [0u8; 8];
            file.read_exact(&mut magic)?;
            if &magic != WAL_MAGIC {
                return Err(FerzError::Internal("Invalid WAL file magic".to_string()));
            }
        }

        Ok(Self {
            file: Mutex::new(file),
            next_lsn: Mutex::new(0),
        })
    }

    /// Append a log record to the WAL
    pub fn append(&self, xid: u64, log_type: LogType, page_id: PageId, data: Vec<u8>) -> Result<u64> {
        let mut n_lsn = self.next_lsn.lock().unwrap();
        let lsn = *n_lsn;
        
        let record = LogRecord {
            lsn,
            xid,
            log_type,
            page_id,
            data,
        };

        let serialized = serde_json::to_vec(&record)
            .map_err(|e| FerzError::Internal(e.to_string()))?;
        
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::End(0))?;
        
        // Write length-prefixed record
        let len = serialized.len() as u32;
        file.write_all(&len.to_be_bytes())?;
        file.write_all(&serialized)?;
        
        // Sync for durability (can be optimized with group commit later)
        file.sync_data()?;
        
        *n_lsn += 1;
        Ok(lsn)
    }

    /// Iterator over log records for recovery
    pub fn recover(&self) -> Result<Vec<LogRecord>> {
        let mut file = self.file.lock().unwrap();
        file.seek(SeekFrom::Start(8))?; // Skip magic
        
        let mut records = Vec::new();
        let mut len_buf = [0u8; 4];
        
        loop {
            match file.read_exact(&mut len_buf) {
                Ok(_) => {
                    let len = u32::from_be_bytes(len_buf) as usize;
                    let mut data = vec![0u8; len];
                    file.read_exact(&mut data)?;
                    let record: LogRecord = serde_json::from_slice(&data)
                        .map_err(|e| FerzError::Internal(e.to_string()))?;
                    records.push(record);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(FerzError::Io(e)),
            }
        }
        
        // Update next_lsn to the latest
        if let Some(last) = records.last() {
            let mut n_lsn = self.next_lsn.lock().unwrap();
            *n_lsn = last.lsn + 1;
        }

        Ok(records)
    }
}
