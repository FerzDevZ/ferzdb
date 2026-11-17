//! Database API - Main entry point for embedded mode

use std::sync::Arc;
use crate::util::error::Result;
use crate::util::types::{ResultSet, PageId};
use crate::storage::buffer_pool::BufferPoolManager;
use crate::storage::ferz_engine::file::FileHandler;
use crate::sql::parser::Parser;
use crate::sql::planner::Planner;
use crate::transaction::txn::TransactionManager;
use crate::storage::catalog::Catalog;
use crate::storage::wal::{WalManager, LogType};

/// Main Database struct for embedded mode
pub struct Database {
    _path: String,
    bpm: Arc<BufferPoolManager>,
    txn_manager: Arc<TransactionManager>,
    pub catalog: Catalog,
    wal: Arc<WalManager>,
}

impl Database {
    /// Create or open a database at the specified path
    pub fn open(path: &str) -> Result<Self> {
        let wal_path = format!("{}.wal", path);
        let wal = Arc::new(WalManager::open(&wal_path)?);
        
        let file_handler = Arc::new(FileHandler::open(path)?);
        let num_pages = file_handler.num_pages()?;
        
        let bpm = Arc::new(BufferPoolManager::new(file_handler, 1024));
        let txn_manager = Arc::new(TransactionManager::new());
        
        let mut db = Database {
            _path: path.to_string(),
            bpm,
            txn_manager,
            catalog: Catalog::new(),
            wal,
        };

        if num_pages > 1 {
            db.load_catalog()?;
            db.recover()?;
        } else if num_pages == 0 {
            // New database, allocate header and schema pages
            db.bpm.allocate_page()?; // Page 0: Header
            db.bpm.allocate_page()?; // Page 1: Schema
        }
        
        Ok(db)
    }

    fn recover(&self) -> Result<()> {
        let records = self.wal.recover()?;
        for record in records {
            match record.log_type {
                LogType::Insert => {
                    // 1. Fetch the page where the record was supposed to be
                    // In v0.4, we might need a more sophisticated way to find the leaf
                    // if it split, but for now we apply to the recorded page_id
                    // if page_id is INVALID, it's a fresh table insert (first page)
                    if record.page_id == PageId::INVALID {
                        // Re-fetch root from catalog
                        // This is simplified; real recovery uses Page LSNs
                        continue; 
                    }
                    
                    let page_arc = self.bpm.fetch_page(record.page_id)?;
                    let mut page = page_arc.lock().unwrap();
                    // Basic idempotency: if record already exists, skip or re-insert
                    // (Real DBs use Page LSN < Record LSN)
                    let _ = page.insert_record(&record.data);
                    self.bpm.mark_dirty(record.page_id)?;
                }
                LogType::Commit => {
                    // Mark transaction as committed in CLOG
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn load_catalog(&mut self) -> Result<()> {
        let page_arc = self.bpm.fetch_page(PageId::SCHEMA_PAGE)?;
        let page = page_arc.lock().unwrap();
        // Simplified: assume catalog fits in one page for v0.3
        self.catalog = Catalog::deserialize(&page.data)?;
        Ok(())
    }

    fn save_catalog(&self) -> Result<()> {
        let data = self.catalog.serialize()?;
        let page_arc = self.bpm.fetch_page(PageId::SCHEMA_PAGE)?;
        {
            let mut page = page_arc.lock().unwrap();
            let len = data.len().min(crate::util::types::PAGE_SIZE);
            page.data[..len].copy_from_slice(&data[..len]);
        }
        self.bpm.mark_dirty(PageId::SCHEMA_PAGE)?;
        Ok(())
    }
    
    /// Execute a SQL command (INSERT, UPDATE, DELETE, CREATE, etc.)
    pub fn execute(&mut self, sql: &str) -> Result<()> {
        let txn = self.txn_manager.begin();
        
        let mut parser = Parser::new(sql)?;
        let stmt = parser.parse()?;
        
        match stmt {
            crate::sql::ast::Statement::CreateTable { name, columns } => {
                // ... Existing CREATE TABLE logic ...
                let root_page_arc = self.bpm.allocate_page()?;
                let root_page_id = {
                    let mut root_page = root_page_arc.lock().unwrap();
                    root_page.set_page_type(crate::storage::ferz_engine::page::PageType::TableLeaf);
                    root_page.set_next_page_id(PageId::INVALID);
                    root_page.id
                };
                self.bpm.mark_dirty(root_page_id)?;

                let col_infos = columns.into_iter().map(|c| {
                    crate::storage::catalog::ColumnInfo {
                        name: c.name,
                        data_type: c.data_type,
                        is_primary: c.primary_key,
                        is_nullable: !c.not_null,
                    }
                }).collect();

                let table_info = crate::storage::catalog::TableInfo {
                    name: name.clone(),
                    columns: col_infos,
                    root_page_id,
                };

                self.catalog.add_table(table_info);
                self.save_catalog()?;
                println!("Table '{}' created with root page {}", name, root_page_id);
            }
            crate::sql::ast::Statement::Insert { table, values, .. } => {
                let table_info = self.catalog.get_table(&table)
                    .ok_or_else(|| crate::util::error::FerzError::Internal(format!("Table not found: {}", table)))?;
                
                let btree = crate::storage::ferz_engine::btree::BTreeIndex::new(
                    table_info.root_page_id, 
                    self.bpm.clone(), 
                    self.wal.clone(),
                    true
                );

                // Convert Expression::Literal to Value
                let row_values: Vec<crate::util::types::Value> = values.into_iter().map(|v| match v {
                    crate::sql::ast::Expression::Literal(val) => val,
                    _ => crate::util::types::Value::Null,
                }).collect();

                btree.insert(&row_values[0], &row_values, txn.xid)?;
                
                // Update Catalog if root changed (tree height increased)
                let new_root = btree.root_id();
                if new_root != table_info.root_page_id {
                    let mut updated_info = table_info.clone();
                    updated_info.root_page_id = new_root;
                    self.catalog.add_table(updated_info);
                    self.save_catalog()?;
                }
                
                println!("Row inserted into '{}'", table);
            }
            _ => {
                // Other statements handled by planner/executor
            }
        }
        
        self.txn_manager.commit(txn);
        Ok(())
    }

    /// Execute a SQL query (SELECT)
    pub fn query(&self, sql: &str) -> Result<ResultSet> {
        let _txn = self.txn_manager.begin();
        
        let mut parser = Parser::new(sql)?;
        let stmt = parser.parse()?;
        
        // Extract column names for the result set
        let columns = match &stmt {
            crate::sql::ast::Statement::Select { columns, .. } => {
                columns.iter().map(|col| match col {
                    crate::sql::ast::SelectColumn::All => "*".to_string(),
                    crate::sql::ast::SelectColumn::Named(n) => n.clone(),
                }).collect()
            }
            _ => vec![],
        };

        let planner = Planner::new(&self.catalog, self.bpm.clone());
        let mut executor = planner.plan(stmt)?;
        
        let mut result_set = ResultSet::new(columns, vec![]);
        
        // Execute and collect results
        while let Some(row) = executor.next()? {
            result_set.add_row(row.values);
        }
        
        Ok(result_set)
    }

    /// Flush all caches to disk.
    ///
    /// This was a stub, so `sync()` reported success while every dirty page —
    /// including the catalog — stayed in memory and died with the process.
    pub fn sync(&self) -> Result<()> {
        self.bpm.flush_all()?;
        Ok(())
    }
}
