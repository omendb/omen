//! Fixed index engine implementations - learned and traditional

use super::{IndexEngine, Result};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// Learned linear index engine - simplified to work with existing trait
#[derive(Debug)]
pub struct LearnedLinearEngine {
    // Store the data directly with position mapping
    data: Vec<(i64, Vec<u8>)>,
    // Simple linear model parameters
    slope: f64,
    intercept: f64,
    max_error: usize,
}

impl LearnedLinearEngine {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            slope: 0.0,
            intercept: 0.0,
            max_error: 100,
        }
    }

    fn train_model(&mut self) {
        if self.data.len() < 2 {
            return;
        }

        let n = self.data.len() as f64;

        // Calculate linear regression
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, (key, _)) in self.data.iter().enumerate() {
            let x = *key as f64;
            let y = i as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        self.slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        self.intercept = (sum_y - self.slope * sum_x) / n;

        // Calculate max error
        let mut max_err = 0;
        for (i, (key, _)) in self.data.iter().enumerate() {
            let predicted = (self.slope * (*key as f64) + self.intercept) as i64;
            let error = (predicted - i as i64).abs() as usize;
            max_err = max_err.max(error);
        }
        self.max_error = (max_err + 10).min(self.data.len() / 10);
    }

    fn predict_position(&self, key: i64) -> usize {
        let pos = self.slope * key as f64 + self.intercept;
        pos.max(0.0).min((self.data.len() - 1) as f64) as usize
    }
}

impl IndexEngine for LearnedLinearEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        self.data = data.to_vec();
        self.data.sort_by_key(|(k, _)| *k);
        self.train_model();
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        self.predict_position(key)
    }

    fn search(&self, key: i64) -> Option<usize> {
        let predicted = self.predict_position(key);
        let start = predicted.saturating_sub(self.max_error);
        let end = (predicted + self.max_error).min(self.data.len());

        // Binary search in the error bounds
        let slice = &self.data[start..end];
        slice.binary_search_by_key(&key, |(k, _)| *k)
            .ok()
            .map(|i| start + i)
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        self.data
            .iter()
            .filter_map(|(k, _)| {
                if *k >= start && *k <= end {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect()
    }

    fn stats(&self) -> String {
        format!(
            "LearnedLinear: {} keys, slope={:.6}, intercept={:.2}, max_error={}",
            self.data.len(), self.slope, self.intercept, self.max_error
        )
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        Box::new(Self {
            data: self.data.clone(),
            slope: self.slope,
            intercept: self.intercept,
            max_error: self.max_error,
        })
    }
}

/// Learned RMI (Recursive Model Index) engine - simplified version
#[derive(Debug)]
pub struct LearnedRMIEngine {
    data: Vec<(i64, Vec<u8>)>,
    // Root model
    root_slope: f64,
    root_intercept: f64,
    // Leaf models (simplified to 2 for now)
    leaf_models: Vec<(f64, f64, usize, usize)>, // (slope, intercept, start, end)
    max_error: usize,
}

impl LearnedRMIEngine {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            root_slope: 0.0,
            root_intercept: 0.0,
            leaf_models: Vec::new(),
            max_error: 100,
        }
    }

    fn train_model(&mut self) {
        if self.data.len() < 2 {
            return;
        }

        let n = self.data.len();
        let num_leaves = 2.max((n as f64).sqrt() as usize / 4).min(10);

        // Train root model to predict leaf
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, (key, _)) in self.data.iter().enumerate() {
            let x = *key as f64;
            let y = (i as f64 / n as f64) * num_leaves as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let n_float = n as f64;
        self.root_slope = (n_float * sum_xy - sum_x * sum_y) / (n_float * sum_xx - sum_x * sum_x);
        self.root_intercept = (sum_y - self.root_slope * sum_x) / n_float;

        // Train leaf models
        self.leaf_models.clear();
        let segment_size = (n + num_leaves - 1) / num_leaves;

        for leaf_idx in 0..num_leaves {
            let start = leaf_idx * segment_size;
            let end = ((leaf_idx + 1) * segment_size).min(n);

            if start >= end {
                break;
            }

            // Train leaf model for this segment
            let segment = &self.data[start..end];
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_xx = 0.0;

            for (i, (key, _)) in segment.iter().enumerate() {
                let x = *key as f64;
                let y = i as f64;
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_xx += x * x;
            }

            let seg_n = segment.len() as f64;
            let slope = if seg_n * sum_xx - sum_x * sum_x != 0.0 {
                (seg_n * sum_xy - sum_x * sum_y) / (seg_n * sum_xx - sum_x * sum_x)
            } else {
                0.0
            };
            let intercept = (sum_y - slope * sum_x) / seg_n;

            self.leaf_models.push((slope, intercept, start, end));
        }

        self.max_error = 50.min(n / 20);
    }

    fn predict_position(&self, key: i64) -> usize {
        if self.leaf_models.is_empty() {
            return 0;
        }

        // Predict which leaf
        let predicted_leaf = (self.root_slope * key as f64 + self.root_intercept)
            .max(0.0)
            .min((self.leaf_models.len() - 1) as f64) as usize;

        // Use leaf model to predict position
        let (slope, intercept, start, end) = self.leaf_models[predicted_leaf];
        let relative_pos = (slope * key as f64 + intercept).max(0.0) as usize;
        (start + relative_pos).min(end - 1).min(self.data.len() - 1)
    }
}

