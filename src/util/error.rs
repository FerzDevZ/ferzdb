//! Error types for FerzDB

use std::fmt;
use std::io;

/// Main error type for FerzDB operations
#[derive(Debug)]
pub enum FerzError {
    // Storage errors
    Io(io::Error),
    Corruption(String),
    PageNotFound(u32),
    InvalidPageType(u8),
    ChecksumMismatch,
    
    // SQL errors
    ParseError(String),
    SyntaxError(String),
    SemanticError(String),
    
    // Transaction errors
    TransactionNotActive,
    TransactionAlreadyActive,
    LockTimeout,
    Deadlock,
    
    // Schema errors
    TableNotFound(String),
    TableAlreadyExists(String),
    ColumnNotFound(String),
    TypeMismatch(String),
    
    // Constraint errors
    PrimaryKeyViolation(String),
    NotNullViolation(String),
    
    // Other
    NotImplemented(String),
    Internal(String),
}

impl fmt::Display for FerzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FerzError::Io(e) => write!(f, "I/O error: {}", e),
            FerzError::Corruption(msg) => write!(f, "Database corruption: {}", msg),
            FerzError::PageNotFound(id) => write!(f, "Page {} not found", id),
            FerzError::InvalidPageType(t) => write!(f, "Invalid page type: {}", t),
            FerzError::ChecksumMismatch => write!(f, "Checksum mismatch - data corruption detected"),
            
            FerzError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            FerzError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            FerzError::SemanticError(msg) => write!(f, "Semantic error: {}", msg),
            
            FerzError::TransactionNotActive => write!(f, "No active transaction"),
            FerzError::TransactionAlreadyActive => write!(f, "Transaction already active"),
            FerzError::LockTimeout => write!(f, "Lock acquisition timeout"),
            FerzError::Deadlock => write!(f, "Deadlock detected"),
            
            FerzError::TableNotFound(name) => write!(f, "Table '{}' does not exist", name),
            FerzError::TableAlreadyExists(name) => write!(f, "Table '{}' already exists", name),
            FerzError::ColumnNotFound(name) => write!(f, "Column '{}' not found", name),
            FerzError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            
            FerzError::PrimaryKeyViolation(msg) => write!(f, "Primary key violation: {}", msg),
            FerzError::NotNullViolation(col) => write!(f, "NOT NULL constraint violated for column '{}'", col),
            
            FerzError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
            FerzError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for FerzError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FerzError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for FerzError {
    fn from(error: io::Error) -> Self {
        FerzError::Io(error)
    }
}

/// Result type alias for FerzDB operations
pub type Result<T> = std::result::Result<T, FerzError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = FerzError::TableNotFound("users".to_string());
        assert_eq!(err.to_string(), "Table 'users' does not exist");
        
        let err = FerzError::ParseError("unexpected token".to_string());
        assert_eq!(err.to_string(), "Parse error: unexpected token");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let ferz_err: FerzError = io_err.into();
        
        match ferz_err {
            FerzError::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }
    }
}
