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
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

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

    /// Get entry point
    pub fn entry_point(&self) -> Option<u32> {
        self.entry_point
    }

    /// Get node level
    pub fn node_level(&self, node_id: u32) -> Option<u8> {
        self.nodes.get(node_id as usize).map(|n| n.level)
    }

    /// Get neighbor count for a node at a level
    pub fn neighbor_count(&self, node_id: u32, level: u8) -> usize {
        self.neighbors.get_neighbors(node_id, level).len()
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

        // Insert into graph
        self.insert_into_graph(node_id, &vector, level)?;

        // Update entry point if this node has higher level than current entry point
        let entry_point_id = self.entry_point.unwrap();
        let entry_level = self.nodes[entry_point_id as usize].level;
        if level > entry_level {
            self.entry_point = Some(node_id);
        }

        Ok(node_id)
    }

    /// Insert node into graph structure
    ///
    /// Implements HNSW insertion algorithm (Malkov & Yashunin 2018)
    fn insert_into_graph(
        &mut self,
        node_id: u32,
        vector: &[f32],
        level: u8,
    ) -> Result<(), String> {
        let entry_point = self.entry_point.expect("Entry point must exist");
        let entry_level = self.nodes[entry_point as usize].level;

        // Search for nearest neighbors at each level above target level
        let mut nearest = vec![entry_point];
        for lc in ((level + 1)..=entry_level).rev() {
            nearest = self.search_layer(vector, &nearest, 1, lc);
        }

        // Insert at levels 0..=level (iterate from top to bottom)
        for lc in (0..=level).rev() {
            // Find ef_construction nearest neighbors at this level
            let candidates = self.search_layer(vector, &nearest, self.params.ef_construction, lc);

            // Select M best neighbors using heuristic
            let m = if lc == 0 {
                self.params.m * 2 // Level 0 has more connections
            } else {
                self.params.m
            };

            let neighbors = self.select_neighbors_heuristic(node_id, &candidates, m, lc, vector);

            // Add bidirectional links
            for &neighbor_id in &neighbors {
                self.neighbors.add_bidirectional_link(node_id, neighbor_id, lc);
            }

            // Update neighbor counts
            self.nodes[node_id as usize].set_neighbor_count(lc, neighbors.len());

            // Prune neighbors' connections if they exceed M
            for &neighbor_id in &neighbors {
                let neighbor_neighbors = self.neighbors.get_neighbors(neighbor_id, lc).to_vec();
                if neighbor_neighbors.len() > m {
                    let neighbor_vec = self.vectors.get(neighbor_id).expect("Neighbor vector must exist");
                    let pruned = self.select_neighbors_heuristic(
                        neighbor_id,
                        &neighbor_neighbors,
                        m,
                        lc,
                        neighbor_vec,
                    );

                    // Clear and reset neighbors
                    self.neighbors.set_neighbors(neighbor_id, lc, pruned.clone());
                    self.nodes[neighbor_id as usize].set_neighbor_count(lc, pruned.len());
                }
            }

            // Update nearest for next level
            nearest = candidates;
        }

        Ok(())
    }

    /// Select neighbors using heuristic (diverse neighbors, better recall)
    ///
    /// Algorithm from Malkov 2018, Section 4
    fn select_neighbors_heuristic(
        &self,
        _node_id: u32,
        candidates: &[u32],
        m: usize,
        _level: u8,
        query_vector: &[f32],
    ) -> Vec<u32> {
        if candidates.len() <= m {
            return candidates.to_vec();
        }

        // Sort candidates by distance to query
        let mut sorted_candidates: Vec<_> = candidates
            .iter()
            .map(|&id| {
                let dist = self.distance_to_query(query_vector, id);
                (id, dist)
            })
            .collect();
        sorted_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut result = Vec::with_capacity(m);
        let mut remaining = Vec::new();

        // Heuristic: Select diverse neighbors
        for (candidate_id, candidate_dist) in &sorted_candidates {
            if result.len() >= m {
                remaining.push(*candidate_id);
                continue;
            }

            // Check if candidate is closer to query than to existing neighbors
            let mut good = true;
            for &result_id in &result {
                let dist_to_result = self.distance(*candidate_id, result_id);
                if dist_to_result < *candidate_dist {
                    good = false;
                    break;
                }
            }

            if good {
                result.push(*candidate_id);
            } else {
                remaining.push(*candidate_id);
            }
        }

        // Fill remaining slots with closest candidates if needed
        for candidate_id in remaining {
            if result.len() >= m {
                break;
            }
            result.push(candidate_id);
        }

        result
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
        let entry_level = self.nodes[entry_point as usize].level;

        // Start from entry point, descend to layer 0
        let mut nearest = vec![entry_point];

        // Greedy search at each layer (find 1 nearest)
        for level in (1..=entry_level).rev() {
            nearest = self.search_layer(query, &nearest, 1, level);
        }

        // Beam search at layer 0 (find ef nearest)
        let candidates = self.search_layer(query, &nearest, ef.max(k), 0);

        // Convert to SearchResult and return k nearest
        let mut results: Vec<SearchResult> = candidates
            .iter()
            .map(|&id| {
                let distance = self.distance_to_query(query, id);
                SearchResult::new(id, distance)
            })
            .collect();

        // Sort by distance (closest first)
        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        // Return top k
        results.truncate(k);
        Ok(results)
    }

    /// Search for nearest neighbors at a specific level
    ///
    /// Returns node IDs of up to ef nearest neighbors.
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[u32],
        ef: usize,
        level: u8,
    ) -> Vec<u32> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new(); // Min-heap (closest first, using Reverse)
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
        while let Some(Reverse(current)) = candidates.pop() {
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

                // If neighbor is closer than farthest in working set, or working set not full, add it
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

        // Return node IDs sorted by distance (closest first)
        let mut results: Vec<_> = working.into_iter().collect();
        results.sort_by_key(|c| c.distance);
        results.into_iter().map(|c| c.node_id).collect()
    }

    /// Get memory usage in bytes (approximate)
    pub fn memory_usage(&self) -> usize {
        let nodes_size = self.nodes.len() * std::mem::size_of::<HNSWNode>();
        let neighbors_size = self.neighbors.memory_usage();
        let vectors_size = self.vectors.memory_usage();

        nodes_size + neighbors_size + vectors_size
    }

    /// Save index to disk
    ///
    /// Format:
    /// - Magic: b"HNSWIDX\0" (8 bytes)
    /// - Version: u32 (4 bytes)
    /// - Dimensions: u32 (4 bytes)
    /// - Num nodes: u32 (4 bytes)
    /// - Entry point: Option<u32> (1 + 4 bytes)
    /// - Distance function: DistanceFunction (bincode)
    /// - Params: HNSWParams (bincode)
    /// - RNG state: u64 (8 bytes)
    /// - Nodes: Vec<HNSWNode> (raw bytes, 64 * num_nodes)
    /// - Neighbors: NeighborLists (bincode)
    /// - Vectors: VectorStorage (bincode)
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        let mut writer = BufWriter::new(file);

        // Write magic bytes
        writer
            .write_all(b"HNSWIDX\0")
            .map_err(|e| format!("Failed to write magic: {}", e))?;

        // Write version
        writer
            .write_all(&1u32.to_le_bytes())
            .map_err(|e| format!("Failed to write version: {}", e))?;

        // Write dimensions
        writer
            .write_all(&(self.dimensions() as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write dimensions: {}", e))?;

        // Write num nodes
        writer
            .write_all(&(self.nodes.len() as u32).to_le_bytes())
            .map_err(|e| format!("Failed to write num_nodes: {}", e))?;

        // Write entry point
        match self.entry_point {
            Some(ep) => {
                writer
                    .write_all(&[1u8])
                    .map_err(|e| format!("Failed to write entry_point flag: {}", e))?;
                writer
                    .write_all(&ep.to_le_bytes())
                    .map_err(|e| format!("Failed to write entry_point: {}", e))?;
            }
            None => {
                writer
                    .write_all(&[0u8])
                    .map_err(|e| format!("Failed to write entry_point flag: {}", e))?;
            }
        }

        // Write distance function
        bincode::serialize_into(&mut writer, &self.distance_fn)
            .map_err(|e| format!("Failed to serialize distance_fn: {}", e))?;

        // Write params
        bincode::serialize_into(&mut writer, &self.params)
            .map_err(|e| format!("Failed to serialize params: {}", e))?;

        // Write RNG state
        writer
            .write_all(&self.rng_state.to_le_bytes())
            .map_err(|e| format!("Failed to write rng_state: {}", e))?;

        // Write nodes (raw bytes for fast I/O)
        if !self.nodes.is_empty() {
            let nodes_bytes = unsafe {
                std::slice::from_raw_parts(
                    self.nodes.as_ptr() as *const u8,
                    self.nodes.len() * std::mem::size_of::<HNSWNode>(),
                )
            };
            writer
                .write_all(nodes_bytes)
                .map_err(|e| format!("Failed to write nodes: {}", e))?;
        }

        // Write neighbor lists
        bincode::serialize_into(&mut writer, &self.neighbors)
            .map_err(|e| format!("Failed to serialize neighbors: {}", e))?;

        // Write vectors
        bincode::serialize_into(&mut writer, &self.vectors)
            .map_err(|e| format!("Failed to serialize vectors: {}", e))?;

        Ok(())
    }

    /// Load index from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Read and verify magic bytes
        let mut magic = [0u8; 8];
        reader
            .read_exact(&mut magic)
            .map_err(|e| format!("Failed to read magic: {}", e))?;
        if &magic != b"HNSWIDX\0" {
            return Err(format!("Invalid magic bytes: {:?}", magic));
        }

        // Read version
        let mut version_bytes = [0u8; 4];
        reader
            .read_exact(&mut version_bytes)
            .map_err(|e| format!("Failed to read version: {}", e))?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 1 {
            return Err(format!("Unsupported version: {}", version));
        }

        // Read dimensions
        let mut dimensions_bytes = [0u8; 4];
        reader
            .read_exact(&mut dimensions_bytes)
            .map_err(|e| format!("Failed to read dimensions: {}", e))?;
        let dimensions = u32::from_le_bytes(dimensions_bytes) as usize;

        // Read num nodes
        let mut num_nodes_bytes = [0u8; 4];
        reader
            .read_exact(&mut num_nodes_bytes)
            .map_err(|e| format!("Failed to read num_nodes: {}", e))?;
        let num_nodes = u32::from_le_bytes(num_nodes_bytes) as usize;

        // Read entry point
        let mut entry_point_flag = [0u8; 1];
        reader
            .read_exact(&mut entry_point_flag)
            .map_err(|e| format!("Failed to read entry_point flag: {}", e))?;
        let entry_point = if entry_point_flag[0] == 1 {
            let mut ep_bytes = [0u8; 4];
            reader
                .read_exact(&mut ep_bytes)
                .map_err(|e| format!("Failed to read entry_point: {}", e))?;
            Some(u32::from_le_bytes(ep_bytes))
        } else {
            None
        };

        // Read distance function
        let distance_fn: DistanceFunction = bincode::deserialize_from(&mut reader)
            .map_err(|e| format!("Failed to deserialize distance_fn: {}", e))?;

        // Read params
        let params: HNSWParams = bincode::deserialize_from(&mut reader)
            .map_err(|e| format!("Failed to deserialize params: {}", e))?;

        // Read RNG state
        let mut rng_state_bytes = [0u8; 8];
        reader
            .read_exact(&mut rng_state_bytes)
            .map_err(|e| format!("Failed to read rng_state: {}", e))?;
        let rng_state = u64::from_le_bytes(rng_state_bytes);

        // Read nodes (raw bytes for fast I/O)
        let mut nodes = vec![HNSWNode::default(); num_nodes];
        if num_nodes > 0 {
            let nodes_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    nodes.as_mut_ptr() as *mut u8,
                    nodes.len() * std::mem::size_of::<HNSWNode>(),
                )
            };
            reader
                .read_exact(nodes_bytes)
                .map_err(|e| format!("Failed to read nodes: {}", e))?;
        }

        // Read neighbor lists
        let neighbors: NeighborLists = bincode::deserialize_from(&mut reader)
            .map_err(|e| format!("Failed to deserialize neighbors: {}", e))?;

        // Read vectors
        let vectors: VectorStorage = bincode::deserialize_from(&mut reader)
            .map_err(|e| format!("Failed to deserialize vectors: {}", e))?;

        // Verify dimensions match
        if vectors.dimensions() != dimensions {
            return Err(format!(
                "Dimension mismatch: header says {}, vectors have {}",
                dimensions,
                vectors.dimensions()
            ));
        }

        Ok(Self {
            nodes,
            neighbors,
            vectors,
            entry_point,
            params,
            distance_fn,
            rng_state,
        })
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

    #[test]
    fn test_hnsw_index_search_multiple() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 5 vectors
        let vecs = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![0.5, 0.5, 0.0],
            vec![0.0, 0.5, 0.5],
        ];

        for vec in vecs {
            index.insert(vec).unwrap();
        }

        // Search for k=3 nearest to [1.0, 0.0, 0.0]
        let query = vec![1.0, 0.0, 0.0];
        let results = index.search(&query, 3, 10).unwrap();

        // Should return 3 results
        assert_eq!(results.len(), 3);

        // First result should be closest (id=0, exact match)
        assert_eq!(results[0].id, 0);
        assert!(results[0].distance < 0.01);

        // Results should be sorted by distance
        for i in 0..results.len() - 1 {
            assert!(results[i].distance <= results[i + 1].distance);
        }
    }

    #[test]
    fn test_hnsw_index_search_with_ef() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 10 vectors
        for i in 0..10 {
            let vec = vec![i as f32, 0.0, 0.0];
            index.insert(vec).unwrap();
        }

        // Search with different ef values
        let query = vec![5.0, 0.0, 0.0];

        let results_ef_5 = index.search(&query, 3, 5).unwrap();
        let results_ef_10 = index.search(&query, 3, 10).unwrap();

        // Both should return 3 results (k=3)
        assert_eq!(results_ef_5.len(), 3);
        assert_eq!(results_ef_10.len(), 3);

        // Higher ef should explore more candidates (potentially better recall)
        // Both should find node 5 as closest
        assert_eq!(results_ef_5[0].id, 5);
        assert_eq!(results_ef_10[0].id, 5);
    }

    #[test]
    fn test_hnsw_levels() {
        let mut params = HNSWParams::default();
        params.seed = 12345; // Fixed seed for reproducibility

        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 100 vectors
        for i in 0..100 {
            let vec = vec![i as f32, 0.0, 0.0];
            index.insert(vec).unwrap();
        }

        // Count how many nodes have their TOP level at each height
        // Note: All nodes exist at level 0, but node.level is their TOP level
        let mut top_level_counts = vec![0; 8];
        for node in &index.nodes {
            top_level_counts[node.level as usize] += 1;
        }

        // Most nodes should have top level = 0 (due to exponential decay)
        assert!(top_level_counts[0] > 80); // Most nodes only at level 0

        // Some nodes should have higher top levels
        let higher_level_count: usize = top_level_counts[1..].iter().sum();
        assert!(higher_level_count > 0); // At least some nodes at higher levels

        // All nodes should exist (sum should be 100)
        let total: usize = top_level_counts.iter().sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_neighbor_count_limits() {
        let mut params = HNSWParams::default();
        params.m = 4; // Small M for easier testing
        params.ef_construction = 10;

        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 20 vectors (enough to test neighbor pruning)
        for i in 0..20 {
            let vec = vec![i as f32, 0.0, 0.0];
            index.insert(vec).unwrap();
        }

        // Check that no node has more than M*2 neighbors at level 0
        for node in &index.nodes {
            let neighbor_count = index.neighbors.get_neighbors(node.id, 0).len();
            assert!(neighbor_count <= params.m * 2);
        }
    }

    #[test]
    fn test_search_recall_simple() {
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 10 vectors in a line
        for i in 0..10 {
            let vec = vec![i as f32, 0.0, 0.0];
            index.insert(vec).unwrap();
        }

        // Query should find exact neighbors
        let query = vec![5.0, 0.0, 0.0];
        let results = index.search(&query, 3, 20).unwrap();

        // Should find nodes 5, 4, and 6 (closest to query)
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, 5); // Exact match

        // Second and third should be 4 or 6
        let ids: Vec<u32> = results.iter().map(|r| r.id).collect();
        assert!(ids.contains(&4));
        assert!(ids.contains(&6));
    }

    #[test]
    fn test_save_load_empty() {
        use tempfile::NamedTempFile;

        let params = HNSWParams::default();
        let index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Save empty index
        let temp_file = NamedTempFile::new().unwrap();
        index.save(temp_file.path()).unwrap();

        // Load it back
        let loaded = HNSWIndex::load(temp_file.path()).unwrap();

        assert_eq!(loaded.dimensions(), 3);
        assert_eq!(loaded.len(), 0);
        assert!(loaded.is_empty());
        assert_eq!(loaded.entry_point, None);
    }

    #[test]
    fn test_save_load_small() {
        use tempfile::NamedTempFile;

        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert 10 vectors
        for i in 0..10 {
            let vec = vec![i as f32, 0.0, 0.0];
            index.insert(vec).unwrap();
        }

        // Save index
        let temp_file = NamedTempFile::new().unwrap();
        index.save(temp_file.path()).unwrap();

        // Load it back
        let loaded = HNSWIndex::load(temp_file.path()).unwrap();

        // Verify basic properties
        assert_eq!(loaded.dimensions(), 3);
        assert_eq!(loaded.len(), 10);
        assert!(!loaded.is_empty());
        assert_eq!(loaded.entry_point, index.entry_point);

        // Verify vectors are preserved
        for i in 0..10 {
            let orig = index.vectors.get(i).unwrap();
            let load = loaded.vectors.get(i).unwrap();
            assert_eq!(orig, load);
        }

        // Verify search works on loaded index
        let query = vec![5.0, 0.0, 0.0];
        let results = loaded.search(&query, 3, 20).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, 5); // Should still find exact match
    }

    #[test]
    fn test_save_load_preserves_graph() {
        use tempfile::NamedTempFile;

        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(3, params, DistanceFunction::L2, false).unwrap();

        // Insert vectors
        for i in 0..20 {
            let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32];
            index.insert(vec).unwrap();
        }

        // Get search results before saving
        let query = vec![10.0, 20.0, 30.0];
        let results_before = index.search(&query, 5, 20).unwrap();

        // Save and load
        let temp_file = NamedTempFile::new().unwrap();
        index.save(temp_file.path()).unwrap();
        let loaded = HNSWIndex::load(temp_file.path()).unwrap();

        // Get search results after loading
        let results_after = loaded.search(&query, 5, 20).unwrap();

        // Results should be identical
        assert_eq!(results_before.len(), results_after.len());
        for (before, after) in results_before.iter().zip(results_after.iter()) {
            assert_eq!(before.id, after.id);
            assert!((before.distance - after.distance).abs() < 1e-5);
        }
    }

    #[test]
    fn test_save_load_with_quantization() {
        use tempfile::NamedTempFile;

        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(8, params, DistanceFunction::L2, true).unwrap();

        // Train quantization
        let samples: Vec<Vec<f32>> = (0..10).map(|i| vec![i as f32; 8]).collect();
        if let VectorStorage::BinaryQuantized {
            ref mut thresholds, ..
        } = index.vectors
        {
            for (i, threshold) in thresholds.iter_mut().enumerate() {
                *threshold = i as f32 + 0.5;
            }
        }

        // Insert vectors
        for i in 0..10 {
            let vec = vec![i as f32; 8];
            index.insert(vec).unwrap();
        }

        // Save and load
        let temp_file = NamedTempFile::new().unwrap();
        index.save(temp_file.path()).unwrap();
        let loaded = HNSWIndex::load(temp_file.path()).unwrap();

        // Verify quantization is preserved
        match (&index.vectors, &loaded.vectors) {
            (
                VectorStorage::BinaryQuantized {
                    thresholds: t1, ..
                },
                VectorStorage::BinaryQuantized {
                    thresholds: t2, ..
                },
            ) => {
                assert_eq!(t1, t2);
            }
            _ => panic!("Expected BinaryQuantized storage"),
        }

        // Search should work
        let query = vec![5.0; 8];
        let results = loaded.search(&query, 3, 20).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_load_invalid_magic() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"INVALID\0").unwrap();
        temp_file.flush().unwrap();

        let result = HNSWIndex::load(temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid magic"));
    }

    #[test]
    fn test_load_unsupported_version() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"HNSWIDX\0").unwrap(); // Magic
        temp_file.write_all(&99u32.to_le_bytes()).unwrap(); // Unsupported version
        temp_file.flush().unwrap();

        let result = HNSWIndex::load(temp_file.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported version"));
    }
}
