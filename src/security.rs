//! Security module for OmenDB
//! Provides authentication, authorization, and TLS encryption
//! Essential for production and enterprise deployments

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use hyper::{HeaderMap, StatusCode};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Username -> password hash mapping
    pub users: HashMap<String, String>,
    /// JWT secret for token generation (if implementing JWT)
    pub jwt_secret: Option<String>,
    /// Session timeout in seconds
    pub session_timeout: u64,
    /// Enable/disable authentication
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        let mut users = HashMap::new();
        // Default admin user (should be changed in production)
        users.insert("admin".to_string(), hash_password("admin123"));

        Self {
            users,
            jwt_secret: Some("default-secret-change-in-production".to_string()),
            session_timeout: 3600, // 1 hour
            enabled: true,
        }
    }
}

impl AuthConfig {
    /// Create new auth config with specific users
    pub fn new() -> Self {
        Self::default()
    }

    /// Add user with password
    pub fn add_user(&mut self, username: &str, password: &str) {
        self.users
            .insert(username.to_string(), hash_password(password));
    }

    /// Remove user
    pub fn remove_user(&mut self, username: &str) {
        self.users.remove(username);
    }

    /// Verify user credentials
    pub fn verify_user(&self, username: &str, password: &str) -> bool {
        if !self.enabled {
            return true; // Auth disabled
        }

        match self.users.get(username) {
            Some(stored_hash) => verify_password(password, stored_hash),
            None => false,
        }
    }

    /// Load from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::new();

        // Check if auth is disabled
        if std::env::var("OMENDB_AUTH_DISABLED").unwrap_or_default() == "true" {
            config.enabled = false;
            return Ok(config);
        }

        // Load admin credentials from env
        if let Ok(admin_user) = std::env::var("OMENDB_ADMIN_USER") {
            if let Ok(admin_pass) = std::env::var("OMENDB_ADMIN_PASSWORD") {
                config.users.clear();
                config.add_user(&admin_user, &admin_pass);
            }
        }

        // Load JWT secret
        if let Ok(secret) = std::env::var("OMENDB_JWT_SECRET") {
            config.jwt_secret = Some(secret);
        }

        // Load session timeout
        if let Ok(timeout) = std::env::var("OMENDB_SESSION_TIMEOUT") {
            config.session_timeout = timeout.parse().unwrap_or(3600);
        }

        Ok(config)
    }
}

/// TLS configuration
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_file: String,
    /// Path to private key file
    pub key_file: String,
    /// Enable/disable TLS
    pub enabled: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            cert_file: "certs/server.crt".to_string(),
            key_file: "certs/server.key".to_string(),
            enabled: false, // Disabled by default for development
        }
    }
}

impl TlsConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        Self {
            cert_file: std::env::var("OMENDB_TLS_CERT")
                .unwrap_or_else(|_| "certs/server.crt".to_string()),
            key_file: std::env::var("OMENDB_TLS_KEY")
                .unwrap_or_else(|_| "certs/server.key".to_string()),
            enabled: std::env::var("OMENDB_TLS_ENABLED").unwrap_or_default() == "true",
        }
    }

    /// Create rustls server config
    pub fn create_server_config(&self) -> Result<Arc<ServerConfig>> {
        let cert_file = File::open(&self.cert_file)
            .map_err(|e| anyhow!("Failed to open cert file {}: {}", self.cert_file, e))?;
        let key_file = File::open(&self.key_file)
            .map_err(|e| anyhow!("Failed to open key file {}: {}", self.key_file, e))?;

        let mut cert_reader = BufReader::new(cert_file);
        let mut key_reader = BufReader::new(key_file);

        let certs = rustls_pemfile::certs(&mut cert_reader)
            .map_err(|e| anyhow!("Failed to parse certificates: {}", e))?
            .into_iter()
            .map(Certificate)
            .collect();

        let mut keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
            .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;

        if keys.is_empty() {
            return Err(anyhow!("No private key found"));
        }

        let key = PrivateKey(keys.remove(0));

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| anyhow!("Failed to create TLS config: {}", e))?;

        Ok(Arc::new(config))
    }
}

/// Security context for request processing
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub auth: AuthConfig,
    pub tls: TlsConfig,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            tls: TlsConfig::default(),
        }
    }
}

