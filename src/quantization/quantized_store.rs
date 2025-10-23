use anyhow::Result;
use hnsw_rs::prelude::*;
use std::sync::Arc;

use super::{QuantizationModel, QuantizedVector};
use crate::vector::Vector;

/// Hamming distance metric for HNSW
///
/// Computes distance between quantized vectors stored as f32 bit representations.
/// Each pair of f32 values encodes one u64 word (via to_bits/from_bits).
#[derive(Clone, Copy)]
pub struct HammingDistance;

impl Distance<f32> for HammingDistance {
    fn eval(&self, va: &[f32], vb: &[f32]) -> f32 {
        // va and vb are f32 arrays encoding u64 bits
        // Each pair of f32 values represents one u64 word
        let num_words = va.len() / 2;
        let mut distance = 0u32;

        for i in 0..num_words {
            let idx = i * 2;

            // Reconstruct u64 from two f32 values
            let a_low = va[idx].to_bits() as u64;
            let a_high = va[idx + 1].to_bits() as u64;
            let a_word = a_low | (a_high << 32);

            let b_low = vb[idx].to_bits() as u64;
            let b_high = vb[idx + 1].to_bits() as u64;
            let b_word = b_low | (b_high << 32);

            // XOR + popcount for Hamming distance
            distance += (a_word ^ b_word).count_ones();
        }

        distance as f32
    }
}

/// Quantized vector store with HNSW index
///
/// Implements two-phase search:
/// 1. Approximate search with Hamming distance on quantized vectors
/// 2. Exact reranking with L2 distance on original vectors
pub struct QuantizedVectorStore<'a> {
    /// Quantization model (trained on sample data)
    quantization_model: Arc<QuantizationModel>,

    /// HNSW index using quantized vectors
    /// Stores quantized vectors as f32 arrays (bit-packed representation)
    hnsw_index: Hnsw<'a, f32, HammingDistance>,

    /// Original float32 vectors (for reranking)
    original_vectors: Vec<Vector>,

    /// Quantized vectors (stored for reference)
    quantized_vectors: Vec<QuantizedVector>,

    /// Metadata
    dimensions: usize,
    max_elements: usize,
    num_vectors: usize,

    /// HNSW parameters
    max_nb_connection: usize, // M parameter
    ef_construction: usize,
    ef_search: usize,
}

impl<'a> QuantizedVectorStore<'a> {
    /// Create new quantized vector store
    ///
    /// # Arguments
    /// * `quantization_model` - Trained quantization model
    /// * `max_elements` - Maximum number of vectors
    /// * `dimensions` - Vector dimensionality
    pub fn new(
        quantization_model: QuantizationModel,
        max_elements: usize,
        dimensions: usize,
    ) -> Self {
        let max_nb_connection = 48; // M=48 for high-dimensional vectors
        let ef_construction = 200;
        let nb_layer = 16.min((max_elements as f32).ln().trunc() as usize);

        let hnsw_index = Hnsw::<f32, HammingDistance>::new(
            max_nb_connection,
            max_elements,
            nb_layer,
            ef_construction,
            HammingDistance,
        );

        Self {
            quantization_model: Arc::new(quantization_model),
            hnsw_index,
            original_vectors: Vec::new(),
            quantized_vectors: Vec::new(),
            dimensions,
            max_elements,
            num_vectors: 0,
            max_nb_connection,
            ef_construction,
            ef_search: 100, // Default ef_search
        }
    }

    /// Insert vector into store
    ///
    /// Quantizes the vector and inserts into HNSW index.
    /// Stores both quantized and original vectors.
    pub fn insert(&mut self, vector: Vector) -> Result<usize> {
        anyhow::ensure!(
            vector.data.len() == self.dimensions,
            "Vector dimension mismatch: expected {}, got {}",
            self.dimensions,
            vector.data.len()
        );

        let id = self.num_vectors;

        // Quantize vector
        let quantized = self.quantization_model.quantize(&vector.data)?;

        // Convert quantized vector to f32 representation for HNSW
        // Pack bits into f32 array (store u64 bits as f32)
        let quantized_f32 = Self::quantized_to_f32(&quantized);

        // Insert into HNSW index
        self.hnsw_index.insert((&quantized_f32, id));

        // Store vectors
        self.original_vectors.push(vector);
        self.quantized_vectors.push(quantized);

        self.num_vectors += 1;
        Ok(id)
    }

