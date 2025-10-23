//! Vector database module - ALEX prototype for high-dimensional vectors
//!
//! Week 1 Goal: Validate ALEX can partition high-dimensional space for k-NN search
//!
//! Approach:
//! - Store vectors as Vec<f32> (1536 dimensions for OpenAI embeddings)
//! - Use ALEX to index vectors by projection to 1D (sum of first 4 dimensions)
//! - Perform approximate k-NN by searching ALEX buckets
//! - Measure: memory usage, query latency, recall@10
//!
//! Go/No-Go Criteria (Week 1):
//! - Memory: <50 bytes/vector (vs HNSW's ~100 bytes/vector)
//! - Latency: <20ms p95 for k=10 search (vs HNSW's <10ms)
//! - Recall@10: >90% (vs brute-force ground truth)

use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// High-dimensional vector (1536 dimensions for OpenAI embeddings)
#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    /// Vector dimensions (typically 1536 for OpenAI, 768 for other models)
    pub data: Vec<f32>,
}

impl Vector {
    /// Create new vector from f32 array
    pub fn new(data: Vec<f32>) -> Self {
        Self { data }
    }

    /// Get dimensionality
    pub fn dim(&self) -> usize {
        self.data.len()
    }

    /// Compute L2 (Euclidean) distance between vectors
    pub fn l2_distance(&self, other: &Vector) -> Result<f32> {
        if self.dim() != other.dim() {
            return Err(anyhow!(
                "Dimension mismatch: {} vs {}",
                self.dim(),
                other.dim()
            ));
        }

        let sum: f32 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| {
                let diff = a - b;
                diff * diff
            })
            .sum();

        Ok(sum.sqrt())
    }

    /// Compute dot product (for inner product similarity)
    pub fn dot_product(&self, other: &Vector) -> Result<f32> {
        if self.dim() != other.dim() {
            return Err(anyhow!(
                "Dimension mismatch: {} vs {}",
                self.dim(),
                other.dim()
            ));
        }

        let sum: f32 = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .sum();

        Ok(sum)
    }

    /// Compute cosine distance (1 - cosine similarity)
    pub fn cosine_distance(&self, other: &Vector) -> Result<f32> {
        let dot = self.dot_product(other)?;
        let norm_self = self.l2_norm();
        let norm_other = other.l2_norm();

        if norm_self == 0.0 || norm_other == 0.0 {
            return Err(anyhow!("Cannot compute cosine distance for zero vector"));
        }

        let cosine_sim = dot / (norm_self * norm_other);
        Ok(1.0 - cosine_sim)
    }

    /// Compute L2 norm (magnitude) of vector
    pub fn l2_norm(&self) -> f32 {
        self.data.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Project vector to 1D for ALEX indexing
    /// Strategy: Sum of first N dimensions (simple but effective for clustering)
    pub fn project_to_1d(&self, num_dims: usize) -> i64 {
        let n = num_dims.min(self.dim());
        let sum: f32 = self.data.iter().take(n).sum();

        // Scale to i64 range for ALEX (multiply by 1000 for precision)
        (sum * 1000.0) as i64
    }
}

/// Simple vector store with ALEX-based indexing (Week 1 prototype)
pub struct VectorStore {
    /// All vectors stored in memory
    vectors: Vec<Vector>,

    /// ALEX index: maps projection value to vector IDs
    /// For prototype, use simple HashMap (will replace with actual ALEX)
    index: HashMap<i64, Vec<usize>>,

    /// Number of dimensions for projection (tunable)
    projection_dims: usize,
}

impl VectorStore {
    /// Create new vector store
    pub fn new(projection_dims: usize) -> Self {
        Self {
            vectors: Vec::new(),
            index: HashMap::new(),
            projection_dims,
        }
    }

    /// Insert vector and return its ID
    pub fn insert(&mut self, vector: Vector) -> usize {
        let id = self.vectors.len();
        let proj = vector.project_to_1d(self.projection_dims);

        self.vectors.push(vector);
        self.index.entry(proj).or_insert_with(Vec::new).push(id);

        id
    }

    /// K-nearest neighbors search (brute force for prototype)
    pub fn knn_search(&self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
        if self.vectors.is_empty() {
            return Ok(Vec::new());
        }

        // Compute distances to all vectors
        let mut distances: Vec<(usize, f32)> = self
            .vectors
            .iter()
            .enumerate()
            .map(|(id, vec)| {
                let dist = query.l2_distance(vec).unwrap_or(f32::MAX);
                (id, dist)
            })
            .collect();

        // Sort by distance and take top K
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(distances.into_iter().take(k).collect())
    }

