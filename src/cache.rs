use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use crate::value::Value;
use crate::row::Row;

/// Large LRU cache for hot data (1-10GB configurable)
///
/// Addresses the 80x in-memory vs disk performance gap (HN validated)
/// Target: Reduce RocksDB overhead from 77% to <30%
pub struct RowCache {
    /// LRU cache: key -> row
    cache: Arc<RwLock<LruCache<Value, Row>>>,

    /// Cache statistics
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,

    /// Configuration
    max_size: usize,  // Max entries (not bytes)
}

impl RowCache {
    /// Create a new RowCache with specified max size
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of entries in the cache
    ///
    /// # Default Sizing
    /// - 100K entries ≈ 1GB for 10KB rows
    /// - Configurable via OMENDB_CACHE_SIZE environment variable
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(max_size).expect("Cache size must be > 0"))
            )),
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_size,
        }
    }

    /// Create cache with size from environment variable or default
    pub fn with_default_size() -> Self {
        let size = default_cache_size();
        Self::new(size)
    }

    /// Get from cache (fast path - 80x faster than disk)
    ///
    /// Returns Some(row) on cache hit, None on cache miss
    pub fn get(&self, key: &Value) -> Option<Row> {
        let mut cache = self.cache.write().unwrap();
        if let Some(row) = cache.get(key) {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(row.clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Insert into cache
    ///
    /// If cache is full, LRU entry will be evicted automatically
    pub fn insert(&self, key: Value, row: Row) {
        let mut cache = self.cache.write().unwrap();
        cache.put(key, row);
    }

    /// Invalidate on update/delete
    ///
    /// Call this when a row is updated or deleted to maintain consistency
    pub fn invalidate(&self, key: &Value) {
        let mut cache = self.cache.write().unwrap();
        cache.pop(key);
    }

    /// Clear all entries from cache
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// Get current cache size (number of entries)
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read().unwrap();
        cache.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            hit_rate,
            size: self.len(),
            capacity: self.max_size,
        }
    }

    /// Reset cache statistics (useful for benchmarking)
    pub fn reset_stats(&self) {
        self.hits.store(0, std::sync::atomic::Ordering::Relaxed);
        self.misses.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Clone for RowCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            hits: Arc::clone(&self.hits),
            misses: Arc::clone(&self.misses),
            max_size: self.max_size,
        }
    }
}

impl std::fmt::Debug for RowCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats();
        f.debug_struct("RowCache")
            .field("max_size", &self.max_size)
            .field("current_size", &stats.size)
            .field("hits", &stats.hits)
            .field("misses", &stats.misses)
            .field("hit_rate", &stats.hit_rate)
            .finish()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,      // Current entries
    pub capacity: usize,  // Max entries
}

impl CacheStats {
    /// Get total requests (hits + misses)
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Get cache utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.capacity > 0 {
            (self.size as f64 / self.capacity as f64) * 100.0
        } else {
            0.0
        }
    }
}

/// Default cache size: 100K entries (estimate ~1GB for 10KB rows)
/// Configurable via OMENDB_CACHE_SIZE environment variable
fn default_cache_size() -> usize {
    std::env::var("OMENDB_CACHE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000)  // 100K entries default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit() {
        let cache = RowCache::new(1000);
        let key = Value::Int64(1);
        let row = Row::new(vec![Value::Int64(1), Value::Text("test".to_string())]);

        cache.insert(key.clone(), row.clone());

        let result = cache.get(&key);
        assert!(result.is_some());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 100.0);
    }

    #[test]
    fn test_cache_miss() {
        let cache = RowCache::new(1000);
        let key = Value::Int64(1);

        let result = cache.get(&key);
        assert!(result.is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = RowCache::new(2);  // Max 2 entries

        cache.insert(Value::Int64(1), Row::new(vec![Value::Int64(1)]));
        cache.insert(Value::Int64(2), Row::new(vec![Value::Int64(2)]));
        cache.insert(Value::Int64(3), Row::new(vec![Value::Int64(3)]));

        // Entry 1 should be evicted (LRU)
        assert!(cache.get(&Value::Int64(1)).is_none());
        assert!(cache.get(&Value::Int64(2)).is_some());
        assert!(cache.get(&Value::Int64(3)).is_some());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = RowCache::new(1000);
        let key = Value::Int64(1);
        let row = Row::new(vec![Value::Int64(1)]);

        cache.insert(key.clone(), row);
        assert!(cache.get(&key).is_some());

        cache.invalidate(&key);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = RowCache::new(1000);

        cache.insert(Value::Int64(1), Row::new(vec![Value::Int64(1)]));
        cache.insert(Value::Int64(2), Row::new(vec![Value::Int64(2)]));

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let cache = RowCache::new(100);

        // Insert 10 entries
        for i in 0..10 {
            cache.insert(Value::Int64(i), Row::new(vec![Value::Int64(i)]));
        }

        // 5 hits
        for i in 0..5 {
            assert!(cache.get(&Value::Int64(i)).is_some());
        }

        // 3 misses
        for i in 10..13 {
            assert!(cache.get(&Value::Int64(i)).is_none());
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 5);
        assert_eq!(stats.misses, 3);
        assert_eq!(stats.total_requests(), 8);
        assert_eq!(stats.size, 10);
        assert_eq!(stats.capacity, 100);

        // Hit rate: 5/8 = 62.5%
        assert!((stats.hit_rate - 62.5).abs() < 0.1);

        // Utilization: 10/100 = 10%
        assert!((stats.utilization() - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_cache_reset_stats() {
        let cache = RowCache::new(100);

        cache.insert(Value::Int64(1), Row::new(vec![Value::Int64(1)]));
        cache.get(&Value::Int64(1));
        cache.get(&Value::Int64(2));

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);

        cache.reset_stats();

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_clone() {
        let cache1 = RowCache::new(100);
        cache1.insert(Value::Int64(1), Row::new(vec![Value::Int64(1)]));

        let cache2 = cache1.clone();

        // Both caches should share the same underlying data
        assert!(cache2.get(&Value::Int64(1)).is_some());

        // Stats should be shared too
        assert_eq!(cache1.stats().hits, 1);
        assert_eq!(cache2.stats().hits, 1);
    }

    #[test]
    fn test_default_cache_size() {
        // Without env var, should default to 100K
        let size = default_cache_size();
        assert_eq!(size, 100_000);
    }

    #[test]
    fn test_with_default_size() {
        let cache = RowCache::with_default_size();
        let stats = cache.stats();
        assert_eq!(stats.capacity, 100_000);
    }
}
