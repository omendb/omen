//! Authentication support for OmenDB PostgreSQL server
//!
//! Implements SCRAM-SHA-256 authentication with in-memory user store.

use anyhow::Result;
use async_trait::async_trait;
use pgwire::api::auth::scram::gen_salted_password;
use pgwire::api::auth::{AuthSource, LoginInfo, Password};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// User credentials stored in memory
#[derive(Debug, Clone)]
pub struct UserCredentials {
    /// Salted password hash using SCRAM-SHA-256
    pub salted_password: Vec<u8>,
    /// Salt used for password hashing
    pub salt: Vec<u8>,
}

/// In-memory user store for authentication
#[derive(Debug, Clone)]
pub struct OmenDbAuthSource {
    /// Map of username -> credentials
    users: Arc<RwLock<HashMap<String, UserCredentials>>>,
    /// Number of PBKDF2 iterations (default: 4096)
    iterations: usize,
}

impl Default for OmenDbAuthSource {
    fn default() -> Self {
        Self::new()
    }
}

impl OmenDbAuthSource {
    /// Create a new authentication source
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            iterations: 4096,
        }
    }

    /// Add a user with plaintext password
    ///
    /// Password will be salted and hashed using SCRAM-SHA-256 PBKDF2.
    pub async fn add_user(&self, username: impl Into<String>, password: &str) -> Result<()> {
        let username = username.into();

        // Generate random salt
        let salt: [u8; 16] = rand::random();

        // Hash password with salt using PBKDF2
        let salted_password = gen_salted_password(password, &salt, self.iterations);

        let credentials = UserCredentials {
            salted_password,
            salt: salt.to_vec(),
        };

        let mut users = self.users.write().await;
        users.insert(username, credentials);

        Ok(())
    }

    /// Remove a user
    pub async fn remove_user(&self, username: &str) -> Result<bool> {
        let mut users = self.users.write().await;
        Ok(users.remove(username).is_some())
    }

    /// Check if a user exists
    pub async fn user_exists(&self, username: &str) -> bool {
        let users = self.users.read().await;
        users.contains_key(username)
    }

    /// Get number of registered users
    pub async fn user_count(&self) -> usize {
        let users = self.users.read().await;
        users.len()
    }
}

#[async_trait]
impl AuthSource for OmenDbAuthSource {
    async fn get_password(&self, login: &LoginInfo) -> PgWireResult<Password> {
        let username = login.user().ok_or_else(|| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "28P01".to_owned(), // invalid_password SQLSTATE
                "No username provided".to_owned(),
            )))
        })?;

        let users = self.users.read().await;

        let credentials = users.get(username).ok_or_else(|| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "28P01".to_owned(), // invalid_password SQLSTATE
                format!("User '{}' does not exist", username),
            )))
        })?;

        Ok(Password::new(
            Some(credentials.salt.clone()),
            credentials.salted_password.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_authenticate_user() {
        let auth = OmenDbAuthSource::new();

        // Add user
        auth.add_user("alice", "secret123").await.unwrap();
        assert!(auth.user_exists("alice").await);
        assert_eq!(auth.user_count().await, 1);

        // Check we can get password
        let login = LoginInfo::new(Some("alice"), Some("omendb"), "127.0.0.1".to_string());
        let password = auth.get_password(&login).await.unwrap();
        assert!(password.salt().is_some());
        assert!(!password.password().is_empty());
    }

    #[tokio::test]
    async fn test_remove_user() {
        let auth = OmenDbAuthSource::new();

        auth.add_user("bob", "password456").await.unwrap();
        assert!(auth.user_exists("bob").await);

        let removed = auth.remove_user("bob").await.unwrap();
        assert!(removed);
        assert!(!auth.user_exists("bob").await);
        assert_eq!(auth.user_count().await, 0);
    }

    #[tokio::test]
    async fn test_nonexistent_user() {
        let auth = OmenDbAuthSource::new();

        let login = LoginInfo::new(Some("nonexistent"), Some("omendb"), "127.0.0.1".to_string());
        let result = auth.get_password(&login).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_username() {
        let auth = OmenDbAuthSource::new();

        let login = LoginInfo::new(None, Some("omendb"), "127.0.0.1".to_string());
        let result = auth.get_password(&login).await;
        assert!(result.is_err());
    }
}
