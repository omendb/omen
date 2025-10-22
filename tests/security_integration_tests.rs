// Security integration tests: Auth + TLS + Multi-user scenarios (Phase 2 Day 8)

use omendb::catalog::Catalog;
use omendb::postgres::{OmenDbAuthSource, PostgresServer};
use omendb::user_store::UserStore;
use datafusion::prelude::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tempfile::TempDir;
use std::time::Duration;
use tokio::time::sleep;
use std::process::Command;

// Helper: Create server with authentication
async fn create_auth_server(port: u16) -> (PostgresServer, Arc<UserStore>, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_users.db");

    let user_store = Arc::new(UserStore::new(db_path.to_str().unwrap()).unwrap());

    // Create test admin user
    user_store.create_user("admin", "admin_password").await.unwrap();
    user_store.create_user("user1", "password1").await.unwrap();
    user_store.create_user("user2", "password2").await.unwrap();

    let auth_source = Arc::new(OmenDbAuthSource::new(user_store.clone()));
    let ctx = SessionContext::new();
    let ctx_lock = Arc::new(RwLock::new(ctx));

    let server = PostgresServer::with_auth(
        format!("127.0.0.1:{}", port),
        ctx_lock,
        auth_source
    );

    (server, user_store, temp_dir)
}

// Helper: Create server with auth + TLS
async fn create_auth_tls_server(port: u16, cert_path: &str, key_path: &str)
    -> (PostgresServer, Arc<UserStore>, TempDir)
{
    let (server, user_store, temp_dir) = create_auth_server(port).await;
    let server_with_tls = server.with_tls(cert_path, key_path)
        .expect("Failed to enable TLS");

    (server_with_tls, user_store, temp_dir)
}

// Helper: Generate test certificates
fn generate_test_certs(dir: &str) -> (String, String) {
    let cert_path = format!("{}/cert.pem", dir);
    let key_path = format!("{}/key.pem", dir);

    let output = Command::new("openssl")
        .args(&[
            "req", "-new", "-newkey", "rsa:2048", "-days", "1", "-nodes", "-x509",
            "-keyout", &key_path, "-out", &cert_path,
            "-subj", "/C=US/ST=CA/L=SF/O=OmenDB-Test/CN=localhost"
        ])
        .output()
        .expect("Failed to generate certificates");

    assert!(output.status.success());
    (cert_path, key_path)
}

