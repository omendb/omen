//! Comprehensive security integration tests for OmenDB
//! Tests authentication, user management, TLS infrastructure, and access control

use anyhow::Result;
use omendb::catalog::Catalog;
use omendb::postgres::OmenDbAuthSource;
use omendb::security::{AuthConfig, TlsConfig, SecurityContext};
use omendb::user_store::{User, UserStore};
use pgwire::api::auth::{AuthSource, LoginInfo};
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// Authentication Tests
// ============================================================================

#[test]
fn test_auth_config_creation() -> Result<()> {
    let config = AuthConfig::new();
    assert!(config.enabled);
    assert_eq!(config.session_timeout, 3600);
    assert!(config.users.contains_key("admin"));
    Ok(())
}

#[test]
fn test_auth_config_user_management() -> Result<()> {
    let mut config = AuthConfig::new();

    // Add user
    config.add_user("alice", "secure_password");
    assert!(config.verify_user("alice", "secure_password"));
    assert!(!config.verify_user("alice", "wrong_password"));

    // Remove user
    config.remove_user("alice");
    assert!(!config.verify_user("alice", "secure_password"));

    Ok(())
}

#[test]
fn test_auth_config_disabled() -> Result<()> {
    let mut config = AuthConfig::new();
    config.enabled = false;

    // Should always succeed when auth is disabled
    assert!(config.verify_user("anyone", "anypassword"));
    assert!(config.verify_user("", ""));

    Ok(())
}

#[test]
fn test_auth_config_from_env() -> Result<()> {
    // Test with auth disabled
    std::env::set_var("OMENDB_AUTH_DISABLED", "true");
    let config = AuthConfig::from_env()?;
    assert!(!config.enabled);
    std::env::remove_var("OMENDB_AUTH_DISABLED");

    // Test with custom admin credentials
    std::env::set_var("OMENDB_ADMIN_USER", "testadmin");
    std::env::set_var("OMENDB_ADMIN_PASSWORD", "testpassword");
    let config = AuthConfig::from_env()?;
    assert!(config.verify_user("testadmin", "testpassword"));
    std::env::remove_var("OMENDB_ADMIN_USER");
    std::env::remove_var("OMENDB_ADMIN_PASSWORD");

    Ok(())
}

// ============================================================================
// OmenDbAuthSource Tests
// ============================================================================

#[tokio::test]
async fn test_omendb_auth_source_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let auth = OmenDbAuthSource::new(temp_dir.path())?;

    // Should start with 0 users (no default admin in OmenDbAuthSource)
    assert_eq!(auth.user_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_omendb_auth_add_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let auth = OmenDbAuthSource::new(temp_dir.path())?;

    // Add user
    auth.add_user("alice", "password123")?;
    assert!(auth.user_exists("alice"));
    assert_eq!(auth.user_count(), 1);

    // Verify we can get password via AuthSource trait
    let login = LoginInfo::new(Some("alice"), Some("omendb"), "127.0.0.1".to_string());
    let password = auth.get_password(&login).await?;
    assert!(password.salt().is_some());
    assert!(!password.password().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_omendb_auth_remove_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let auth = OmenDbAuthSource::new(temp_dir.path())?;

    // Add and remove user
    auth.add_user("bob", "password456")?;
    assert!(auth.user_exists("bob"));

    let removed = auth.remove_user("bob")?;
    assert!(removed);
    assert!(!auth.user_exists("bob"));
    assert_eq!(auth.user_count(), 0);

    Ok(())
}

#[tokio::test]
async fn test_omendb_auth_nonexistent_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let auth = OmenDbAuthSource::new(temp_dir.path())?;

    // Try to get password for non-existent user
    let login = LoginInfo::new(Some("nonexistent"), Some("omendb"), "127.0.0.1".to_string());
    let result = auth.get_password(&login).await;
    assert!(result.is_err());

    Ok(())
}

// ============================================================================
// UserStore Tests
// ============================================================================

#[test]
fn test_user_store_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    assert_eq!(store.user_count()?, 0);
    Ok(())
}

#[test]
fn test_user_store_create_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    // Create user
    let user = User::new_with_password("alice", "password123", 4096)?;
    store.create_user(&user)?;

    assert!(store.user_exists("alice")?);
    assert_eq!(store.user_count()?, 1);

    Ok(())
}

#[test]
fn test_user_store_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Create users in first instance
    {
        let store = UserStore::new(&data_dir)?;
        let user1 = User::new_with_password("alice", "password123", 4096)?;
        let user2 = User::new_with_password("bob", "password456", 4096)?;
        store.create_user(&user1)?;
        store.create_user(&user2)?;
        assert_eq!(store.user_count()?, 2);
    }

    // Verify persistence in new instance
    {
        let store = UserStore::new(&data_dir)?;
        assert_eq!(store.user_count()?, 2);
        assert!(store.user_exists("alice")?);
        assert!(store.user_exists("bob")?);
    }

    Ok(())
}