    /// Two-phase k-NN search
    ///
    /// Phase 1: Approximate search with Hamming distance (retrieve `candidates` results)
    /// Phase 2: Rerank candidates with exact L2 distance (return top `k`)
    ///
    /// # Arguments
    /// * `query` - Query vector
    /// * `k` - Number of results to return
    /// * `candidates` - Number of candidates to retrieve in Phase 1 (default: k * 20)
    pub fn knn_search(
        &self,
        query: &Vector,
        k: usize,
        candidates: Option<usize>,
    ) -> Result<Vec<(usize, f32)>> {
        anyhow::ensure!(
            query.data.len() == self.dimensions,
            "Query dimension mismatch: expected {}, got {}",
            self.dimensions,
            query.data.len()
        );

        let num_candidates = candidates.unwrap_or(k * 20); // Default 20x expansion

        // Phase 1: Approximate search with Hamming distance
        let query_quantized = self.quantization_model.quantize(&query.data)?;
        let query_f32 = Self::quantized_to_f32(&query_quantized);

        let approximate_results = self.hnsw_index.search(&query_f32, num_candidates, self.ef_search);

        // Phase 2: Rerank with exact L2 distance
        let mut exact_results: Vec<(usize, f32)> = approximate_results
            .iter()
            .map(|neighbor| {
                let id = neighbor.d_id;
                let original = &self.original_vectors[id];
                let l2_dist = query.l2_distance(original).unwrap();
                (id, l2_dist)
            })
            .collect();

        // Sort by exact distance and return top k
        exact_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(exact_results.into_iter().take(k).collect())
    }

    /// Set ef_search parameter (runtime tuning)
    pub fn set_ef_search(&mut self, ef_search: usize) {
        self.ef_search = ef_search;
    }

    /// Get current ef_search
    pub fn get_ef_search(&self) -> usize {
        self.ef_search
    }

    /// Number of vectors in store
    pub fn len(&self) -> usize {
        self.num_vectors
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.num_vectors == 0
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Convert QuantizedVector to f32 representation for HNSW
    ///
    /// Packs u64 bits into f32 array by reinterpreting bits.
    /// This is a workaround since hnsw_rs expects &[f32].
    fn quantized_to_f32(quantized: &QuantizedVector) -> Vec<f32> {
        let bytes = quantized.to_bytes();

        // Skip first 4 bytes (dimensions header)
        let bit_bytes = &bytes[4..];

        // Convert u64 words to f32 (reinterpret bits)
        let num_words = bit_bytes.len() / 8;
        let mut f32_vec = Vec::with_capacity(num_words);

        for i in 0..num_words {
            let offset = i * 8;
            let word = u64::from_le_bytes([
                bit_bytes[offset],
                bit_bytes[offset + 1],
                bit_bytes[offset + 2],
                bit_bytes[offset + 3],
                bit_bytes[offset + 4],
                bit_bytes[offset + 5],
                bit_bytes[offset + 6],
                bit_bytes[offset + 7],
            ]);

            // Store as two f32 values (32 bits each from 64-bit word)
            let low = (word & 0xFFFFFFFF) as u32;
            let high = ((word >> 32) & 0xFFFFFFFF) as u32;

            f32_vec.push(f32::from_bits(low));
            f32_vec.push(f32::from_bits(high));
        }

        f32_vec
    }

    /// Memory usage breakdown
    pub fn memory_usage(&self) -> MemoryUsage {
        let quantized_size: usize = self
            .quantized_vectors
            .iter()
            .map(|q| q.memory_size())
            .sum();

        let original_size: usize = self
            .original_vectors
            .iter()
            .map(|v| v.data.len() * std::mem::size_of::<f32>())
            .sum();

        // Estimate HNSW graph overhead
        // Each node has M connections × 2 bytes per ID ≈ 100 bytes per node
        let graph_size = self.num_vectors * 100;

        MemoryUsage {
            quantized_vectors: quantized_size,
            original_vectors: original_size,
            hnsw_graph: graph_size,
            total: quantized_size + original_size + graph_size,
        }
    }
}

/// Memory usage breakdown
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub quantized_vectors: usize,
    pub original_vectors: usize,
    pub hnsw_graph: usize,
    pub total: usize,
}

