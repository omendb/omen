//! AlexTree: Adaptive learned index tree structure
//!
//! Simplified single-level ALEX tree implementation:
//! - Array of leaf nodes (GappedNode)
//! - Each leaf handles a key range
//! - Splits create new leaves
//!
//! Future: Add inner nodes for multi-level tree

use super::GappedNode;
use anyhow::Result;

/// AlexTree: Single-level adaptive learned index
///
/// Current implementation uses a simple array of leaf nodes.
/// When a leaf splits, we insert the new leaf into the array.
#[derive(Debug)]
pub struct AlexTree {
    /// Leaf nodes, sorted by key range
    leaves: Vec<GappedNode>,

    /// Split keys between leaves (leaves[i] handles keys < split_keys[i])
    /// Last leaf handles all remaining keys
    split_keys: Vec<i64>,

    /// Default expansion factor for new nodes
    expansion_factor: f64,
}

impl AlexTree {
    /// Create new ALEX tree
    pub fn new() -> Self {
        Self {
            leaves: vec![GappedNode::new(100, 1.0)], // Start with one leaf
            split_keys: vec![],
            expansion_factor: 1.0,
        }
    }

    /// Create ALEX tree with custom expansion factor
    pub fn with_expansion(expansion_factor: f64) -> Self {
        Self {
            leaves: vec![GappedNode::new(100, expansion_factor)],
            split_keys: vec![],
            expansion_factor,
        }
    }

    /// Insert key-value pair
    ///
    /// Routes to appropriate leaf, splits if necessary.
    /// **Time complexity**: O(log n) to find leaf + O(log error) to insert
    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<()> {
        // Find leaf for this key
        let leaf_idx = self.find_leaf_index(key);

        // Try to insert
        let insert_result = self.leaves[leaf_idx].insert(key, value.clone())?;

        if !insert_result {
            // Leaf is full - split it
            let (split_key, right_leaf) = self.leaves[leaf_idx].split()?;

            // Insert new leaf and split key
            self.split_keys.insert(leaf_idx, split_key);
            self.leaves.insert(leaf_idx + 1, right_leaf);

            // Retry insert - route to correct leaf after split
            let new_leaf_idx = self.find_leaf_index(key);
            self.leaves[new_leaf_idx].insert(key, value)?;
        }

        Ok(())
    }

    /// Get value for key
    ///
    /// **Time complexity**: O(log n) to find leaf + O(log error) to search
    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        let leaf_idx = self.find_leaf_index(key);
        self.leaves[leaf_idx].get(key)
    }

    /// Find leaf index for key using binary search on split keys
    ///
    /// Leaf routing: split_keys[i] is the FIRST key of leaf[i+1]
    /// - leaf[0]: keys < split_keys[0]
    /// - leaf[i]: keys in [split_keys[i-1], split_keys[i])
    /// - leaf[n-1]: keys >= split_keys[n-2]
    fn find_leaf_index(&self, key: i64) -> usize {
        // Binary search for first split_key > key
        match self.split_keys.binary_search(&key) {
            Ok(idx) => idx + 1,  // key == split_keys[idx] → in leaf[idx+1]
            Err(idx) => idx,     // key should be inserted at idx → in leaf[idx]
        }
    }

    /// Get total number of keys across all leaves
    pub fn len(&self) -> usize {
        self.leaves.iter().map(|leaf| leaf.num_keys()).sum()
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get number of leaf nodes
    pub fn num_leaves(&self) -> usize {
        self.leaves.len()
    }
}

impl Default for AlexTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert_get() {
        let mut tree = AlexTree::new();

        tree.insert(10, vec![1]).unwrap();
        tree.insert(20, vec![2]).unwrap();
        tree.insert(30, vec![3]).unwrap();

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.get(10).unwrap(), Some(vec![1]));
        assert_eq!(tree.get(20).unwrap(), Some(vec![2]));
        assert_eq!(tree.get(30).unwrap(), Some(vec![3]));
        assert_eq!(tree.get(40).unwrap(), None);
    }

    #[test]
    fn test_split_creates_new_leaf() {
        let mut tree = AlexTree::with_expansion(0.0); // No expansion - splits quickly

        // Fill first leaf to capacity (will trigger split)
        for i in 0..100 {
            tree.insert(i, vec![i as u8]).unwrap();
        }

        // Should have split into multiple leaves
        assert!(tree.num_leaves() > 1);
        assert_eq!(tree.len(), 100);

        // All keys should still be retrievable
        for i in 0..100 {
            assert!(tree.get(i).unwrap().is_some(), "Missing key {}", i);
        }
    }

    #[test]
    fn test_out_of_order_inserts() {
        let mut tree = AlexTree::new();

        tree.insert(50, vec![5]).unwrap();
        tree.insert(10, vec![1]).unwrap();
        tree.insert(30, vec![3]).unwrap();
        tree.insert(20, vec![2]).unwrap();
        tree.insert(40, vec![4]).unwrap();

        assert_eq!(tree.len(), 5);

        // All keys should be found
        for i in [10, 20, 30, 40, 50] {
            assert!(tree.get(i).unwrap().is_some());
        }
    }

    #[test]
    fn test_large_scale() {
        let mut tree = AlexTree::new();

        // Insert 10K keys
        for i in 0..10000 {
            tree.insert(i, vec![(i % 256) as u8]).unwrap();
        }

        assert_eq!(tree.len(), 10000);

        // Sample lookups
        for i in (0..10000).step_by(100) {
            assert!(tree.get(i).unwrap().is_some());
        }
    }
}
