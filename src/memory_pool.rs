//! Memory pool optimization for OmenDB
//!
//! Reduces allocation overhead and memory fragmentation through:
//! 1. Object pooling for frequently allocated structures
//! 2. Pre-allocated buffer reuse
//! 3. Optimized memory layout for ALEX nodes
//! 4. Cache-friendly memory alignment

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Memory pool for reusing Vec<u8> buffers
pub struct ByteBufferPool {
    /// Pool of reusable buffers organized by size class
    pools: Vec<Mutex<VecDeque<Vec<u8>>>>,
    /// Size classes (powers of 2)
    size_classes: Vec<usize>,
}

impl ByteBufferPool {
    /// Create new buffer pool
    pub fn new() -> Self {
        // Size classes: 64B, 128B, 256B, 512B, 1KB, 2KB, 4KB, 8KB
        let size_classes = vec![64, 128, 256, 512, 1024, 2048, 4096, 8192];
        let pools = size_classes
            .iter()
            .map(|_| Mutex::new(VecDeque::new()))
            .collect();

        Self {
            pools,
            size_classes,
        }
    }

    /// Get buffer of at least `size` bytes
    pub fn get_buffer(&self, size: usize) -> Vec<u8> {
        // Find appropriate size class
        if let Some(class_idx) = self.find_size_class(size) {
            if let Ok(mut pool) = self.pools[class_idx].try_lock() {
                if let Some(mut buffer) = pool.pop_front() {
                    buffer.clear();
                    buffer.reserve(size);
                    return buffer;
                }
            }
        }

        // No pooled buffer available, allocate new one
        Vec::with_capacity(size.next_power_of_two().max(64))
    }

    /// Return buffer to pool for reuse
    pub fn return_buffer(&self, buffer: Vec<u8>) {
        let capacity = buffer.capacity();

        if let Some(class_idx) = self.find_size_class(capacity) {
            if let Ok(mut pool) = self.pools[class_idx].try_lock() {
                if pool.len() < 16 { // Limit pool size
                    pool.push_back(buffer);
                }
            }
        }
        // If can't return to pool, just drop (normal deallocation)
    }

    fn find_size_class(&self, size: usize) -> Option<usize> {
        self.size_classes
            .iter()
            .position(|&class_size| size <= class_size)
    }
}

/// Memory-optimized vector that reuses capacity
pub struct PooledVec<T> {
    data: Vec<T>,
    pool: Option<Arc<VecPool<T>>>,
}

impl<T> PooledVec<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pool: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            pool: None,
        }
    }

    pub fn from_pool(pool: Arc<VecPool<T>>) -> Self {
        let data = pool.get_vec();
        Self {
            data,
            pool: Some(pool),
        }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn into_inner(mut self) -> Vec<T> {
        std::mem::take(&mut self.data)
    }
}

impl<T> Drop for PooledVec<T> {
    fn drop(&mut self) {
        if let Some(pool) = &self.pool {
            let mut vec = std::mem::take(&mut self.data);
            vec.clear();
            pool.return_vec(vec);
        }
    }
}

/// Pool for reusing Vec<T> allocations
pub struct VecPool<T> {
    pool: Mutex<VecDeque<Vec<T>>>,
    max_pool_size: usize,
}

impl<T> VecPool<T> {
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            pool: Mutex::new(VecDeque::new()),
            max_pool_size,
        }
    }

    pub fn get_vec(&self) -> Vec<T> {
        if let Ok(mut pool) = self.pool.try_lock() {
            if let Some(vec) = pool.pop_front() {
                return vec;
            }
        }
        Vec::new()
    }

    pub fn return_vec(&self, vec: Vec<T>) {
        if let Ok(mut pool) = self.pool.try_lock() {
            if pool.len() < self.max_pool_size {
                pool.push_back(vec);
            }
        }
    }
}

/// Cache-optimized key-value pair storage
#[repr(C)]
#[derive(Clone)]
pub struct OptimizedKeyValue {
    /// 8-byte aligned key
    pub key: i64,
    /// Length of value data
    pub value_len: u32,
    /// Inline storage for small values (<= 28 bytes)
    /// This avoids heap allocation for most values
    pub inline_data: [u8; 28],
    /// Heap storage for large values (> 28 bytes)
    pub heap_data: Option<Box<[u8]>>,
}

impl OptimizedKeyValue {
    /// Create new key-value pair with optimized storage
    pub fn new(key: i64, value: Vec<u8>) -> Self {
        let value_len = value.len() as u32;

        if value.len() <= 28 {
            // Use inline storage
            let mut inline_data = [0u8; 28];
            inline_data[..value.len()].copy_from_slice(&value);

            Self {
                key,
                value_len,
                inline_data,
                heap_data: None,
            }
        } else {
            // Use heap storage with Box for efficient allocation
            Self {
                key,
                value_len,
                inline_data: [0u8; 28],
                heap_data: Some(value.into_boxed_slice()),
            }
        }
    }

    /// Get value data (zero-copy for large values)
    pub fn value(&self) -> &[u8] {
        if self.value_len <= 28 {
            &self.inline_data[..self.value_len as usize]
        } else if let Some(heap_data) = &self.heap_data {
            &heap_data[..]
        } else {
            &[]
        }
    }

