//! FerzDB CLI Client

#[cfg(feature = "cli")]
use std::io::{self, Read, Write};
#[cfg(feature = "cli")]
use std::net::TcpStream;
#[cfg(feature = "cli")]
use serde_json::json;
#[cfg(feature = "cli")]
use ferzdb::{FerzError, Database, ResultSet};

#[cfg(feature = "cli")]
fn main() -> Result<(), FerzError> {
    println!("FerzDB CLI v0.6.0 (Security Update)");
    println!("1. Local Mode (ferz.fdb)");
    println!("2. Remote Login (XAMPP-style)");
    print!("Choose mode [1]: ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    if choice.trim() == "2" {
        run_remote_mode()
    } else {
        run_local_mode()
    }
}

#[cfg(feature = "cli")]
fn run_remote_mode() -> Result<(), FerzError> {
    print!("Server IP [127.0.0.1:5432]: ");
    io::stdout().flush().unwrap();
    let mut ip = String::new();
    io::stdin().read_line(&mut ip).unwrap();
    let ip = if ip.trim().is_empty() { "127.0.0.1:5432" } else { ip.trim() };

    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut user = String::new();
    io::stdin().read_line(&mut user).unwrap();
    let user = user.trim();

    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut pass = String::new();
    io::stdin().read_line(&mut pass).unwrap();
    let pass = pass.trim();

    print!("Database: ");
    io::stdout().flush().unwrap();
    let mut db_name = String::new();
    io::stdin().read_line(&mut db_name).unwrap();
    let db_name = db_name.trim();

    let mut stream = TcpStream::connect(ip)
        .map_err(|e| FerzError::Internal(format!("Failed to connect: {}", e)))?;

    // --- XAMPP-style Handshake ---
    let handshake = json!({
        "user": user,
        "pass": pass,
        "db": db_name
    });
    
    stream.write_all(handshake.to_string().as_bytes()).unwrap();
    
    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer).unwrap();
    let response = String::from_utf8_lossy(&buffer[..n]);
    
    if !response.contains("AUTH_OK") {
        println!("❌ Login Failed: {}", response);
        return Ok(());
    }

    println!("✅ Authenticated to {} as {}", ip, user);
    println!("Type .quit to exit");

    loop {
        print!("{}@{}> ", user, db_name);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == ".quit" { break; }
        if input.is_empty() { continue; }

        stream.write_all(input.as_bytes()).unwrap();
        let n = stream.read(&mut buffer).unwrap();
        println!("{}", String::from_utf8_lossy(&buffer[..n]));
    }

    Ok(())
}

#[cfg(feature = "cli")]
fn run_local_mode() -> Result<(), FerzError> {
    println!("Opening local database ferz.fdb...");
    let mut db = Database::open("ferz_local.fdb")?;
    
    loop {
        print!("ferzcli (local)> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == ".quit" { break; }
        if input.is_empty() { continue; }

        if input.to_uppercase().starts_with("SELECT") {
            match db.query(input) {
                Ok(results) => print_results(results),
                Err(e) => println!("Query Error: {}", e),
            }
        } else {
            match db.execute(input) {
                Ok(_) => println!("Command executed successfully."),
                Err(e) => println!("Execution Error: {}", e),
            }
        }
    }
    Ok(())
}

fn print_results(results: ResultSet) {
    if results.columns.is_empty() {
        println!("No results found.");
        return;
    }

    // Print header
    for col in &results.columns {
        print!("{:<15} | ", col);
    }
    println!();
    println!("{}", "-".repeat(results.columns.len() * 18));

    // Print rows
    let mut count = 0;
    for row in results {
        for val in row {
            print!("{:<15} | ", format!("{}", val));
        }
        println!();
        count += 1;
    }
    println!("({} rows)", count);
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("ferzcli requires the 'cli' feature to be enabled");
    eprintln!("Build with: cargo build --features cli");
    std::process::exit(1);
}
