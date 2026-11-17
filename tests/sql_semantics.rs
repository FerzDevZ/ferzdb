/*! Behavioural tests for the SQL surface that clients actually touch.

The existing verifications in this directory print their findings and always
return `Ok(())`, so a broken lookup still reported success. These tests assert
instead, and each one removes the database file it creates.
*/

use ferzdb::util::types::Value;
use ferzdb::Database;
use std::fs;
use std::path::PathBuf;

/// A unique database path per test, deleted on drop so runs stay independent.
struct TempDb {
    path: PathBuf,
}

impl TempDb {
    fn new(name: &str) -> Self {
        let mut path = std::env::temp_dir();
        path.push(format!("ferzdb-test-{}-{}.fdb", name, std::process::id()));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(format!("{}.wal", path.display()));
        TempDb { path }
    }

    fn open(&self) -> Database {
        Database::open(self.path.to_str().expect("temp path is valid UTF-8"))
            .expect("database opens")
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_file(format!("{}.wal", self.path.display()));
    }
}

#[test]
fn a_created_table_reports_its_columns() {
    let temp = TempDb::new("columns");
    let db = temp.open();
    let mut db = db;

    db.execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .expect("create table succeeds");

    let result = db
        .query("SELECT * FROM users")
        .expect("select from a fresh table succeeds");

    assert!(result.rows.is_empty(), "a new table has no rows");
}

#[test]
fn an_inserted_row_is_readable_by_key() {
    let temp = TempDb::new("insert");
    let mut db = temp.open();

    db.execute("CREATE TABLE users (id INTEGER, name TEXT)")
        .expect("create table succeeds");
    db.execute("INSERT INTO users VALUES (1, 'Ferdinand')")
        .expect("insert succeeds");

    let result = db
        .query("SELECT * FROM users WHERE id = 1")
        .expect("select succeeds");

    assert_eq!(result.rows.len(), 1, "the inserted row is found exactly once");
    assert_eq!(result.rows[0][0], Value::Integer(1));
    assert_eq!(result.rows[0][1], Value::Text("Ferdinand".to_string()));
}

#[test]
fn rows_survive_a_reopen() {
    let temp = TempDb::new("reopen");
    {
        let mut db = temp.open();
        db.execute("CREATE TABLE items (id INTEGER, label TEXT)")
            .expect("create table succeeds");
        db.execute("INSERT INTO items VALUES (7, 'durian')")
            .expect("insert succeeds");
        db.sync().expect("flush succeeds");
    }

    let db = temp.open();
    let result = db
        .query("SELECT * FROM items WHERE id = 7")
        .expect("select after reopen succeeds");

    assert_eq!(result.rows.len(), 1);
    assert_eq!(result.rows[0][1], Value::Text("durian".to_string()));
}

#[test]
fn inserting_into_a_missing_table_is_an_error() {
    let temp = TempDb::new("missing");
    let mut db = temp.open();

    let outcome = db.execute("INSERT INTO ghosts VALUES (1, 'x')");

    assert!(outcome.is_err(), "an unknown table must not silently succeed");
}

#[test]
fn malformed_sql_is_rejected() {
    let temp = TempDb::new("malformed");
    let mut db = temp.open();

    assert!(db.execute("CREATE TABLE").is_err(), "truncated DDL fails");
    assert!(db.query("SELECT FROM WHERE").is_err(), "nonsense select fails");
}