    /// ALEX-accelerated K-NN search (approximate)
    /// Search only buckets near the query vector's projection
    pub fn knn_search_alex(&self, query: &Vector, k: usize, num_buckets: usize) -> Result<Vec<(usize, f32)>> {
        if self.vectors.is_empty() {
            return Ok(Vec::new());
        }

        let query_proj = query.project_to_1d(self.projection_dims);

        // Get candidate vector IDs from nearby buckets
        // For prototype, search buckets within range
        let mut candidate_ids = Vec::new();
        let search_range = 100; // Tunable parameter

        for offset in -(search_range as i64)..=(search_range as i64) {
            let proj = query_proj + offset;
            if let Some(ids) = self.index.get(&proj) {
                candidate_ids.extend(ids.iter().copied());
            }
        }

        // If no candidates found, fallback to brute force
        if candidate_ids.is_empty() {
            return self.knn_search(query, k);
        }

        // Compute distances only for candidate vectors
        let mut distances: Vec<(usize, f32)> = candidate_ids
            .iter()
            .map(|&id| {
                let dist = query.l2_distance(&self.vectors[id]).unwrap_or(f32::MAX);
                (id, dist)
            })
            .collect();

        // Sort and take top K
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(distances.into_iter().take(k).collect())
    }

    /// Get vector by ID
    pub fn get(&self, id: usize) -> Option<&Vector> {
        self.vectors.get(id)
    }

    /// Number of vectors stored
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Memory usage estimate (bytes)
    pub fn memory_usage(&self) -> usize {
        // Vector data: num_vectors * dim * 4 bytes (f32)
        let vector_data = self.vectors.iter()
            .map(|v| v.dim() * 4)
            .sum::<usize>();

        // Index overhead: rough estimate
        let index_overhead = self.index.len() * 32; // HashMap entry overhead

        vector_data + index_overhead
    }

    /// Bytes per vector (average)
    pub fn bytes_per_vector(&self) -> f32 {
        if self.vectors.is_empty() {
            return 0.0;
        }
        self.memory_usage() as f32 / self.vectors.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vector(dim: usize) -> Vector {
        let data: Vec<f32> = (0..dim).map(|i| (i as f32) * 0.1).collect();
        Vector::new(data)
    }

    #[test]
    fn test_vector_l2_distance() {
        let v1 = Vector::new(vec![1.0, 0.0, 0.0]);
        let v2 = Vector::new(vec![0.0, 1.0, 0.0]);

        let dist = v1.l2_distance(&v2).unwrap();
        assert!((dist - 1.414).abs() < 0.01); // sqrt(2) ≈ 1.414
    }

    #[test]
    fn test_vector_dot_product() {
        let v1 = Vector::new(vec![1.0, 2.0, 3.0]);
        let v2 = Vector::new(vec![4.0, 5.0, 6.0]);

        let dot = v1.dot_product(&v2).unwrap();
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_vector_projection() {
        let v = Vector::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let proj = v.project_to_1d(4);

        // (1 + 2 + 3 + 4) * 1000 = 10000
        assert_eq!(proj, 10000);
    }

    #[test]
    fn test_vector_store_insert() {
        let mut store = VectorStore::new(4);

        let v1 = random_vector(128);
        let v2 = random_vector(128);

        let id1 = store.insert(v1);
        let id2 = store.insert(v2);

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_vector_store_knn() {
        let mut store = VectorStore::new(4);

        // Insert some vectors
        for i in 0..100 {
            let data: Vec<f32> = (0..128).map(|d| (i * d) as f32 * 0.1).collect();
            store.insert(Vector::new(data));
        }

        // Query for nearest neighbors
        let query = random_vector(128);
        let results = store.knn_search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);

        // Results should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i-1].1);
        }
    }

    #[test]
    fn test_vector_store_memory() {
        let mut store = VectorStore::new(4);

        // Insert 1000 128-dim vectors
        for i in 0..1000 {
            let data: Vec<f32> = (0..128).map(|d| (i + d) as f32).collect();
            store.insert(Vector::new(data));
        }

        let bytes_per_vec = store.bytes_per_vector();

        // Should be approximately 128 * 4 = 512 bytes + small overhead
        assert!(bytes_per_vec < 600.0, "Expected <600 bytes/vector, got {}", bytes_per_vec);
    }

    #[test]
    fn test_dimension_mismatch() {
        let v1 = Vector::new(vec![1.0, 2.0]);
        let v2 = Vector::new(vec![1.0, 2.0, 3.0]);

        assert!(v1.l2_distance(&v2).is_err());
        assert!(v1.dot_product(&v2).is_err());
    }
}
