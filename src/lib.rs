//! # FerzDB
//!
//! FerzDB is a modern SQL database with offline-first capabilities.

// --- Core Abstractions ---

/// The central trait for any storage engine used by FerzDB.
/// Allows swapping the internal B-Tree engine with other storage backends.
pub trait StorageEngine: Send + Sync {
    fn read_page(&self, page_id: u64) -> Result<Vec<u8>, FerzError>;
    fn write_page(&self, page_id: u64, data: &[u8]) -> Result<(), FerzError>;
    fn allocate_page(&self) -> Result<u64, FerzError>;
}

/// Interface for the SQL Execution layer.
pub trait QueryExecutor: Send + Sync {
    fn execute(&self, sql: &str) -> Result<ResultSet, FerzError>;
}

// --- Public API ---

pub use database::Database;
pub use util::error::FerzError;
pub use util::types::{DataType, Value, ResultSet};

// Internal modules
mod database;
pub mod sql;
pub mod storage;
pub mod transaction;
pub mod util;
pub mod ffi;

/// Convenience re-exports
pub mod prelude {
    pub use crate::database::Database;
    pub use crate::util::error::FerzError;
    pub use crate::util::types::{DataType, Value, ResultSet};
    pub use crate::{StorageEngine, QueryExecutor};
}
