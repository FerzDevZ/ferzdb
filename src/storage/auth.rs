//! Authentication and Session Management
use crate::util::error::{FerzError, Result};
use std::sync::Arc;

/// Represents an active user session
#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub current_db: String,
    pub authenticated: bool,
}

impl Session {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            current_db: String::from("main"),
            authenticated: false,
        }
    }
}

pub struct AuthManager;

impl AuthManager {
    /// Verify credentials against system records (Hardcoded 'admin' for v0.6)
    pub fn verify(user: &str, pass: &str) -> Result<bool> {
        // In next phase, this will use 'ferz_system.fdb'
        if user == "admin" && pass == "ferz123" {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
