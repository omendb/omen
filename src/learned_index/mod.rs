//! OmenDB: World's first database using only learned indexes
//!
//! This module implements the core learned index technology that replaces
//! traditional B-trees with machine learning models for 10x performance.

pub mod linear;
pub mod hierarchical;
pub mod traits;

pub use linear::LinearLearnedIndex;
pub use hierarchical::HierarchicalLearnedIndex;
pub use traits::{LearnedIndex, LearnedIndexError, Result};

use serde::{Deserialize, Serialize};

/// Key type for our database (time-series focused)
pub type Key = i64;  // Unix timestamp in microseconds

/// Value type (for now just bytes)
pub type Value = Vec<u8>;

/// Position in storage
pub type Position = usize;

/// Configuration for learned indexes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedIndexConfig {
    /// Maximum error bound for predictions (binary search range)
    pub max_error: usize,

    /// Number of models in hierarchy
    pub num_models: usize,

    /// Whether to use GPU acceleration
    pub use_gpu: bool,

    /// Retrain threshold (when error gets too high)
    pub retrain_threshold: f64,
}

impl Default for LearnedIndexConfig {
    fn default() -> Self {
        Self {
            max_error: 100,      // Binary search within 100 positions
            num_models: 10,      // 10 models in second level
            use_gpu: false,      // CPU by default
            retrain_threshold: 2.0, // Retrain when error doubles
        }
    }
}

/// Statistics about learned index performance
#[derive(Debug, Clone, Default)]
pub struct LearnedIndexStats {
    pub lookups: u64,
    pub inserts: u64,
    pub avg_error: f64,
    pub max_observed_error: usize,
    pub retrains: u64,
}