//! Recursive Model Index (RMI) - Two-stage learned index for 10x performance
//!
//! The key insight: Use a hierarchy of models instead of a single model.
//! 1. Root model predicts which leaf model to use
//! 2. Leaf models are specialized for their data segment
//!
//! This should achieve better accuracy and push toward 10x speedup.

use crate::{LearnedIndex, Result, Error};

/// A two-stage Recursive Model Index
///
/// Stage 1: Root model predicts which leaf model to use
/// Stage 2: Leaf models predict position within their segment
#[derive(Debug, Clone)]
pub struct RMIIndex<V: Clone> {
    // Root model parameters (predicts which leaf to use)
    root_slope: f64,
    root_intercept: f64,

    // Leaf models (each handles a segment of the data)
    pub leaf_models: Vec<LeafModel>,

    // Data storage (sorted)
    keys: Vec<i64>,
    values: Vec<V>,

    // Configuration
    num_leaf_models: usize,
    max_error: usize,
}

/// A single leaf model in the RMI hierarchy
#[derive(Debug, Clone)]
pub struct LeafModel {
    slope: f64,
    intercept: f64,
    start_idx: usize,
    end_idx: usize,
    max_error: usize,
}

impl<V: Clone> RMIIndex<V> {
    /// Train the two-stage RMI model
    fn train_rmi(keys: &[i64], num_leaf_models: usize) -> (f64, f64, Vec<LeafModel>, usize) {
        let n = keys.len();

        // Stage 1: Train root model to predict leaf model index
        let mut root_sum_x = 0.0;
        let mut root_sum_y = 0.0;
        let mut root_sum_xy = 0.0;
        let mut root_sum_xx = 0.0;

        for (i, &key) in keys.iter().enumerate() {
            let x = key as f64;
            // Predict which leaf model this key should use (0 to num_leaf_models-1)
            let y = ((i as f64 / n as f64) * num_leaf_models as f64).min((num_leaf_models - 1) as f64);

            root_sum_x += x;
            root_sum_y += y;
            root_sum_xy += x * y;
            root_sum_xx += x * x;
        }

        let n_float = n as f64;
        let root_slope = (n_float * root_sum_xy - root_sum_x * root_sum_y) /
                        (n_float * root_sum_xx - root_sum_x * root_sum_x);
        let root_intercept = (root_sum_y - root_slope * root_sum_x) / n_float;

        // Stage 2: Train leaf models for each segment
        let mut leaf_models = Vec::new();
        let segment_size = (n + num_leaf_models - 1) / num_leaf_models; // Ceiling division

        for i in 0..num_leaf_models {
            let start_idx = i * segment_size;
            let end_idx = ((i + 1) * segment_size).min(n);

            if start_idx >= end_idx {
                break;
            }

            let segment_keys = &keys[start_idx..end_idx];
            let leaf_model = Self::train_leaf_model(segment_keys, start_idx, end_idx);
            leaf_models.push(leaf_model);
        }

        // Calculate overall max error for the RMI
        let mut max_error = 0;
        for (actual_idx, &key) in keys.iter().enumerate() {
            // Root model prediction
            let predicted_leaf = ((root_slope * key as f64 + root_intercept) as usize)
                .min(leaf_models.len() - 1);

            // Leaf model prediction
            let leaf = &leaf_models[predicted_leaf];
            let predicted_pos = leaf.start_idx +
                ((leaf.slope * key as f64 + leaf.intercept) as usize)
                    .min(leaf.end_idx - leaf.start_idx);

            let error = (predicted_pos as i64 - actual_idx as i64).abs() as usize;
            max_error = max_error.max(error);
        }

        // Add safety buffer
        max_error = (max_error + 50).min(n / 20);

        (root_slope, root_intercept, leaf_models, max_error)
    }

