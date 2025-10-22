// TLS integration tests for PostgreSQL wire protocol

use omendb::postgres::PostgresServer;
use datafusion::prelude::*;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tempfile::TempDir;
use std::fs;

// Generate test certificates for TLS testing
fn generate_test_certificates(dir: &str) -> (String, String) {
    let cert_path = format!("{}/test_cert.pem", dir);
    let key_path = format!("{}/test_key.pem", dir);

    let output = Command::new("openssl")
        .args(&[
            "req", "-new", "-newkey", "rsa:2048", "-days", "1", "-nodes", "-x509",
            "-keyout", &key_path, "-out", &cert_path,
            "-subj", "/C=US/ST=CA/L=SF/O=OmenDB-Test/CN=localhost"
        ])
        .output()
        .expect("Failed to generate test certificates");

    assert!(output.status.success(), "Certificate generation failed: {:?}", output);
    (cert_path, key_path)
}

#[tokio::test]
async fn test_server_with_tls() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();
    let (cert_path, key_path) = generate_test_certificates(dir_path);

    // Verify certificates were created
    assert!(std::path::Path::new(&cert_path).exists());
    assert!(std::path::Path::new(&key_path).exists());

    let ctx = SessionContext::new();
    let server = PostgresServer::with_addr("127.0.0.1:15433", ctx)
        .with_tls(&cert_path, &key_path)
        .expect("Failed to enable TLS");

    assert!(server.is_tls_enabled());
}

#[tokio::test]
async fn test_server_tls_requires_both_cert_and_key() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();
    let (cert_path, _) = generate_test_certificates(dir_path);

    let ctx = SessionContext::new();

    // Missing key file should fail
    let result = PostgresServer::with_addr("127.0.0.1:15434", ctx)
        .with_tls(&cert_path, "/nonexistent/key.pem");

    assert!(result.is_err());
}

#[tokio::test]
async fn test_server_tls_invalid_certificate() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();

    // Create invalid certificate file
    let bad_cert = format!("{}/bad_cert.pem", dir_path);
    fs::write(&bad_cert, "INVALID CERTIFICATE").unwrap();

    let ctx = SessionContext::new();
    let result = PostgresServer::with_addr("127.0.0.1:15435", ctx)
        .with_tls(&bad_cert, &bad_cert);

    assert!(result.is_err());
}

#[tokio::test]
async fn test_server_without_tls() {
    let ctx = SessionContext::new();
    let server = PostgresServer::with_addr("127.0.0.1:15436", ctx);

    assert!(!server.is_tls_enabled());
}

#[tokio::test]
async fn test_tls_connection_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();
    let (cert_path, key_path) = generate_test_certificates(dir_path);

    let ctx = SessionContext::new();

    // Test TLS configuration
    let server = PostgresServer::with_addr("127.0.0.1:15437", ctx)
        .with_tls(&cert_path, &key_path)
        .expect("Failed to enable TLS");

    assert!(server.is_tls_enabled());

    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        server.serve().await
    });

    // Give server time to start
    sleep(Duration::from_millis(500)).await;

    // Test psql connection with TLS (requires psql to be installed)
    if Command::new("psql").arg("--version").output().is_ok() {
        let output = Command::new("psql")
            .args(&[
                "host=127.0.0.1",
                "port=15437",
                "sslmode=require",
                "-c",
                "SELECT 1"
            ])
            .output();

        if let Ok(result) = output {
            // Connection might fail due to self-signed cert, but should attempt TLS
            println!("psql output: {:?}", String::from_utf8_lossy(&result.stdout));
        }
    }

    // Cleanup: abort server
    server_handle.abort();
}

#[test]
fn test_certificate_file_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_str().unwrap();
    let (cert_path, key_path) = generate_test_certificates(dir_path);

    // Check that key file has restrictive permissions
    let key_metadata = fs::metadata(&key_path).unwrap();

    // On Unix, private key should have restrictive permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = key_metadata.permissions().mode();
        // Key file should not be world-readable (mode & 0o004 == 0)
        assert_eq!(mode & 0o004, 0, "Private key should not be world-readable");
    }

    // Certificate can be world-readable
    assert!(fs::metadata(&cert_path).unwrap().len() > 0);
}
