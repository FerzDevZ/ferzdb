//! Buffer Pool Manager for in-memory page caching

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::util::error::{FerzError, Result};
use crate::util::types::{PageId}; // Removed unused PAGE_SIZE
use crate::storage::ferz_engine::page::Page;
use crate::storage::ferz_engine::file::FileHandler;

/// Frame in the buffer pool
struct Frame {
    page: Arc<Mutex<Page>>,
    is_dirty: bool,
    pin_count: u32,
    usage_count: u32,
}

pub struct BufferPoolManager {
    file_handler: Arc<FileHandler>,
    frames: Mutex<Vec<Frame>>,
    page_table: Mutex<HashMap<PageId, usize>>,
    pool_size: usize,
}

impl BufferPoolManager {
    pub fn new(file_handler: Arc<FileHandler>, pool_size: usize) -> Self {
        Self {
            file_handler,
            frames: Mutex::new(Vec::with_capacity(pool_size)),
            page_table: Mutex::new(HashMap::new()),
            pool_size,
        }
    }

    pub fn fetch_page(&self, page_id: PageId) -> Result<Arc<Mutex<Page>>> {
        let mut page_table = self.page_table.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let mut frames = self.frames.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;

        if let Some(&frame_idx) = page_table.get(&page_id) {
            let frame = &mut frames[frame_idx];
            frame.pin_count += 1;
            frame.usage_count += 1;
            return Ok(Arc::clone(&frame.page));
        }

        let victim_idx = self.find_victim(&mut frames, &mut page_table)?;
        
        let mut new_page = Page::new(page_id);
        self.file_handler.read_page(page_id, &mut new_page.data)?;

        let page_arc = Arc::new(Mutex::new(new_page));
        let frame = Frame {
            page: Arc::clone(&page_arc),
            is_dirty: false,
            pin_count: 1,
            usage_count: 1,
        };

        if victim_idx < frames.len() {
            frames[victim_idx] = frame;
        } else {
            frames.push(frame);
        }
        
        page_table.insert(page_id, victim_idx);
        Ok(page_arc)
    }

    fn find_victim(&self, frames: &mut [Frame], page_table: &mut HashMap<PageId, usize>) -> Result<usize> {
        if frames.len() < self.pool_size {
            return Ok(frames.len());
        }

        for i in 0..frames.len() {
            if frames[i].pin_count == 0 {
                if frames[i].usage_count > 0 {
                    frames[i].usage_count -= 1;
                } else {
                    if frames[i].is_dirty {
                        let page = frames[i].page.lock().unwrap();
                        self.file_handler.write_page(page.id, &page.data)?;
                    }
                    let page_to_remove = frames[i].page.lock().unwrap().id;
                    page_table.remove(&page_to_remove);
                    return Ok(i);
                }
            }
        }

        Err(FerzError::Internal("Buffer pool full".to_string()))
    }

    pub fn mark_dirty(&self, page_id: PageId) -> Result<()> {
        let page_table = self.page_table.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let mut frames = self.frames.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;

        if let Some(&frame_idx) = page_table.get(&page_id) {
            frames[frame_idx].is_dirty = true;
            Ok(())
        } else {
            Err(FerzError::PageNotFound(page_id.as_u32()))
        }
    }

    pub fn unpin_page(&self, page_id: PageId) -> Result<()> {
        let page_table = self.page_table.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let mut frames = self.frames.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;

        if let Some(&frame_idx) = page_table.get(&page_id) {
            if frames[frame_idx].pin_count > 0 {
                frames[frame_idx].pin_count -= 1;
            }
            Ok(())
        } else {
            Err(FerzError::PageNotFound(page_id.as_u32()))
        }
    }

    /// Write every dirty frame back to the file and clear the dirty bit.
    ///
    /// Without this the pool only reached disk through victim eviction, so a
    /// process that closed cleanly lost every page still pinned in memory —
    /// the catalog included, which made a reopened database forget its tables.
    pub fn flush_all(&self) -> Result<()> {
        let page_table = self.page_table.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;
        let mut frames = self.frames.lock().map_err(|_| FerzError::Internal("Lock poisoned".to_string()))?;

        for frame in frames.iter_mut() {
            if !frame.is_dirty {
                continue;
            }
            let page = frame.page.lock().unwrap();
            self.file_handler.write_page(page.id, &page.data)?;
            drop(page);
            frame.is_dirty = false;
        }

        drop(page_table);
        Ok(())
    }

    /// Allocate a new page in the storage engine and fetch it
    pub fn allocate_page(&self) -> Result<Arc<Mutex<Page>>> {
        let page_id = self.file_handler.allocate_page()?;
        self.fetch_page(page_id)
    }
}