    /// Train a single leaf model for a segment of the data
    fn train_leaf_model(segment_keys: &[i64], start_idx: usize, end_idx: usize) -> LeafModel {
        let n = segment_keys.len() as f64;

        if n == 0.0 {
            return LeafModel {
                slope: 0.0,
                intercept: 0.0,
                start_idx,
                end_idx,
                max_error: 0,
            };
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, &key) in segment_keys.iter().enumerate() {
            let x = key as f64;
            let y = i as f64; // Position within the segment

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let slope = if n * sum_xx - sum_x * sum_x != 0.0 {
            (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
        } else {
            0.0
        };
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate max error for this leaf model
        let mut max_error = 0;
        for (i, &key) in segment_keys.iter().enumerate() {
            let predicted = (slope * key as f64 + intercept) as i64;
            let error = (predicted - i as i64).abs() as usize;
            max_error = max_error.max(error);
        }
        max_error = (max_error + 10).min(segment_keys.len() / 4);

        LeafModel {
            slope,
            intercept,
            start_idx,
            end_idx,
            max_error,
        }
    }

    /// Predict position using the two-stage RMI
    #[inline]
    fn predict_position(&self, key: i64) -> usize {
        // Stage 1: Root model predicts which leaf model to use
        let predicted_leaf_f = (self.root_slope * key as f64 + self.root_intercept).max(0.0);
        let predicted_leaf_idx = (predicted_leaf_f as usize).min(self.leaf_models.len() - 1);

        // Stage 2: Leaf model predicts position within its segment
        let leaf = &self.leaf_models[predicted_leaf_idx];
        let relative_pos_f = (leaf.slope * key as f64 + leaf.intercept).max(0.0);
        let relative_pos = (relative_pos_f as usize).min(leaf.end_idx - leaf.start_idx - 1);

        (leaf.start_idx + relative_pos).min(self.keys.len() - 1)
    }

    /// Binary search within error bounds using the appropriate leaf model
    fn search_in_bounds(&self, key: i64, predicted: usize) -> Option<usize> {
        let start = predicted.saturating_sub(self.max_error);
        let end = (predicted + self.max_error).min(self.keys.len());

        if start >= self.keys.len() {
            return None;
        }

        let slice = &self.keys[start..end];
        slice.binary_search(&key).ok().map(|i| start + i)
    }
}

impl<V: Clone> LearnedIndex<i64, V> for RMIIndex<V> {
    fn train(mut data: Vec<(i64, V)>) -> Result<Self> {
        if data.is_empty() {
            return Err(Error::InvalidData("Cannot train on empty data".into()));
        }

        // Sort by key
        data.sort_by_key(|(k, _)| *k);

        // Separate keys and values
        let (keys, values): (Vec<_>, Vec<_>) = data.into_iter().unzip();

        // Choose number of leaf models based on data size
        // Use fewer models for better performance on smaller datasets
        let num_leaf_models = if keys.len() < 10_000 {
            2  // Just 2 models for small datasets
        } else if keys.len() < 100_000 {
            ((keys.len() as f64).sqrt() / 4.0) as usize  // Fewer models than sqrt(n)
        } else {
            ((keys.len() as f64).sqrt() as usize).max(10).min(50)  // Cap at 50 models
        }.max(2);

        // Train the RMI
        let (root_slope, root_intercept, leaf_models, max_error) =
            Self::train_rmi(&keys, num_leaf_models);

        Ok(RMIIndex {
            root_slope,
            root_intercept,
            leaf_models,
            keys,
            values,
            num_leaf_models,
            max_error,
        })
    }

    fn get(&self, key: &i64) -> Option<V> {
        // Stage 1 & 2: Predict position using RMI
        let predicted = self.predict_position(*key);

        // Stage 3: Binary search within error bounds
        self.search_in_bounds(*key, predicted)
            .map(|idx| self.values[idx].clone())
    }

    fn range(&self, start: &i64, end: &i64) -> Vec<V> {
        let start_pos = self.predict_position(*start);
        let end_pos = self.predict_position(*end);

        // Find actual positions with binary search
        let start_idx = self.search_in_bounds(*start, start_pos)
            .unwrap_or_else(|| {
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
    use std::collections::BTreeMap;
    use std::time::Instant;

    #[test]
    fn test_rmi_correctness() {
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push((i * 2, format!("value_{}", i)));
        }

        let index = RMIIndex::train(data.clone()).unwrap();

        // Test point lookups
        for (key, expected_value) in &data {
            let result = index.get(key);
            assert_eq!(result, Some(expected_value.clone()),
                "Failed lookup for key {}", key);
        }

        // Test missing keys
        assert_eq!(index.get(&1), None);
        assert_eq!(index.get(&999), None);
    }

    #[test]
    fn test_rmi_vs_linear_performance() {
        use crate::LinearIndex;

        let n = 50_000;
        let mut data = Vec::new();
        let mut btree = BTreeMap::new();

        // Create data with some non-uniform distribution
        for i in 0..n {
            let key = if i < n/2 {
                i * 2  // Dense in first half
            } else {
                n + (i - n/2) * 10  // Sparse in second half
            };
            let value = i;
            data.push((key, value));
            btree.insert(key, value);
        }

        // Train both indexes
        let rmi_index = RMIIndex::train(data.clone()).unwrap();
        let linear_index = LinearIndex::train(data).unwrap();

        println!("RMI uses {} leaf models", rmi_index.leaf_models.len());

        // Test queries
        let num_queries = 5000;
        let test_keys: Vec<i64> = (0..num_queries)
            .map(|i| {
                if i < num_queries/2 {
                    (i % (n/2)) * 2
                } else {
                    n + ((i - num_queries/2) % (n/2)) * 10
                }
            })
            .collect();

        // Benchmark RMI
        let start = Instant::now();
        let mut rmi_found = 0;
        for &key in &test_keys {
            if rmi_index.get(&key).is_some() {
                rmi_found += 1;
            }
        }
        let rmi_time = start.elapsed();

        // Benchmark Linear Index
        let start = Instant::now();
        let mut linear_found = 0;
        for &key in &test_keys {
            if linear_index.get(&key).is_some() {
                linear_found += 1;
            }
        }
        let linear_time = start.elapsed();

        // Benchmark B-tree
        let start = Instant::now();
        let mut btree_found = 0;
        for &key in &test_keys {
            if btree.get(&key).is_some() {
                btree_found += 1;
            }
        }
        let btree_time = start.elapsed();

        let rmi_qps = num_queries as f64 / rmi_time.as_secs_f64();
        let linear_qps = num_queries as f64 / linear_time.as_secs_f64();
        let btree_qps = num_queries as f64 / btree_time.as_secs_f64();

        println!("Performance comparison ({} keys, {} queries):", n, num_queries);
        println!("  RMI Index:    {:.0} queries/sec ({} found)", rmi_qps, rmi_found);
        println!("  Linear Index: {:.0} queries/sec ({} found)", linear_qps, linear_found);
        println!("  BTreeMap:     {:.0} queries/sec ({} found)", btree_qps, btree_found);
        println!("  RMI vs BTree: {:.2}x speedup", rmi_qps / btree_qps);
        println!("  RMI vs Linear: {:.2}x speedup", rmi_qps / linear_qps);

        // RMI should be at least as good as linear index
        assert!(rmi_found == linear_found, "RMI and Linear should find same number of keys");
        assert!(rmi_found == btree_found, "Should find all existing keys");

        // RMI should be faster than B-tree (the main goal)
        assert!(rmi_time < btree_time, "RMI should be faster than BTreeMap");
    }

    #[test]
    fn test_rmi_range_queries() {
        let mut data = Vec::new();
        for i in 0..1000 {
            data.push((i * 3, i)); // Keys: 0, 3, 6, 9, ...
        }

        let index = RMIIndex::train(data).unwrap();

        // Test various range sizes
        let result = index.range(&100, &200);
        assert!(!result.is_empty());

        // All results should be in range
        let keys_in_range: Vec<i64> = (100..=200).step_by(3).collect();
        let expected_count = keys_in_range.len();
        assert_eq!(result.len(), expected_count);
    }
}