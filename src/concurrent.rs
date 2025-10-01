//! Concurrency support for production OmenDB with durability
//! Thread-safe wrappers with read-write locking and WAL integration

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::Path;
use anyhow::{Result, Context};
use crate::index::RecursiveModelIndex;
use crate::storage::ArrowStorage;

/// Thread-safe OmenDB with concurrent access support
pub struct ConcurrentOmenDB {
    /// Protected learned index
    index: Arc<RwLock<RecursiveModelIndex>>,

    /// Protected storage
    storage: Arc<RwLock<ArrowStorage>>,

    /// Connection counter for monitoring
    active_connections: AtomicUsize,

    /// Query counter for metrics
    query_count: AtomicUsize,

    /// Write counter for metrics
    write_count: AtomicUsize,
}

impl ConcurrentOmenDB {
    /// Create new concurrent database instance
    pub fn new(expected_size: usize) -> Self {
        Self {
            index: Arc::new(RwLock::new(RecursiveModelIndex::new(expected_size))),
            storage: Arc::new(RwLock::new(ArrowStorage::new())),
            active_connections: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
            write_count: AtomicUsize::new(0),
        }
    }

    /// Create with persistence and WAL support
    pub fn with_persistence<P: AsRef<Path>>(expected_size: usize, data_dir: P) -> Result<Self> {
        let storage = ArrowStorage::with_persistence(data_dir)?;

        Ok(Self {
            index: Arc::new(RwLock::new(RecursiveModelIndex::new(expected_size))),
            storage: Arc::new(RwLock::new(storage)),
            active_connections: AtomicUsize::new(0),
            query_count: AtomicUsize::new(0),
            write_count: AtomicUsize::new(0),
        })
    }

    /// Get read access to index
    pub fn read_index(&self) -> Result<RwLockReadGuard<RecursiveModelIndex>> {
        self.index.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))
    }

    /// Get write access to index
    pub fn write_index(&self) -> Result<RwLockWriteGuard<RecursiveModelIndex>> {
        self.index.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))
    }

    /// Get read access to storage
    pub fn read_storage(&self) -> Result<RwLockReadGuard<ArrowStorage>> {
        self.storage.read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage read lock: {}", e))
    }

    /// Get write access to storage
    pub fn write_storage(&self) -> Result<RwLockWriteGuard<ArrowStorage>> {
        self.storage.write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire storage write lock: {}", e))
    }

    /// Thread-safe insert with WAL support
    pub fn insert(&self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        self.write_count.fetch_add(1, Ordering::Relaxed);

        // First update storage (includes WAL write)
        {
            let mut storage = self.write_storage()?;
            storage.insert(timestamp, value, series_id)?;
        }

        // Then update index (separate lock to minimize contention)
        {
            let mut index = self.write_index()?;
            index.add_key(timestamp);
        }

        Ok(())
    }

    /// Sync data to disk
    pub fn sync(&self) -> Result<()> {
        let storage = self.read_storage()?;
        storage.sync()
    }

    /// Thread-safe point query
    pub fn search(&self, key: i64) -> Result<Option<usize>> {
        self.query_count.fetch_add(1, Ordering::Relaxed);

        let index = self.read_index()?;
        Ok(index.search(key))
    }

    /// Thread-safe range query
    pub fn range_search(&self, start: i64, end: i64) -> Result<Vec<usize>> {
        self.query_count.fetch_add(1, Ordering::Relaxed);

        let index = self.read_index()?;
        Ok(index.range_search(start, end))
    }

    /// Get metrics
    pub fn metrics(&self) -> Metrics {
        Metrics {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            query_count: self.query_count.load(Ordering::Relaxed),
            write_count: self.write_count.load(Ordering::Relaxed),
        }
    }

    /// Register new connection
    pub fn connect(&self) -> ConnectionGuard {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        ConnectionGuard { db: self }
    }
}

/// RAII guard for connection tracking
pub struct ConnectionGuard<'a> {
    db: &'a ConcurrentOmenDB,
}

