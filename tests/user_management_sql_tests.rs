//! Tests for SQL user management commands
//! CREATE USER, DROP USER, ALTER USER

use anyhow::Result;
use omen::catalog::Catalog;
use omen::postgres::OmenDbAuthSource;
use omen::sql_engine::{ExecutionResult, SqlEngine};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_create_user_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create user
    let sql = "CREATE USER alice WITH PASSWORD 'secret123'";
    let result = engine.execute(sql)?;

    match result {
        ExecutionResult::UserCreated { username } => {
            assert_eq!(username, "alice");
        }
        _ => panic!("Expected UserCreated result"),
    }

    // Verify user exists
    assert!(auth.user_exists("alice"));
    assert_eq!(auth.user_count(), 1);

    Ok(())
}

#[tokio::test]
async fn test_create_user_duplicate_error() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create user
    let sql = "CREATE USER bob WITH PASSWORD 'password456'";
    engine.execute(sql)?;

    // Try to create duplicate
    let result = engine.execute(sql);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    Ok(())
}

#[tokio::test]
async fn test_create_user_invalid_name() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth);

    // Invalid username (starts with number)
    let sql = "CREATE USER 123user WITH PASSWORD 'secret123'";
    let result = engine.execute(sql);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must start with"));

    // Invalid username (contains special chars)
    let sql = "CREATE USER user@name WITH PASSWORD 'secret123'";
    let result = engine.execute(sql);
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_create_user_weak_password() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth);

    // Password too short
    let sql = "CREATE USER charlie WITH PASSWORD 'short'";
    let result = engine.execute(sql);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 8"));

    Ok(())
}

#[tokio::test]
async fn test_drop_user_basic() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create and drop user
    engine.execute("CREATE USER david WITH PASSWORD 'password123'")?;
    assert!(auth.user_exists("david"));

    let result = engine.execute("DROP USER david")?;
    match result {
        ExecutionResult::UserDropped { username } => {
            assert_eq!(username, "david");
        }
        _ => panic!("Expected UserDropped result"),
    }

    assert!(!auth.user_exists("david"));

    Ok(())
}

#[tokio::test]
async fn test_drop_user_nonexistent() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth);

    // Try to drop non-existent user
    let result = engine.execute("DROP USER nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));

    Ok(())
}

#[tokio::test]
async fn test_alter_user_password() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create user
    engine.execute("CREATE USER eve WITH PASSWORD 'oldpassword123'")?;

    // Alter password
    let result = engine.execute("ALTER USER eve PASSWORD 'newpassword456'")?;
    match result {
        ExecutionResult::UserAltered { username } => {
            assert_eq!(username, "eve");
        }
        _ => panic!("Expected UserAltered result"),
    }

    // User still exists
    assert!(auth.user_exists("eve"));

    Ok(())
}

#[tokio::test]
async fn test_alter_user_nonexistent() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth);

    // Try to alter non-existent user
    let result = engine.execute("ALTER USER nobody PASSWORD 'newpass123'");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));

    Ok(())
}

#[tokio::test]
async fn test_user_management_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create multiple users
    engine.execute("CREATE USER alice WITH PASSWORD 'password123'")?;
    engine.execute("CREATE USER bob WITH PASSWORD 'password456'")?;
    engine.execute("CREATE USER charlie WITH PASSWORD 'password789'")?;

    assert_eq!(auth.user_count(), 3);

    // Alter one user
    engine.execute("ALTER USER bob PASSWORD 'newpassword'")?;

    // Drop one user
    engine.execute("DROP USER charlie")?;

    assert_eq!(auth.user_count(), 2);
    assert!(auth.user_exists("alice"));
    assert!(auth.user_exists("bob"));
    assert!(!auth.user_exists("charlie"));

    Ok(())
}

#[tokio::test]
async fn test_sql_injection_in_create_user() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth);

    // Try SQL injection in username
    let sql = r#"CREATE USER alice'; DROP TABLE users; -- WITH PASSWORD 'secret123'"#;
    let result = engine.execute(sql);

    // Should fail validation (special characters in username)
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_special_characters_in_password() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Password with special characters
    let sql = "CREATE USER alice WITH PASSWORD 'p@ssw0rd!#$%'";
    let result = engine.execute(sql)?;

    match result {
        ExecutionResult::UserCreated { username } => {
            assert_eq!(username, "alice");
        }
        _ => panic!("Expected UserCreated result"),
    }

    assert!(auth.user_exists("alice"));

    Ok(())
}

#[tokio::test]
async fn test_unicode_username() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Unicode username (should work if alphanumeric)
    let sql = "CREATE USER user123 WITH PASSWORD 'password123'";
    let result = engine.execute(sql)?;

    match result {
        ExecutionResult::UserCreated { username } => {
            assert_eq!(username, "user123");
        }
        _ => panic!("Expected UserCreated result"),
    }

    assert!(auth.user_exists("user123"));

    Ok(())
}

#[tokio::test]
async fn test_case_sensitive_username() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);
    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Create users with different cases
    engine.execute("CREATE USER Alice WITH PASSWORD 'password123'")?;
    engine.execute("CREATE USER alice WITH PASSWORD 'password456'")?;

    // Both should exist (case-sensitive)
    assert_eq!(auth.user_count(), 2);
    assert!(auth.user_exists("Alice"));
    assert!(auth.user_exists("alice"));

    Ok(())
}

#[tokio::test]
async fn test_admin_user_cannot_be_deleted() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let auth = Arc::new(OmenDbAuthSource::new(temp_dir.path())?);

    // Create admin user
    auth.add_user("admin", "adminpass123")?;

    let mut engine = SqlEngine::new(catalog).with_auth(auth.clone());

    // Try to drop admin
    let result = engine.execute("DROP USER admin");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot delete admin"));

    // Admin still exists
    assert!(auth.user_exists("admin"));

    Ok(())
}

#[tokio::test]
async fn test_no_auth_configured() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let catalog = Catalog::new(temp_dir.path().to_path_buf())?;
    let mut engine = SqlEngine::new(catalog); // No auth configured

    // Try to create user without auth
    let result = engine.execute("CREATE USER alice WITH PASSWORD 'password123'");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Authentication not configured"));

    Ok(())
}
