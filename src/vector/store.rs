//! Vector storage with HNSW indexing
//!
//! VectorStore manages a collection of vectors and provides k-NN search
//! using HNSW (Hierarchical Navigable Small World) algorithm.

use super::types::Vector;
use anyhow::Result;

/// Vector store with HNSW indexing
pub struct VectorStore {
    /// All vectors stored in memory
    vectors: Vec<Vector>,

    /// Vector dimensionality
    dimensions: usize,
}

impl VectorStore {
    /// Create new vector store
    pub fn new(dimensions: usize) -> Self {
        Self {
            vectors: Vec::new(),
            dimensions,
        }
    }

    /// Insert vector and return its ID
    pub fn insert(&mut self, vector: Vector) -> Result<usize> {
        if vector.dim() != self.dimensions {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimensions,
                vector.dim()
            );
        }

        let id = self.vectors.len();
        self.vectors.push(vector);
        Ok(id)
    }

    /// K-nearest neighbors search (brute force for now, will use HNSW)
    pub fn knn_search(&self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
        if query.dim() != self.dimensions {
            anyhow::bail!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.dim()
            );
        }

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

    /// Get vector by ID
    pub fn get(&self, id: usize) -> Option<&Vector> {
        self.vectors.get(id)
    }

    /// Number of vectors stored
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// Memory usage estimate (bytes)
    pub fn memory_usage(&self) -> usize {
        // Vector data: num_vectors * dim * 4 bytes (f32)
        self.vectors.iter().map(|v| v.dim() * 4).sum::<usize>()
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

    fn random_vector(dim: usize, seed: usize) -> Vector {
        let data: Vec<f32> = (0..dim).map(|i| ((seed + i) as f32) * 0.1).collect();
        Vector::new(data)
    }

    #[test]
    fn test_vector_store_insert() {
        let mut store = VectorStore::new(128);

        let v1 = random_vector(128, 0);
        let v2 = random_vector(128, 1);

        let id1 = store.insert(v1).unwrap();
        let id2 = store.insert(v2).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_vector_store_knn() {
        let mut store = VectorStore::new(128);

        // Insert some vectors
        for i in 0..100 {
            store.insert(random_vector(128, i)).unwrap();
        }

        // Query for nearest neighbors
        let query = random_vector(128, 50);
        let results = store.knn_search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);

        // Results should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i - 1].1);
        }
    }

    #[test]
    fn test_dimension_mismatch() {
        let mut store = VectorStore::new(128);
        let wrong_dim = Vector::new(vec![1.0; 64]);

        assert!(store.insert(wrong_dim).is_err());
    }
}
