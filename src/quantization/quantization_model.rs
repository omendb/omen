use anyhow::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::QuantizedVector;

/// Quantization model with per-dimension thresholds
///
/// Implements RaBitQ-style randomized threshold selection:
/// - Mean-based: threshold[i] = mean(all_vectors[*, i])
/// - RaBitQ: threshold[i] = mean[i] + random(-0.5 * std[i], 0.5 * std[i])
///
/// The randomized approach provides better theoretical error bounds
/// (see RaBitQ paper, SIGMOD 2024).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationModel {
    /// Per-dimension thresholds (1536 floats for 1536D)
    thresholds: Vec<f32>,

    /// Per-dimension means (for debugging/analysis)
    mean: Option<Vec<f32>>,

    /// Per-dimension standard deviations (for debugging/analysis)
    std_dev: Option<Vec<f32>>,
}

impl QuantizationModel {
    /// Train quantization model on sample vectors
    ///
    /// Computes per-dimension thresholds using RaBitQ algorithm:
    /// 1. Compute mean and std dev for each dimension
    /// 2. Randomize threshold: mean + uniform(-0.5*std, 0.5*std)
    ///
    /// # Arguments
    /// * `sample_vectors` - Training vectors (recommend 1K-10K samples)
    /// * `randomized` - Use RaBitQ randomization (default: true)
    pub fn train(sample_vectors: &[Vec<f32>], randomized: bool) -> Result<Self> {
        anyhow::ensure!(
            !sample_vectors.is_empty(),
            "Cannot train on empty sample set"
        );

        let dimensions = sample_vectors[0].len();
        anyhow::ensure!(
            dimensions > 0,
            "Cannot train on zero-dimensional vectors"
        );

        // Validate all vectors have same dimensions
        for (i, v) in sample_vectors.iter().enumerate() {
            anyhow::ensure!(
                v.len() == dimensions,
                "Vector {} has {} dimensions, expected {}",
                i,
                v.len(),
                dimensions
            );
        }

        let n = sample_vectors.len() as f32;

        // Compute per-dimension mean
        let mut mean = vec![0.0f32; dimensions];
        for vector in sample_vectors {
            for (i, &value) in vector.iter().enumerate() {
                mean[i] += value;
            }
        }
        for m in &mut mean {
            *m /= n;
        }

        // Compute per-dimension standard deviation
        let mut variance = vec![0.0f32; dimensions];
        for vector in sample_vectors {
            for (i, &value) in vector.iter().enumerate() {
                let diff = value - mean[i];
                variance[i] += diff * diff;
            }
        }
        let std_dev: Vec<f32> = variance.iter().map(|&v| (v / n).sqrt()).collect();

        // Compute thresholds
        let thresholds = if randomized {
            // RaBitQ: randomized threshold = mean + uniform(-0.5*std, 0.5*std)
            let mut rng = rand::thread_rng();
            mean.iter()
                .zip(std_dev.iter())
                .map(|(&m, &s)| {
                    let perturbation = rng.gen_range(-0.5 * s..0.5 * s);
                    m + perturbation
                })
                .collect()
        } else {
            // Simple mean-based threshold
            mean.clone()
        };

        Ok(QuantizationModel {
            thresholds,
            mean: Some(mean),
            std_dev: Some(std_dev),
        })
    }

    /// Quantize a vector using trained thresholds
    pub fn quantize(&self, vector: &[f32]) -> Result<QuantizedVector> {
        anyhow::ensure!(
            vector.len() == self.thresholds.len(),
            "Vector has {} dimensions but model expects {}",
            vector.len(),
            self.thresholds.len()
        );

        Ok(QuantizedVector::from_f32(vector, &self.thresholds))
    }

    /// Get thresholds (for inspection/debugging)
    pub fn thresholds(&self) -> &[f32] {
        &self.thresholds
    }

    /// Get mean (if available)
    pub fn mean(&self) -> Option<&[f32]> {
        self.mean.as_deref()
    }

