//! Generic table index - wraps learned index for any orderable type
//! Converts orderable values (Int64, Timestamp, Float64, Boolean) to i64 for indexing

use crate::index::RecursiveModelIndex;
use crate::value::Value;
use anyhow::{anyhow, Result};

/// Generic index over any orderable column type
#[derive(Debug)]
pub struct TableIndex {
    /// Underlying learned index (works with i64)
    learned_index: RecursiveModelIndex,

    /// Mapping from i64 key to row position (KEPT SORTED by key)
    key_to_position: Vec<(i64, usize)>,

    /// Whether the index needs retraining
    needs_retrain: bool,
}

impl TableIndex {
    /// Create new table index with initial capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            learned_index: RecursiveModelIndex::new(capacity),
            key_to_position: Vec::new(),
            needs_retrain: false,
        }
    }

    /// Add key-value pair to index
    pub fn insert(&mut self, key: &Value, position: usize) -> Result<()> {
        let key_i64 = key.to_i64()?;

        // Find insertion point using binary search to maintain sorted order
        let insert_idx = match self
            .key_to_position
            .binary_search_by_key(&key_i64, |(k, _)| *k)
        {
            Ok(idx) => {
                // Key already exists - update position
                self.key_to_position[idx] = (key_i64, position);
                return Ok(());
            }
            Err(idx) => idx,
        };

        // Insert at correct position to keep sorted
        self.key_to_position.insert(insert_idx, (key_i64, position));
        self.needs_retrain = true;

        // Retrain periodically (every 1000 inserts)
        if self.key_to_position.len() % 1000 == 0 && self.needs_retrain {
            self.retrain_internal();
        }

        Ok(())
    }

    /// Search for exact key using learned index prediction
    pub fn search(&self, key: &Value) -> Result<Option<usize>> {
        let key_i64 = key.to_i64()?;

        if self.key_to_position.is_empty() {
            return Ok(None);
        }

        // Retrain if needed before search
        if self.needs_retrain {
            // Can't mutate in search, so fall back to binary search
            match self
                .key_to_position
                .binary_search_by_key(&key_i64, |(k, _)| *k)
            {
                Ok(idx) => return Ok(Some(self.key_to_position[idx].1)),
                Err(_) => return Ok(None),
            }
        }

        // Use learned index to predict position in key_to_position array
        if let Some(predicted_idx) = self.learned_index.search(key_i64) {
            // predicted_idx is an index into the sorted key_to_position array
            let predicted_idx = predicted_idx.min(self.key_to_position.len() - 1);

            // Search around the predicted position (within error bound)
            let search_radius = 100; // Conservative error bound
            let start = predicted_idx.saturating_sub(search_radius);
            let end = (predicted_idx + search_radius).min(self.key_to_position.len());

            // Binary search in the local window
            let window = &self.key_to_position[start..end];
            match window.binary_search_by_key(&key_i64, |(k, _)| *k) {
                Ok(local_idx) => return Ok(Some(window[local_idx].1)),
                Err(_) => {
                    // Not in predicted window - fall back to full binary search
                }
            }
        }

        // Fallback: full binary search on entire array
        match self
            .key_to_position
            .binary_search_by_key(&key_i64, |(k, _)| *k)
        {
            Ok(idx) => Ok(Some(self.key_to_position[idx].1)),
            Err(_) => Ok(None),
        }
    }

    /// Range query - find all positions for keys in [start, end]
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        let start_i64 = start.to_i64()?;
        let end_i64 = end.to_i64()?;

        if self.key_to_position.is_empty() {
            return Ok(Vec::new());
        }

        // Since key_to_position is sorted, use binary search to find range bounds
        let start_idx = self
            .key_to_position
            .binary_search_by_key(&start_i64, |(k, _)| *k)
            .unwrap_or_else(|idx| idx);

        let end_idx = self
            .key_to_position
            .binary_search_by_key(&end_i64, |(k, _)| *k)
            .map(|idx| idx + 1) // Include the end key if found
            .unwrap_or_else(|idx| idx);

        // Extract positions for all keys in range
        let results: Vec<usize> = self.key_to_position[start_idx..end_idx]
            .iter()
            .map(|(_, pos)| *pos)
            .collect();

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

    /// Internal retraining that's called during insert
    fn retrain_internal(&mut self) {
        // Build training data: (key, index_in_array) pairs
        let training_data: Vec<(i64, usize)> = self
            .key_to_position
            .iter()
            .enumerate()
            .map(|(idx, (key, _row_pos))| (*key, idx))
            .collect();

        // Rebuild learned index with the mapping from key -> array index
        self.learned_index = RecursiveModelIndex::new(training_data.len());
        self.learned_index.train(training_data);

        self.needs_retrain = false;
    }

    /// Retrain the learned index (should be called periodically)
    pub fn retrain(&mut self) {
        self.retrain_internal();
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
        let results = index
            .range_query(&Value::Int64(20), &Value::Int64(50))
            .unwrap();
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