    /// Get owned value data
    pub fn into_value(self) -> Vec<u8> {
        if self.value_len <= 28 {
            self.inline_data[..self.value_len as usize].to_vec()
        } else if let Some(heap_data) = self.heap_data {
            heap_data.into_vec()
        } else {
            Vec::new()
        }
    }

    /// Memory footprint in bytes
    pub fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>() +
        if self.value_len > 28 {
            self.heap_data.as_ref().map_or(0, |data| data.len())
        } else {
            0
        }
    }
}

/// Memory-efficient batch for bulk operations
pub struct OptimizedBatch {
    /// Pre-allocated storage for key-value pairs
    pairs: Vec<OptimizedKeyValue>,
    /// Buffer pool for temporary allocations
    buffer_pool: Arc<ByteBufferPool>,
}

impl OptimizedBatch {
    /// Create new batch with pre-allocated capacity
    pub fn with_capacity(capacity: usize, buffer_pool: Arc<ByteBufferPool>) -> Self {
        Self {
            pairs: Vec::with_capacity(capacity),
            buffer_pool,
        }
    }

    /// Add key-value pair to batch
    pub fn push(&mut self, key: i64, value: Vec<u8>) {
        self.pairs.push(OptimizedKeyValue::new(key, value));
    }

    /// Sort batch by key (in-place, cache-efficient)
    pub fn sort_by_key(&mut self) {
        self.pairs.sort_unstable_by_key(|kv| kv.key);
    }

    /// Get pairs for iteration
    pub fn pairs(&self) -> &[OptimizedKeyValue] {
        &self.pairs
    }

    /// Convert to legacy format for compatibility
    pub fn into_legacy_pairs(self) -> Vec<(i64, Vec<u8>)> {
        self.pairs
            .into_iter()
            .map(|kv| (kv.key, kv.into_value()))
            .collect()
    }

    /// Clear batch for reuse
    pub fn clear(&mut self) {
        self.pairs.clear();
    }

    /// Get temporary buffer from pool
    pub fn get_temp_buffer(&self, size: usize) -> Vec<u8> {
        self.buffer_pool.get_buffer(size)
    }

    /// Return temporary buffer to pool
    pub fn return_temp_buffer(&self, buffer: Vec<u8>) {
        self.buffer_pool.return_buffer(buffer);
    }
}

/// Global memory pools (lazy-initialized)
lazy_static::lazy_static! {
    static ref GLOBAL_BUFFER_POOL: Arc<ByteBufferPool> = Arc::new(ByteBufferPool::new());
    static ref GLOBAL_I64_POOL: Arc<VecPool<i64>> = Arc::new(VecPool::new(32));
    static ref GLOBAL_U8_POOL: Arc<VecPool<u8>> = Arc::new(VecPool::new(32));
}

/// Get global buffer pool
pub fn global_buffer_pool() -> Arc<ByteBufferPool> {
    GLOBAL_BUFFER_POOL.clone()
}

/// Get global i64 vector pool
pub fn global_i64_pool() -> Arc<VecPool<i64>> {
    GLOBAL_I64_POOL.clone()
}

/// Get global u8 vector pool
pub fn global_u8_pool() -> Arc<VecPool<u8>> {
    GLOBAL_U8_POOL.clone()
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total allocated bytes
    pub total_allocated: usize,
    /// Bytes in pools (reusable)
    pub pooled_bytes: usize,
    /// Number of active allocations
    pub active_allocations: usize,
    /// Cache hit rate for pools
    pub pool_hit_rate: f64,
}

impl MemoryStats {
    pub fn efficiency(&self) -> f64 {
        if self.total_allocated == 0 {
            1.0
        } else {
            1.0 - (self.pooled_bytes as f64 / self.total_allocated as f64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool() {
        let pool = ByteBufferPool::new();

        // Get buffer
        let mut buffer = pool.get_buffer(100);
        assert!(buffer.capacity() >= 100);

        // Use buffer
        buffer.extend_from_slice(b"test data");

        // Return to pool
        pool.return_buffer(buffer);

        // Get again - should reuse
        let buffer2 = pool.get_buffer(100);
        assert!(buffer2.is_empty()); // Should be cleared
    }

    #[test]
    fn test_optimized_key_value() {
        // Small value (inline)
        let kv1 = OptimizedKeyValue::new(42, b"small".to_vec());
        assert_eq!(kv1.key, 42);
        assert_eq!(kv1.value(), b"small");
        assert!(kv1.heap_data.is_none());

        // Large value (heap)
        let large_data = vec![0u8; 100];
        let kv2 = OptimizedKeyValue::new(123, large_data.clone());
        assert_eq!(kv2.key, 123);
        assert_eq!(kv2.value(), &large_data[..]);
        assert!(kv2.heap_data.is_some());
    }

    #[test]
    fn test_optimized_batch() {
        let pool = Arc::new(ByteBufferPool::new());
        let mut batch = OptimizedBatch::with_capacity(10, pool);

        batch.push(30, b"third".to_vec());
        batch.push(10, b"first".to_vec());
        batch.push(20, b"second".to_vec());

        batch.sort_by_key();

        let pairs = batch.pairs();
        assert_eq!(pairs[0].key, 10);
        assert_eq!(pairs[1].key, 20);
        assert_eq!(pairs[2].key, 30);
    }
}