//! Generic table index - wraps learned index for any orderable type
//! Converts orderable values (Int64, Timestamp, Float64, Boolean) to i64 for indexing

use crate::index::RecursiveModelIndex;
use crate::value::Value;
use anyhow::{Result, anyhow};

/// Generic index over any orderable column type
#[derive(Debug)]
pub struct TableIndex {
    /// Underlying learned index (works with i64)
    learned_index: RecursiveModelIndex,

    /// Mapping from i64 key to row position
    key_to_position: Vec<(i64, usize)>,
}

impl TableIndex {
    /// Create new table index with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            learned_index: RecursiveModelIndex::new(capacity),
            key_to_position: Vec::new(),
        }
    }

    /// Add key-value pair to index
    pub fn insert(&mut self, key: &Value, position: usize) -> Result<()> {
        // Convert key to i64
        let key_i64 = key.to_i64()?;

        // Add to learned index
        self.learned_index.add_key(key_i64);

        // Store mapping
        self.key_to_position.push((key_i64, position));

        Ok(())
    }

    /// Search for exact key
    pub fn search(&self, key: &Value) -> Result<Option<usize>> {
        let key_i64 = key.to_i64()?;

        // Use learned index hint for optimization (not yet implemented)
        let _predicted_pos = self.learned_index.search(key_i64);

        // Linear search in key_to_position
        // TODO: Use predicted position for binary search around the hint
        for (stored_key, position) in &self.key_to_position {
            if *stored_key == key_i64 {
                return Ok(Some(*position));
            }
        }

        Ok(None)
    }

    /// Range query - find all positions for keys in [start, end]
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        let start_i64 = start.to_i64()?;
        let end_i64 = end.to_i64()?;

        let mut results = Vec::new();

        for (key, position) in &self.key_to_position {
            if *key >= start_i64 && *key <= end_i64 {
                results.push(*position);
            }
        }

        Ok(results)
    }

    /// Get number of keys in index
    pub fn len(&self) -> usize {
        self.key_to_position.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.key_to_position.is_empty()
    }

    /// Retrain the learned index (should be called periodically)
    pub fn retrain(&mut self) {
        // Extract all keys
        let keys: Vec<i64> = self.key_to_position.iter()
            .map(|(k, _)| *k)
            .collect();

        // Rebuild learned index
        self.learned_index = RecursiveModelIndex::new(keys.len());
        for key in keys {
            self.learned_index.add_key(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_insert_and_search() {
        let mut index = TableIndex::new(100);

        // Insert some keys
        index.insert(&Value::Int64(100), 0).unwrap();
        index.insert(&Value::Int64(200), 1).unwrap();
        index.insert(&Value::Int64(300), 2).unwrap();

        // Search
        assert_eq!(index.search(&Value::Int64(200)).unwrap(), Some(1));
        assert_eq!(index.search(&Value::Int64(999)).unwrap(), None);
    }

    #[test]
    fn test_index_with_timestamps() {
        let mut index = TableIndex::new(100);

        // Insert timestamps
        index.insert(&Value::Timestamp(1000000), 0).unwrap();
        index.insert(&Value::Timestamp(2000000), 1).unwrap();
        index.insert(&Value::Timestamp(3000000), 2).unwrap();

        // Search
        assert_eq!(index.search(&Value::Timestamp(2000000)).unwrap(), Some(1));
    }

    #[test]
    fn test_index_range_query() {
        let mut index = TableIndex::new(100);

        // Insert keys
        for i in 0..10 {
            index.insert(&Value::Int64(i * 10), i as usize).unwrap();
        }

        // Range query [20, 50]
        let results = index.range_query(&Value::Int64(20), &Value::Int64(50)).unwrap();
        assert_eq!(results.len(), 4); // 20, 30, 40, 50
    }

    #[test]
    fn test_index_with_floats() {
        let mut index = TableIndex::new(100);

        // Float64 values can be indexed (converted to i64 via bit representation)
        index.insert(&Value::Float64(1.5), 0).unwrap();
        index.insert(&Value::Float64(2.5), 1).unwrap();
        index.insert(&Value::Float64(3.5), 2).unwrap();

        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_index_retrain() {
        let mut index = TableIndex::new(100);

        // Insert keys
        for i in 0..10 {
            index.insert(&Value::Int64(i), i as usize).unwrap();
        }

        // Retrain
        index.retrain();

        // Verify searches still work
        assert_eq!(index.search(&Value::Int64(5)).unwrap(), Some(5));
    }
}