impl<'a> Drop for ConnectionGuard<'a> {
    fn drop(&mut self) {
        self.db.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Database metrics
#[derive(Debug, Clone)]
pub struct Metrics {
    pub active_connections: usize,
    pub query_count: usize,
    pub write_count: usize,
}

/// Connection pool for managing database connections
pub struct ConnectionPool {
    db: Arc<ConcurrentOmenDB>,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(db: Arc<ConcurrentOmenDB>, max_connections: usize) -> Self {
        Self {
            db,
            max_connections,
        }
    }

    /// Get connection from pool
    pub fn get_connection(&self) -> Result<PooledConnection<'_>> {
        let metrics = self.db.metrics();

        if metrics.active_connections >= self.max_connections {
            return Err(anyhow::anyhow!(
                "Connection pool exhausted: {}/{} connections in use",
                metrics.active_connections,
                self.max_connections
            ));
        }

        Ok(PooledConnection {
            db: Arc::clone(&self.db),
            _guard: self.db.connect(),
        })
    }
}

/// Pooled connection wrapper
pub struct PooledConnection<'a> {
    db: Arc<ConcurrentOmenDB>,
    _guard: ConnectionGuard<'a>,
}

impl<'a> PooledConnection<'a> {
    pub fn search(&self, key: i64) -> Result<Option<usize>> {
        self.db.search(key)
    }

    pub fn insert(&self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        self.db.insert(timestamp, value, series_id)
    }

    pub fn range_search(&self, start: i64, end: i64) -> Result<Vec<usize>> {
        self.db.range_search(start, end)
    }
}

/// Lock-free metrics collector using atomics
pub struct MetricsCollector {
    pub total_queries: AtomicUsize,
    pub total_inserts: AtomicUsize,
    pub failed_queries: AtomicUsize,
    pub failed_inserts: AtomicUsize,
    pub avg_query_time_ns: AtomicUsize,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            total_queries: AtomicUsize::new(0),
            total_inserts: AtomicUsize::new(0),
            failed_queries: AtomicUsize::new(0),
            failed_inserts: AtomicUsize::new(0),
            avg_query_time_ns: AtomicUsize::new(0),
        }
    }

    pub fn record_query(&self, success: bool, duration_ns: usize) {
        self.total_queries.fetch_add(1, Ordering::Relaxed);

        if !success {
            self.failed_queries.fetch_add(1, Ordering::Relaxed);
        }

        // Update rolling average (simplified)
        let current_avg = self.avg_query_time_ns.load(Ordering::Relaxed);
        let new_avg = (current_avg * 99 + duration_ns) / 100;
        self.avg_query_time_ns.store(new_avg, Ordering::Relaxed);
    }

    pub fn record_insert(&self, success: bool) {
        self.total_inserts.fetch_add(1, Ordering::Relaxed);

        if !success {
            self.failed_inserts.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn get_stats(&self) -> Stats {
        Stats {
            total_queries: self.total_queries.load(Ordering::Relaxed),
            total_inserts: self.total_inserts.load(Ordering::Relaxed),
            failed_queries: self.failed_queries.load(Ordering::Relaxed),
            failed_inserts: self.failed_inserts.load(Ordering::Relaxed),
            avg_query_time_ns: self.avg_query_time_ns.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub total_queries: usize,
    pub total_inserts: usize,
    pub failed_queries: usize,
    pub failed_inserts: usize,
    pub avg_query_time_ns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_access() {
        let db = Arc::new(ConcurrentOmenDB::new(10000));

        // Initialize with data
        for i in 0..1000 {
            db.insert(i as i64 * 10, i as f64, 1).unwrap();
        }

        // Train index
        {
            let data: Vec<(i64, usize)> = (0..1000)
                .map(|i| (i as i64 * 10, i))
                .collect();
            let mut index = db.write_index().unwrap();
            index.train(data);
        }

        // Spawn multiple readers
        let mut handles = vec![];

        for _ in 0..10 {
            let db_clone = Arc::clone(&db);

            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let result = db_clone.search(i as i64 * 10);
                    assert!(result.is_ok());
                }
            });

            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Check metrics
        let metrics = db.metrics();
        assert!(metrics.query_count >= 1000);
    }

    #[test]
    fn test_connection_pool() {
        let db = Arc::new(ConcurrentOmenDB::new(1000));
        let pool = ConnectionPool::new(db, 10);

        // Get connections
        let conn1 = pool.get_connection();
        assert!(conn1.is_ok());

        let conn2 = pool.get_connection();
        assert!(conn2.is_ok());
    }

    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        collector.record_query(true, 100);
        collector.record_query(false, 200);
        collector.record_insert(true);

        let stats = collector.get_stats();
        assert_eq!(stats.total_queries, 2);
        assert_eq!(stats.failed_queries, 1);
        assert_eq!(stats.total_inserts, 1);
    }
}