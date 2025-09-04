//! LSM buffer for write optimization
//!
//! Absorbs write bursts without blocking readers (Bf-Tree inspired).

use anyhow::Result;
use std::collections::BTreeMap;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct LSMBuffer {
    buffer: Arc<RwLock<BTreeMap<Vec<u8>, Vec<u8>>>>,
    capacity: usize,
}

impl LSMBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(BTreeMap::new())),
            capacity,
        }
    }
    
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.buffer.read().get(key).cloned()
    }
    
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<bool> {
        let mut buffer = self.buffer.write();
        buffer.insert(key, value);
        
        // Return true if buffer is full and needs flushing
        Ok(buffer.len() > self.capacity)
    }
    
    pub fn flush(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut buffer = self.buffer.write();
        let entries: Vec<_> = buffer.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        buffer.clear();
        entries
    }
}