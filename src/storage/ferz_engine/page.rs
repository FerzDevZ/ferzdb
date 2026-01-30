//! Slotted Page management
//! 
//! A slotted page is a format that allows variable-length records to be stored
//! in a fixed-size page. Records grow from the bottom, while slots grow from the top.

use crate::util::types::{PageId, PAGE_SIZE, PAGE_HEADER_SIZE};

/// Page types for identifying the content of a page
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    Unused = 0,
    TableLeaf = 1,
    TableInternal = 2,
    IndexLeaf = 3,
    IndexInternal = 4,
}

impl From<u8> for PageType {
    fn from(v: u8) -> Self {
        match v {
            1 => PageType::TableLeaf,
            2 => PageType::TableInternal,
            3 => PageType::IndexLeaf,
            4 => PageType::IndexInternal,
            _ => PageType::Unused,
        }
    }
}

/// Slotted Page structure
/// 
/// [ Header (64 bytes) ]
/// [ Slot Array        ]
/// [ ... Free Space ... ]
/// [ Row Data          ]
pub struct Page {
    pub id: PageId,
    pub data: [u8; PAGE_SIZE],
}

impl Page {
    pub fn new(id: PageId) -> Self {
        let mut data = [0u8; PAGE_SIZE];
        // Initialize header
        data[0] = PageType::Unused as u8;
        // Default next_page_id to INVALID (u32::MAX) at offset 8
        data[8..12].copy_from_slice(&0xFFFFFFFFu32.to_be_bytes());
        
        Self { id, data }
    }

    /// Page Type (offset 0)
    pub fn page_type(&self) -> PageType {
        PageType::from(self.data[0])
    }

    pub fn set_page_type(&mut self, t: PageType) {
        self.data[0] = t as u8;
    }

    /// Number of slots (offset 2-3)
    pub fn num_slots(&self) -> u16 {
        u16::from_be_bytes([self.data[2], self.data[3]])
    }

    pub fn set_num_slots(&mut self, n: u16) {
        let bytes = n.to_be_bytes();
        self.data[2] = bytes[0];
        self.data[3] = bytes[1];
    }

    /// Start of free space (offset 4-5)
    pub fn free_space_start(&self) -> u16 {
        u16::from_be_bytes([self.data[4], self.data[5]])
    }

    pub fn set_free_space_start(&mut self, offset: u16) {
        let bytes = offset.to_be_bytes();
        self.data[4] = bytes[0];
        self.data[5] = bytes[1];
    }

    /// End of free space / beginning of data (offset 6-7)
    pub fn free_space_end(&self) -> u16 {
        u16::from_be_bytes([self.data[6], self.data[7]])
    }

    pub fn set_free_space_end(&mut self, offset: u16) {
        let bytes = offset.to_be_bytes();
        self.data[6] = bytes[0];
        self.data[7] = bytes[1];
    }

    /// Next Page ID (for linked leaf nodes) (offset 8-11)
    pub fn next_page_id(&self) -> PageId {
        PageId(u32::from_be_bytes([self.data[8], self.data[9], self.data[10], self.data[11]]))
    }

    pub fn set_next_page_id(&mut self, id: PageId) {
        let bytes = id.0.to_be_bytes();
        self.data[8..12].copy_from_slice(&bytes);
    }

    /// Parent Page ID (offset 12-15)
    pub fn parent_id(&self) -> PageId {
        PageId(u32::from_be_bytes([self.data[12], self.data[13], self.data[14], self.data[15]]))
    }

    pub fn set_parent_id(&mut self, id: PageId) {
        let bytes = id.0.to_be_bytes();
        self.data[12..16].copy_from_slice(&bytes);
    }

    /// Initial Child ID (for internal nodes) (offset 16-19)
    pub fn initial_child_id(&self) -> PageId {
        PageId(u32::from_be_bytes([self.data[16], self.data[17], self.data[18], self.data[19]]))
    }

    pub fn set_initial_child_id(&mut self, id: PageId) {
        let bytes = id.0.to_be_bytes();
        self.data[16..20].copy_from_slice(&bytes);
    }

    /// Insert a record into the page
    pub fn insert_record(&mut self, record: &[u8]) -> Option<u16> {
        let n = self.num_slots();
        let _slot_offset = PAGE_HEADER_SIZE + (n as usize * 4);
        let free_start = self.free_space_start();
        let free_end = self.free_space_end();
        
        // If first insert, initialize free space boundaries
        let actual_start = if free_start == 0 { PAGE_HEADER_SIZE as u16 } else { free_start };
        let actual_end = if free_end == 0 { PAGE_SIZE as u16 } else { free_end };

        let record_size = record.len() as u16;
        let required_space = record_size + 4; // record + slot entry

        if (actual_end - actual_start) < required_space {
            return None; // No space
        }

        // Write record at the end of free space
        let record_offset = actual_end - record_size;
        self.data[record_offset as usize..(actual_end as usize)].copy_from_slice(record);

        // Write slot entry (offset, length)
        let slot_data_pos = PAGE_HEADER_SIZE + (n as usize * 4);
        self.data[slot_data_pos..slot_data_pos+2].copy_from_slice(&record_offset.to_be_bytes());
        self.data[slot_data_pos+2..slot_data_pos+4].copy_from_slice(&record_size.to_be_bytes());

        // Update headers
        self.set_num_slots(n + 1);
        self.set_free_space_start((slot_data_pos + 4) as u16);
        self.set_free_space_end(record_offset);

        Some(n)
    }

    /// Get record by slot ID
    pub fn get_record(&self, slot_id: u16) -> Option<&[u8]> {
        if slot_id >= self.num_slots() {
            return None;
        }

        let slot_pos = PAGE_HEADER_SIZE + (slot_id as usize * 4);
        let offset = u16::from_be_bytes([self.data[slot_pos], self.data[slot_pos+1]]) as usize;
        let length = u16::from_be_bytes([self.data[slot_pos+2], self.data[slot_pos+3]]) as usize;

        if offset == 0 { return None; } // Deleted

        Some(&self.data[offset..offset+length])
    }
}
