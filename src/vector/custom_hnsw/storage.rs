// Vector and neighbor storage for custom HNSW
//
// Design goals:
// - Separate neighbors from nodes (fetch only when needed)
// - Support quantized and full precision vectors
// - Memory-efficient neighbor list storage

use serde::{Deserialize, Serialize};

/// Storage for neighbor lists
///
/// Neighbors are stored separately from nodes to improve cache utilization.
/// Only fetch neighbors when traversing the graph.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborLists {
    /// Neighbor storage: neighbors[node_id][level] = Vec<neighbor_ids>
    ///
    /// Simple 2D structure: first index is node_id, second is level
    neighbors: Vec<Vec<Vec<u32>>>,

    /// Maximum levels supported
    max_levels: usize,
}

impl NeighborLists {
    /// Create empty neighbor lists
    pub fn new(max_levels: usize) -> Self {
        Self {
            neighbors: Vec::new(),
            max_levels,
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(num_nodes: usize, max_levels: usize) -> Self {
        Self {
            neighbors: Vec::with_capacity(num_nodes),
            max_levels,
        }
    }

    /// Get neighbors for a node at a specific level
    pub fn get_neighbors(&self, node_id: u32, level: u8) -> &[u32] {
        let node_idx = node_id as usize;
        let level_idx = level as usize;

        if node_idx >= self.neighbors.len() {
            return &[];
        }

        if level_idx >= self.neighbors[node_idx].len() {
            return &[];
        }

        &self.neighbors[node_idx][level_idx]
    }

    /// Set neighbors for a node at a specific level
    pub fn set_neighbors(&mut self, node_id: u32, level: u8, neighbors_list: Vec<u32>) {
        let node_idx = node_id as usize;
        let level_idx = level as usize;

        // Ensure we have enough nodes
        while self.neighbors.len() <= node_idx {
            self.neighbors.push(vec![Vec::new(); self.max_levels]);
        }

        // Set the neighbors at this level
        self.neighbors[node_idx][level_idx] = neighbors_list;
    }

    /// Add a bidirectional link between two nodes at a level
    pub fn add_bidirectional_link(&mut self, node_a: u32, node_b: u32, level: u8) {
        let node_a_idx = node_a as usize;
        let node_b_idx = node_b as usize;
        let level_idx = level as usize;

        // Ensure we have enough nodes
        let max_idx = node_a_idx.max(node_b_idx);
        while self.neighbors.len() <= max_idx {
            self.neighbors.push(vec![Vec::new(); self.max_levels]);
        }

        // Add node_b to node_a's neighbors (if not already present)
        if !self.neighbors[node_a_idx][level_idx].contains(&node_b) {
            self.neighbors[node_a_idx][level_idx].push(node_b);
        }

        // Add node_a to node_b's neighbors (if not already present)
        if !self.neighbors[node_b_idx][level_idx].contains(&node_a) {
            self.neighbors[node_b_idx][level_idx].push(node_a);
        }
    }

    /// Get total number of neighbor entries
    pub fn total_neighbors(&self) -> usize {
        self.neighbors
            .iter()
            .flat_map(|node| node.iter())
            .map(|level| level.len())
            .sum()
    }

    /// Get memory usage in bytes (approximate)
    pub fn memory_usage(&self) -> usize {
        let mut total = 0;

        // Size of outer Vec
        total += self.neighbors.capacity() * std::mem::size_of::<Vec<Vec<u32>>>();

        // Size of each node's level vecs
        for node in &self.neighbors {
            total += node.capacity() * std::mem::size_of::<Vec<u32>>();

            // Size of actual neighbor data
            for level in node {
                total += level.len() * std::mem::size_of::<u32>();
            }
        }

        total
    }
}

/// Vector storage (quantized or full precision)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VectorStorage {
    /// Full precision f32 vectors
    ///
    /// Memory: dimensions * 4 bytes per vector
    /// Example: 1536D = 6144 bytes per vector
    FullPrecision {
        vectors: Vec<Vec<f32>>,
        dimensions: usize,
    },

    /// Binary quantized vectors
    ///
    /// Memory: dimensions / 8 bytes per vector (1 bit per dimension)
    /// Example: 1536D = 192 bytes per vector (32x compression)
    BinaryQuantized {
        /// Quantized vectors (1 bit per dimension, packed into bytes)
        quantized: Vec<Vec<u8>>,

        /// Original vectors for reranking (optional)
        ///
        /// If present: Memory = quantized + original
        /// If absent: Faster but lower recall
        original: Option<Vec<Vec<f32>>>,

        /// Quantization thresholds (one per dimension)
        thresholds: Vec<f32>,

        /// Vector dimensions
        dimensions: usize,
    },
}

impl VectorStorage {
    /// Create empty full precision storage
    pub fn new_full_precision(dimensions: usize) -> Self {
        Self::FullPrecision {
            vectors: Vec::new(),
            dimensions,
        }
    }

