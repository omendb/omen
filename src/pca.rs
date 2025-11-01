//! PCA (Principal Component Analysis) for Vector Dimensionality Reduction
//!
//! Simple implementation using power iteration for eigenvalue decomposition.
//! Reduces high-dimensional vectors (e.g., 1536D OpenAI embeddings) to lower dimensions
//! (e.g., 64D) while preserving maximum variance.
//!
//! References:
//! - docs/architecture/research/pca_alex_approach_oct_2025.md
//! - LIDER paper (VLDB 2023): Learned indexes for high-dimensional vectors

use anyhow::{Context, Result};
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

/// PCA model for vector dimensionality reduction
///
/// Uses power iteration to find principal components.
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

    /// Train PCA on sample vectors using power iteration
    ///
    /// Algorithm:
    /// 1. Center data: X_centered = X - mean(X)
    /// 2. Compute covariance matrix: C = X^T * X / n
    /// 3. Find top k eigenvectors using power iteration
    /// 4. Explained variance = sum(eigenvalues_k) / sum(eigenvalues_all)
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

        // Compute covariance matrix: C = X^T * X / n
        let cov = centered.t().dot(&centered) / (n_samples as f64);

        // Find principal components using power iteration
        let mut components = Array2::zeros((self.input_dims, self.output_dims));
        let mut eigenvalues = Vec::new();

        for k in 0..self.output_dims {
            // Power iteration to find k-th eigenvector
            let (eigenvec, eigenval) = self.power_iteration(&cov, k, &components)?;

            // Store eigenvector as k-th column
            for i in 0..self.input_dims {
                components[[i, k]] = eigenvec[i];
            }

            eigenvalues.push(eigenval);
        }

        // Estimate total variance (trace of covariance matrix)
        let total_variance: f64 = (0..self.input_dims).map(|i| cov[[i, i]]).sum();
        let explained: f64 = eigenvalues.iter().sum();
        self.explained_variance = (explained / total_variance) as f32;

        self.mean = Some(mean);
        self.components = Some(components);

        Ok(self.explained_variance)
    }

    /// Power iteration to find eigenvector
    ///
    /// Finds the dominant eigenvector of matrix A that is orthogonal
    /// to previously found eigenvectors.
    fn power_iteration(
        &self,
        matrix: &Array2<f64>,
        k: usize,
        prev_components: &Array2<f64>,
    ) -> Result<(Array1<f64>, f64)> {
        let n = matrix.nrows();

        // Start with random vector
        let mut v = Array1::from_vec(
            (0..n).map(|i| ((i + k + 1) as f64).sin()).collect()
        );

        // Orthogonalize against previous components
        for j in 0..k {
            let prev = prev_components.column(j);
            let proj = v.dot(&prev);
            for i in 0..n {
                v[i] -= proj * prev[i];
            }
        }

        // Normalize
        let norm = v.dot(&v).sqrt();
        if norm > 0.0 {
            v /= norm;
        }

        // Power iteration
        for _ in 0..100 {  // Max 100 iterations
            // v_new = A * v
            let mut v_new = Array1::zeros(n);
            for i in 0..n {
                for j in 0..n {
                    v_new[i] += matrix[[i, j]] * v[j];
                }
            }

            // Orthogonalize against previous components
            for j in 0..k {
                let prev = prev_components.column(j);
                let proj = v_new.dot(&prev);
                for i in 0..n {
                    v_new[i] -= proj * prev[i];
                }
            }

            // Normalize
            let norm = v_new.dot(&v_new).sqrt();
            if norm < 1e-10 {
                break;  // Converged to zero (shouldn't happen)
            }
            v_new /= norm;

            // Check convergence
            let diff: f64 = (&v_new - &v).iter().map(|x| x * x).sum::<f64>().sqrt();
            v = v_new;

            if diff < 1e-6 {
                break;  // Converged
            }
        }

        // Compute eigenvalue: λ = v^T * A * v
        let mut av = Array1::zeros(n);
        for i in 0..n {
            for j in 0..n {
                av[i] += matrix[[i, j]] * v[j];
            }
        }
        let eigenvalue = v.dot(&av);

        Ok((v, eigenvalue))
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
        vectors.iter().map(|v| self.project(v)).collect()
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
