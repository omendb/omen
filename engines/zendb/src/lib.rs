//! ZenDB - Next-generation database system
//! 
//! ZenDB is a hybrid database that scales from embedded to distributed deployment
//! with PostgreSQL wire protocol compatibility and multi-modal data support.

pub mod storage;
pub mod transaction;
pub mod query;
pub mod consensus;
pub mod network;
pub mod wal;

use anyhow::Result;
use std::path::Path;

/// Configuration for ZenDB instance
#[derive(Debug, Clone)]
pub struct Config {
    /// Database file path for embedded mode
    pub data_path: Option<String>,
    /// Enable distributed mode
    pub distributed: bool,
    /// Network bind address for distributed mode
    pub bind_address: Option<String>,
    /// Cluster peers for distributed mode
    pub cluster_peers: Vec<String>,
    /// Enable WAL
    pub enable_wal: bool,
    /// Page size in bytes (default: 16384)
    pub page_size: usize,
    /// Buffer pool size in MB
    pub buffer_pool_mb: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_path: Some("zenithdb.db".to_string()),
            distributed: false,
            bind_address: None,
            cluster_peers: Vec::new(),
            enable_wal: true,
            page_size: 16384, // 16KB pages optimized for SSDs
            buffer_pool_mb: 64,
        }
    }
}

/// Main ZenDB database instance
pub struct ZenDB {
    config: Config,
    storage_engine: Box<dyn storage::StorageEngine>,
    transaction_manager: transaction::TransactionManager,
}

impl ZenDB {
    /// Create a new ZenDB instance with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }
    
    /// Create a new ZenDB instance with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Initialize storage engine based on configuration
        let storage_engine: Box<dyn storage::StorageEngine> = if config.distributed {
            // TODO: Initialize distributed storage engine
            todo!("Distributed storage engine not yet implemented")
        } else {
            // Initialize embedded storage engine
            let data_path = config.data_path.as_ref()
                .ok_or_else(|| anyhow::anyhow!("data_path required for embedded mode"))?;
            Box::new(storage::EmbeddedEngine::open(data_path)?)
        };
        
        let transaction_manager = transaction::TransactionManager::new();
        
        Ok(Self {
            config,
            storage_engine,
            transaction_manager,
        })
    }
    
    /// Execute a SQL query
    pub async fn query(&self, sql: &str) -> Result<Vec<serde_json::Value>> {
        // TODO: Implement SQL query execution
        todo!("Query execution not yet implemented")
    }
    
    /// Start a new transaction
    pub async fn begin_transaction(&self) -> Result<transaction::Transaction> {
        self.transaction_manager.begin().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = ZenDB::new();
        assert!(db.is_ok());
    }
}