    /// Create empty binary quantized storage
    pub fn new_binary_quantized(dimensions: usize, keep_original: bool) -> Self {
        Self::BinaryQuantized {
            quantized: Vec::new(),
            original: if keep_original {
                Some(Vec::new())
            } else {
                None
            },
            thresholds: vec![0.0; dimensions], // Will be computed during training
            dimensions,
        }
    }

    /// Get number of vectors stored
    pub fn len(&self) -> usize {
        match self {
            Self::FullPrecision { vectors, .. } => vectors.len(),
            Self::BinaryQuantized { quantized, .. } => quantized.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        match self {
            Self::FullPrecision { dimensions, .. } => *dimensions,
            Self::BinaryQuantized { dimensions, .. } => *dimensions,
        }
    }

    /// Insert a full precision vector
    pub fn insert(&mut self, vector: Vec<f32>) -> Result<u32, String> {
        match self {
            Self::FullPrecision { vectors, dimensions } => {
                if vector.len() != *dimensions {
                    return Err(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        dimensions,
                        vector.len()
                    ));
                }
                let id = vectors.len() as u32;
                vectors.push(vector);
                Ok(id)
            }
            Self::BinaryQuantized {
                quantized,
                original,
                thresholds,
                dimensions,
            } => {
                if vector.len() != *dimensions {
                    return Err(format!(
                        "Vector dimension mismatch: expected {}, got {}",
                        dimensions,
                        vector.len()
                    ));
                }

                // Quantize vector
                let quant = Self::quantize_binary(&vector, thresholds);
                let id = quantized.len() as u32;
                quantized.push(quant);

                // Store original if requested
                if let Some(orig) = original {
                    orig.push(vector);
                }

                Ok(id)
            }
        }
    }

    /// Get a vector by ID (full precision)
    pub fn get(&self, id: u32) -> Option<&[f32]> {
        match self {
            Self::FullPrecision { vectors, .. } => {
                vectors.get(id as usize).map(|v| v.as_slice())
            }
            Self::BinaryQuantized { original, .. } => {
                original.as_ref().and_then(|o| o.get(id as usize).map(|v| v.as_slice()))
            }
        }
    }

    /// Binary quantize a vector
    ///
    /// Each dimension is quantized to 1 bit based on threshold:
    /// - value >= threshold[dim] => 1
    /// - value < threshold[dim] => 0
    fn quantize_binary(vector: &[f32], thresholds: &[f32]) -> Vec<u8> {
        debug_assert_eq!(vector.len(), thresholds.len());

        let num_bytes = (vector.len() + 7) / 8; // Round up
        let mut quantized = vec![0u8; num_bytes];

        for (i, (&value, &threshold)) in vector.iter().zip(thresholds.iter()).enumerate() {
            if value >= threshold {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                quantized[byte_idx] |= 1 << bit_idx;
            }
        }

        quantized
    }

