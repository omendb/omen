//! Adapter for custom HNSW implementation
//!
//! Provides an API-compatible wrapper around the custom HNSW implementation,
//! matching the interface of the hnsw_rs-based HNSWIndex.

use super::custom_hnsw::{DistanceFunction, HNSWIndex as CustomHNSW, HNSWParams as CustomParams};
use anyhow::Result;
use std::path::Path;

/// Adapter for custom HNSW implementation
///
/// Provides the same API as the hnsw_rs-based HNSWIndex for easy migration
pub struct CustomHNSWAdapter {
    /// Custom HNSW index
    index: CustomHNSW,

    /// Index parameters
    max_elements: usize,
    max_nb_connection: usize, // M parameter
    ef_construction: usize,
    dimensions: usize,

    /// Runtime search parameter (tunable)
    ef_search: usize,

    /// Number of vectors inserted
    num_vectors: usize,
}

/// HNSW parameters (matches hnsw_index.rs API)
#[derive(Debug, Clone)]
pub struct HNSWParams {
    pub max_elements: usize,
    pub max_nb_connection: usize,
    pub ef_construction: usize,
    pub ef_search: usize,
    pub dimensions: usize,
}

impl CustomHNSWAdapter {
    /// Create new HNSW index
    ///
    /// Parameters recommended for 1536D OpenAI embeddings:
    /// - max_elements: Maximum number of vectors (e.g., 1_000_000)
    /// - dimensions: Vector dimensionality (e.g., 1536)
    pub fn new(max_elements: usize, dimensions: usize) -> Self {
        // Parameters matching pgvector defaults for fair comparison
        let max_nb_connection = 16; // M=16 (pgvector default)
        let ef_construction = 64; // ef_construction=64 (pgvector default)

        let params = CustomParams {
            m: max_nb_connection,
            ef_construction,
            ml: 1.0 / (max_nb_connection as f32).ln(),
            seed: 42,
            max_level: 8,
        };

        let index = CustomHNSW::new(dimensions, params, DistanceFunction::L2, false)
            .expect("Failed to create custom HNSW index");

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

        let id = self.index.insert(vector.to_vec()).map_err(|e| anyhow::anyhow!(e))?;
        self.num_vectors += 1;
        Ok(id as usize)
    }

    /// Insert batch of vectors in parallel
    ///
    /// Note: Current custom HNSW implementation inserts sequentially.
    /// Parallel insertion will be added in future optimization phase.
    ///
    /// Returns Vec of IDs for inserted vectors
    pub fn batch_insert(&mut self, vectors: &[Vec<f32>]) -> Result<Vec<usize>> {
        // Validate all vectors have correct dimensions
        for (i, vector) in vectors.iter().enumerate() {
            if vector.len() != self.dimensions {
                anyhow::bail!(
                    "Vector {} dimension mismatch: expected {}, got {}",
                    i,
                    self.dimensions,
                    vector.len()
                );
            }
        }

        let mut ids = Vec::with_capacity(vectors.len());

        // Insert sequentially (TODO: add parallel batch_insert to custom HNSW)
        for vector in vectors {
            let id = self.index.insert(vector.clone()).map_err(|e| anyhow::anyhow!(e))?;
            ids.push(id as usize);
            self.num_vectors += 1;
        }

        Ok(ids)
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

        // Search with custom HNSW
        let results = self.index.search(query, k, self.ef_search).map_err(|e| anyhow::anyhow!(e))?;

        // Convert from custom HNSW format to (id, distance) tuples
        let neighbors: Vec<(usize, f32)> = results
            .iter()
            .map(|r| (r.id as usize, r.distance))
            .collect();

        Ok(neighbors)
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

    /// Save index to disk
    ///
    /// Uses custom HNSW's fast binary serialization format.
    /// Saves both graph structure and vector data.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.index.save(path).map_err(|e| anyhow::anyhow!(e))
    }

    /// Load index from disk
    ///
    /// Loads index saved with save() method.
    /// Fast loading: <1 second for 100K vectors (vs minutes for rebuild)
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = CustomHNSW::load(path).map_err(|e| anyhow::anyhow!(e))?;

        // Extract parameters from loaded index
        let dimensions = index.dimensions();
        let num_vectors = index.len();

        // Note: We don't have direct access to all params from loaded index,
        // so we use defaults. This is fine because the index behavior is
        // determined by the saved graph structure, not these params.
        Ok(Self {
            index,
            max_elements: num_vectors.max(1_000_000),
            max_nb_connection: 16, // Default
            ef_construction: 64,   // Default
            ef_search: 100,        // Default
            dimensions,
            num_vectors,
        })
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.index.memory_usage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_basic() {
        let mut adapter = CustomHNSWAdapter::new(1000, 4);

        // Insert vectors
        let v1 = vec![1.0, 0.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0, 0.0];

        let id1 = adapter.insert(&v1).unwrap();
        let id2 = adapter.insert(&v2).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(adapter.len(), 2);

        // Search
        let query = vec![0.9, 0.1, 0.0, 0.0];
        let results = adapter.search(&query, 1).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0); // Closest to v1
    }

    #[test]
    fn test_adapter_batch_insert() {
        let mut adapter = CustomHNSWAdapter::new(1000, 3);

        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let ids = adapter.batch_insert(&vectors).unwrap();

        assert_eq!(ids.len(), 3);
        assert_eq!(adapter.len(), 3);
    }

    #[test]
    fn test_adapter_ef_search() {
        let mut adapter = CustomHNSWAdapter::new(1000, 4);

        assert_eq!(adapter.get_ef_search(), 100); // Default

        adapter.set_ef_search(200);
        assert_eq!(adapter.get_ef_search(), 200);
    }
}
