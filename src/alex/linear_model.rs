//! Linear regression models for ALEX learned index
//!
//! ALEX uses simple linear models (y = slope * x + intercept) to predict
//! positions in sorted arrays. Linear models are:
//! - Fast to train: O(n) single pass
//! - Fast to predict: O(1) multiplication + addition
//! - Good enough: For sorted data, linear model approximates CDF well
//!
//! From ALEX paper: "Linear models provide a good balance between prediction
//! accuracy and computational cost for learned indexes."

use std::fmt;

/// Linear regression model: y = slope * x + intercept
///
/// Used to predict position of a key in a sorted array.
/// Trained using least squares regression on (key, position) pairs.
#[derive(Debug, Clone)]
pub struct LinearModel {
    slope: f64,
    intercept: f64,
}

impl LinearModel {
    /// Create a new untrained linear model
    ///
    /// Default model: y = x (identity function)
    pub fn new() -> Self {
        Self {
            slope: 1.0,
            intercept: 0.0,
        }
    }

    /// Train model using least squares linear regression
    ///
    /// Given (key, position) pairs, finds best-fit line that minimizes
    /// squared error: min Σ(predicted_pos - actual_pos)²
    ///
    /// **Algorithm**: Standard least squares regression
    /// ```text
    /// slope = (n*Σ(xy) - Σ(x)*Σ(y)) / (n*Σ(x²) - (Σ(x))²)
    /// intercept = (Σ(y) - slope*Σ(x)) / n
    /// ```
    ///
    /// **Time complexity**: O(n) single pass over data
    ///
    /// # Arguments
    /// * `data` - Slice of (key, position) pairs. Keys should be sorted.
    ///
    /// # Examples
    /// ```
    /// use omendb::alex::linear_model::LinearModel;
    ///
    /// let data = vec![(0, 0), (10, 1), (20, 2), (30, 3)];
    /// let mut model = LinearModel::new();
    /// model.train(&data);
    ///
    /// // Model should predict positions close to actual
    /// assert!((model.predict(15) as i64 - 1).abs() <= 1);
    /// ```
    pub fn train(&mut self, data: &[(i64, usize)]) {
        if data.is_empty() {
            // No data - keep identity function
            return;
        }

        if data.len() == 1 {
            // Single point - map to that position
            let (_key, pos) = data[0];
            self.slope = 0.0;
            self.intercept = pos as f64;
            return;
        }

        // Compute statistics in single pass
        let n = data.len() as f64;
        let sum_x = data.iter().map(|(k, _)| *k as f64).sum::<f64>();
        let sum_y = data.iter().map(|(_, p)| *p as f64).sum::<f64>();
        let sum_xy = data
            .iter()
            .map(|(k, p)| *k as f64 * *p as f64)
            .sum::<f64>();
        let sum_x2 = data.iter().map(|(k, _)| (*k as f64).powi(2)).sum::<f64>();

        // Compute slope and intercept
        let denominator = n * sum_x2 - sum_x * sum_x;

        if denominator.abs() < 1e-10 {
            // All keys are identical - vertical line
            // Map all to average position
            self.slope = 0.0;
            self.intercept = sum_y / n;
        } else {
            self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
            self.intercept = (sum_y - self.slope * sum_x) / n;
        }
    }

    /// Predict position for given key
    ///
    /// Returns predicted array index for key using: y = slope * key + intercept
    ///
    /// **Time complexity**: O(1)
    ///
    /// # Arguments
    /// * `key` - Key to predict position for
    ///
    /// # Returns
    /// Predicted position (clamped to valid array indices)
    ///
    /// # Examples
    /// ```
    /// use omendb::alex::linear_model::LinearModel;
    ///
    /// let data = vec![(0, 0), (100, 100), (200, 200)];
    /// let mut model = LinearModel::new();
    /// model.train(&data);
    ///
    /// // Predict position for key=50 (should be near 50)
    /// let pos = model.predict(50);
    /// assert!((pos as i64 - 50).abs() < 10);
    /// ```
    pub fn predict(&self, key: i64) -> usize {
        let pos = self.slope * key as f64 + self.intercept;
        // Clamp to valid range (non-negative)
        pos.max(0.0) as usize
    }

    /// Get model slope
    pub fn slope(&self) -> f64 {
        self.slope
    }

    /// Get model intercept
    pub fn intercept(&self) -> f64 {
        self.intercept
    }

    /// Compute maximum prediction error on training data
    ///
    /// Returns worst-case error: max|predicted_pos - actual_pos|
    ///
    /// Used to determine search window size for exponential search.
    ///
    /// **Time complexity**: O(n)
    pub fn max_error(&self, data: &[(i64, usize)]) -> usize {
        data.iter()
            .map(|(key, pos)| {
                let predicted = self.predict(*key);
                (predicted as i64 - *pos as i64).abs() as usize
            })
            .max()
            .unwrap_or(0)
    }

    /// Compute average prediction error on training data
    ///
    /// Returns mean absolute error: avg(|predicted_pos - actual_pos|)
    ///
    /// **Time complexity**: O(n)
    pub fn avg_error(&self, data: &[(i64, usize)]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let total_error: usize = data
            .iter()
            .map(|(key, pos)| {
                let predicted = self.predict(*key);
                (predicted as i64 - *pos as i64).abs() as usize
            })
            .sum();

        total_error as f64 / data.len() as f64
    }
}

