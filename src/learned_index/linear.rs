//! Linear learned index - simple but effective for sequential data

use super::{Key, Position, LearnedIndexConfig, LearnedIndexStats};
use super::traits::{LearnedIndex, LearnedIndexError, Result};
use ndarray::{Array1, Array2};
use linfa::prelude::*;
use linfa_linear::LinearRegression;
use std::collections::BTreeMap;

/// Simple linear learned index using linear regression
///
/// Perfect for time-series data where keys are mostly sequential
pub struct LinearLearnedIndex {
    /// Linear model: position = slope * key + intercept
    slope: f64,
    intercept: f64,

    /// Sorted data for refinement
    data: Vec<(Key, Position)>,

    /// Configuration
    config: LearnedIndexConfig,

    /// Performance statistics
    stats: LearnedIndexStats,

    /// Maximum error observed during training
    max_training_error: usize,

    /// Is the model trained?
    trained: bool,
}

impl LinearLearnedIndex {
    pub fn new(config: LearnedIndexConfig) -> Self {
        Self {
            slope: 0.0,
            intercept: 0.0,
            data: Vec::new(),
            config,
            stats: LearnedIndexStats::default(),
            max_training_error: 0,
            trained: false,
        }
    }

    /// Train using simple linear regression
    fn train_model(&mut self) -> Result<()> {
        if self.data.len() < 2 {
            return Err(LearnedIndexError::TrainingFailed(
                "Need at least 2 data points".to_string()
            ));
        }

        let n = self.data.len() as f64;

        // Calculate linear regression manually for speed
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for (key, pos) in &self.data {
            let x = *key as f64;
            let y = *pos as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        // Calculate slope and intercept
        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator == 0.0 {
            return Err(LearnedIndexError::TrainingFailed(
                "All keys are identical".to_string()
            ));
        }

        self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
        self.intercept = (sum_y - self.slope * sum_x) / n;

        // Calculate max error for binary search bounds
        self.max_training_error = 0;
        for (key, actual_pos) in &self.data {
            let predicted = self.predict_raw(*key);
            let error = (predicted as i64 - *actual_pos as i64).abs() as usize;
            self.max_training_error = self.max_training_error.max(error);
        }

        // Add buffer to max error
        self.max_training_error = (self.max_training_error + 10)
            .min(self.data.len() / 10)
            .max(self.config.max_error);

        self.trained = true;
        self.stats.retrains += 1;

        Ok(())
    }

    /// Raw prediction without bounds checking
    #[inline]
    fn predict_raw(&self, key: Key) -> Position {
        (self.slope * key as f64 + self.intercept).max(0.0) as Position
    }

    /// Binary search within error bounds
    fn search_in_bounds(&self, key: Key, predicted: Position) -> Result<Position> {
        let start = predicted.saturating_sub(self.max_training_error);
        let end = (predicted + self.max_training_error).min(self.data.len());

        // Binary search in the narrowed range
        let slice = &self.data[start..end];

        match slice.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(idx) => Ok(self.data[start + idx].1),
            Err(_) => Err(LearnedIndexError::KeyNotFound(key)),
        }
    }
}

impl LearnedIndex for LinearLearnedIndex {
    fn train(&mut self, data: &[(Key, Position)]) -> Result<()> {
        self.data = data.to_vec();
        self.data.sort_by_key(|(k, _)| *k);
        self.train_model()
    }

    fn predict(&self, key: Key) -> Position {
        if !self.trained {
            return 0;
        }
        let pos = self.predict_raw(key);
        pos.min(self.data.len().saturating_sub(1))
    }

    fn search(&self, key: Key) -> Result<Position> {
        if !self.trained {
            return Err(LearnedIndexError::NotTrained);
        }

        self.stats.lookups += 1;

        let predicted = self.predict(key);
        self.search_in_bounds(key, predicted)
    }

    fn insert(&mut self, key: Key, position: Position) -> Result<()> {
        self.stats.inserts += 1;

        // Simple append for now (time-series usually appends)
        self.data.push((key, position));

        // Retrain if error threshold exceeded
        if self.needs_retrain() {
            self.train_model()?;
        }

        Ok(())
    }

    fn range(&self, start: Key, end: Key) -> Result<Vec<Position>> {
        if !self.trained {
            return Err(LearnedIndexError::NotTrained);
        }

        // Use predictions to narrow search
        let start_pos = self.predict(start);
        let end_pos = self.predict(end);

        // Expand by error bounds
        let search_start = start_pos.saturating_sub(self.max_training_error);
        let search_end = (end_pos + self.max_training_error).min(self.data.len());

        let mut results = Vec::new();
        for i in search_start..search_end {
            let (k, pos) = self.data[i];
            if k >= start && k <= end {
                results.push(pos);
            }
        }

        Ok(results)
    }

    fn error_bound(&self) -> usize {
        self.max_training_error
    }

    fn needs_retrain(&self) -> bool {
        // Retrain if we've inserted 10% new data
        let insert_ratio = self.stats.inserts as f64 / self.data.len().max(1) as f64;
        insert_ratio > 0.1
    }

    fn stats(&self) -> String {
        format!(
            "LinearLearnedIndex: {} keys, slope={:.6}, intercept={:.2}, max_error={}, lookups={}, inserts={}",
            self.data.len(), self.slope, self.intercept, self.max_training_error,
            self.stats.lookups, self.stats.inserts
        )
    }
}