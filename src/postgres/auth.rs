//! Authentication support for OmenDB PostgreSQL server
//!
//! Implements SCRAM-SHA-256 authentication with persistent user store.

use anyhow::Result;
use async_trait::async_trait;
use crate::user_store::{User, UserStore};
use pgwire::api::auth::scram::gen_salted_password;
use pgwire::api::auth::{AuthSource, LoginInfo, Password};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use std::path::Path;
use std::sync::Arc;

/// Persistent user store for authentication
pub struct OmenDbAuthSource {
    /// Persistent user storage backend
    user_store: Arc<UserStore>,
    /// Number of PBKDF2 iterations (default: 4096)
    iterations: usize,
}

impl OmenDbAuthSource {
    /// Create a new authentication source with persistent storage
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let user_store = UserStore::new(data_dir.as_ref().join("users"))?;

        Ok(Self {
            user_store: Arc::new(user_store),
            iterations: 4096,
        })
    }

    /// Create in-memory authentication source for testing
    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let temp_dir = tempfile::TempDir::new()?;
        let user_store = UserStore::new(temp_dir.path())?;

        Ok(Self {
            user_store: Arc::new(user_store),
            iterations: 4096,
        })
    }

    /// Add a user with plaintext password
    ///
    /// Password will be salted and hashed using SCRAM-SHA-256 PBKDF2.
    pub fn add_user(&self, username: impl Into<String>, password: &str) -> Result<()> {
        let username = username.into();

        // Generate random salt
        let salt: [u8; 16] = rand::random();

        // Hash password with salt using PBKDF2
        let salted_password = gen_salted_password(password, &salt, self.iterations);

        let user = User::new(username, salted_password, salt.to_vec());

        self.user_store.create_user(&user)?;

        Ok(())
    }

    /// Remove a user
    pub fn remove_user(&self, username: &str) -> Result<bool> {
        self.user_store.delete_user(username)
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> bool {
        self.user_store.user_exists(username).unwrap_or(false)
    }

    /// Get number of registered users
    pub fn user_count(&self) -> usize {
        self.user_store.user_count().unwrap_or(0)
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

        let user = self.user_store.get_user(username).map_err(|e| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "28P01".to_owned(),
                format!("Failed to retrieve user: {}", e),
            )))
        })?;

        let user = user.ok_or_else(|| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "28P01".to_owned(), // invalid_password SQLSTATE
                format!("User '{}' does not exist", username),
            )))
        })?;

        Ok(Password::new(
            Some(user.salt.clone()),
            user.salted_password.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_authenticate_user() {
        let auth = OmenDbAuthSource::new_in_memory().unwrap();

        // Add user
        auth.add_user("alice", "secret123").unwrap();
        assert!(auth.user_exists("alice"));
        assert_eq!(auth.user_count(), 1);

        // Check we can get password
        let login = LoginInfo::new(Some("alice"), Some("omendb"), "127.0.0.1".to_string());
        let password = auth.get_password(&login).await.unwrap();
        assert!(password.salt().is_some());
        assert!(!password.password().is_empty());
    }

    #[tokio::test]
    async fn test_remove_user() {
        let auth = OmenDbAuthSource::new_in_memory().unwrap();

        auth.add_user("bob", "password456").unwrap();
        assert!(auth.user_exists("bob"));

        let removed = auth.remove_user("bob").unwrap();
        assert!(removed);
        assert!(!auth.user_exists("bob"));
        assert_eq!(auth.user_count(), 0);
    }

    #[tokio::test]
    async fn test_nonexistent_user() {
        let auth = OmenDbAuthSource::new_in_memory().unwrap();

        let login = LoginInfo::new(Some("nonexistent"), Some("omendb"), "127.0.0.1".to_string());
        let result = auth.get_password(&login).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_username() {
        let auth = OmenDbAuthSource::new_in_memory().unwrap();

        let login = LoginInfo::new(None, Some("omendb"), "127.0.0.1".to_string());
        let result = auth.get_password(&login).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_persistence_across_restarts() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create user in first instance
        {
            let auth = OmenDbAuthSource::new(data_dir).unwrap();
            auth.add_user("alice", "secret123").unwrap();
            assert!(auth.user_exists("alice"));
        }

        // Reopen and verify user persisted
        {
            let auth = OmenDbAuthSource::new(data_dir).unwrap();
            assert!(auth.user_exists("alice"));
            assert_eq!(auth.user_count(), 1);

            // Verify password works
            let login = LoginInfo::new(Some("alice"), Some("omendb"), "127.0.0.1".to_string());
            let password = auth.get_password(&login).await.unwrap();
            assert!(password.salt().is_some());
            assert!(!password.password().is_empty());
        }
    }
}
