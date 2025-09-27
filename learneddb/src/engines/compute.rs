//! Compute engine implementations for SIMD and vector operations

use super::{ComputeEngine, Result};

/// Rust SIMD compute engine using packed_simd2
#[derive(Debug)]
pub struct RustSIMDEngine {
    // Future: Add SIMD configuration
}

impl RustSIMDEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComputeEngine for RustSIMDEngine {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        // TODO: Implement with packed_simd2
        // For now, fallback to scalar
        ScalarEngine::euclidean_distance(a, b)
    }

    fn batch_distance(&self, query: &[f32], vectors: &[&[f32]]) -> Vec<f32> {
        // TODO: Implement SIMD batch processing
        vectors
            .iter()
            .map(|v| self.distance(query, v))
            .collect()
    }

    fn knn(&self, query: &[f32], vectors: &[&[f32]], k: usize) -> Vec<(usize, f32)> {
        let mut distances: Vec<_> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, self.distance(query, v)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.truncate(k);
        distances
    }

    fn stats(&self) -> String {
        format!("RustSIMD: Vectorized operations (pending packed_simd2 integration)")
    }
}

/// Scalar compute engine (fallback)
#[derive(Debug)]
pub struct ScalarEngine;

impl ScalarEngine {
    pub fn new() -> Self {
        Self
    }

    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let diff = x - y;
                diff * diff
            })
            .sum::<f32>()
            .sqrt()
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

impl ComputeEngine for ScalarEngine {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        Self::euclidean_distance(a, b)
    }

    fn batch_distance(&self, query: &[f32], vectors: &[&[f32]]) -> Vec<f32> {
        vectors
            .iter()
            .map(|v| self.distance(query, v))
            .collect()
    }

    fn knn(&self, query: &[f32], vectors: &[&[f32]], k: usize) -> Vec<(usize, f32)> {
        let mut distances: Vec<_> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, self.distance(query, v)))
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        distances.truncate(k);
        distances
    }

    fn stats(&self) -> String {
        format!("Scalar: Simple CPU operations without vectorization")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_distance() {
        let engine = ScalarEngine::new();
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let dist = engine.distance(&a, &b);
        assert!((dist - 5.196).abs() < 0.01); // sqrt(27) ≈ 5.196
    }

    #[test]
    fn test_scalar_knn() {
        let engine = ScalarEngine::new();
        let query = vec![0.0, 0.0];
        let vectors: Vec<&[f32]> = vec![
            &[1.0, 1.0],
            &[2.0, 2.0],
            &[3.0, 3.0],
            &[0.5, 0.5],
        ];

        let results = engine.knn(&query, &vectors, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 3); // Closest is [0.5, 0.5]
        assert_eq!(results[1].0, 0); // Second closest is [1.0, 1.0]
    }
}