impl MemoryUsage {
    pub fn total_gb(&self) -> f64 {
        self.total as f64 / 1_000_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_random_vector(dim: usize, seed: usize) -> Vector {
        let data: Vec<f32> = (0..dim)
            .map(|i| ((seed + i) as f32) * 0.01 - 0.5)
            .collect();
        Vector { data }
    }

    #[test]
    fn test_quantized_store_insert() {
        // Train quantization model
        let samples: Vec<Vec<f32>> = (0..100)
            .map(|i| generate_random_vector(128, i).data)
            .collect();
        let model = QuantizationModel::train(&samples, false).unwrap();

        let mut store = QuantizedVectorStore::new(model, 1000, 128);

        for i in 0..100 {
            let vector = generate_random_vector(128, i);
            let id = store.insert(vector).unwrap();
            assert_eq!(id, i);
        }

        assert_eq!(store.len(), 100);
    }

    #[test]
    fn test_quantized_store_search() {
        // Train quantization model
        let samples: Vec<Vec<f32>> = (0..100)
            .map(|i| generate_random_vector(128, i).data)
            .collect();
        let model = QuantizationModel::train(&samples, false).unwrap();

        let mut store = QuantizedVectorStore::new(model, 1000, 128);

        // Insert vectors
        let mut vectors = Vec::new();
        for i in 0..100 {
            let vector = generate_random_vector(128, i);
            vectors.push(vector.clone());
            store.insert(vector).unwrap();
        }

        // Search should return exact match in top results
        // Use high candidate expansion since quantization recall may be lower for small datasets
        let query_id = 50;
        let results = store.knn_search(&vectors[query_id], 10, Some(50)).unwrap();

        assert_eq!(results.len(), 10);

        // First result should have low distance (high recall expected)
        // Since we're testing on small dataset, quantization might not be perfect
        // but after L2 reranking, nearest should be close
        assert!(results[0].1 < 5.0, "Top result distance {} too high", results[0].1);

        // At least one of top-3 should be very close (distance < 1.0)
        let has_close_match = results.iter().take(3).any(|(_, d)| *d < 1.0);
        assert!(has_close_match, "No close matches in top-3 results");
    }

    #[test]
    fn test_two_phase_search_expansion() {
        // Test different candidate expansion factors
        let samples: Vec<Vec<f32>> = (0..200)
            .map(|i| generate_random_vector(128, i).data)
            .collect();
        let model = QuantizationModel::train(&samples, false).unwrap();

        let mut store = QuantizedVectorStore::new(model, 1000, 128);

        for i in 0..200 {
            let vector = generate_random_vector(128, i);
            store.insert(vector).unwrap();
        }

        let query = generate_random_vector(128, 100);

        // Try different expansion factors
        let result_10x = store.knn_search(&query, 10, Some(100)).unwrap();
        let result_20x = store.knn_search(&query, 10, Some(200)).unwrap();

        assert_eq!(result_10x.len(), 10);
        assert_eq!(result_20x.len(), 10);

        // Higher expansion should give equal or better results
        // (lower or equal distance to top result)
        assert!(result_20x[0].1 <= result_10x[0].1 + 0.1);
    }

    #[test]
    fn test_memory_usage() {
        let samples: Vec<Vec<f32>> = (0..100)
            .map(|i| generate_random_vector(1536, i).data)
            .collect();
        let model = QuantizationModel::train(&samples, false).unwrap();

        let mut store = QuantizedVectorStore::new(model, 10000, 1536);

        for i in 0..100 {
            let vector = generate_random_vector(1536, i);
            store.insert(vector).unwrap();
        }

        let usage = store.memory_usage();

        // Verify memory breakdown
        assert!(usage.quantized_vectors > 0);
        assert!(usage.original_vectors > 0);
        assert!(usage.hnsw_graph > 0);
        assert_eq!(usage.total, usage.quantized_vectors + usage.original_vectors + usage.hnsw_graph);

        // Check realistic sizes for 100 × 1536D vectors
        let expected_original = 100 * 1536 * 4; // 100 vectors × 1536 dims × 4 bytes
        assert!((usage.original_vectors as i64 - expected_original as i64).abs() < 1000);
    }
}
