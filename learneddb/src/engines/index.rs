//! Index engine implementations - learned and traditional

use super::{IndexEngine, Result};
use omendb::{LinearIndex, RMIIndex, LearnedIndex};
use std::collections::BTreeMap;
use std::sync::RwLock;

/// Learned linear index engine
#[derive(Debug)]
pub struct LearnedLinearEngine {
    index: Option<LinearIndex<usize>>,
    data: Vec<(i64, Vec<u8>)>,
}

impl LearnedLinearEngine {
    pub fn new() -> Self {
        Self {
            index: None,
            data: Vec::new(),
        }
    }
}

impl IndexEngine for LearnedLinearEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        self.data = data.to_vec();

        // Create training data: key -> position
        let training_data: Vec<(i64, usize)> = data
            .iter()
            .enumerate()
            .map(|(pos, (key, _))| (*key, pos))
            .collect();

        self.index = Some(LinearIndex::train(training_data)?);
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        if let Some(ref index) = self.index {
            index.get(&key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn search(&self, key: i64) -> Option<usize> {
        if let Some(ref index) = self.index {
            index.get(&key).copied()
        } else {
            self.data.binary_search_by_key(&key, |(k, _)| *k).ok()
        }
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        if let Some(ref index) = self.index {
            index.range(&start, &end).into_iter().collect()
        } else {
            self.data
                .iter()
                .filter(|(k, _)| *k >= start && *k <= end)
                .map(|(k, _)| *k)
                .collect()
        }
    }

    fn stats(&self) -> String {
        if let Some(ref index) = self.index {
            format!("LearnedLinear: {} keys, slope={:.6}, intercept={:.2}, max_error={}",
                self.data.len(), index.slope(), index.intercept(), index.max_error())
        } else {
            format!("LearnedLinear: untrained")
        }
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        Box::new(Self {
            index: self.index.clone(),
            data: self.data.clone(),
        })
    }
}

/// Learned RMI (Recursive Model Index) engine
#[derive(Debug)]
pub struct LearnedRMIEngine {
    index: Option<RMIIndex<usize>>,
    data: Vec<(i64, Vec<u8>)>,
}

impl LearnedRMIEngine {
    pub fn new() -> Self {
        Self {
            index: None,
            data: Vec::new(),
        }
    }
}

impl IndexEngine for LearnedRMIEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        self.data = data.to_vec();

        // Create training data: key -> position
        let training_data: Vec<(i64, usize)> = data
            .iter()
            .enumerate()
            .map(|(pos, (key, _))| (*key, pos))
            .collect();

        self.index = Some(RMIIndex::train(training_data)?);
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        if let Some(ref index) = self.index {
            index.get(&key).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn search(&self, key: i64) -> Option<usize> {
        if let Some(ref index) = self.index {
            index.get(&key).copied()
        } else {
            self.data.binary_search_by_key(&key, |(k, _)| *k).ok()
        }
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        if let Some(ref index) = self.index {
            index.range(&start, &end).into_iter().collect()
        } else {
            self.data
                .iter()
                .filter(|(k, _)| *k >= start && *k <= end)
                .map(|(k, _)| *k)
                .collect()
        }
    }

    fn stats(&self) -> String {
        if let Some(ref index) = self.index {
            format!("LearnedRMI: {} keys, {} leaf models, max_error={}",
                self.data.len(), index.num_leaf_models(), index.max_error())
        } else {
            format!("LearnedRMI: untrained")
        }
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        Box::new(Self {
            index: self.index.clone(),
            data: self.data.clone(),
        })
    }
}

/// Traditional B-tree index engine (fallback)
#[derive(Debug)]
pub struct BTreeEngine {
    tree: RwLock<BTreeMap<i64, usize>>,
}

impl BTreeEngine {
    pub fn new() -> Self {
        Self {
            tree: RwLock::new(BTreeMap::new()),
        }
    }
}

impl IndexEngine for BTreeEngine {
    fn train(&mut self, data: &[(i64, Vec<u8>)]) -> Result<()> {
        let mut tree = self.tree.write().unwrap();
        tree.clear();

        for (pos, (key, _)) in data.iter().enumerate() {
            tree.insert(*key, pos);
        }
        Ok(())
    }

    fn predict(&self, key: i64) -> usize {
        // B-tree doesn't predict, just searches
        self.search(key).unwrap_or(0)
    }

    fn search(&self, key: i64) -> Option<usize> {
        let tree = self.tree.read().unwrap();
        tree.get(&key).copied()
    }

    fn range(&self, start: i64, end: i64) -> Vec<i64> {
        let tree = self.tree.read().unwrap();
        tree.range(start..=end).map(|(k, _)| *k).collect()
    }

    fn stats(&self) -> String {
        let tree = self.tree.read().unwrap();
        format!("BTree: {} keys", tree.len())
    }

    fn clone_box(&self) -> Box<dyn IndexEngine> {
        let tree = self.tree.read().unwrap();
        Box::new(Self {
            tree: RwLock::new(tree.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> Vec<(i64, Vec<u8>)> {
        (0..1000)
            .map(|i| (i * 2, format!("value_{}", i).into_bytes()))
            .collect()
    }

    #[test]
    fn test_learned_linear_engine() {
        let mut engine = LearnedLinearEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }

    #[test]
    fn test_learned_rmi_engine() {
        let mut engine = LearnedRMIEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }

    #[test]
    fn test_btree_engine() {
        let mut engine = BTreeEngine::new();
        let data = test_data();

        engine.train(&data).unwrap();

        // Test search
        assert_eq!(engine.search(100), Some(50));
        assert_eq!(engine.search(101), None);

        // Test range
        let range = engine.range(100, 200);
        assert_eq!(range.len(), 51);
    }

    #[test]
    fn test_engine_comparison() {
        let data = test_data();
        let mut engines: Vec<Box<dyn IndexEngine>> = vec![
            Box::new(LearnedLinearEngine::new()),
            Box::new(LearnedRMIEngine::new()),
            Box::new(BTreeEngine::new()),
        ];

        // Train all engines
        for engine in &mut engines {
            engine.train(&data).unwrap();
        }

        // All should return same results
        for engine in &engines {
            assert_eq!(engine.search(100), Some(50));
            assert_eq!(engine.search(999), None);
            assert_eq!(engine.range(0, 10).len(), 6);
        }
    }
}