impl Default for LinearModel {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LinearModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LinearModel(y = {:.6}x + {:.6})",
            self.slope, self.intercept
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_function() {
        // Default model should be y = x
        let model = LinearModel::new();
        assert_eq!(model.predict(0), 0);
        assert_eq!(model.predict(100), 100);
        assert_eq!(model.predict(1000), 1000);
    }

    #[test]
    fn test_perfect_linear_data() {
        // Perfect linear: keys = positions
        let data: Vec<(i64, usize)> = (0..100).map(|i| (i, i as usize)).collect();
        let mut model = LinearModel::new();
        model.train(&data);

        // Should have slope ≈ 1, intercept ≈ 0
        assert!((model.slope() - 1.0).abs() < 0.01);
        assert!(model.intercept().abs() < 0.01);

        // Predictions should be exact
        for i in 0..100 {
            let predicted = model.predict(i);
            assert_eq!(predicted, i as usize);
        }

        // Max error should be 0
        assert_eq!(model.max_error(&data), 0);
    }

    #[test]
    fn test_scaled_data() {
        // Keys = 10 * positions
        let data: Vec<(i64, usize)> = (0..100).map(|i| (i * 10, i as usize)).collect();
        let mut model = LinearModel::new();
        model.train(&data);

        // Should have slope ≈ 0.1, intercept ≈ 0
        assert!((model.slope() - 0.1).abs() < 0.01);
        assert!(model.intercept().abs() < 0.01);

        // Predictions should be close
        for i in 0..100 {
            let predicted = model.predict(i * 10);
            assert!((predicted as i64 - i).abs() <= 1);
        }
    }

    #[test]
    fn test_offset_data() {
        // Keys = positions + 1000 (shifted)
        let data: Vec<(i64, usize)> = (0..100).map(|i| (i + 1000, i as usize)).collect();
        let mut model = LinearModel::new();
        model.train(&data);

        // Should have slope ≈ 1, intercept ≈ -1000
        assert!((model.slope() - 1.0).abs() < 0.01);
        assert!((model.intercept() + 1000.0).abs() < 0.01);

        // Predictions should be exact
        for i in 0..100 {
            let predicted = model.predict(i + 1000);
            assert_eq!(predicted, i as usize);
        }
    }

    #[test]
    fn test_single_point() {
        // Single data point
        let data = vec![(42, 10)];
        let mut model = LinearModel::new();
        model.train(&data);

        // Should map all keys to that position
        assert_eq!(model.predict(0), 10);
        assert_eq!(model.predict(42), 10);
        assert_eq!(model.predict(100), 10);
    }

    #[test]
    fn test_duplicate_keys() {
        // All keys identical
        let data = vec![(5, 0), (5, 1), (5, 2), (5, 3)];
        let mut model = LinearModel::new();
        model.train(&data);

        // Should map to average position (≈ 1.5)
        let predicted = model.predict(5);
        assert!((predicted as i64 - 1).abs() <= 1);
    }

    #[test]
    fn test_sparse_data() {
        // Sparse keys with gaps
        let data = vec![(0, 0), (1000, 1), (2000, 2), (3000, 3)];
        let mut model = LinearModel::new();
        model.train(&data);

        // Should predict intermediate positions
        let mid = model.predict(1500);
        assert!((mid as i64 - 1).abs() <= 1);
    }

    #[test]
    fn test_error_metrics() {
        // Slightly noisy data
        let data = vec![(0, 0), (10, 2), (20, 3), (30, 5)];
        let mut model = LinearModel::new();
        model.train(&data);

        let max_err = model.max_error(&data);
        let avg_err = model.avg_error(&data);

        // Should have some error due to noise
        assert!(max_err > 0);
        assert!(avg_err > 0.0);
        assert!(max_err < 5); // But not too large
    }

    #[test]
    fn test_empty_data() {
        // Empty training data
        let data: Vec<(i64, usize)> = vec![];
        let mut model = LinearModel::new();
        model.train(&data);

        // Should keep identity function
        assert_eq!(model.slope(), 1.0);
        assert_eq!(model.intercept(), 0.0);
    }

    #[test]
    fn test_negative_keys() {
        // Negative keys
        let data: Vec<(i64, usize)> = (-50..50).map(|i| (i, (i + 50) as usize)).collect();
        let mut model = LinearModel::new();
        model.train(&data);

        // Should handle negative keys correctly
        assert_eq!(model.predict(-50), 0);
        assert_eq!(model.predict(0), 50);
        assert_eq!(model.predict(49), 99);
    }

    #[test]
    fn test_large_scale() {
        // 1M data points
        let data: Vec<(i64, usize)> = (0..1_000_000)
            .map(|i| (i as i64, i as usize))
            .collect();
        let mut model = LinearModel::new();
        model.train(&data);

        // Should have perfect fit
        assert!((model.slope() - 1.0).abs() < 0.001);
        assert!(model.intercept().abs() < 0.001);

        // Sample predictions
        assert_eq!(model.predict(0), 0);
        assert_eq!(model.predict(500_000), 500_000);
        assert_eq!(model.predict(999_999), 999_999);
    }
}
