//! Lock Manager for concurrency control
//! 
//! Implements table-level and row-level locking to ensure isolation.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, Condvar};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum LockResource {
    Table(String),
    Row(String, u64), // Table name + Row ID
}

pub enum LockMode {
    Shared,
    Exclusive,
}

struct LockRequest {
    tx_id: u64,
    mode: LockMode,
}

pub struct LockManager {
    locks: Mutex<HashMap<LockResource, Vec<LockRequest>>>,
    cv: Condvar,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
            cv: Condvar::new(),
        }
    }

    pub fn acquire_lock(&self, tx_id: u64, resource: LockResource, mode: LockMode) -> bool {
        let mut locks = self.locks.lock().unwrap();
        
        // Very simplified lock acquisition logic (no deadlock detection yet)
        let requests = locks.entry(resource).or_insert_with(Vec::new);
        
        // For v0.1: Immediate success if no other locks, else wait/fail
        if requests.is_empty() {
            requests.push(LockRequest { tx_id, mode });
            return true;
        }
        
        false
    }

    pub fn release_lock(&self, tx_id: u64, resource: LockResource) {
        let mut locks = self.locks.lock().unwrap();
        if let Some(requests) = locks.get_mut(&resource) {
            requests.retain(|r| r.tx_id != tx_id);
            if requests.is_empty() {
                locks.remove(&resource);
                self.cv.notify_all();
            }
        }
    }
}
