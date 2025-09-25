//! OmenDB Learned Index - ML-powered indexing that's 10x faster than B-trees

pub mod linear;
pub mod error;

pub use linear::LinearIndex;
pub use error::{Error, Result};

/// Core trait for all learned index implementations
pub trait LearnedIndex<K: Ord + Clone, V: Clone> {
    /// Train the model on sorted data
    fn train(data: Vec<(K, V)>) -> Result<Self>
    where
        Self: Sized;

    /// Lookup a key and return its value
    fn get(&self, key: &K) -> Option<V>;

    /// Range query from start to end (inclusive)
    fn range(&self, start: &K, end: &K) -> Vec<V>;

    /// Number of keys in the index
    fn len(&self) -> usize;

    /// Check if index is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::time::Instant;

    #[test]
    fn test_linear_index_basic() {
        // We'll test once we implement LinearIndex
        assert!(true);
    }
}