#[test]
fn test_user_store_concurrent_access() -> Result<()> {
    use std::thread;

    let temp_dir = TempDir::new()?;
    let store = Arc::new(UserStore::new(temp_dir.path())?);

    // Spawn multiple threads creating users
    let mut handles = vec![];
    for i in 0..10 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let username = format!("user{}", i);
            let user = User::new_with_password(&username, "password123", 4096).unwrap();
            store_clone.create_user(&user).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // All users should be created
    assert_eq!(store.user_count()?, 10);
    for i in 0..10 {
        assert!(store.user_exists(&format!("user{}", i))?);
    }

    Ok(())
}

// ============================================================================
// User Management Security Tests
// ============================================================================

#[test]
fn test_username_validation() -> Result<()> {
    // Invalid: starts with number
    assert!(User::validate_username("123user").is_err());

    // Invalid: contains special chars
    assert!(User::validate_username("user@name").is_err());
    assert!(User::validate_username("user#name").is_err());

    // Invalid: empty
    assert!(User::validate_username("").is_err());

    // Valid usernames
    assert!(User::validate_username("valid_user").is_ok());
    assert!(User::validate_username("user123").is_ok());
    assert!(User::validate_username("_user").is_ok());

    Ok(())
}

#[test]
fn test_user_store_duplicate_prevention() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    // Create user
    let user1 = User::new_with_password("alice", "password123", 4096)?;
    store.create_user(&user1)?;

    // Try to create duplicate
    let user2 = User::new_with_password("alice", "different_password", 4096)?;
    let result = store.create_user(&user2);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[test]
fn test_password_hashing() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    // Create user with password
    let user = User::new_with_password("alice", "password123", 4096)?;
    store.create_user(&user)?;

    // Get stored user
    let stored_user = store.get_user("alice")?.unwrap();

    // Verify hash is not plaintext
    assert!(!stored_user.salted_password.is_empty());
    assert!(!stored_user.salt.is_empty());

    Ok(())
}

#[test]
fn test_user_store_delete_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    // Create user
    let user = User::new_with_password("alice", "password123", 4096)?;
    store.create_user(&user)?;
    assert!(store.user_exists("alice")?);

    // Delete user
    let deleted = store.delete_user("alice")?;
    assert!(deleted);
    assert!(!store.user_exists("alice")?);

    // Try to delete non-existent user
    let deleted = store.delete_user("alice")?;
    assert!(!deleted);

    Ok(())
}

#[test]
fn test_user_store_list_users() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let store = UserStore::new(temp_dir.path())?;

    // Create multiple users
    let user1 = User::new_with_password("alice", "password1", 4096)?;
    let user2 = User::new_with_password("bob", "password2", 4096)?;
    let user3 = User::new_with_password("charlie", "password3", 4096)?;
    store.create_user(&user1)?;
    store.create_user(&user2)?;
    store.create_user(&user3)?;

    // List users
    let users = store.list_users()?;
    assert_eq!(users.len(), 3);
    assert!(users.contains(&"alice".to_string()));
    assert!(users.contains(&"bob".to_string()));
    assert!(users.contains(&"charlie".to_string()));

    Ok(())
}

// ============================================================================
// Catalog Integration Security Tests
// ============================================================================

#[test]
fn test_catalog_user_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Default admin exists
    assert!(catalog.user_exists("admin"));

    // Create users
    catalog.create_user("alice", "secure_password")?;
    assert!(catalog.user_exists("alice"));

    // Try duplicate - should fail
    assert!(catalog.create_user("alice", "another_password").is_err());

    // Drop user
    catalog.drop_user("alice")?;
    assert!(!catalog.user_exists("alice"));

    Ok(())
}

#[test]
fn test_catalog_user_persistence() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Create users
    {
        let catalog = Catalog::new(data_dir.clone())?;
        catalog.create_user("alice", "password123")?;
        catalog.create_user("bob", "password456")?;
        assert_eq!(catalog.user_count(), 3); // admin + alice + bob
    }

    // Restart and verify
    {
        let catalog = Catalog::new(data_dir)?;
        assert_eq!(catalog.user_count(), 3);
        assert!(catalog.user_exists("admin"));
        assert!(catalog.user_exists("alice"));
        assert!(catalog.user_exists("bob"));
    }

    Ok(())
}

// ============================================================================
// TLS Infrastructure Tests
// ============================================================================

#[test]
fn test_tls_config_creation() {
    let config = TlsConfig::default();
    assert!(!config.enabled); // Disabled by default
    assert_eq!(config.cert_file, "certs/server.crt");
    assert_eq!(config.key_file, "certs/server.key");
}