impl SecurityContext {
    /// Create from environment
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            auth: AuthConfig::from_env()?,
            tls: TlsConfig::from_env(),
        })
    }

    /// Authenticate HTTP request using Basic Auth
    pub fn authenticate_request(&self, headers: &HeaderMap) -> Result<bool, StatusCode> {
        if !self.auth.enabled {
            return Ok(true);
        }

        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Basic ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let encoded = &auth_header[6..];
        let decoded = general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let credentials = String::from_utf8(decoded).map_err(|_| StatusCode::UNAUTHORIZED)?;

        let parts: Vec<&str> = credentials.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let username = parts[0];
        let password = parts[1];

        if self.auth.verify_user(username, password) {
            Ok(true)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }

    /// Generate authentication challenge header
    pub fn auth_challenge_header(&self) -> (&'static str, &'static str) {
        ("WWW-Authenticate", "Basic realm=\"OmenDB\"")
    }
}

/// Simple password hashing (in production, use bcrypt or argon2)
fn hash_password(password: &str) -> String {
    // This is a simple hash for demonstration
    // In production, use bcrypt, scrypt, or argon2
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Verify password against hash
fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

/// Generate self-signed certificate for development
pub fn generate_self_signed_cert(cert_path: &str, key_path: &str) -> Result<()> {
    // This would use rcgen crate to generate certificates
    // For now, provide instructions for manual generation
    println!("To generate self-signed certificates for development:");
    println!("mkdir -p certs");
    println!("openssl req -x509 -newkey rsa:4096 -keyout {} -out {} -days 365 -nodes -subj '/CN=localhost'", key_path, cert_path);

    Err(anyhow!(
        "Please generate certificates manually using the command above"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert!(config.users.contains_key("admin"));
        assert_eq!(config.session_timeout, 3600);
    }

    #[test]
    fn test_user_management() {
        let mut config = AuthConfig::new();

        // Add user
        config.add_user("testuser", "testpass");
        assert!(config.verify_user("testuser", "testpass"));
        assert!(!config.verify_user("testuser", "wrongpass"));
        assert!(!config.verify_user("wronguser", "testpass"));

        // Remove user
        config.remove_user("testuser");
        assert!(!config.verify_user("testuser", "testpass"));
    }

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash1 = hash_password(password);
        let hash2 = hash_password(password);

        // Same password should produce same hash
        assert_eq!(hash1, hash2);

        // Verification should work
        assert!(verify_password(password, &hash1));
        assert!(!verify_password("wrong", &hash1));
    }

    #[test]
    fn test_auth_disabled() {
        let mut config = AuthConfig::new();
        config.enabled = false;

        // Should always return true when disabled
        assert!(config.verify_user("any", "password"));
        assert!(config.verify_user("", ""));
    }

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert_eq!(config.cert_file, "certs/server.crt");
        assert_eq!(config.key_file, "certs/server.key");
    }

    #[test]
    fn test_basic_auth_header_parsing() {
        let security_ctx = SecurityContext::default();
        let mut headers = HeaderMap::new();

        // Valid Basic Auth
        let credentials = general_purpose::STANDARD.encode("admin:admin123");
        headers.insert(
            "authorization",
            format!("Basic {}", credentials).parse().unwrap(),
        );

        let result = security_ctx.authenticate_request(&headers);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_invalid_auth_header() {
        let security_ctx = SecurityContext::default();
        let mut headers = HeaderMap::new();

        // Invalid header
        headers.insert("authorization", "Bearer token123".parse().unwrap());

        let result = security_ctx.authenticate_request(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_missing_auth_header() {
        let security_ctx = SecurityContext::default();
        let headers = HeaderMap::new();

        let result = security_ctx.authenticate_request(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_wrong_credentials() {
        let security_ctx = SecurityContext::default();
        let mut headers = HeaderMap::new();

        // Wrong credentials
        let credentials = general_purpose::STANDARD.encode("admin:wrongpass");
        headers.insert(
            "authorization",
            format!("Basic {}", credentials).parse().unwrap(),
        );

        let result = security_ctx.authenticate_request(&headers);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::UNAUTHORIZED);
    }
}
