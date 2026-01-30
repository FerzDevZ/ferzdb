//! Low-level file I/O for database pages

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

use crate::util::error::{FerzError, Result};
use crate::util::types::{PageId, PAGE_SIZE};

/// Handles physical disk I/O for database files
pub struct FileHandler {
    file: Mutex<File>,
    path: String,
}

impl FileHandler {
    /// Open or create a database file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        
        Ok(Self {
            file: Mutex::new(file),
            path: path.as_ref().to_string_lossy().to_string(),
        })
    }

    /// Read a page from disk
    pub fn read_page(&self, page_id: PageId, buffer: &mut [u8; PAGE_SIZE]) -> Result<()> {
        let mut file = self.file.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let offset = page_id.as_u32() as u64 * PAGE_SIZE as u64;
        
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(buffer)?;
        
        Ok(())
    }

    /// Write a page to disk
    pub fn write_page(&self, page_id: PageId, buffer: &[u8; PAGE_SIZE]) -> Result<()> {
        let mut file = self.file.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let offset = page_id.as_u32() as u64 * PAGE_SIZE as u64;
        
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(buffer)?;
        file.sync_data()?; // Ensure durability
        
        Ok(())
    }

    /// Get current number of pages in the file
    pub fn num_pages(&self) -> Result<u32> {
        let file = self.file.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let metadata = file.metadata()?;
        Ok((metadata.len() / PAGE_SIZE as u64) as u32)
    }

    /// Allocate a new page by extending the file
    pub fn allocate_page(&self) -> Result<PageId> {
        let mut file = self.file.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let metadata = file.metadata()?;
        let offset = metadata.len();
        let new_id = (offset / PAGE_SIZE as u64) as u32;
        
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&[0u8; PAGE_SIZE])?;
        file.sync_data()?;
        
        Ok(PageId(new_id))
    }
}

pub struct DatabaseFile {
    // Will be implemented in next phase
}
