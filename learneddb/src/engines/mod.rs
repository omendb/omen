//! Modular engine traits for OmenDB - allows swapping implementations

use std::sync::Arc;
use std::fmt::Debug;

pub mod index;
pub mod storage;
pub mod compute;

pub use index::{IndexEngine, LearnedLinearEngine, LearnedRMIEngine, BTreeEngine};
pub use storage::{StorageEngine, RocksDBEngine, InMemoryEngine};
pub use compute::{ComputeEngine, RustSIMDEngine, ScalarEngine};

/// Result type for engine operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Core trait for index operations
pub trait IndexEngine: Send + Sync + Debug {
    /// Train the index on data
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()>;

    /// Predict position for a key (O(1) for learned indexes)
    fn predict(&self, key: i64) -> usize;

    /// Search for exact key position
    fn search(&self, key: i64) -> Option<usize>;

    /// Range query
    fn range(&self, start: i64, end: i64) -> Vec<i64>;

    /// Get statistics about the index
    fn stats(&self) -> String;

    /// Clone the engine
    fn clone_box(&self) -> Box<dyn IndexEngine>;
}

/// Core trait for storage operations
pub trait StorageEngine: Send + Sync + Debug {
    /// Get a value by key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Put a key-value pair
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;

    /// Delete a key
    fn delete(&self, key: &[u8]) -> Result<()>;

    /// Scan range of keys
    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>>;

    /// Batch write operations
    fn batch_write(&self, ops: Vec<BatchOp>) -> Result<()>;

    /// Get storage statistics
    fn stats(&self) -> String;
}

/// Core trait for compute operations (SIMD, distance calculations)
pub trait ComputeEngine: Send + Sync + Debug {
    /// Calculate distance between two vectors
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;

    /// Batch distance calculations
    fn batch_distance(&self, query: &[f32], vectors: &[&[f32]]) -> Vec<f32>;

    /// Find k nearest neighbors
    fn knn(&self, query: &[f32], vectors: &[&[f32]], k: usize) -> Vec<(usize, f32)>;

    /// Get compute statistics
    fn stats(&self) -> String;
}

/// Batch operation for storage
#[derive(Debug, Clone)]
pub enum BatchOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}

/// Factory for creating engines based on configuration
pub struct EngineFactory;

impl EngineFactory {
    /// Create index engine based on type
    pub fn create_index_engine(engine_type: &str) -> Result<Box<dyn IndexEngine>> {
        match engine_type {
            "learned_linear" => Ok(Box::new(LearnedLinearEngine::new())),
            "learned_rmi" => Ok(Box::new(LearnedRMIEngine::new())),
            "btree" => Ok(Box::new(BTreeEngine::new())),
            _ => Err(format!("Unknown index engine type: {}", engine_type).into()),
        }
    }

    /// Create storage engine based on type
    pub fn create_storage_engine(engine_type: &str, path: &str) -> Result<Box<dyn StorageEngine>> {
        match engine_type {
            "rocksdb" => Ok(Box::new(RocksDBEngine::open(path)?)),
            "memory" => Ok(Box::new(InMemoryEngine::new())),
            _ => Err(format!("Unknown storage engine type: {}", engine_type).into()),
        }
    }

    /// Create compute engine based on type
    pub fn create_compute_engine(engine_type: &str) -> Result<Box<dyn ComputeEngine>> {
        match engine_type {
            "simd" => Ok(Box::new(RustSIMDEngine::new())),
            "scalar" => Ok(Box::new(ScalarEngine::new())),
            _ => Err(format!("Unknown compute engine type: {}", engine_type).into()),
        }
    }
}

// Enable cloning for trait objects
impl Clone for Box<dyn IndexEngine> {
    fn clone(&self) -> Box<dyn IndexEngine> {
        self.clone_box()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        // Test index engine creation
        let index = EngineFactory::create_index_engine("learned_linear").unwrap();
        assert!(index.stats().contains("Linear"));

        // Test storage engine creation
        let storage = EngineFactory::create_storage_engine("memory", "").unwrap();
        assert!(storage.stats().contains("Memory"));

        // Test compute engine creation
        let compute = EngineFactory::create_compute_engine("scalar").unwrap();
        assert!(compute.stats().contains("Scalar"));
    }

    #[test]
    fn test_engine_modularity() {
        // This test demonstrates that engines are truly modular
        let engines = vec![
            EngineFactory::create_index_engine("learned_linear").unwrap(),
            EngineFactory::create_index_engine("btree").unwrap(),
        ];

        for engine in engines {
            // All engines implement the same interface
            let _pos = engine.predict(42);
            let _result = engine.search(42);
            let _stats = engine.stats();
        }
    }
}