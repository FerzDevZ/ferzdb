//! FerzDB Network Server (Daemon)
//! 
//! This binary provides a TCP interface for FerzDB, allowing remote clients
//! to execute SQL commands and queries.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread;
use ferzdb::Database;
use ferzdb::util::error::Result;
use std::sync::Mutex;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    // Create data directory if it doesn't exist
    if !Path::new("data").exists() {
        fs::create_dir("data").unwrap();
    }

    let listener = TcpListener::bind("127.0.0.1:5432")?;
    println!("FerzDB Server (XAMPP-style) listening on 127.0.0.1:5432");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Client error: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {:?}", e),
        }
    }

    Ok(())
}

use ferzdb::storage::auth::{Session, AuthManager};
use serde_json::Value as JsonValue;

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0u8; 4096];
    let mut session = Session::new();
    let mut current_db: Option<Database> = None;
    
    loop {
        let n = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        
        let input = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
        if input.is_empty() { continue; }

        if !session.authenticated {
            // --- Stage 1: Handshake (JSON expected) ---
            match serde_json::from_slice::<JsonValue>(&buffer[..n]) {
                Ok(json) => {
                    let user = json["user"].as_str().unwrap_or("");
                    let pass = json["pass"].as_str().unwrap_or("");
                    let target_db_name = json["db"].as_str();

                    if AuthManager::verify(user, pass)? {
                        session.username = user.to_string();
                        session.authenticated = true;
                        
                        // Optional auto-select DB if provided
                        if let Some(db_name) = target_db_name {
                            let path = format!("data/{}.fdb", db_name);
                            if Path::new(&path).exists() {
                                current_db = Some(Database::open(&path)?);
                                session.current_db = db_name.to_string();
                            }
                        }

                        stream.write_all(b"AUTH_OK\n")?;
                        println!("User '{}' authenticated globally", user);
                    } else {
                        stream.write_all(b"AUTH_FAILED\n")?;
                        return Ok(());
                    }
                }
                Err(_) => {
                    stream.write_all(b"ERR: Handshake required. Send JSON login.\n")?;
                    return Ok(());
                }
            }
            continue;
        }

        // --- Stage 2: SQL Execution & Intercepted Commands ---
        let cmd_upper = input.to_uppercase();
        
        let response = if cmd_upper == "SHOW DATABASES" {
            let mut dbs = Vec::new();
            if let Ok(entries) = fs::read_dir("data") {
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.ends_with(".fdb") {
                            dbs.push(name.replace(".fdb", ""));
                        }
                    }
                }
            }
            format!("Databases:\n{:?}\n", dbs)
        } else if cmd_upper == "SHOW TABLES" {
            if let Some(db) = &current_db {
                let tables = db.catalog.table_names();
                format!("Tables in {}:\n{:?}\n", session.current_db, tables)
            } else {
                "Error: No database selected.\n".to_string()
            }
        } else if cmd_upper.starts_with("SHOW SCHEMA") {
            if let Some(db) = &current_db {
                let table_name = input.split_whitespace().nth(2).unwrap_or("").trim_matches(';');
                if let Some(info) = db.catalog.get_table(table_name) {
                    format!("Schema for {}:\n{:?}\n", table_name, info.columns)
                } else {
                    format!("Error: Table '{}' not found.\n", table_name)
                }
            } else {
                "Error: No database selected.\n".to_string()
            }
        } else if cmd_upper.starts_with("CREATE DATABASE") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() >= 3 {
                let db_name = parts[2].trim_matches(';');
                let path = format!("data/{}.fdb", db_name);
                if !Path::new(&path).exists() {
                    let _ = Database::open(&path)?; // Creates the file
                    format!("Database '{}' created successfully.\n", db_name)
                } else {
                    format!("Error: Database '{}' already exists.\n", db_name)
                }
            } else {
                "Error: Invalid syntax. Use CREATE DATABASE name;\n".to_string()
            }
        } else if cmd_upper.starts_with("USE") {
            let db_name = input.split_whitespace().nth(1).unwrap_or("").trim_matches(';');
            let path = format!("data/{}.fdb", db_name);
            if Path::new(&path).exists() {
                current_db = Some(Database::open(&path)?);
                session.current_db = db_name.to_string();
                format!("Switched to database '{}'.\n", db_name)
            } else {
                format!("Error: Database '{}' not found.\n", db_name)
            }
        } else if cmd_upper.starts_with("SELECT") {
            if let Some(db) = &current_db {
                match db.query(&input) {
                    Ok(results) => {
                        let mut output = format!("Database: {}\nColumns: {:?}\n", session.current_db, results.columns);
                        for row in results.rows {
                            output.push_str(&format!("{:?}\n", row));
                        }
                        output
                    }
                    Err(e) => format!("Query Error: {:?}\n", e),
                }
            } else {
                "Error: No database selected. Use 'USE <db>;' first.\n".to_string()
            }
        } else {
            if let Some(mut db) = current_db.take() {
                let res = match db.execute(&input) {
                    Ok(_) => "Command executed successfully.\n".to_string(),
                    Err(e) => format!("Execution Error: {:?}\n", e),
                };
                current_db = Some(db); // Return it back
                res
            } else {
                "Error: No database selected. Use 'USE <db>;' first.\n".to_string()
            }
        };

        if let Err(_) = stream.write_all(response.as_bytes()) {
            break;
        }
    }
    
    Ok(())
}
