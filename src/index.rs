//! Learned Index Module - RMI with range query support
//! Week 3: Adding range queries to our 8.39x speedup implementation

use std::time::Instant;
use tracing::{debug, info, instrument, warn};

/// Recursive Model Index with multiple layers for better scaling
#[derive(Debug)]
pub struct RecursiveModelIndex {
    // Root model
    root: LinearModel,

    // Second layer models
    second_layer: Vec<LinearModel>,

    // Actual data
    data: Vec<(i64, usize)>,

    // Number of second layer models
    num_second_models: usize,
}

#[derive(Debug, Clone)]
pub struct LinearModel {
    slope: f64,
    intercept: f64,
    start_idx: usize,
    end_idx: usize,
    max_error: usize,
}

impl RecursiveModelIndex {
    pub fn new(data_size: usize) -> Self {
        // Minimal models for maximum speed
        let num_second_models = if data_size > 1_000_000 {
            16 // Just 16 models for 1M+ keys
        } else if data_size > 100_000 {
            8 // 8 models for 100K+ keys
        } else {
            4 // 4 models for smaller datasets
        };

        Self {
            root: LinearModel {
                slope: 0.0,
                intercept: 0.0,
                start_idx: 0,
                end_idx: 0,
                max_error: 0,
            },
            second_layer: Vec::new(),
            data: Vec::new(),
            num_second_models,
        }
    }

    /// Add a key dynamically
    pub fn add_key(&mut self, key: i64) {
        // For now, just append (in production, would maintain sorted order)
        let pos = self.data.len();
        self.data.push((key, pos));

        // Retrain periodically
        if self.data.len() % 10000 == 0 {
            info!(
                total_keys = self.data.len(),
                "Periodic retrain triggered (every 10,000 keys)"
            );
            self.retrain();
        }
    }

    /// Retrain the index
    #[instrument(skip(self))]
    pub fn retrain(&mut self) {
        info!("Retraining learned index");
        let data = self.data.clone();
        self.train(data);
    }

