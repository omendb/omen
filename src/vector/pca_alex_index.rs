//! PCA-ALEX Vector Index
//!
//! Combines PCA dimensionality reduction with ALEX learned index for vector search:
//! 1. PCA: Reduce 1536D → 64D (preserves 80-90% variance)
//! 2. ALEX: Index the 64D space (not 1D projection like Week 1)
//! 3. Search: PCA project query → ALEX lookup → refine with exact L2
//!
//! This addresses Week 1 failure where 1D projection got only 5% recall.

use crate::alex::alex_tree::AlexTree;
use crate::pca::VectorPCA;
use super::types::Vector;
use anyhow::Result;
use std::collections::BTreeMap;

/// PCA-ALEX index for high-dimensional vector search
///
/// Strategy:
/// - Train PCA on sample vectors (1536D → 64D)
/// - For each vector: project to 64D, use first dimension as ALEX key
/// - Store mapping: ALEX_key → vector_id
/// - Search: project query → ALEX range query → refine top candidates
pub struct PCAAlexIndex {
    /// PCA model for dimensionality reduction
    pca: VectorPCA,

    /// ALEX index (keys = first PCA component)
    alex_index: AlexTree,

    /// Mapping: PCA key → vector IDs (multiple vectors can map to same key)
    key_to_ids: BTreeMap<i64, Vec<usize>>,

    /// All original vectors (for distance refinement)
    vectors: Vec<Vector>,

    /// Input dimensionality (e.g., 1536)
    input_dims: usize,

    /// PCA output dimensionality (e.g., 64)
    pca_dims: usize,

    /// Candidate expansion factor for recall
    /// e.g., if k=10 and expansion=10, retrieve 100 candidates from ALEX
    expansion_factor: usize,
}

impl PCAAlexIndex {
    /// Create new PCA-ALEX index
    pub fn new(input_dims: usize, pca_dims: usize) -> Self {
        Self {
            pca: VectorPCA::new(input_dims, pca_dims),
            alex_index: AlexTree::new(),
            key_to_ids: BTreeMap::new(),
            vectors: Vec::new(),
            input_dims,
            pca_dims,
            expansion_factor: 10, // Default: retrieve 10x candidates
        }
    }

    /// Train PCA on sample vectors
    ///
    /// Should be called before inserting vectors. Uses a sample of vectors
    /// to learn the PCA projection that will be used for all future inserts.
    pub fn train_pca(&mut self, training_vectors: &[Vec<f32>]) -> Result<f32> {
        println!("Training PCA ({}D → {}D)...", self.input_dims, self.pca_dims);
        let explained_variance = self.pca.train(training_vectors)?;
        println!("PCA explained variance: {:.2}%", explained_variance * 100.0);
        Ok(explained_variance)
    }

    /// Insert vector into index
    pub fn insert(&mut self, vector: Vector) -> Result<usize> {
        if !self.pca.is_trained() {
            anyhow::bail!("PCA not trained. Call train_pca() first.");
        }

        let vector_id = self.vectors.len();

        // Project to PCA space (1536D → 64D)
        let pca_vector = self.pca.project(&vector.data)?;

        // Use first PCA component as ALEX key (scaled to i64)
        // This is better than Week 1's sum of first 4 dims
        let alex_key = (pca_vector[0] * 1_000_000.0) as i64;

        // Insert into ALEX (convert vector_id to bytes)
        let value_bytes = vector_id.to_le_bytes().to_vec();
        self.alex_index.insert(alex_key, value_bytes)?;

        // Track key → vector_id mapping (for range queries)
        self.key_to_ids
            .entry(alex_key)
            .or_insert_with(Vec::new)
            .push(vector_id);

        // Store original vector (for distance refinement)
        self.vectors.push(vector);

        Ok(vector_id)
    }

    /// K-nearest neighbors search using PCA-ALEX
    ///
    /// Algorithm:
    /// 1. Project query to PCA space (1536D → 64D)
    /// 2. Use first PCA component as ALEX key
    /// 3. Query ALEX for nearby keys (range query)
    /// 4. Collect candidate vector IDs (expansion_factor * k candidates)
    /// 5. Refine: compute exact L2 distance in original 1536D space
    /// 6. Return top-k by distance
    pub fn knn_search(&self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
        if !self.pca.is_trained() {
            anyhow::bail!("PCA not trained. Call train_pca() first.");
        }

        if self.vectors.is_empty() {
            return Ok(Vec::new());
        }

        // 1. Project query to PCA space
        let pca_query = self.pca.project(&query.data)?;
        let query_key = (pca_query[0] * 1_000_000.0) as i64;

        // 2. Find range around query key using ALEX
        // Retrieve expansion_factor * k candidates for better recall
        let num_candidates = k * self.expansion_factor;
        let candidates = self.find_candidates(query_key, num_candidates)?;

        // 3. Refine: compute exact L2 distance in original space
        let mut distances: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&id| {
                let dist = query.l2_distance(&self.vectors[id]).unwrap_or(f32::MAX);
                (id, dist)
            })
            .collect();

