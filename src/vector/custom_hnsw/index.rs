// HNSW Index - Main implementation
//
// Architecture:
// - Flattened index (contiguous nodes, u32 node IDs)
// - Separate neighbor storage (fetch only when needed)
// - Cache-optimized layout (64-byte aligned hot data)

use super::storage::{NeighborLists, VectorStorage};
use super::types::{Candidate, DistanceFunction, HNSWNode, HNSWParams, SearchResult};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

/// HNSW Index
///
/// Hierarchical graph index for approximate nearest neighbor search.
/// Optimized for cache locality and memory efficiency.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HNSWIndex {
    /// Node metadata (cache-line aligned)
    nodes: Vec<HNSWNode>,

    /// Neighbor lists (stored separately for cache efficiency)
    neighbors: NeighborLists,

    /// Vector storage (full precision or quantized)
    vectors: VectorStorage,

    /// Entry point (top-level node)
    entry_point: Option<u32>,

    /// Construction parameters
    params: HNSWParams,

    /// Distance function
    distance_fn: DistanceFunction,

    /// Random number generator seed state
    rng_state: u64,
}

impl HNSWIndex {
    /// Create a new empty HNSW index
    pub fn new(
        dimensions: usize,
        params: HNSWParams,
        distance_fn: DistanceFunction,
        use_quantization: bool,
    ) -> Result<Self, String> {
        params.validate()?;

        let vectors = if use_quantization {
            VectorStorage::new_binary_quantized(dimensions, true)
        } else {
            VectorStorage::new_full_precision(dimensions)
        };

        Ok(Self {
            nodes: Vec::new(),
            neighbors: NeighborLists::new(params.max_level as usize),
            vectors,
            entry_point: None,
            params,
            distance_fn,
            rng_state: params.seed,
        })
    }

