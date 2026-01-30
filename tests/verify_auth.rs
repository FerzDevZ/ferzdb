use ferzdb::storage::auth::AuthManager;

#[test]
fn test_auth_handshake() {
    // Test default admin credentials
    assert!(AuthManager::verify("admin", "ferz123").unwrap());
    
    // Test failure
    assert!(!AuthManager::verify("user", "wrong").unwrap());
    assert!(!AuthManager::verify("admin", "wrong").unwrap());
}
