//! User storage layer for OmenDB authentication
//!
//! Provides persistent storage for user credentials using RocksDB.
//! Users are stored with SCRAM-SHA-256 salted password hashes.

use anyhow::{anyhow, Result};
use rocksdb::{ColumnFamilyDescriptor, Options, DB};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// User account information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// Unique username (case-sensitive)
    pub username: String,
    /// SCRAM-SHA-256 salted password hash
    pub salted_password: Vec<u8>,
    /// Random salt used for password hashing
    pub salt: Vec<u8>,
    /// Unix timestamp of user creation
    pub created_at: i64,
    /// User roles for authorization (future RBAC support)
    pub roles: Vec<String>,
}

impl User {
    /// Create a new user with password hashing parameters
    pub fn new(
        username: impl Into<String>,
        salted_password: Vec<u8>,
        salt: Vec<u8>,
    ) -> Self {
        Self {
            username: username.into(),
            salted_password,
            salt,
            created_at: chrono::Utc::now().timestamp(),
            roles: vec![],
        }
    }

    /// Create a new user with roles
    pub fn with_roles(
        username: impl Into<String>,
        salted_password: Vec<u8>,
        salt: Vec<u8>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            username: username.into(),
            salted_password,
            salt,
            created_at: chrono::Utc::now().timestamp(),
            roles,
        }
    }

    /// Validate username format
    pub fn validate_username(username: &str) -> Result<()> {
        if username.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }

        if username.len() > 63 {
            return Err(anyhow!("Username cannot exceed 63 characters"));
        }

        // PostgreSQL-compatible username rules: alphanumeric + underscore, must start with letter or underscore
        let first_char = username.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(anyhow!("Username must start with a letter or underscore"));
        }

        for c in username.chars() {
            if !c.is_alphanumeric() && c != '_' {
                return Err(anyhow!(
                    "Username must contain only letters, numbers, and underscores"
                ));
            }
        }

        Ok(())
    }
}

/// Persistent user storage using RocksDB
pub struct UserStore {
    db: Arc<DB>,
}

const CF_USERS: &str = "users";

impl UserStore {
    /// Create or open user store at the given path
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cf_users = ColumnFamilyDescriptor::new(CF_USERS, Options::default());

        let db = DB::open_cf_descriptors(&opts, path, vec![cf_users])
            .map_err(|e| anyhow!("Failed to open user store: {}", e))?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Create a new user
    ///
    /// Returns error if username already exists or is invalid.
    pub fn create_user(&self, user: &User) -> Result<()> {
        User::validate_username(&user.username)?;

        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        // Check if user already exists
        if self
            .db
            .get_cf(&cf, &user.username)?
            .is_some()
        {
            return Err(anyhow!("User '{}' already exists", user.username));
        }

        // Serialize and store user
        let serialized = bincode::serialize(user)
            .map_err(|e| anyhow!("Failed to serialize user: {}", e))?;

        self.db
            .put_cf(&cf, &user.username, serialized)
            .map_err(|e| anyhow!("Failed to store user: {}", e))?;

        Ok(())
    }

    /// Delete a user
    ///
    /// Returns true if user was deleted, false if user did not exist.
    pub fn delete_user(&self, username: &str) -> Result<bool> {
        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        // Check if user exists
        let exists = self.db.get_cf(&cf, username)?.is_some();

        if exists {
            self.db
                .delete_cf(&cf, username)
                .map_err(|e| anyhow!("Failed to delete user: {}", e))?;
        }

        Ok(exists)
    }

