//! Query Planner - Converts AST into execution operators

use crate::sql::ast::Statement;
use crate::sql::executor::{Executor, SeqScan, Filter, Projection};
use crate::util::error::{FerzError, Result};

use crate::storage::catalog::Catalog;

use crate::storage::buffer_pool::BufferPoolManager;
use std::sync::Arc;

pub struct Planner<'a> {
    catalog: &'a Catalog,
    bpm: Arc<BufferPoolManager>,
}

impl<'a> Planner<'a> {
    pub fn new(catalog: &'a Catalog, bpm: Arc<BufferPoolManager>) -> Self {
        Self { catalog, bpm }
    }

    /// Plan a SELECT statement
    pub fn plan(&self, stmt: Statement) -> Result<Box<dyn Executor>> {
        match stmt {
            Statement::Select { columns, from, where_clause, .. } => {
                let table_info = self.catalog.get_table(&from)
                    .ok_or_else(|| FerzError::Internal(format!("Table not found: {}", from)))?
                    .clone();

                let mut exec: Box<dyn Executor> = Box::new(SeqScan::new(table_info.clone(), self.bpm.clone()));
                
                if let Some(predicate) = where_clause {
                    exec = Box::new(Filter { 
                        source: exec, 
                        predicate: predicate,
                        table_info: table_info.clone(),
                    });
                }
                
                exec = Box::new(Projection { source: exec, columns });
                Ok(exec)
            }
            _ => Err(FerzError::Internal("Only SELECT is currently planned".to_string())),
        }
    }
}
