//! Common type definitions for FerzDB

use std::fmt;
use serde::{Serialize, Deserialize};

/// Page identifier (u32 allows ~4 billion pages = 32TB at 8KB/page)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PageId(pub u32);

impl PageId {
    pub const INVALID: PageId = PageId(u32::MAX);
    pub const HEADER_PAGE: PageId = PageId(0);
    pub const SCHEMA_PAGE: PageId = PageId(1);
    
    pub fn new(id: u32) -> Self {
        PageId(id)
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Page#{}", self.0)
    }
}

/// Transaction identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub u64);

impl TransactionId {
    pub fn new(id: u64) -> Self {
        TransactionId(id)
    }
}

/// Row identifier (combines page and slot)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RowId(pub u64);

impl RowId {
    pub fn new(page_id: u32, slot: u16) -> Self {
        let id = ((page_id as u64) << 32) | (slot as u64);
        RowId(id)
    }
    
    pub fn page_id(&self) -> u32 {
        (self.0 >> 32) as u32
    }
    
    pub fn slot(&self) -> u16 {
        (self.0 & 0xFFFF) as u16
    }
}

/// SQL data types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Blob,
    Boolean,
}

impl DataType {
    /// Returns the fixed size in bytes, or None for variable-length types
    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            DataType::Integer => Some(8),
            DataType::Float => Some(8),
            DataType::Boolean => Some(1),
            DataType::Text | DataType::Blob => None,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Blob => write!(f, "BLOB"),
            DataType::Boolean => write!(f, "BOOLEAN"),
        }
    }
}

/// Runtime value
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
    Null,
}

impl Value {
    /// Returns the data type of this value
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Value::Integer(_) => Some(DataType::Integer),
            Value::Float(_) => Some(DataType::Float),
            Value::Text(_) => Some(DataType::Text),
            Value::Blob(_) => Some(DataType::Blob),
            Value::Boolean(_) => Some(DataType::Boolean),
            Value::Null => None,
        }
    }
    
    /// Checks if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Text(s) => write!(f, "'{}'", s),
            Value::Blob(b) => write!(f, "<blob {} bytes>", b.len()),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "NULL"),
        }
    }
}

/// Page size constant (8KB)
pub const PAGE_SIZE: usize = 8192;

/// Page header size (64 bytes)
pub const PAGE_HEADER_SIZE: usize = 64;

/// Magic number for database files: "FERZDB\0\0"
pub const DB_MAGIC: &[u8; 8] = b"FERZDB\0\0";

/// Magic number for WAL files: "FERZWAL\0"
pub const WAL_MAGIC: &[u8; 8] = b"FERZWAL\0";

/// Database version
pub const DB_VERSION: u16 = 1;

/// Result set from a query
#[derive(Debug, Clone, PartialEq)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pos: usize,
}

impl ResultSet {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Value>>) -> Self {
        ResultSet { columns, rows, pos: 0 }
    }
    
    pub fn empty() -> Self {
        ResultSet { columns: vec![], rows: vec![], pos: 0 }
    }

    pub fn add_row(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }
}

impl Iterator for ResultSet {
    type Item = Vec<Value>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.rows.len() {
            let row = self.rows[self.pos].clone();
            self.pos += 1;
            Some(row)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_id() {
        let page = PageId::new(42);
        assert_eq!(page.as_u32(), 42);
        assert_eq!(format!("{}", page), "Page#42");
    }

    #[test]
    fn test_row_id() {
        let row = RowId::new(100, 5);
        assert_eq!(row.page_id(), 100);
        assert_eq!(row.slot(), 5);
    }

    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Integer.fixed_size(), Some(8));
        assert_eq!(DataType::Float.fixed_size(), Some(8));
        assert_eq!(DataType::Boolean.fixed_size(), Some(1));
        assert_eq!(DataType::Text.fixed_size(), None);
        assert_eq!(DataType::Blob.fixed_size(), None);
    }

    #[test]
    fn test_value_type() {
        let v = Value::Integer(42);
        assert_eq!(v.data_type(), Some(DataType::Integer));
        assert!(!v.is_null());
        
        let v = Value::Null;
        assert_eq!(v.data_type(), None);
        assert!(v.is_null());
    }
}