    /// Compute quantization thresholds from sample vectors
    ///
    /// Uses median of each dimension as threshold
    pub fn train_quantization(&mut self, sample_vectors: &[Vec<f32>]) -> Result<(), String> {
        match self {
            Self::BinaryQuantized {
                thresholds,
                dimensions,
                ..
            } => {
                if sample_vectors.is_empty() {
                    return Err("Cannot train on empty sample".to_string());
                }

                // Verify all vectors have correct dimensions
                for vec in sample_vectors {
                    if vec.len() != *dimensions {
                        return Err("Sample vector dimension mismatch".to_string());
                    }
                }

                // Compute median for each dimension
                for dim in 0..*dimensions {
                    let mut values: Vec<f32> = sample_vectors.iter().map(|v| v[dim]).collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

                    let median = if values.len() % 2 == 0 {
                        let mid = values.len() / 2;
                        (values[mid - 1] + values[mid]) / 2.0
                    } else {
                        values[values.len() / 2]
                    };

                    thresholds[dim] = median;
                }

                Ok(())
            }
            Self::FullPrecision { .. } => {
                Err("Cannot train quantization on full precision storage".to_string())
            }
        }
    }

    /// Get memory usage in bytes (approximate)
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::FullPrecision { vectors, dimensions } => {
                vectors.len() * dimensions * std::mem::size_of::<f32>()
            }
            Self::BinaryQuantized {
                quantized,
                original,
                thresholds,
                dimensions,
            } => {
                let quantized_size = quantized.len() * (dimensions + 7) / 8;
                let original_size = original
                    .as_ref()
                    .map(|o| o.len() * dimensions * std::mem::size_of::<f32>())
                    .unwrap_or(0);
                let thresholds_size = thresholds.len() * std::mem::size_of::<f32>();
                quantized_size + original_size + thresholds_size
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighbor_lists_basic() {
        let mut lists = NeighborLists::new(8);

        // Set neighbors for node 0, level 0
        lists.set_neighbors(0, 0, vec![1, 2, 3]);

        let neighbors = lists.get_neighbors(0, 0);
        assert_eq!(neighbors, &[1, 2, 3]);

        // Empty level
        let empty = lists.get_neighbors(0, 1);
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_neighbor_lists_bidirectional() {
        let mut lists = NeighborLists::new(8);

        lists.add_bidirectional_link(0, 1, 0);

        assert_eq!(lists.get_neighbors(0, 0), &[1]);
        assert_eq!(lists.get_neighbors(1, 0), &[0]);
    }

    #[test]
    fn test_vector_storage_full_precision() {
        let mut storage = VectorStorage::new_full_precision(3);

        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![4.0, 5.0, 6.0];

        let id1 = storage.insert(vec1.clone()).unwrap();
        let id2 = storage.insert(vec2.clone()).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(storage.len(), 2);

        assert_eq!(storage.get(0), Some(vec1.as_slice()));
        assert_eq!(storage.get(1), Some(vec2.as_slice()));
    }

    #[test]
    fn test_vector_storage_dimension_check() {
        let mut storage = VectorStorage::new_full_precision(3);

        let wrong_dim = vec![1.0, 2.0]; // Only 2 dimensions
        assert!(storage.insert(wrong_dim).is_err());
    }

    #[test]
    fn test_binary_quantization() {
        let vector = vec![0.5, -0.3, 0.8, -0.1];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let quantized = VectorStorage::quantize_binary(&vector, &thresholds);

        // First 4 bits should be: 1, 0, 1, 0 (based on >= 0.0)
        // Packed as: bit0=1, bit1=0, bit2=1, bit3=0 => 0b00000101 = 5
        assert_eq!(quantized[0], 5);
    }

    #[test]
    fn test_quantization_training() {
        let mut storage = VectorStorage::new_binary_quantized(2, true);

        let samples = vec![
            vec![1.0, 5.0],
            vec![2.0, 6.0],
            vec![3.0, 7.0],
        ];

        storage.train_quantization(&samples).unwrap();

        // Thresholds should be medians: [2.0, 6.0]
        match storage {
            VectorStorage::BinaryQuantized { thresholds, .. } => {
                assert_eq!(thresholds, vec![2.0, 6.0]);
            }
            _ => panic!("Expected BinaryQuantized storage"),
        }
    }
}