impl IndexEngine for LearnedRMIEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        self.data = data.to_vec();
        self.data.sort_by_key(|(k, _)| *k);
        self.train_model();
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        self.predict_position(key)
    }

    fn search(&self, key: i64) -> Option<usize> {
        let predicted = self.predict_position(key);
        let start = predicted.saturating_sub(self.max_error);
        let end = (predicted + self.max_error).min(self.data.len());

        // Binary search in the error bounds
        let slice = &self.data[start..end];
        slice.binary_search_by_key(&key, |(k, _)| *k)
            .ok()
            .map(|i| start + i)
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        self.data
            .iter()
            .filter_map(|(k, _)| {
                if *k >= start && *k <= end {
                    Some(*k)
                } else {
                    None
                }
            })
            .collect()
    }

    fn stats(&self) -> String {
        format!(
            "LearnedRMI: {} keys, {} leaf models, max_error={}",
            self.data.len(), self.leaf_models.len(), self.max_error
        )
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        Box::new(Self {
            data: self.data.clone(),
            root_slope: self.root_slope,
            root_intercept: self.root_intercept,
            leaf_models: self.leaf_models.clone(),
            max_error: self.max_error,
        })
    }
}

/// Traditional B-tree index engine (fallback)
#[derive(Debug)]
pub struct BTreeEngine {
    tree: RwLock<BTreeMap<i64, usize>>,
    data: RwLock<Vec<(i64, Vec<u8>)>>,
}

impl BTreeEngine {
    pub fn new() -> Self {
        Self {
            tree: RwLock::new(BTreeMap::new()),
            data: RwLock::new(Vec::new()),
        }
    }
}

impl IndexEngine for BTreeEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        let mut tree = self.tree.write().unwrap();
        let mut data_store = self.data.write().unwrap();

        tree.clear();
        *data_store = data.to_vec();
        data_store.sort_by_key(|(k, _)| *k);

        for (pos, (key, _)) in data_store.iter().enumerate() {
            tree.insert(*key, pos);
        }
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        // B-tree doesn't predict, just searches
        self.search(key).unwrap_or(0)
    }

    fn search(&self, key: i64) -> Option<usize> {
        let tree = self.tree.read().unwrap();
        tree.get(&key).copied()
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        let tree = self.tree.read().unwrap();
        tree.range(start..=end).map(|(k, _)| *k).collect()
    }

    fn stats(&self) -> String {
        let tree = self.tree.read().unwrap();
        format!("BTree: {} keys", tree.len())
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        let tree = self.tree.read().unwrap();
        let data = self.data.read().unwrap();
        Box::new(Self {
            tree: RwLock::new(tree.clone()),
            data: RwLock::new(data.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> Vec<(i64, Vec<u8>)> {
        (0..1000)
            .map(|i| (i * 2, format!("value_{}", i).into_bytes()))
            .collect()
    }

    #[test]
    fn test_learned_linear_engine() {
        let mut engine = LearnedLinearEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }

    #[test]
    fn test_learned_rmi_engine() {
        let mut engine = LearnedRMIEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }

    #[test]
    fn test_btree_engine() {
        let mut engine = BTreeEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }
}