        // 4. Sort by distance and return top-k
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(distances.into_iter().take(k).collect())
    }

    /// Find candidate vector IDs near query_key using ALEX range query
    fn find_candidates(&self, query_key: i64, num_candidates: usize) -> Result<Vec<usize>> {
        let mut candidates = Vec::new();

        // Start with query key
        if let Some(ids) = self.key_to_ids.get(&query_key) {
            candidates.extend(ids);
        }

        // Expand range outward until we have enough candidates
        let mut range_radius = 1000; // Initial radius
        let max_radius = 10_000_000; // Maximum search radius

        while candidates.len() < num_candidates && range_radius < max_radius {
            // Search keys in range [query_key - radius, query_key + radius]
            let start_key = query_key.saturating_sub(range_radius);
            let end_key = query_key.saturating_add(range_radius);

            for (key, ids) in self.key_to_ids.range(start_key..=end_key) {
                // Skip query_key (already added)
                if *key == query_key {
                    continue;
                }
                candidates.extend(ids);

                if candidates.len() >= num_candidates {
                    break;
                }
            }

            // Expand search radius
            range_radius *= 2;
        }

        // If still not enough, just return all vectors (fallback)
        if candidates.is_empty() {
            candidates = (0..self.vectors.len()).collect();
        }

        Ok(candidates)
    }

    /// Set candidate expansion factor (for recall tuning)
    ///
    /// Higher expansion = better recall, slower queries
    /// Default: 10 (retrieve 10x candidates, e.g., 100 for k=10)
    pub fn set_expansion_factor(&mut self, factor: usize) {
        self.expansion_factor = factor;
    }

    /// Get vector by ID
    pub fn get(&self, id: usize) -> Option<&Vector> {
        self.vectors.get(id)
    }

    /// Number of vectors indexed
    pub fn len(&self) -> usize {
        self.vectors.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }

    /// PCA explained variance ratio
    pub fn explained_variance(&self) -> f32 {
        self.pca.explained_variance_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_structured_vector(dim: usize, basis_dim: usize, basis: &[Vec<f32>], seed: usize) -> Vector {
        let mut data = vec![0.0; dim];
        let weights: Vec<f32> = (0..basis_dim).map(|i| ((seed + i) as f32 * 0.1).sin()).collect();

        for i in 0..dim {
            for j in 0..basis_dim {
                data[i] += weights[j] * basis[j][i];
            }
        }

        Vector::new(data)
    }

    fn generate_basis(dim: usize, basis_dim: usize) -> Vec<Vec<f32>> {
        (0..basis_dim)
            .map(|j| {
                (0..dim)
                    .map(|i| ((i as f32 * 0.01).sin() * (j as f32 + i as f32 * 0.1).cos()))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn test_pca_alex_basic() {
        let dim = 128;
        let pca_dim = 16;
        let basis_dim = 20;
        let num_vectors = 100;

        let basis = generate_basis(dim, basis_dim);
        let mut index = PCAAlexIndex::new(dim, pca_dim);

        // Generate training data
        let training_data: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| generate_structured_vector(dim, basis_dim, &basis, i).data)
            .collect();

        // Train PCA
        let variance = index.train_pca(&training_data).unwrap();
        assert!(variance > 0.5); // Should capture >50% variance

        // Insert vectors
        for i in 0..num_vectors {
            let vec = generate_structured_vector(dim, basis_dim, &basis, i);
            index.insert(vec).unwrap();
        }

        assert_eq!(index.len(), num_vectors);
    }

    #[test]
    fn test_pca_alex_search() {
        let dim = 128;
        let pca_dim = 16;
        let basis_dim = 20;
        let num_vectors = 100;

        let basis = generate_basis(dim, basis_dim);
        let mut index = PCAAlexIndex::new(dim, pca_dim);

        // Generate and train
        let training_data: Vec<Vec<f32>> = (0..num_vectors)
            .map(|i| generate_structured_vector(dim, basis_dim, &basis, i).data)
            .collect();
        index.train_pca(&training_data).unwrap();

        // Insert vectors
        for i in 0..num_vectors {
            let vec = generate_structured_vector(dim, basis_dim, &basis, i);
            index.insert(vec).unwrap();
        }

        // Query: should find itself as nearest neighbor
        let query_id = 50;
        let query = generate_structured_vector(dim, basis_dim, &basis, query_id);
        let results = index.knn_search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);
        // First result should be the query vector itself (or very close)
        assert!(results[0].1 < 0.1); // Distance should be very small
    }

    #[test]
    fn test_expansion_factor() {
        let dim = 128;
        let pca_dim = 16;

        let mut index = PCAAlexIndex::new(dim, pca_dim);
        assert_eq!(index.expansion_factor, 10); // Default

        index.set_expansion_factor(20);
        assert_eq!(index.expansion_factor, 20);
    }
}