    /// Get user by username
    pub fn get_user(&self, username: &str) -> Result<Option<User>> {
        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        match self.db.get_cf(&cf, username)? {
            Some(bytes) => {
                let user: User = bincode::deserialize(&bytes)
                    .map_err(|e| anyhow!("Failed to deserialize user: {}", e))?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    /// Update an existing user
    ///
    /// Returns error if user does not exist.
    pub fn update_user(&self, user: &User) -> Result<()> {
        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        // Check if user exists
        if self.db.get_cf(&cf, &user.username)?.is_none() {
            return Err(anyhow!("User '{}' does not exist", user.username));
        }

        // Serialize and update user
        let serialized = bincode::serialize(user)
            .map_err(|e| anyhow!("Failed to serialize user: {}", e))?;

        self.db
            .put_cf(&cf, &user.username, serialized)
            .map_err(|e| anyhow!("Failed to update user: {}", e))?;

        Ok(())
    }

    /// List all usernames
    pub fn list_users(&self) -> Result<Vec<String>> {
        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        let mut usernames = Vec::new();
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, _) = item.map_err(|e| anyhow!("Iterator error: {}", e))?;
            let username = String::from_utf8(key.to_vec())
                .map_err(|e| anyhow!("Invalid UTF-8 in username: {}", e))?;
            usernames.push(username);
        }

        Ok(usernames)
    }

    /// Get total number of users
    pub fn user_count(&self) -> Result<usize> {
        Ok(self.list_users()?.len())
    }

    /// Check if a user exists
    pub fn user_exists(&self, username: &str) -> Result<bool> {
        let cf = self
            .db
            .cf_handle(CF_USERS)
            .ok_or_else(|| anyhow!("Users column family not found"))?;

        Ok(self.db.get_cf(&cf, username)?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_user(username: &str) -> User {
        User::new(
            username,
            vec![1, 2, 3, 4], // Mock salted password
            vec![5, 6, 7, 8], // Mock salt
        )
    }

    #[test]
    fn test_create_user() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        let user = create_test_user("alice");
        assert!(store.create_user(&user).is_ok());

        // Verify user was created
        assert!(store.user_exists("alice").unwrap());
        assert_eq!(store.user_count().unwrap(), 1);
    }

    #[test]
    fn test_delete_user() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        let user = create_test_user("bob");
        store.create_user(&user).unwrap();

        // Delete user
        let deleted = store.delete_user("bob").unwrap();
        assert!(deleted);
        assert!(!store.user_exists("bob").unwrap());

        // Delete non-existent user
        let deleted = store.delete_user("bob").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_get_user() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        let user = create_test_user("charlie");
        store.create_user(&user).unwrap();

        // Get existing user
        let retrieved = store.get_user("charlie").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().username, "charlie");

        // Get non-existent user
        let retrieved = store.get_user("nonexistent").unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_list_users() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        // Empty store
        assert_eq!(store.list_users().unwrap().len(), 0);

        // Add users
        store.create_user(&create_test_user("alice")).unwrap();
        store.create_user(&create_test_user("bob")).unwrap();
        store.create_user(&create_test_user("charlie")).unwrap();

        let usernames = store.list_users().unwrap();
        assert_eq!(usernames.len(), 3);
        assert!(usernames.contains(&"alice".to_string()));
        assert!(usernames.contains(&"bob".to_string()));
        assert!(usernames.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_duplicate_user_error() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        let user = create_test_user("alice");
        assert!(store.create_user(&user).is_ok());

        // Try to create duplicate
        let result = store.create_user(&user);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_persistence_across_restart() {
        let temp_dir = TempDir::new().unwrap();

        // Create user in first instance
        {
            let store = UserStore::new(temp_dir.path()).unwrap();
            let user = create_test_user("alice");
            store.create_user(&user).unwrap();
        }

        // Reopen and verify user persisted
        {
            let store = UserStore::new(temp_dir.path()).unwrap();
            assert!(store.user_exists("alice").unwrap());
            let user = store.get_user("alice").unwrap().unwrap();
            assert_eq!(user.username, "alice");
            assert_eq!(user.salted_password, vec![1, 2, 3, 4]);
            assert_eq!(user.salt, vec![5, 6, 7, 8]);
        }
    }

    #[test]
    fn test_concurrent_user_creation() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let store = Arc::new(UserStore::new(temp_dir.path()).unwrap());

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    let user = create_test_user(&format!("user{}", i));
                    store.create_user(&user)
                })
            })
            .collect();

        // All should succeed
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }

        assert_eq!(store.user_count().unwrap(), 10);
    }

    #[test]
    fn test_user_serialization() {
        let user = User::with_roles(
            "admin",
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec!["admin".to_string(), "superuser".to_string()],
        );

        // Serialize and deserialize
        let serialized = bincode::serialize(&user).unwrap();
        let deserialized: User = bincode::deserialize(&serialized).unwrap();

        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_empty_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        assert_eq!(store.user_count().unwrap(), 0);
        assert_eq!(store.list_users().unwrap().len(), 0);
        assert!(!store.user_exists("anyone").unwrap());
    }

    #[test]
    fn test_invalid_username() {
        // Empty username
        assert!(User::validate_username("").is_err());

        // Too long (>63 chars)
        let long_name = "a".repeat(64);
        assert!(User::validate_username(&long_name).is_err());

        // Invalid characters
        assert!(User::validate_username("user-name").is_err());
        assert!(User::validate_username("user.name").is_err());
        assert!(User::validate_username("user@name").is_err());
        assert!(User::validate_username("user name").is_err());

        // Must start with letter or underscore
        assert!(User::validate_username("123user").is_err());

        // Valid usernames
        assert!(User::validate_username("alice").is_ok());
        assert!(User::validate_username("Alice").is_ok()); // Case-sensitive
        assert!(User::validate_username("user_123").is_ok());
        assert!(User::validate_username("_user").is_ok());
    }

    #[test]
    fn test_update_user() {
        let temp_dir = TempDir::new().unwrap();
        let store = UserStore::new(temp_dir.path()).unwrap();

        let mut user = create_test_user("alice");
        store.create_user(&user).unwrap();

        // Update user roles
        user.roles = vec!["admin".to_string()];
        assert!(store.update_user(&user).is_ok());

        // Verify update
        let updated = store.get_user("alice").unwrap().unwrap();
        assert_eq!(updated.roles, vec!["admin".to_string()]);

        // Update non-existent user
        let nonexistent = create_test_user("bob");
        let result = store.update_user(&nonexistent);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }
}
