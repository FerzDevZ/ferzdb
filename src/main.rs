//! FerzDB Server implementation
//! 
//! Handles TCP connections and routes requests to the Database engine.

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::sync::Arc;
use ferzdb::prelude::*;

pub struct Server {
    db: Arc<Database>,
}

impl Server {
    pub fn new(db: Database) -> Self {
        Self { db: Arc::new(db) }
    }

    pub fn listen(&self, addr: &str) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr)?;
        println!("FerzDB Server listening on {}", addr);

        for stream in listener.incoming() {
            let stream = stream?;
            // let db = Arc::clone(&self.db);
            thread::spawn(move || {
                handle_client(stream);
            });
        }
        Ok(())
    }
}

fn handle_client(mut _stream: TcpStream) {
    // Protocol implementation (v0.1 simplified)
    let mut _buffer = [0u8; 1024];
    // stream.read(&mut buffer).unwrap();
    // stream.write_all(b"FerzDB v0.1\n").unwrap();
}

fn main() {
    // Server entry point will be implemented in v0.2
    println!("FerzDB Server mode is under development.");
}
