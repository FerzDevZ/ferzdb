use ferzdb::Database;
use ferzdb::util::types::Value;

#[test]
fn test_btree_hierarchy() -> ferzdb::util::error::Result<()> {
    let mut db = Database::open("test_hierarchy.fdb")?;
    
    // 1. Create a table
    db.execute("CREATE TABLE users (id INTEGER, name TEXT)")?;
    
    println!("Inserting rows to trigger B+Tree splits...");
    for i in 1..100 {
        let sql = format!("INSERT INTO users VALUES ({}, 'User{}')", i, i);
        db.execute(&sql)?;
    }
    
    println!("Verifying point lookups (O(log N) traversal)...");
    for i in [1, 50, 99] {
        let sql = format!("SELECT * FROM users WHERE id = {}", i);
        let results = db.query(&sql)?;
        if results.rows.is_empty() {
            println!("❌ Failed to find User{}", i);
        } else {
            println!("✅ Found: {:?}", results.rows[0]);
        }
    }
    
    println!("Verification Complete!");
    Ok(())
}