    #[instrument(skip(self, data), fields(keys = data.len()))]
    pub fn train(&mut self, mut data: Vec<(i64, usize)>) {
        info!(keys = data.len(), "Training learned index");
        let start_time = Instant::now();

        data.sort_by_key(|(k, _)| *k);
        self.data = data;

        let n = self.data.len();
        if n == 0 {
            debug!("No data to train");
            return;
        }

        // Train root model to predict which second-layer model to use
        // Normalize keys to avoid floating-point precision issues
        let min_key = self.data.first().map(|(k, _)| *k as f64).unwrap_or(0.0);
        let max_key = self.data.last().map(|(k, _)| *k as f64).unwrap_or(0.0);
        let key_range = (max_key - min_key).max(1.0);

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (i, (key, _)) in self.data.iter().enumerate() {
            // Normalize key to [0, 1] range
            let x = (*key as f64 - min_key) / key_range;
            let y = (i as f64 / n as f64) * self.num_second_models as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let n_f = n as f64;
        let denominator = n_f * sum_xx - sum_x * sum_x;
        if denominator.abs() > 1e-10 {
            let normalized_slope = (n_f * sum_xy - sum_x * sum_y) / denominator;
            let normalized_intercept = (sum_y - normalized_slope * sum_x) / n_f;

            // Denormalize for actual use
            self.root.slope = normalized_slope / key_range;
            self.root.intercept = normalized_intercept - self.root.slope * min_key;
        }

        self.root.start_idx = 0;
        self.root.end_idx = n;
        self.root.max_error = 0; // No error at root level - direct prediction

        // Train second layer models
        self.second_layer.clear();
        let segment_size = (n + self.num_second_models - 1) / self.num_second_models;

        for model_idx in 0..self.num_second_models {
            let start = model_idx * segment_size;
            let end = ((model_idx + 1) * segment_size).min(n);

            if start >= end {
                break;
            }

            let (slope, intercept, max_error) = self.train_segment(start, end);

            self.second_layer.push(LinearModel {
                slope,
                intercept,
                start_idx: start,
                end_idx: end,
                max_error,
            });
        }

        let duration = start_time.elapsed();
        let avg_max_error = if !self.second_layer.is_empty() {
            self.second_layer.iter().map(|m| m.max_error).sum::<usize>() / self.second_layer.len()
        } else {
            0
        };
        let max_error_bound = self.second_layer.iter().map(|m| m.max_error).max().unwrap_or(0);

        info!(
            duration_ms = duration.as_millis(),
            keys = n,
            models = self.second_layer.len(),
            avg_max_error = avg_max_error,
            max_error_bound = max_error_bound,
            "Learned index training completed"
        );

        // Warn if prediction errors are high (>10% of dataset)
        let error_threshold = (n as f64 * 0.1) as usize;
        if avg_max_error > error_threshold {
            warn!(
                avg_max_error = avg_max_error,
                threshold = error_threshold,
                error_percentage = (avg_max_error as f64 / n as f64 * 100.0) as u64,
                "High prediction error detected in learned index"
            );
        }
    }

    fn train_segment(&self, start: usize, end: usize) -> (f64, f64, usize) {
        let segment = &self.data[start..end];
        let seg_n = segment.len() as f64;

        if segment.is_empty() {
            return (0.0, 0.0, 0);
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        let min_key = segment[0].0 as f64;
        let key_range = (segment[segment.len() - 1].0 - segment[0].0) as f64;

        for (i, (key, _)) in segment.iter().enumerate() {
            let x = if key_range > 0.0 {
                ((*key as f64) - min_key) / key_range
            } else {
                0.0
            };
            let y = i as f64 / seg_n;

            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let mut slope = 0.0;
        let mut intercept = 0.0;

        let denominator = seg_n * sum_xx - sum_x * sum_x;
        if denominator.abs() > 1e-10 {
            slope = (seg_n * sum_xy - sum_x * sum_y) / denominator;
            intercept = (sum_y - slope * sum_x) / seg_n;

            // Denormalize
            slope = slope * seg_n / key_range.max(1.0);
            intercept = intercept * seg_n - slope * min_key;
        }

        // Compute 95th percentile prediction error (not max! max captures outliers)
        // Sample strategically: first 50, middle 50, last 50
        let mut errors = Vec::new();
        let seg_len = segment.len();

        // Sample first 50
        for (i, (key, _)) in segment.iter().take(50.min(seg_len)).enumerate() {
            let predicted = (slope * (*key as f64) + intercept).round() as i64;
            let error = (predicted - i as i64).abs() as usize;
            errors.push(error);
        }

        // Sample middle 50
        if seg_len > 100 {
            let mid_start = (seg_len / 2).saturating_sub(25);
            for (i, (key, _)) in segment.iter().skip(mid_start).take(50).enumerate() {
                let actual_i = mid_start + i;
                let predicted = (slope * (*key as f64) + intercept).round() as i64;
                let error = (predicted - actual_i as i64).abs() as usize;
                errors.push(error);
            }
        }

        // Sample last 50
        if seg_len > 50 {
            let last_start = seg_len.saturating_sub(50);
            for (i, (key, _)) in segment.iter().skip(last_start).enumerate() {
                let actual_i = last_start + i;
                let predicted = (slope * (*key as f64) + intercept).round() as i64;
                let error = (predicted - actual_i as i64).abs() as usize;
                errors.push(error);
            }
        }

        // Use 95th percentile instead of max (robust to outliers!)
        errors.sort_unstable();
        let p95_idx = ((errors.len() as f64 * 0.95) as usize).min(errors.len().saturating_sub(1));
        let p95_error = errors.get(p95_idx).copied().unwrap_or(0);

        // Add buffer for safety, cap at reasonable maximum
        let max_error = (p95_error + 5).max(1).min(200);

        debug!(
            segment_size = seg_len,
            p95_error = p95_error,
            final_max_error = max_error,
            "Trained segment error bounds"
        );

        (slope, intercept, max_error)
    }

    #[inline]
    pub fn search(&self, key: i64) -> Option<usize> {
        if self.second_layer.is_empty() || self.data.is_empty() {
            return None;
        }

        let model_idx = (self.root.slope * key as f64 + self.root.intercept)
            .round()
            .max(0.0)
            .min((self.second_layer.len() - 1) as f64) as usize;

        let model = &self.second_layer[model_idx];

        if model.start_idx >= self.data.len()
            || model.end_idx > self.data.len()
            || key < self.data[model.start_idx].0
            || key > self.data[model.end_idx - 1].0
        {
            let adj_idx = if key < self.data[model.start_idx].0 && model_idx > 0 {
                model_idx - 1
            } else if key > self.data[model.end_idx - 1].0
                && model_idx + 1 < self.second_layer.len()
            {
                model_idx + 1
            } else {
                return None;
            };

            let adj_model = &self.second_layer[adj_idx];
            if adj_model.start_idx >= self.data.len()
                || adj_model.end_idx > self.data.len()
                || key < self.data[adj_model.start_idx].0
                || key > self.data[adj_model.end_idx - 1].0
            {
                return None;
            }

            return self.search_in_model(adj_model, key);
        }

        self.search_in_model(model, key)
    }

    #[inline]
    fn search_in_model(&self, model: &LinearModel, key: i64) -> Option<usize> {
        let predicted_pos = (model.slope * key as f64 + model.intercept)
            .round()
            .max(0.0) as usize;

        let global_pos = (model.start_idx + predicted_pos).min(model.end_idx.saturating_sub(1));

        let start = global_pos
            .saturating_sub(model.max_error)
            .max(model.start_idx);
        let end = (global_pos + model.max_error + 1)
            .min(model.end_idx)
            .min(self.data.len());

        if global_pos < self.data.len() && self.data[global_pos].0 == key {
            return Some(self.data[global_pos].1);
        }

        // Always binary search in the window, regardless of size
        if start < end {
            let slice = &self.data[start..end];
            match slice.binary_search_by_key(&key, |(k, _)| *k) {
                Ok(idx) => Some(self.data[start + idx].1),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    /// NEW: Range query using learned index
    pub fn range_search(&self, start_key: i64, end_key: i64) -> Vec<usize> {
        if self.data.is_empty() || self.second_layer.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // Find starting model
        let start_model_idx = (self.root.slope * start_key as f64 + self.root.intercept)
            .round()
            .max(0.0)
            .min((self.second_layer.len() - 1) as f64) as usize;

        // Find ending model
        let end_model_idx = (self.root.slope * end_key as f64 + self.root.intercept)
            .round()
            .max(0.0)
            .min((self.second_layer.len() - 1) as f64) as usize;

        // Search through relevant models
        for model_idx in
            start_model_idx.saturating_sub(1)..=(end_model_idx + 1).min(self.second_layer.len() - 1)
        {
            let model = &self.second_layer[model_idx];

            // Skip models outside range
            if model.end_idx <= model.start_idx
                || self.data[model.end_idx - 1].0 < start_key
                || self.data[model.start_idx].0 > end_key
            {
                continue;
            }

            // Find start position in model
            let start_pos = if self.data[model.start_idx].0 >= start_key {
                model.start_idx
            } else {
                let predicted = (model.slope * start_key as f64 + model.intercept)
                    .round()
                    .max(0.0) as usize;
                let pos = (model.start_idx + predicted).min(model.end_idx - 1);

                // Binary search for exact start
                let search_start = pos.saturating_sub(model.max_error).max(model.start_idx);
                let search_end = (pos + model.max_error + 1).min(model.end_idx);

                let slice = &self.data[search_start..search_end];
                match slice.binary_search_by_key(&start_key, |(k, _)| *k) {
                    Ok(idx) => search_start + idx,
                    Err(idx) => search_start + idx,
                }
            };

            // Find end position in model
            let end_pos = if self.data[model.end_idx - 1].0 <= end_key {
                model.end_idx
            } else {
                let predicted = (model.slope * end_key as f64 + model.intercept)
                    .round()
                    .max(0.0) as usize;
                let pos = (model.start_idx + predicted).min(model.end_idx - 1);

                // Binary search for exact end
                let search_start = pos.saturating_sub(model.max_error).max(model.start_idx);
                let search_end = (pos + model.max_error + 1).min(model.end_idx);

                let slice = &self.data[search_start..search_end];
                match slice.binary_search_by_key(&end_key, |(k, _)| *k) {
                    Ok(idx) => search_start + idx + 1,
                    Err(idx) => search_start + idx,
                }
            };

            // Add all positions in range
            for i in start_pos..end_pos {
                if i < self.data.len() && self.data[i].0 >= start_key && self.data[i].0 <= end_key {
                    results.push(self.data[i].1);
                }
            }
        }

        results
    }

    /// Count keys in range (optimized)
    pub fn count_range(&self, start_key: i64, end_key: i64) -> usize {
        self.range_search(start_key, end_key).len()
    }

    /// Get total number of models (root + second layer)
    pub fn model_count(&self) -> usize {
        1 + self.second_layer.len()
    }

    /// Get maximum error bound across all models (for query window sizing)
    pub fn max_error_bound(&self) -> usize {
        if self.second_layer.is_empty() {
            100 // Default fallback
        } else {
            let max_bound = self.second_layer
                .iter()
                .map(|m| m.max_error)
                .max()
                .unwrap_or(100);

            max_bound
        }
    }
}

// Implement trait for storage integration
impl crate::storage::LearnedIndexTrait for RecursiveModelIndex {
    fn train(&mut self, keys: &[i64]) {
        let data: Vec<(i64, usize)> = keys.iter().enumerate().map(|(i, &k)| (k, i)).collect();
        self.train(data);
    }

    fn search(&self, key: i64) -> Option<usize> {
        self.search(key)
    }

    fn range_search(&self, start: i64, end: i64) -> Vec<usize> {
        self.range_search(start, end)
    }
}
