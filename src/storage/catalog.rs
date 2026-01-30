//! System Catalog for storing database metadata (tables, columns, root pages)

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::util::error::Result;
use crate::util::types::{DataType, PageId};

/// Metadata for a single column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: DataType,
    pub is_primary: bool,
    pub is_nullable: bool,
}

/// Metadata for a table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub root_page_id: PageId,
}

/// The System Catalog maintains the schema of all tables
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Catalog {
    pub tables: HashMap<String, TableInfo>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_table(&mut self, info: TableInfo) {
        self.tables.insert(info.name.clone(), info);
    }

    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.tables.get(name)
    }

    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    /// Serialize catalog to bytes for storage in Page 0 (simplified)
    pub fn serialize(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| crate::util::error::FerzError::Internal(e.to_string()))
    }

    /// Deserialize catalog from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        // Trim null bytes (common in uninitialized pages)
        let trimmed = match data.iter().position(|&b| b == 0) {
            Some(0) => return Ok(Self::default()), // Starts with null, empty catalog
            Some(pos) => &data[..pos],
            None => data,
        };

        if trimmed.is_empty() {
            return Ok(Self::default());
        }

        serde_json::from_slice(trimmed).map_err(|e| crate::util::error::FerzError::Internal(e.to_string()))
    }
}
