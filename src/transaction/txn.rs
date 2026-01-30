use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

static GLOBAL_XID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents a point-in-time view of the database
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub min_xid: u64,           // Any XID < min_xid is committed
    pub max_xid: u64,           // Any XID >= max_xid is in progress/future
    pub active_xids: HashSet<u64>, // List of XIDs currently active (not committed)
}

impl Snapshot {
    pub fn is_visible(&self, xid: u64) -> bool {
        if xid < self.min_xid {
            return true;
        }
        if xid >= self.max_xid {
            return false;
        }
        !self.active_xids.contains(&xid)
    }
}

pub struct Transaction {
    pub xid: u64,
    pub snapshot: Snapshot,
}

impl Transaction {
    pub fn new(active_xids: HashSet<u64>) -> Self {
        let xid = GLOBAL_XID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let min_xid = active_xids.iter().min().cloned().unwrap_or(xid);
        
        Self {
            xid,
            snapshot: Snapshot {
                min_xid,
                max_xid: xid,
                active_xids,
            },
        }
    }
}

pub struct TransactionManager {
    active_xids: Mutex<HashSet<u64>>,
}

use std::sync::Mutex;

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_xids: Mutex::new(HashSet::new()),
        }
    }

    pub fn begin(&self) -> Transaction {
        let mut active = self.active_xids.lock().unwrap();
        let txn = Transaction::new(active.clone());
        active.insert(txn.xid);
        txn
    }

    pub fn commit(&self, txn: Transaction) {
        let mut active = self.active_xids.lock().unwrap();
        active.remove(&txn.xid);
        // In a real DB, we'd persist the commit in the WAL/CLOG here
    }

    pub fn rollback(&self, txn: Transaction) {
        let mut active = self.active_xids.lock().unwrap();
        active.remove(&txn.xid);
        // In a real DB, we'd mark the XID as aborted
    }
}
