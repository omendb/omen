//! Generic table index - wraps ALEX learned index for any orderable type
//! Converts orderable values (Int64, Timestamp, Float64, Boolean) to i64 for indexing

use crate::alex::AlexTree;
use crate::value::Value;
use anyhow::Result;

/// Generic index over any orderable column type
///
/// Uses ALEX (Adaptive Learned indEX) internally for dynamic workloads.
/// ALEX handles inserts/deletes efficiently with gapped arrays and local node splits,
/// avoiding O(n) rebuilds that plague RMI.
#[derive(Debug)]
pub struct TableIndex {
    /// ALEX tree stores (key → row_position) mappings
    alex: AlexTree,
}

impl TableIndex {
    /// Create new table index
    ///
    /// Note: capacity parameter ignored - ALEX auto-sizes
    pub fn new(_capacity: usize) -> Self {
        Self {
            alex: AlexTree::new(),
        }
    }

    /// Add key-value pair to index
    ///
    /// **Time complexity**: O(log n) amortized (ALEX handles splits automatically)
    pub fn insert(&mut self, key: &Value, position: usize) -> Result<()> {
        let key_i64 = key.to_i64()?;
        let value = position.to_le_bytes().to_vec();
        self.alex.insert(key_i64, value)
    }

    /// Search for exact key using ALEX prediction + exponential search
    ///
    /// **Time complexity**: O(log n) to find leaf + O(log error) to search
    pub fn search(&self, key: &Value) -> Result<Option<usize>> {
        let key_i64 = key.to_i64()?;

        match self.alex.get(key_i64)? {
            Some(bytes) => {
                let pos_bytes: [u8; 8] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid position encoding"))?;
                let position = usize::from_le_bytes(pos_bytes);
                Ok(Some(position))
            }
            None => Ok(None),
        }
    }

    /// Range query - find all positions for keys in [start, end]
    ///
    /// **Time complexity**: O(log n) to find start + O(result_size)
    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        let start_i64 = start.to_i64()?;
        let end_i64 = end.to_i64()?;

        let results = self.alex.range(start_i64, end_i64)?;

        results
            .into_iter()
            .map(|(_, bytes)| {
                let pos_bytes: [u8; 8] = bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid position encoding"))?;
                Ok(usize::from_le_bytes(pos_bytes))
            })
            .collect()
    }

    /// Get number of keys in index
    pub fn len(&self) -> usize {
        self.alex.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.alex.is_empty()
    }

    /// Retrain the index (no-op for ALEX - it auto-retrains)
    ///
    /// Kept for API compatibility. ALEX handles retraining automatically
    /// during node splits, so this is now a no-op.
    pub fn retrain(&mut self) {
        // ALEX handles retraining automatically - no manual intervention needed
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

        // Retrain (no-op with ALEX)
        index.retrain();

        // Verify searches still work
        assert_eq!(index.search(&Value::Int64(5)).unwrap(), Some(5));
    }
}