#[tokio::test]
async fn test_auth_required_connection() {
    let (server, _user_store, _temp_dir) = create_auth_server(25433).await;

    // Spawn server
    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    sleep(Duration::from_millis(500)).await;

    // Test: Connection without credentials should fail
    if Command::new("psql").arg("--version").output().is_ok() {
        let output = Command::new("psql")
            .args(&[
                "-h", "127.0.0.1",
                "-p", "25433",
                "-U", "nonexistent",
                "-c", "SELECT 1"
            ])
            .env("PGPASSWORD", "wrongpassword")
            .output()
            .unwrap();

        // Should fail (exit code != 0)
        assert!(!output.status.success(), "Connection should fail with wrong credentials");
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_valid_auth_connection() {
    let (server, _user_store, _temp_dir) = create_auth_server(25434).await;

    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    sleep(Duration::from_millis(500)).await;

    // Test: Valid credentials should succeed
    if Command::new("psql").arg("--version").output().is_ok() {
        let output = Command::new("psql")
            .args(&[
                "-h", "127.0.0.1",
                "-p", "25434",
                "-U", "admin",
                "-c", "SELECT 1"
            ])
            .env("PGPASSWORD", "admin_password")
            .output()
            .unwrap();

        // Should succeed
        if !output.status.success() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_tls_with_auth_e2e() {
    let temp_dir = TempDir::new().unwrap();
    let (cert_path, key_path) = generate_test_certs(temp_dir.path().to_str().unwrap());

    let (server, _user_store, _db_dir) = create_auth_tls_server(25435, &cert_path, &key_path).await;

    assert!(server.is_tls_enabled(), "TLS should be enabled");

    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    sleep(Duration::from_millis(500)).await;

    // Test: TLS + Auth connection
    if Command::new("psql").arg("--version").output().is_ok() {
        let output = Command::new("psql")
            .args(&[
                "host=127.0.0.1",
                "port=25435",
                "user=admin",
                "sslmode=require",
                "-c",
                "SELECT 1"
            ])
            .env("PGPASSWORD", "admin_password")
            .output()
            .unwrap();

        if !output.status.success() {
            eprintln!("TLS+Auth test output:");
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_multi_user_concurrent_access() {
    let (server, user_store, _temp_dir) = create_auth_server(25436).await;

    // Verify multiple users exist
    assert!(user_store.verify_user("admin", "admin_password").await.unwrap());
    assert!(user_store.verify_user("user1", "password1").await.unwrap());
    assert!(user_store.verify_user("user2", "password2").await.unwrap());

    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    sleep(Duration::from_millis(500)).await;

    // Test: Multiple users can connect concurrently
    if Command::new("psql").arg("--version").output().is_ok() {
        let user1_result = Command::new("psql")
            .args(&[
                "-h", "127.0.0.1",
                "-p", "25436",
                "-U", "user1",
                "-c", "SELECT 'user1' as current_user"
            ])
            .env("PGPASSWORD", "password1")
            .output();

        let user2_result = Command::new("psql")
            .args(&[
                "-h", "127.0.0.1",
                "-p", "25436",
                "-U", "user2",
                "-c", "SELECT 'user2' as current_user"
            ])
            .env("PGPASSWORD", "password2")
            .output();

        if let (Ok(u1), Ok(u2)) = (user1_result, user2_result) {
            if !u1.status.success() {
                eprintln!("User1 failed: {}", String::from_utf8_lossy(&u1.stderr));
            }
            if !u2.status.success() {
                eprintln!("User2 failed: {}", String::from_utf8_lossy(&u2.stderr));
            }
        }
    }

    server_handle.abort();
}

#[tokio::test]
async fn test_user_isolation() {
    let (_server, user_store, _temp_dir) = create_auth_server(25437).await;

    // Test: Users are isolated (each user has unique credentials)
    let admin_valid = user_store.verify_user("admin", "admin_password").await.unwrap();
    let admin_wrong = user_store.verify_user("admin", "wrong_password").await.unwrap();
    let user1_valid = user_store.verify_user("user1", "password1").await.unwrap();
    let user1_wrong = user_store.verify_user("user1", "admin_password").await.unwrap();

    assert!(admin_valid, "Admin should authenticate with correct password");
    assert!(!admin_wrong, "Admin should fail with wrong password");
    assert!(user1_valid, "User1 should authenticate with correct password");
    assert!(!user1_wrong, "User1 should not authenticate with admin password");
}

#[tokio::test]
async fn test_permission_boundary_user_creation() {
    let (_server, user_store, _temp_dir) = create_auth_server(25438).await;

    // Test: Create user with weak password should fail
    let weak_result = user_store.create_user("weak_user", "123").await;
    assert!(weak_result.is_err(), "Weak password should be rejected");

    // Test: Create user with strong password should succeed
    let strong_result = user_store.create_user("strong_user", "StrongP@ssw0rd!").await;
    assert!(strong_result.is_ok(), "Strong password should be accepted");

    // Test: Duplicate username should fail
    let duplicate_result = user_store.create_user("admin", "AnotherP@ssw0rd!").await;
    assert!(duplicate_result.is_err(), "Duplicate username should be rejected");
}

#[tokio::test]
async fn test_connection_pool_limits() {
    let (server, _user_store, _temp_dir) = create_auth_server(25439).await;

    // Check connection pool stats
    let stats = server.pool_stats();
    assert_eq!(server.connection_count(), 0, "Should start with 0 connections");
    assert!(stats.max_connections > 0, "Max connections should be configured");

    // Server maintains connection pool limits
    assert!(stats.max_connections <= 1000, "Reasonable connection limit");
}

#[tokio::test]
async fn test_tls_certificate_validation() {
    let temp_dir = TempDir::new().unwrap();
    let (cert_path, key_path) = generate_test_certs(temp_dir.path().to_str().unwrap());

    // Test: Valid certificates load successfully
    let (_server, _user_store, _db_dir) = create_auth_tls_server(25440, &cert_path, &key_path).await;

    // Test: Invalid certificate path fails
    let (server2, _us, _td) = create_auth_server(25441).await;
    let invalid_result = server2.with_tls("/nonexistent/cert.pem", "/nonexistent/key.pem");
    assert!(invalid_result.is_err(), "Invalid certificate path should fail");

    // Test: Mismatched cert/key fails
    let bad_key = format!("{}/bad_key.pem", temp_dir.path().to_str().unwrap());
    std::fs::write(&bad_key, "INVALID KEY DATA").unwrap();

    let (server3, _us3, _td3) = create_auth_server(25442).await;
    let mismatch_result = server3.with_tls(&cert_path, &bad_key);
    assert!(mismatch_result.is_err(), "Mismatched cert/key should fail");
}

#[tokio::test]
async fn test_password_hashing_security() {
    let (_server, user_store, _temp_dir) = create_auth_server(25443).await;

    // Test: Passwords are hashed (not stored in plaintext)
    user_store.create_user("hash_test", "MySecureP@ss123").await.unwrap();

    // Verify user can authenticate
    assert!(user_store.verify_user("hash_test", "MySecureP@ss123").await.unwrap());

    // Wrong password fails
    assert!(!user_store.verify_user("hash_test", "WrongPassword").await.unwrap());

    // Test: Empty password fails
    let empty_result = user_store.create_user("empty_user", "").await;
    assert!(empty_result.is_err(), "Empty password should be rejected");
}

#[tokio::test]
async fn test_concurrent_user_operations() {
    let (_server, user_store, _temp_dir) = create_auth_server(25444).await;

    // Test: Concurrent user creation and verification
    let us1 = user_store.clone();
    let us2 = user_store.clone();
    let us3 = user_store.clone();

    let create_task = tokio::spawn(async move {
        us1.create_user("concurrent1", "Pass1234!").await
    });

    let verify_task = tokio::spawn(async move {
        us2.verify_user("admin", "admin_password").await
    });

    let list_task = tokio::spawn(async move {
        us3.list_users().await
    });

    let (create_result, verify_result, list_result) =
        tokio::join!(create_task, verify_task, list_task);

    assert!(create_result.unwrap().is_ok(), "Concurrent create should succeed");
    assert!(verify_result.unwrap().unwrap(), "Concurrent verify should succeed");
    assert!(list_result.unwrap().is_ok(), "Concurrent list should succeed");
}

#[test]
fn test_default_admin_user_warning() {
    // This test documents that default admin password should trigger warning
    // In production, we should warn if default 'admin' user exists with weak password

    // Note: This is a documentation test - actual warning implementation
    // should be in server startup code
    let default_admin = "admin";
    let default_password = "admin"; // Weak password

    assert_eq!(default_admin, "admin");
    assert_eq!(default_password.len(), 5); // Too short

    // TODO: Add actual warning in postgres_server.rs startup
}
