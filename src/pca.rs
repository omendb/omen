// PCA (Principal Component Analysis) for Vector Dimensionality Reduction
//
// Custom implementation using SVD for maximum control and simplicity.
// Reduces high-dimensional vectors (e.g., 1536D OpenAI embeddings) to lower dimensions
// (e.g., 64D) while preserving maximum variance.
//
// References:
// - docs/architecture/research/pca_alex_approach_oct_2025.md
// - LIDER paper (VLDB 2023): Learned indexes for high-dimensional vectors

use anyhow::{Context, Result};
use ndarray::{s, Array1, Array2, Axis};
use ndarray_linalg::SVDInto; // SVDInto trait for consuming arrays
use serde::{Deserialize, Serialize};

/// PCA model for vector dimensionality reduction
///
/// Uses Singular Value Decomposition (SVD) to find principal components.
/// Trained on sample vectors, then projects new vectors to PCA space.
#[derive(Clone, Serialize, Deserialize)]
pub struct VectorPCA {
    /// Number of input dimensions (e.g., 1536 for OpenAI embeddings)
    input_dims: usize,

    /// Number of output PCA components (e.g., 64)
    output_dims: usize,

    /// Mean vector for centering (subtract before projection)
    #[serde(skip)]
    mean: Option<Array1<f64>>,

    /// Principal components (eigenvectors)
    /// Shape: (input_dims, output_dims)
    #[serde(skip)]
    components: Option<Array2<f64>>,

    /// Explained variance ratio (cumulative)
    /// How much of total variance is preserved by PCA components
    explained_variance: f32,
}

impl VectorPCA {
    /// Create a new PCA model with specified dimensions
    pub fn new(input_dims: usize, output_dims: usize) -> Self {
        Self {
            input_dims,
            output_dims,
            mean: None,
            components: None,
            explained_variance: 0.0,
        }
    }

    /// Train PCA on sample vectors using SVD
    ///
    /// Algorithm:
    /// 1. Center data: X_centered = X - mean(X)
    /// 2. SVD: X_centered = U * Σ * V^T
    /// 3. Principal components = first k columns of V
    /// 4. Explained variance = (Σ_k^2) / sum(Σ^2)
    ///
    /// Returns explained variance ratio (0.0-1.0)
    pub fn train(&mut self, training_vectors: &[Vec<f32>]) -> Result<f32> {
        if training_vectors.is_empty() {
            anyhow::bail!("Training vectors cannot be empty");
        }

        if training_vectors[0].len() != self.input_dims {
            anyhow::bail!(
                "Training vector dimension mismatch: expected {}, got {}",
                self.input_dims,
                training_vectors[0].len()
            );
        }

        let n_samples = training_vectors.len();

        // Convert to Array2<f64> (n_samples × input_dims)
        let mut data = Array2::zeros((n_samples, self.input_dims));
        for (i, vec) in training_vectors.iter().enumerate() {
            for (j, &val) in vec.iter().enumerate() {
                data[[i, j]] = val as f64;
            }
        }

        // Compute mean vector
        let mean = data.mean_axis(Axis(0)).context("Failed to compute mean")?;

        // Center data: subtract mean from each row
        let mut centered = data.clone();
        for mut row in centered.rows_mut() {
            row -= &mean;
        }

        // Perform SVD: X = U * Σ * V^T
        // We want V (right singular vectors) which are the principal components
        let svd_result = centered.svd_into(true, true)
            .context("SVD decomposition failed")?;

        let (_u, sigma, v_t) = svd_result;

        let v_t = v_t.context("SVD did not return V^T")?;

        // Principal components are rows of V^T (or columns of V)
        // We want first output_dims components
        let components = v_t.slice(s![0..self.output_dims, ..]).t().to_owned();

        // Calculate explained variance
        let total_variance: f64 = sigma.iter().map(|s| s * s).sum();
        let explained: f64 = sigma.iter().take(self.output_dims).map(|s| s * s).sum();
        self.explained_variance = (explained / total_variance) as f32;

        self.mean = Some(mean);
        self.components = Some(components);

        Ok(self.explained_variance)
    }

    /// Project a single vector to PCA space
    ///
    /// Algorithm:
    /// 1. Center: x_centered = x - mean
    /// 2. Project: x_pca = x_centered * components
    pub fn project(&self, vector: &[f32]) -> Result<Vec<f32>> {
        let mean = self.mean.as_ref()
            .context("PCA model not trained yet. Call train() first.")?;
        let components = self.components.as_ref()
            .context("PCA model not trained yet. Call train() first.")?;

        if vector.len() != self.input_dims {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.input_dims,
                vector.len()
            );
        }

        // Convert to Array1<f64>
        let x: Array1<f64> = vector.iter().map(|&v| v as f64).collect();

        // Center: subtract mean
        let centered = &x - mean;

        // Project: multiply by components matrix
        let projected = components.t().dot(&centered);

        // Convert back to Vec<f32>
        let result: Vec<f32> = projected.iter().map(|&v| v as f32).collect();

