//! Integration tests for ZenDB

use zendb::{Config, ZenDB};

#[tokio::test]
async fn test_database_creation() {
    let config = Config {
        data_path: Some("test_zendb.db".to_string()),
        ..Default::default()
    };
    
    let db = ZenDB::with_config(config);
    assert!(db.is_ok(), "Database should be created successfully");
}

#[tokio::test]
async fn test_embedded_mode() {
    let mut config = Config::default();
    config.distributed = false;
    config.data_path = Some("test_embedded.db".to_string());
    
    let db = ZenDB::with_config(config);
    assert!(db.is_ok(), "Embedded mode should initialize");
}

// TODO: Add more integration tests as features are implemented
// - Test time-travel queries
// - Test real-time subscriptions
// - Test vector operations
// - Test concurrent access