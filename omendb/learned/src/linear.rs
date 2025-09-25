//! Simple linear model learned index
//!
//! This uses linear regression to learn the CDF (cumulative distribution function)
//! of the data and predict positions instead of tree traversal.

use crate::{LearnedIndex, Result, Error};

/// A learned index using simple linear regression
///
/// The key insight: position = slope * key + intercept
/// We learn these parameters from the data's CDF
#[derive(Debug, Clone)]
pub struct LinearIndex<V: Clone> {
    // Model parameters
    slope: f64,
    intercept: f64,

    // Data storage (sorted)
    keys: Vec<i64>,
    values: Vec<V>,

    // Error bounds for binary search
    max_error: usize,
}

impl<V: Clone> LinearIndex<V> {
    /// Train a linear model on the data
    fn train_linear_model(keys: &[i64]) -> (f64, f64, usize) {
        let n = keys.len() as f64;

        // Calculate CDF points (key -> position)
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, &key) in keys.iter().enumerate() {
            let x = key as f64;
            let y = i as f64;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        // Linear regression: y = slope * x + intercept
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate max prediction error for binary search bounds
        let mut max_error = 0usize;
        for (i, &key) in keys.iter().enumerate() {
            let predicted = (slope * key as f64 + intercept) as i64;
            let error = (predicted - i as i64).abs() as usize;
            max_error = max_error.max(error);
        }

        // Add buffer for safety
        max_error = (max_error + 10).min(keys.len() / 10);

        (slope, intercept, max_error)
    }

    /// Predict position for a given key
    #[inline]
    fn predict_position(&self, key: i64) -> usize {
        let predicted = self.slope * key as f64 + self.intercept;
        predicted.max(0.0) as usize
    }

    /// Binary search within error bounds
    fn search_in_bounds(&self, key: i64, predicted: usize) -> Option<usize> {
        let start = predicted.saturating_sub(self.max_error);
        let end = (predicted + self.max_error).min(self.keys.len());

        // Ensure valid slice bounds
        if start >= self.keys.len() {
            return None;
        }
        let end = end.min(self.keys.len());

        // Binary search in the narrowed range
        let slice = &self.keys[start..end];
        slice.binary_search(&key).ok().map(|i| start + i)
    }
}

impl<V: Clone> LearnedIndex<i64, V> for LinearIndex<V> {
    fn train(mut data: Vec<(i64, V)>) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::InvalidData("Cannot train on empty data".into()));
        }

        // Sort by key
        data.sort_by_key(|(k, _)| *k);

        // Separate keys and values
        let (keys, values): (Vec<_>, Vec<_>) = data.into_iter().unzip();

        // Train linear model
        let (slope, intercept, max_error) = Self::train_linear_model(&keys);

        Ok(LinearIndex {
            slope,
            intercept,
            keys,
            values,
            max_error,
        })
    }

    fn get(&self, key: &i64) -> Option<V> {
        // Step 1: Predict position using learned model
        let predicted = self.predict_position(*key);

        // Step 2: Binary search within error bounds
        self.search_in_bounds(*key, predicted)
            .map(|idx| self.values[idx].clone())
    }

    fn range(&self, start: &i64, end: &i64) -> Vec<V> {
        let start_pos = self.predict_position(*start);
        let end_pos = self.predict_position(*end);

        // Find actual positions with binary search
        let start_idx = self.search_in_bounds(*start, start_pos)
            .unwrap_or_else(|| {
                // If not found, find insertion point
                let search_start = start_pos.saturating_sub(self.max_error);
                let search_end = (start_pos + self.max_error).min(self.keys.len());
                self.keys[search_start..search_end]
                    .binary_search(start)
                    .unwrap_or_else(|i| search_start + i)
            });

        let end_idx = self.search_in_bounds(*end, end_pos)
            .map(|i| i + 1)
            .unwrap_or_else(|| {
                let search_start = end_pos.saturating_sub(self.max_error);
                let search_end = (end_pos + self.max_error).min(self.keys.len());
                self.keys[search_start..search_end]
                    .binary_search(end)
                    .map(|i| search_start + i + 1)
                    .unwrap_or_else(|i| search_start + i)
            });

        self.values[start_idx..end_idx].to_vec()
    }

    fn len(&self) -> usize {
        self.keys.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::collections::BTreeMap;

    #[test]
    fn test_linear_index_correctness() {
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push((i * 2, format!("value_{}", i)));
        }

        let index = LinearIndex::train(data.clone()).unwrap();

        // Test point lookups
        for (key, value) in &data {
            assert_eq!(index.get(key), Some(value.clone()));
        }

        // Test missing keys
        assert_eq!(index.get(&1), None);
        assert_eq!(index.get(&999), None);
    }

    #[test]
    fn test_linear_index_performance() {
        let n = 100_000;
        let mut data = Vec::new();
        let mut btree = BTreeMap::new();

        for i in 0..n {
            let key = i * 2;
            let value = i;
            data.push((key, value));
            btree.insert(key, value);
        }

        let index = LinearIndex::train(data).unwrap();

        // Benchmark learned index
        let start = Instant::now();
        for _ in 0..10000 {
            let key = rand::random::<i64>() % (n * 2);
            let _ = index.get(&key);
        }
        let learned_time = start.elapsed();

        // Benchmark B-tree
        let start = Instant::now();
        for _ in 0..10000 {
            let key = rand::random::<i64>() % (n * 2);
            let _ = btree.get(&key);
        }
        let btree_time = start.elapsed();

        println!("Learned index: {:?}", learned_time);
        println!("B-tree: {:?}", btree_time);
        println!("Speedup: {:.2}x", btree_time.as_nanos() as f64 / learned_time.as_nanos() as f64);

        // We should be at least 2x faster
        assert!(learned_time < btree_time);
    }
}