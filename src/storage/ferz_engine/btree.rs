//! B+Tree Index Implementation
//! 
//! This module implements a B+Tree for clustered and secondary indices.
//! It interacts with the BufferPoolManager to fetch and modify pages.

use std::sync::Arc;
use crate::util::error::{FerzError, Result};
use crate::util::types::{PageId, Value};
use crate::storage::buffer_pool::BufferPoolManager;
use crate::storage::ferz_engine::page::PageType;

use crate::transaction::txn::Snapshot;
use crate::storage::ferz_engine::row::{RowHeader, RowSerializer};

use crate::storage::wal::{WalManager, LogType};
use std::sync::Mutex;

pub struct BTreeIndex {
    root_page_id: Arc<Mutex<PageId>>,
    bpm: Arc<BufferPoolManager>,
    wal: Arc<WalManager>,
    pub is_clustered: bool,
}

impl BTreeIndex {
    pub fn new(root_page_id: PageId, bpm: Arc<BufferPoolManager>, wal: Arc<WalManager>, is_clustered: bool) -> Self {
        Self { 
            root_page_id: Arc::new(Mutex::new(root_page_id)), 
            bpm, 
            wal, 
            is_clustered 
        }
    }
    
    pub fn root_id(&self) -> PageId {
        *self.root_page_id.lock().unwrap()
    }

    /// Find a record by its key with MVCC visibility checks
    pub fn find(&self, key: &Value, snapshot: &Snapshot) -> Result<Option<Vec<Value>>> {
        let _root_page = self.bpm.fetch_page(self.root_id())?;
        
        // 1. Traverse to leaf (Skeleton)
        let leaf_page_id = self.find_leaf(key)?;
        let leaf_page_arc = self.bpm.fetch_page(leaf_page_id)?;
        let leaf_page = leaf_page_arc.lock().unwrap();

        // 2. Search in slotted page
        for i in 0..leaf_page.num_slots() {
            if let Some(record_bytes) = leaf_page.get_record(i) {
                let (header, values) = RowSerializer::deserialize(record_bytes)?;
                
                // --- Identity Check ---
                if values.is_empty() || values[0] != *key {
                    continue;
                }

                // --- MVCC Visibility Logic ---
                if snapshot.is_visible(header.xid) {
                    // This version is visible!
                    return Ok(Some(values));
                } else if header.rollback_ptr != 0 {
                    // TODO: Follow rollback pointer to Undo Log for older version
                }
            }
        }
        
        Ok(None)
    }

    /// Insert a record with transaction context and B+Tree balancing
    pub fn insert(&self, key: &Value, values: &[Value], xid: u64) -> Result<()> {
        let header = RowHeader {
            xid,
            rollback_ptr: 0,
        };
        let record = RowSerializer::serialize(&header, values);
        
        // --- Write-Ahead Logging ---
        // Log the insert BEFORE modifying the data pages
        self.wal.append(xid, LogType::Insert, PageId::INVALID, record.clone())?;
        
        let leaf_page_id = self.find_leaf(key)?;
        let leaf_page_arc = self.bpm.fetch_page(leaf_page_id)?;
        
        if let Some(_) = {
            let mut leaf_page = leaf_page_arc.lock().unwrap();
            let slot = leaf_page.insert_record(&record);
            if slot.is_some() {
                self.bpm.mark_dirty(leaf_page_id)?;
                // Log the commit for this specific insertion (simplified txn)
                self.wal.append(xid, LogType::Commit, PageId::INVALID, vec![])?;
            }
            slot
        } {
            return Ok(());
        }

        // --- Node is full, perform split ---
        let new_leaf_id = self.split_node(leaf_page_id)?;
        println!("DEBUG: Splitting leaf {} -> new leaf {}", leaf_page_id, new_leaf_id);

        // Re-decide which page to insert into based on the key
        // Simplified v0.3: just insert into the new page if it was a split
        let target_page_id = new_leaf_id;
        let target_page_arc = self.bpm.fetch_page(target_page_id)?;
        let mut target_page = target_page_arc.lock().unwrap();
        target_page.insert_record(&record);
        
        self.bpm.mark_dirty(target_page_id)?;
        self.bpm.unpin_page(target_page_id)?;
        self.bpm.unpin_page(leaf_page_id)?;
        
        Ok(())
    }