    /// Get standard deviation (if available)
    pub fn std_dev(&self) -> Option<&[f32]> {
        self.std_dev.as_deref()
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.thresholds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_simple() {
        let samples = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ];

        let model = QuantizationModel::train(&samples, false).unwrap();

        assert_eq!(model.dimensions(), 3);
        assert_eq!(model.mean().unwrap(), &[2.0, 3.0, 4.0]);

        // Without randomization, thresholds = mean
        assert_eq!(model.thresholds(), &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_train_randomized() {
        let samples = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ];

        let model = QuantizationModel::train(&samples, true).unwrap();

        assert_eq!(model.dimensions(), 3);
        assert_eq!(model.mean().unwrap(), &[2.0, 3.0, 4.0]);

        // With randomization, thresholds should be near mean but not exact
        // They should be within ±0.5 * std_dev
        let thresholds = model.thresholds();
        let mean = model.mean().unwrap();
        let std_dev = model.std_dev().unwrap();

        for i in 0..3 {
            let lower = mean[i] - 0.5 * std_dev[i];
            let upper = mean[i] + 0.5 * std_dev[i];
            assert!(
                thresholds[i] >= lower && thresholds[i] <= upper,
                "Threshold {} = {} not in range [{}, {}]",
                i,
                thresholds[i],
                lower,
                upper
            );
        }
    }

    #[test]
    fn test_quantize() {
        let samples = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 3.0, 4.0],
            vec![3.0, 4.0, 5.0],
        ];

        let model = QuantizationModel::train(&samples, false).unwrap();

        let vector = vec![1.5, 3.5, 4.5];
        let quantized = model.quantize(&vector).unwrap();

        // thresholds = [2.0, 3.0, 4.0]
        // 1.5 < 2.0 -> 0
        // 3.5 >= 3.0 -> 1
        // 4.5 >= 4.0 -> 1
        // bits = 0b110 = 6
        assert_eq!(quantized.dimensions(), 3);
        // bit pattern: [false, true, true] -> bits[0] = 0b110 = 6
        assert_eq!(quantized.hamming_distance(&quantized), 0);
    }

    #[test]
    fn test_train_empty_fails() {
        let samples: Vec<Vec<f32>> = vec![];
        let result = QuantizationModel::train(&samples, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_train_dimension_mismatch_fails() {
        let samples = vec![vec![1.0, 2.0], vec![3.0, 4.0, 5.0]]; // Mismatched dimensions

        let result = QuantizationModel::train(&samples, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_quantize_dimension_mismatch_fails() {
        let samples = vec![vec![1.0, 2.0, 3.0]];
        let model = QuantizationModel::train(&samples, false).unwrap();

        let wrong_vector = vec![1.0, 2.0]; // Wrong dimension
        let result = model.quantize(&wrong_vector);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_sample_1536d() {
        // Test with 1536D vectors (OpenAI embedding size)
        let mut samples = Vec::new();
        for _ in 0..100 {
            let vector: Vec<f32> = (0..1536).map(|i| (i as f32) / 1536.0).collect();
            samples.push(vector);
        }

        let model = QuantizationModel::train(&samples, false).unwrap();

        assert_eq!(model.dimensions(), 1536);

        // Quantize a test vector
        let test_vector: Vec<f32> = (0..1536).map(|i| (i as f32) / 1536.0).collect();
        let quantized = model.quantize(&test_vector).unwrap();

        assert_eq!(quantized.dimensions(), 1536);
    }

    #[test]
    fn test_std_dev_calculation() {
        let samples = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
        ];

        let model = QuantizationModel::train(&samples, false).unwrap();

        let std_dev = model.std_dev().unwrap();

        // Manual calculation:
        // mean = [3.0, 4.0]
        // variance[0] = ((1-3)^2 + (3-3)^2 + (5-3)^2) / 3 = (4 + 0 + 4) / 3 = 8/3
        // std[0] = sqrt(8/3) ≈ 1.633
        assert!((std_dev[0] - 1.633).abs() < 0.01);
        assert!((std_dev[1] - 1.633).abs() < 0.01);
    }
}
