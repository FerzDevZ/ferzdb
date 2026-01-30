//! Volcano-style Query Execution Engine

use crate::storage::ferz_engine::row::Row;
use crate::util::error::{FerzError, Result};
use crate::util::types::{Value, PageId};
use crate::sql::ast::{Expression, SelectColumn};
use crate::storage::catalog::TableInfo;

use std::sync::Arc;
use crate::storage::buffer_pool::BufferPoolManager;
use crate::storage::ferz_engine::row::RowSerializer;
use crate::storage::ferz_engine::page::PageType;

/// Core trait for all execution operators
pub trait Executor {
    fn next(&mut self) -> Result<Option<Row>>;
}

/// Scans a table using a clustered index
pub struct SeqScan {
    pub table_info: TableInfo,
    pub bpm: Arc<BufferPoolManager>,
    current_page_id: PageId,
    current_slot: u16,
}

impl SeqScan {
    pub fn new(table_info: TableInfo, bpm: Arc<BufferPoolManager>) -> Self {
        let mut current_page_id = table_info.root_page_id;
        
        // Traverse to the leftmost leaf
        loop {
            if let Ok(page_arc) = bpm.fetch_page(current_page_id) {
                let page = page_arc.lock().unwrap();
                match page.page_type() {
                    PageType::TableInternal | PageType::IndexInternal => {
                        current_page_id = page.initial_child_id();
                    }
                    _ => break, // Hit a leaf or uninitialized
                }
            } else {
                break;
            }
        }

        Self {
            table_info,
            bpm,
            current_page_id,
            current_slot: 0,
        }
    }
}

impl Executor for SeqScan {
    fn next(&mut self) -> Result<Option<Row>> {
        while self.current_page_id != PageId::INVALID {
            let page_arc = self.bpm.fetch_page(self.current_page_id)?;
            let page = page_arc.lock().unwrap();
            
            if self.current_slot < page.num_slots() {
                if let Some(record_bytes) = page.get_record(self.current_slot) {
                    self.current_slot += 1;
                    let (header, values) = RowSerializer::deserialize(record_bytes)?;
                    // For SeqScan in v0.3, we return the row if visible (placeholder: all visible)
                    // In a real DB, we'd check against a snapshot
                    return Ok(Some(Row { header, values }));
                } else {
                    self.current_slot += 1; // Skip deleted
                }
            } else {
                // Move to next page
                let next_id = page.next_page_id();
                if next_id == PageId::INVALID || next_id == PageId::HEADER_PAGE || next_id == PageId::SCHEMA_PAGE {
                    self.current_page_id = PageId::INVALID;
                } else {
                    self.current_page_id = next_id;
                }
                self.current_slot = 0;
            }
        }
        Ok(None)
    }
}

/// Filters rows based on a predicate
pub struct Filter {
    pub source: Box<dyn Executor>,
    pub predicate: Expression,
    pub table_info: TableInfo,
}

impl Filter {
    fn evaluate(&self, row: &Row, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::Column(name) => {
                let idx = self.table_info.columns.iter().position(|c| &c.name == name)
                    .ok_or_else(|| FerzError::Internal(format!("Column not found: {}", name)))?;
                Ok(row.values[idx].clone())
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate(row, left)?;
                let right_val = self.evaluate(row, right)?;
                
                match op {
                    crate::sql::ast::BinaryOperator::Eq => Ok(Value::Boolean(left_val == right_val)),
                    crate::sql::ast::BinaryOperator::Ne => Ok(Value::Boolean(left_val != right_val)),
                    // Add more operators as needed...
                    _ => Err(FerzError::Internal("Operator not implemented".to_string())),
                }
            }
        }
    }
}

impl Executor for Filter {
    fn next(&mut self) -> Result<Option<Row>> {
        while let Some(row) = self.source.next()? {
            match self.evaluate(&row, &self.predicate)? {
                Value::Boolean(true) => return Ok(Some(row)),
                _ => continue,
            }
        }
        Ok(None)
    }
}

/// Projects specific columns from the row
pub struct Projection {
    pub source: Box<dyn Executor>,
    pub columns: Vec<SelectColumn>,
}

impl Executor for Projection {
    fn next(&mut self) -> Result<Option<Row>> {
        if let Some(row) = self.source.next()? {
            let mut projected_values = Vec::new();
            
            for col in &self.columns {
                match col {
                    SelectColumn::All => {
                        projected_values = row.values.clone();
                        break;
                    }
                    SelectColumn::Named(_name) => {
                        // This assumes we have access to table_info to find index
                        // Since Projection doesn't have table_info, we might need to add it
                        // OR we assume SeqScan/Filter handled everything and we just return all for now
                        // Simplified for v0.5: just return all values for now
                        projected_values = row.values.clone();
                        break;
                    }
                }
            }
            
            return Ok(Some(Row {
                header: row.header,
                values: projected_values,
            }));
        }
        Ok(None)
    }
}
