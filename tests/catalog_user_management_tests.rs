//! Integration tests for Catalog user management
//! Tests the integration of UserStore with Catalog

use anyhow::Result;
use omendb::catalog::Catalog;
use tempfile::TempDir;

#[test]
fn test_catalog_user_management() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Default admin user should exist
    assert_eq!(catalog.user_count(), 1);
    assert!(catalog.user_exists("admin"));

    // Create users
    catalog.create_user("alice", "password123")?;
    catalog.create_user("bob", "password456")?;

    assert_eq!(catalog.user_count(), 3);
    assert!(catalog.user_exists("alice"));
    assert!(catalog.user_exists("bob"));

    // List users
    let users = catalog.list_users()?;
    assert_eq!(users.len(), 3);
    assert!(users.contains(&"admin".to_string()));
    assert!(users.contains(&"alice".to_string()));
    assert!(users.contains(&"bob".to_string()));

    // Drop user
    let dropped = catalog.drop_user("alice")?;
    assert!(dropped);
    assert_eq!(catalog.user_count(), 2);
    assert!(!catalog.user_exists("alice"));

    Ok(())
}

#[test]
fn test_default_admin_user_creation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Default admin user should be created on first initialization
    assert_eq!(catalog.user_count(), 1);
    assert!(catalog.user_exists("admin"));

    // Verify admin in user list
    let users = catalog.list_users()?;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0], "admin");

    Ok(())
}

#[test]
fn test_catalog_restart_preserves_users() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Create catalog and add users
    {
        let catalog = Catalog::new(data_dir.clone())?;
        catalog.create_user("alice", "password123")?;
        catalog.create_user("bob", "password456")?;
        assert_eq!(catalog.user_count(), 3); // admin + alice + bob
    }

    // Reopen catalog
    {
        let catalog = Catalog::new(data_dir)?;

        // Users should persist
        assert_eq!(catalog.user_count(), 3);
        assert!(catalog.user_exists("admin"));
        assert!(catalog.user_exists("alice"));
        assert!(catalog.user_exists("bob"));

        let users = catalog.list_users()?;
        assert_eq!(users.len(), 3);
    }

    Ok(())
}

#[test]
fn test_user_isolation_per_catalog() -> Result<()> {
    let temp_dir1 = TempDir::new()?;
    let temp_dir2 = TempDir::new()?;

    let catalog1 = Catalog::new(temp_dir1.path().to_path_buf())?;
    let catalog2 = Catalog::new(temp_dir2.path().to_path_buf())?;

    // Create user in catalog1
    catalog1.create_user("alice", "password123")?;

    // User should only exist in catalog1
    assert!(catalog1.user_exists("alice"));
    assert!(!catalog2.user_exists("alice"));

    // Create user in catalog2
    catalog2.create_user("bob", "password456")?;

    // Each catalog has its own users
    assert!(catalog1.user_exists("alice"));
    assert!(!catalog1.user_exists("bob"));
    assert!(!catalog2.user_exists("alice"));
    assert!(catalog2.user_exists("bob"));

    Ok(())
}

#[test]
fn test_concurrent_catalog_user_ops() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new()?;
    let catalog = Arc::new(Catalog::new(temp_dir.path().to_path_buf())?);

    // Spawn multiple threads creating users
    let mut handles = vec![];
    for i in 0..5 {
        let cat = Arc::clone(&catalog);
        let handle = thread::spawn(move || {
            let username = format!("user{}", i);
            cat.create_user(&username, "password123").unwrap();
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // All users should be created (5 + admin)
    assert_eq!(catalog.user_count(), 6);
    for i in 0..5 {
        let username = format!("user{}", i);
        assert!(catalog.user_exists(&username));
    }

    Ok(())
}

#[test]
fn test_catalog_create_user_validation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Invalid username (starts with number)
    let result = catalog.create_user("123user", "password123");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must start with"));

    // Invalid username (special chars)
    let result = catalog.create_user("user@name", "password123");
    assert!(result.is_err());

    // Valid username should work
    catalog.create_user("validuser", "password123")?;
    assert!(catalog.user_exists("validuser"));

    Ok(())
}

#[test]
fn test_catalog_drop_nonexistent_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Try to drop non-existent user
    let dropped = catalog.drop_user("nonexistent")?;
    assert!(!dropped);

    Ok(())
}

#[test]
fn test_catalog_duplicate_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;

    // Create user
    catalog.create_user("alice", "password123")?;

    // Try to create duplicate
    let result = catalog.create_user("alice", "password456");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}
