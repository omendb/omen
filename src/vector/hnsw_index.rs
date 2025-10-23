//! HNSW (Hierarchical Navigable Small World) index wrapper
//!
//! Wraps the hnsw_rs crate to provide a clean API for vector indexing.
//!
//! Parameters (for 1536D vectors):
//! - M (max_nb_connection): 48-64 (high-dimensional embeddings)
//! - ef_construction: 200-400 (index build quality)
//! - ef_search: 100-500 (runtime search quality, tunable)

use anyhow::Result;
use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::prelude::*;

/// HNSW index for high-dimensional vectors
pub struct HNSWIndex<'a> {
    /// HNSW index from hnsw_rs crate
    index: Hnsw<'a, f32, DistL2>,

    /// Index parameters
    max_elements: usize,
    max_nb_connection: usize, // M parameter
    ef_construction: usize,

    /// Runtime search parameter (tunable)
    ef_search: usize,

    /// Vector dimensionality
    dimensions: usize,

    /// Number of vectors inserted
    num_vectors: usize,
}

impl<'a> HNSWIndex<'a> {
    /// Create new HNSW index for high-dimensional vectors
    ///
    /// Parameters recommended for 1536D OpenAI embeddings:
    /// - max_elements: Maximum number of vectors (e.g., 1_000_000)
    /// - dimensions: Vector dimensionality (e.g., 1536)
    pub fn new(max_elements: usize, dimensions: usize) -> Self {
        // Parameters for high-dimensional vectors (based on research)
        let max_nb_connection = 48; // M=48 for high-dim embeddings
        let ef_construction = 200; // Balanced quality/speed
        let nb_layer = 16.min((max_elements as f32).ln().trunc() as usize);

        let index = Hnsw::<f32, DistL2>::new(
            max_nb_connection,
            max_elements,
            nb_layer,
            ef_construction,
            DistL2 {},
        );

        Self {
            index,
            max_elements,
            max_nb_connection,
            ef_construction,
            ef_search: 100, // Default ef_search
            dimensions,
            num_vectors: 0,
        }
    }

    /// Insert vector into index and return its ID
    pub fn insert(&mut self, vector: &[f32]) -> Result<usize> {
        if vector.len() != self.dimensions {
            anyhow::bail!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimensions,
                vector.len()
            );
        }

        let id = self.num_vectors;

        // Insert into HNSW
        // hnsw_rs expects (data, id) tuple
        self.index.insert((vector, id));

        self.num_vectors += 1;
        Ok(id)
    }

    /// Search for K nearest neighbors
    ///
    /// Returns Vec<(id, distance)> sorted by distance (ascending)
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        if query.len() != self.dimensions {
            anyhow::bail!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.len()
            );
        }

        // HNSW search
        let neighbors = self.index.search(query, k, self.ef_search);

        // Convert from hnsw_rs format to our format
        // hnsw_rs returns Vec<Neighbour> where Neighbour has .d_id (data_id) and .distance
        let results: Vec<(usize, f32)> = neighbors
            .iter()
            .map(|n| (n.d_id, n.distance))
            .collect();

        Ok(results)
    }

    /// Set ef_search parameter (runtime tuning)
    ///
    /// Higher ef_search = better recall, slower queries
    /// - ef=50: ~85-90% recall, ~1ms
    /// - ef=100: ~90-95% recall, ~2ms
    /// - ef=200: ~95-98% recall, ~5ms
    /// - ef=500: ~98-99% recall, ~10ms
    pub fn set_ef_search(&mut self, ef_search: usize) {
        self.ef_search = ef_search;
    }

    /// Get current ef_search value
    pub fn get_ef_search(&self) -> usize {
        self.ef_search
    }

    /// Number of vectors in index
    pub fn len(&self) -> usize {
        self.num_vectors
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.num_vectors == 0
    }

    /// Get index parameters
    pub fn params(&self) -> HNSWParams {
        HNSWParams {
            max_elements: self.max_elements,
            max_nb_connection: self.max_nb_connection,
            ef_construction: self.ef_construction,
            ef_search: self.ef_search,
            dimensions: self.dimensions,
        }
    }
}

/// HNSW index parameters
#[derive(Debug, Clone)]
pub struct HNSWParams {
    pub max_elements: usize,
    pub max_nb_connection: usize, // M
    pub ef_construction: usize,
    pub ef_search: usize,
    pub dimensions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_random_vector(dim: usize, seed: usize) -> Vec<f32> {
        (0..dim).map(|i| ((seed + i) as f32) * 0.1).collect()
    }

    #[test]
    fn test_hnsw_insert() {
        let mut index = HNSWIndex::new(1000, 128);

        for i in 0..100 {
            let vector = generate_random_vector(128, i);
            let id = index.insert(&vector).unwrap();
            assert_eq!(id, i);
        }

        assert_eq!(index.len(), 100);
    }

    #[test]
    fn test_hnsw_search() {
        let mut index = HNSWIndex::new(1000, 128);

        // Insert some vectors
        for i in 0..100 {
            let vector = generate_random_vector(128, i);
            index.insert(&vector).unwrap();
        }

        // Search for nearest neighbors
        let query = generate_random_vector(128, 50);
        let results = index.search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);

        // Results should be sorted by distance (ascending)
        for i in 1..results.len() {
            assert!(results[i].1 >= results[i - 1].1);
        }
    }

    #[test]
    fn test_hnsw_recall() {
        let mut index = HNSWIndex::new(1000, 128);

        // Insert vectors
        let mut vectors = Vec::new();
        for i in 0..1000 {
            let vector = generate_random_vector(128, i);
            vectors.push(vector.clone());
            index.insert(&vector).unwrap();
        }

        // Query should return itself as nearest neighbor
        let query_id = 500;
        let results = index.search(&vectors[query_id], 1).unwrap();

        assert_eq!(results[0].0, query_id);
        assert!(results[0].1 < 0.01); // Distance to itself should be ~0
    }

    #[test]
    fn test_hnsw_dimension_mismatch() {
        let mut index = HNSWIndex::new(1000, 128);

        let wrong_dim = vec![1.0; 64];
        assert!(index.insert(&wrong_dim).is_err());

        let query_wrong = vec![1.0; 64];
        assert!(index.search(&query_wrong, 10).is_err());
    }

    #[test]
    fn test_hnsw_ef_search() {
        let mut index = HNSWIndex::new(1000, 128);

        assert_eq!(index.get_ef_search(), 100); // Default

        index.set_ef_search(200);
        assert_eq!(index.get_ef_search(), 200);
    }

    #[test]
    fn test_hnsw_params() {
        let index = HNSWIndex::new(10000, 1536);
        let params = index.params();

        assert_eq!(params.max_elements, 10000);
        assert_eq!(params.max_nb_connection, 48); // M=48 for high-dim
        assert_eq!(params.ef_construction, 200);
        assert_eq!(params.ef_search, 100);
        assert_eq!(params.dimensions, 1536);
    }
}