    fn find_leaf(&self, key: &Value) -> Result<PageId> {
        let mut curr_id = self.root_id();
        
        loop {
            let page_arc = self.bpm.fetch_page(curr_id)?;
            let page = page_arc.lock().unwrap();
            
            match page.page_type() {
                PageType::TableLeaf | PageType::IndexLeaf => return Ok(curr_id),
                PageType::TableInternal | PageType::IndexInternal => {
                    // Internal Node: entries are [Key, ChildPageId]
                    // If Key <= entry_key, go to this child?
                    // Standard B+Tree: Child_i contains keys < Key_i+1
                    
                    let mut found_child = page.initial_child_id();
                    
                    for i in 0..page.num_slots() {
                        if let Some(entry_bytes) = page.get_record(i) {
                            // Internal entry: [Serialized Key | 4-byte PageId]
                            let key_len = entry_bytes.len() - 4;
                            let entry_key = crate::storage::ferz_engine::row::RowSerializer::deserialize_value(&entry_bytes[..key_len])?;
                            let child_id = PageId(u32::from_be_bytes([
                                entry_bytes[key_len], entry_bytes[key_len+1], 
                                entry_bytes[key_len+2], entry_bytes[key_len+3]
                            ]));
                            
                            if key < &entry_key {
                                // key is smaller than this pivot, use previous child (already in found_child)
                                break;
                            }
                            found_child = child_id;
                        }
                    }
                    curr_id = found_child;
                }
                _ => return Err(FerzError::Internal(format!("Unexpected page type: {:?}", page.page_type()))),
            }
        }
    }

    /// Delete a record from the B-Tree
    pub fn delete(&self, _key: &Value, _xid: u64) -> Result<()> {
        // 1. Find leaf page containing the key
        // 2. Remove from slotted page
        // 3. If page underflow (< 50% full), handle merge/redistribution
        
        Ok(())
    }

    /// Redistribution logic (stealing from sibling)
    fn redistribute(&self, _page_id: PageId, _sibling_id: PageId) -> Result<()> {
        Ok(())
    }

    /// Merge two nodes into one
    fn merge_nodes(&self, _page_id: PageId, _sibling_id: PageId) -> Result<()> {
        Ok(())
    }

    /// Split a node (Internal or Leaf) and promote key to parent
    fn split_node(&self, page_id: PageId) -> Result<PageId> {
        let old_page_arc = self.bpm.fetch_page(page_id)?;
        let new_page_arc = self.bpm.allocate_page()?;
        let new_page_id = { new_page_arc.lock().unwrap().id };

        let mut old_page = old_page_arc.lock().unwrap();
        let mut new_page = new_page_arc.lock().unwrap();
        
        // 1. Set type and basic headers
        let page_type = old_page.page_type();
        new_page.set_page_type(page_type);
        new_page.set_parent_id(old_page.parent_id());
        
        // 2. Move half of the entries
        let num_slots = old_page.num_slots();
        let mid = num_slots / 2;
        
        let mut split_key = Value::Null;

        if page_type == PageType::TableLeaf || page_type == PageType::IndexLeaf {
            // Leaf Split: Keep split key in new leaf
            new_page.set_next_page_id(old_page.next_page_id());
            old_page.set_next_page_id(new_page_id);
            
            for i in mid..num_slots {
                if let Some(record) = old_page.get_record(i) {
                    new_page.insert_record(record);
                }
            }
            
            // Get the first key of the new page for promotion
            if let Some(record) = new_page.get_record(0) {
                let (_, values) = RowSerializer::deserialize(record)?;
                split_key = values[0].clone(); // Assuming first col is PK
            }
        } else {
            // Internal Split: Move entries, middle key goes to parent
            // First entry in new page is special (initial_child)
            // Initial child of new page is the child pointer of the "split entry"
            
            // 1. Get the entry that will be promoted
            let promote_entry = old_page.get_record(mid).unwrap();
            let key_len = promote_entry.len() - 4;
            split_key = RowSerializer::deserialize_value(&promote_entry[..key_len])?;
            let promoted_child = PageId(u32::from_be_bytes([
                promote_entry[key_len], promote_entry[key_len+1], 
                promote_entry[key_len+2], promote_entry[key_len+3]
            ]));
            
            new_page.set_initial_child_id(promoted_child);

            // 2. Move remaining entries
            for i in (mid+1)..num_slots {
                if let Some(record) = old_page.get_record(i) {
                    new_page.insert_record(record);
                }
            }
        }

        old_page.set_num_slots(mid);
        self.bpm.mark_dirty(page_id)?;
        self.bpm.mark_dirty(new_page_id)?;

        // 3. Promote split_key to parent
        let parent_id = old_page.parent_id();
        drop(old_page);
        drop(new_page);
        
        if page_id == self.root_id() {
            self.create_new_root(split_key, page_id, new_page_id)?;
        } else {
            self.insert_into_internal(parent_id, split_key, new_page_id)?;
        }

        Ok(new_page_id)
    }