        Ok(result)
    }

    /// Project multiple vectors to PCA space (batch operation)
    pub fn project_batch(&self, vectors: &[Vec<f32>]) -> Result<Vec<Vec<f32>>> {
        let mean = self.mean.as_ref()
            .context("PCA model not trained yet. Call train() first.")?;
        let components = self.components.as_ref()
            .context("PCA model not trained yet. Call train() first.")?;

        if vectors.is_empty() {
            return Ok(vec![]);
        }

        // Convert to Array2<f64>
        let n_vectors = vectors.len();
        let mut data = Array2::zeros((n_vectors, self.input_dims));
        for (i, vec) in vectors.iter().enumerate() {
            if vec.len() != self.input_dims {
                anyhow::bail!(
                    "Vector {} dimension mismatch: expected {}, got {}",
                    i,
                    self.input_dims,
                    vec.len()
                );
            }
            for (j, &val) in vec.iter().enumerate() {
                data[[i, j]] = val as f64;
            }
        }

        // Center: subtract mean from each row
        let mut centered = data;
        for mut row in centered.rows_mut() {
            row -= mean;
        }

        // Project all vectors: (n × input_dims) @ (input_dims × output_dims) = (n × output_dims)
        let projected = centered.dot(components);

        // Convert back to Vec<Vec<f32>>
        let mut results = Vec::with_capacity(n_vectors);
        for i in 0..n_vectors {
            let row: Vec<f32> = projected.row(i).iter().map(|&v| v as f32).collect();
            results.push(row);
        }

        Ok(results)
    }

    /// Get explained variance ratio
    pub fn explained_variance_ratio(&self) -> f32 {
        self.explained_variance
    }

    /// Get input dimensionality
    pub fn input_dims(&self) -> usize {
        self.input_dims
    }

    /// Get output dimensionality
    pub fn output_dims(&self) -> usize {
        self.output_dims
    }

    /// Check if model is trained
    pub fn is_trained(&self) -> bool {
        self.mean.is_some() && self.components.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    fn generate_random_vectors(n: usize, dims: usize) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        (0..n)
            .map(|_| (0..dims).map(|_| rng.gen::<f32>()).collect())
            .collect()
    }

    #[test]
    fn test_pca_basic() {
        let mut pca = VectorPCA::new(128, 16);

        let training_data = generate_random_vectors(1000, 128);
        let variance = pca.train(&training_data).unwrap();

        assert!(variance > 0.0);
        assert!(variance <= 1.0);
        assert!(pca.is_trained());

        println!("PCA (128D → 16D) preserves {:.1}% variance", variance * 100.0);
    }

    #[test]
    fn test_pca_projection() {
        let mut pca = VectorPCA::new(128, 16);
        let training_data = generate_random_vectors(1000, 128);
        pca.train(&training_data).unwrap();

        let test_vector = generate_random_vectors(1, 128);
        let projected = pca.project(&test_vector[0]).unwrap();

        assert_eq!(projected.len(), 16);
    }

    #[test]
    fn test_pca_batch_projection() {
        let mut pca = VectorPCA::new(128, 16);
        let training_data = generate_random_vectors(1000, 128);
        pca.train(&training_data).unwrap();

        let test_vectors = generate_random_vectors(100, 128);
        let projected = pca.project_batch(&test_vectors).unwrap();

        assert_eq!(projected.len(), 100);
        assert_eq!(projected[0].len(), 16);
    }

    #[test]
    fn test_pca_preserves_variance() {
        let mut pca = VectorPCA::new(256, 64);
        let training_data = generate_random_vectors(10000, 256);
        let variance = pca.train(&training_data).unwrap();

        // With 64 components out of 256, should preserve reasonable variance
        assert!(variance > 0.5, "Variance too low: {}", variance);

        println!("PCA (256D → 64D) preserves {:.1}% variance", variance * 100.0);
    }

    #[test]
    fn test_pca_high_dimensional() {
        // Test with realistic dimensions (OpenAI embeddings)
        let mut pca = VectorPCA::new(1536, 64);
        let training_data = generate_random_vectors(5000, 1536);
        let variance = pca.train(&training_data).unwrap();

        assert!(variance > 0.0);
        assert!(pca.is_trained());

        println!("PCA (1536D → 64D) preserves {:.1}% variance", variance * 100.0);

        // Project a test vector
        let test_vec = generate_random_vectors(1, 1536);
        let projected = pca.project(&test_vec[0]).unwrap();
        assert_eq!(projected.len(), 64);
    }

    #[test]
    fn test_pca_dimension_mismatch() {
        let mut pca = VectorPCA::new(128, 16);
        let training_data = generate_random_vectors(1000, 128);
        pca.train(&training_data).unwrap();

        let wrong_dims = vec![0.0f32; 64];
        let result = pca.project(&wrong_dims);

        assert!(result.is_err());
    }

    #[test]
    fn test_pca_not_trained() {
        let pca = VectorPCA::new(128, 16);
        let test_vector = vec![0.0f32; 128];
        let result = pca.project(&test_vector);

        assert!(result.is_err());
    }
}