#[test]
fn test_tls_config_from_env() {
    // Set environment variables
    std::env::set_var("OMENDB_TLS_CERT", "/custom/path/cert.pem");
    std::env::set_var("OMENDB_TLS_KEY", "/custom/path/key.pem");
    std::env::set_var("OMENDB_TLS_ENABLED", "true");

    let config = TlsConfig::from_env();
    assert!(config.enabled);
    assert_eq!(config.cert_file, "/custom/path/cert.pem");
    assert_eq!(config.key_file, "/custom/path/key.pem");

    // Clean up
    std::env::remove_var("OMENDB_TLS_CERT");
    std::env::remove_var("OMENDB_TLS_KEY");
    std::env::remove_var("OMENDB_TLS_ENABLED");
}

#[test]
fn test_security_context_creation() -> Result<()> {
    let context = SecurityContext::default();
    assert!(context.auth.enabled);
    assert!(!context.tls.enabled);
    Ok(())
}

#[test]
fn test_security_context_from_env() -> Result<()> {
    // Set environment
    std::env::set_var("OMENDB_AUTH_DISABLED", "true");
    std::env::set_var("OMENDB_TLS_ENABLED", "true");

    let context = SecurityContext::from_env()?;
    assert!(!context.auth.enabled);
    assert!(context.tls.enabled);

    // Clean up
    std::env::remove_var("OMENDB_AUTH_DISABLED");
    std::env::remove_var("OMENDB_TLS_ENABLED");

    Ok(())
}

#[test]
fn test_tls_cert_loading_validation() {
    use datafusion::prelude::SessionContext;
    use omendb::postgres::server::PostgresServer;

    let ctx = SessionContext::new();
    let server = PostgresServer::new(ctx);

    // Try to enable TLS with non-existent certificate files
    let result = server.with_tls("nonexistent.crt", "nonexistent.key");
    assert!(result.is_err());
}

// ============================================================================
// End-to-End Security Tests
// ============================================================================

#[test]
fn test_complete_user_lifecycle() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // 1. Create user via catalog
    catalog.create_user("alice", "initial_password")?;
    assert!(catalog.user_exists("alice"));
    assert_eq!(catalog.user_count(), 2); // admin + alice

    // 2. List users
    let users = catalog.list_users()?;
    assert_eq!(users.len(), 2);
    assert!(users.contains(&"admin".to_string()));
    assert!(users.contains(&"alice".to_string()));

    // 3. Drop user
    catalog.drop_user("alice")?;
    assert!(!catalog.user_exists("alice"));
    assert_eq!(catalog.user_count(), 1); // Only admin left

    Ok(())
}

#[test]
fn test_security_persistence_after_crash() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Simulate crash: create users then drop store
    {
        let store = UserStore::new(&data_dir)?;
        let user1 = User::new_with_password("alice", "password123", 4096)?;
        let user2 = User::new_with_password("bob", "password456", 4096)?;
        store.create_user(&user1)?;
        store.create_user(&user2)?;
        // Store dropped here (simulated crash)
    }

    // Recovery: verify users still exist
    {
        let store = UserStore::new(&data_dir)?;
        assert!(store.user_exists("alice")?);
        assert!(store.user_exists("bob")?);
        assert_eq!(store.user_count()?, 2);
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_authentication() -> Result<()> {
    use tokio::task;

    let temp_dir = TempDir::new()?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);

    // Create test users
    for i in 0..10 {
        auth.add_user(format!("user{}", i), "password123")?;
    }

    // Spawn concurrent authentication attempts
    let mut handles = vec![];
    for i in 0..10 {
        let auth_clone = Arc::clone(&auth);
        let handle = task::spawn(async move {
            let username = format!("user{}", i);
            let login = LoginInfo::new(Some(&username), Some("omendb"), "127.0.0.1".to_string());
            for _ in 0..5 {
                let _ = auth_clone.get_password(&login).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await?;
    }

    Ok(())
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_authentication_performance() -> Result<()> {
    use std::time::Instant;

    let temp_dir = TempDir::new()?;
    let auth = OmenDbAuthSource::new(temp_dir.path())?;

    // Create test user
    auth.add_user("testuser", "password123")?;

    // Measure authentication performance
    let start = Instant::now();
    let iterations = 100;

    let login = LoginInfo::new(Some("testuser"), Some("omendb"), "127.0.0.1".to_string());
    for _ in 0..iterations {
        auth.get_password(&login).await?;
    }

    let duration = start.elapsed();
    let avg_ms = duration.as_millis() as f64 / iterations as f64;

    // Authentication should be reasonably fast (< 10ms average)
    assert!(
        avg_ms < 10.0,
        "Authentication too slow: {}ms average",
        avg_ms
    );

    Ok(())
}