    fn create_new_root(&self, key: Value, left_id: PageId, right_id: PageId) -> Result<()> {
        // Root split is special: it's the only way height increases
        // Actually, we usually keep root_page_id fixed for convenience, 
        // but it's easier to just allocate a new one and update the catalog/manager.
        // Wait, for now let's just use the root_page_id from BTreeIndex.
        // If we change it, we need to update the catalog.
        
        // This is a bit complex for a simple v0.5. 
        // Shortcut: Root split creates a new internal node and that becomes the new root.
        let new_root_arc = self.bpm.allocate_page()?;
        let new_root_id = { new_root_arc.lock().unwrap().id };
        
        {
            let mut page = new_root_arc.lock().unwrap();
            page.set_page_type(PageType::TableInternal);
            page.set_initial_child_id(left_id);
            
            let mut entry = RowSerializer::serialize_value(&key);
            entry.extend_from_slice(&right_id.0.to_be_bytes());
            page.insert_record(&entry);
        }
        
        // Update parents of children
        self.bpm.fetch_page(left_id)?.lock().unwrap().set_parent_id(new_root_id);
        self.bpm.fetch_page(right_id)?.lock().unwrap().set_parent_id(new_root_id);
        
        // --- CRITICAL ---
        // Update the root_page_id in the index instance
        {
            let mut root_id = self.root_page_id.lock().unwrap();
            *root_id = new_root_id;
        }
        
        println!("DEBUG: Tree height increased. New root: {}", new_root_id);
        Ok(())
    }

    fn insert_into_internal(&self, page_id: PageId, key: Value, child_id: PageId) -> Result<()> {
        let page_arc = self.bpm.fetch_page(page_id)?;
        let mut page = page_arc.lock().unwrap();
        
        let mut entry = RowSerializer::serialize_value(&key);
        entry.extend_from_slice(&child_id.0.to_be_bytes());
        
        if let Some(_) = page.insert_record(&entry) {
            self.bpm.mark_dirty(page_id)?;
            Ok(())
        } else {
            // Parent is also full! Recursive split
            drop(page);
            self.split_node(page_id)?;
            // After split, search for the correct node again or propagate?
            // Simplified: just re-call find_leaf for the key if we split parent.
            // But here we are inserting into internal, so we need to be careful.
            Ok(())
        }
    }
}

/// Orchestrates multiple indices (Clustered + Secondary) for a single table
pub struct IndexManager {
    pub table_name: String,
    pub clustered_index: BTreeIndex,
    pub secondary_indices: std::collections::HashMap<String, BTreeIndex>,
    wal: Arc<WalManager>,
}

impl IndexManager {
    pub fn new(table_name: &str, clustered_index: BTreeIndex, wal: Arc<WalManager>) -> Self {
        Self {
            table_name: table_name.to_string(),
            clustered_index,
            secondary_indices: std::collections::HashMap::new(),
            wal,
        }
    }

    pub fn insert_row(&self, values: &[Value], xid: u64) -> Result<()> {
        // 1. Insert into Clustered Index (data)
        // PK is usually the first value for v0.2
        let pk = &values[0];
        self.clustered_index.insert(pk, values, xid)?;

        // 2. Update all secondary indices with (IndexKey -> PK)
        for (_name, index) in &self.secondary_indices {
            // Placeholder: assume secondary index on second column
            let secondary_key = &values[1];
            index.insert(secondary_key, &[pk.clone()], xid)?;
        }
        
        Ok(())
    }
}

pub struct BTree {
    // Will be implemented in next phase
}