    /// Get number of vectors in index
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.vectors.dimensions()
    }

    /// Assign random level to new node
    ///
    /// Uses exponential decay: P(level = l) = (1/M)^l
    /// This ensures most nodes are at level 0, fewer at higher levels.
    fn random_level(&mut self) -> u8 {
        // Simple LCG for deterministic random numbers
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let rand_val = (self.rng_state >> 32) as f32 / u32::MAX as f32;

        // Exponential distribution: -ln(uniform) / ln(M)
        let level = (-rand_val.ln() * self.params.ml) as u8;
        level.min(self.params.max_level - 1)
    }

    /// Compute distance between two vectors
    fn distance(&self, id_a: u32, id_b: u32) -> f32 {
        let vec_a = self.vectors.get(id_a).expect("Vector A not found");
        let vec_b = self.vectors.get(id_b).expect("Vector B not found");
        self.distance_fn.distance(vec_a, vec_b)
    }

    /// Compute distance between query and vector
    fn distance_to_query(&self, query: &[f32], id: u32) -> f32 {
        let vec = self.vectors.get(id).expect("Vector not found");
        self.distance_fn.distance(query, vec)
    }

    /// Insert a vector into the index
    ///
    /// Returns the node ID assigned to this vector.
    pub fn insert(&mut self, vector: Vec<f32>) -> Result<u32, String> {
        // Validate dimensions
        if vector.len() != self.dimensions() {
            return Err(format!(
                "Vector dimension mismatch: expected {}, got {}",
                self.dimensions(),
                vector.len()
            ));
        }

        // Store vector and get ID
        let node_id = self.vectors.insert(vector.clone())?;

        // Assign random level
        let level = self.random_level();

        // Create node
        let node = HNSWNode::new(node_id, level);
        self.nodes.push(node);

        // If this is the first node, set as entry point
        if self.entry_point.is_none() {
            self.entry_point = Some(node_id);
            return Ok(node_id);
        }

        // Insert into graph (simplified for now - full algorithm in next iteration)
        self.insert_into_graph(node_id, &vector, level)?;

        Ok(node_id)
    }

    /// Insert node into graph structure
    ///
    /// TODO: Full HNSW insertion algorithm (greedy search + neighbor selection)
    /// For now: simplified version that connects to entry point
    fn insert_into_graph(
        &mut self,
        node_id: u32,
        vector: &[f32],
        level: u8,
    ) -> Result<(), String> {
        let entry_point = self.entry_point.expect("Entry point must exist");

        // Simplified: just connect to entry point at level 0
        // Full algorithm will search for nearest neighbors at each level
        if level >= 0 {
            self.neighbors.add_bidirectional_link(node_id, entry_point, 0);

            // Update neighbor counts
            self.nodes[node_id as usize].set_neighbor_count(0, 1);
            let entry_neighbors = self.neighbors.get_neighbors(entry_point, 0).len();
            self.nodes[entry_point as usize].set_neighbor_count(0, entry_neighbors);
        }

        Ok(())
    }

    /// Search for k nearest neighbors
    ///
    /// Returns up to k nearest neighbors sorted by distance (closest first).
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<SearchResult>, String> {
        // Validate dimensions
        if query.len() != self.dimensions() {
            return Err(format!(
                "Query dimension mismatch: expected {}, got {}",
                self.dimensions(),
                query.len()
            ));
        }

        // Handle empty index
        if self.is_empty() {
            return Ok(Vec::new());
        }

        let entry_point = self.entry_point.expect("Entry point must exist");

        // Simplified search: just return entry point
        // Full algorithm will traverse graph from top level to bottom
        let distance = self.distance_to_query(query, entry_point);
        let result = SearchResult::new(entry_point, distance);

        Ok(vec![result])
    }

    /// Search for nearest neighbors at a specific level
    ///
    /// Returns candidate set sorted by distance.
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[u32],
        ef: usize,
        level: u8,
    ) -> Vec<Candidate> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new(); // Min-heap (closest first)
        let mut working = BinaryHeap::new(); // Max-heap (farthest first for pruning)

        // Initialize with entry points
        for &ep in entry_points {
            let dist = self.distance_to_query(query, ep);
            let candidate = Candidate::new(ep, dist);

            candidates.push(Reverse(candidate));
            working.push(candidate);
            visited.insert(ep);
        }

        // Greedy search
        while let Some(current) = candidates.pop() {
            let current = current.0; // Unwrap Reverse

            // If current is farther than farthest in working set, stop
            if let Some(&farthest) = working.peek() {
                if current.distance > farthest.distance {
                    break;
                }
            }

            // Explore neighbors
            let neighbors = self.neighbors.get_neighbors(current.node_id, level);
            for &neighbor_id in neighbors {
                if visited.contains(&neighbor_id) {
                    continue;
                }
                visited.insert(neighbor_id);

                let dist = self.distance_to_query(query, neighbor_id);
                let neighbor = Candidate::new(neighbor_id, dist);

                // If neighbor is closer than farthest in working set, add it
                if let Some(&farthest) = working.peek() {
                    if dist < farthest.distance.0 || working.len() < ef {
                        candidates.push(Reverse(neighbor));
                        working.push(neighbor);

                        // Prune working set to ef size
                        if working.len() > ef {
                            working.pop();
                        }
                    }
                } else {
                    candidates.push(Reverse(neighbor));
                    working.push(neighbor);
                }
            }
        }

        // Return working set sorted by distance (closest first)
        let mut results: Vec<_> = working.into_iter().collect();
        results.sort_by_key(|c| c.distance);
        results
    }

    /// Get memory usage in bytes (approximate)
    pub fn memory_usage(&self) -> usize {
        let nodes_size = self.nodes.len() * std::mem::size_of::<HNSWNode>();
        let neighbors_size = self.neighbors.memory_usage();
        let vectors_size = self.vectors.memory_usage();

        nodes_size + neighbors_size + vectors_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_index_creation() {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(128, params, DistanceFunction::L2, false).unwrap();

        assert_eq!(index.len(), 0);
        assert_eq!(index.dimensions(), 128);
        assert!(index.is_empty());
    }

    #[test]
    fn test_hnsw_index_insert_single() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let vec = vec![1.0, 2.0, 3.0];
        let id = index.insert(vec).unwrap();

        assert_eq!(id, 0);
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_hnsw_index_insert_multiple() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];
        let vec3 = vec![7.0, 8.0, 9.0];

        let id1 = index.insert(vec1).unwrap();
        let id2 = index.insert(vec2).unwrap();
        let id3 = index.insert(vec3).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);
        assert_eq!(index.len(), 3);
    }

    #[test]
    fn test_hnsw_index_dimension_validation() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let wrong_dim = vec![1.0, 2.0]; // Only 2 dimensions
        assert!(index.insert(wrong_dim).is_err());
    }

    #[test]
    fn test_hnsw_index_search_empty() {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let query = vec![1.0, 2.0, 3.0];
        let results = index.search(&query, 5, 100).unwrap();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_hnsw_index_search_single() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let vec = vec![1.0, 2.0, 3.0];
        index.insert(vec.clone()).unwrap();

        let results = index.search(&vec, 5, 100).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 0);
        assert!(results[0].distance < 0.01); // Should be ~0 (same vector)
    }

    #[test]
    fn test_random_level_distribution() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        let mut level_counts = vec![0; 8];

        // Generate 1000 random levels
        for _ in 0..1000 {
            let level = index.random_level();
            level_counts[level as usize] += 1;
        }

        // Level 0 should have most nodes (exponential decay)
        assert!(level_counts[0] > level_counts[1]);
        assert!(level_counts[1] > level_counts[2]);
    }

    #[test]
    fn test_memory_usage() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(128, params, DistanceFunction::L2, false).unwrap();

        // Insert 10 vectors
        for i in 0..10 {
            let vec = vec![i as f32; 128];
            index.insert(vec).unwrap();
        }

        let memory = index.memory_usage();

        // Should have memory for:
        // - 10 nodes (64 bytes each = 640 bytes)
        // - 10 vectors (128 * 4 bytes = 5120 bytes)
        // - Some neighbor storage
        assert!(memory > 5000); // At least vectors + nodes
        assert!(memory < 50000); // Not excessive
    }
}
