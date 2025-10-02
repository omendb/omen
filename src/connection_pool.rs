//! Connection pooling for OmenDB
//! Manages database connections with limits, timeouts, and resource tracking

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};

/// Connection pool configuration
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Maximum number of concurrent connections (default: 100)
    pub max_connections: usize,

    /// Connection idle timeout (default: 300 seconds / 5 minutes)
    pub idle_timeout: Duration,

    /// Maximum time to wait for a connection (default: 30 seconds)
    pub acquire_timeout: Duration,

    /// Enable connection validation before use (default: true)
    pub validate_connections: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            idle_timeout: Duration::from_secs(300),
            acquire_timeout: Duration::from_secs(30),
            validate_connections: true,
        }
    }
}

/// Connection metadata for tracking
#[derive(Debug, Clone)]
struct ConnectionMetadata {
    /// Connection ID
    id: u64,

    /// Time when connection was created
    created_at: Instant,

    /// Time of last activity
    last_active: Instant,

    /// Number of queries executed on this connection
    query_count: u64,

    /// Total bytes processed by this connection
    bytes_processed: u64,

    /// Current connection state
    state: ConnectionState,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    /// Connection is idle and available
    Idle,

    /// Connection is in use
    Active,

    /// Connection is being validated
    Validating,

    /// Connection is closed
    Closed,
}

/// Connection pool manager
pub struct ConnectionPool {
    /// Pool configuration
    config: PoolConfig,

    /// Active connections
    connections: Arc<Mutex<HashMap<u64, ConnectionMetadata>>>,

    /// Next connection ID
    next_id: Arc<Mutex<u64>>,

    /// Pool statistics
    stats: Arc<Mutex<PoolStats>>,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Total connections created
    pub total_created: u64,

    /// Total connections closed
    pub total_closed: u64,

    /// Current active connections
    pub active_connections: usize,

    /// Current idle connections
    pub idle_connections: usize,

    /// Total connection acquisitions
    pub total_acquisitions: u64,

    /// Total connection releases
    pub total_releases: u64,

    /// Total wait time for connections (milliseconds)
    pub total_wait_time_ms: u64,

    /// Number of times max connections was reached
    pub max_connections_reached: u64,

    /// Number of connections timed out (idle)
    pub idle_timeouts: u64,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_created: 0,
            total_closed: 0,
            active_connections: 0,
            idle_connections: 0,
            total_acquisitions: 0,
            total_releases: 0,
            total_wait_time_ms: 0,
            max_connections_reached: 0,
            idle_timeouts: 0,
        }
    }
}

impl ConnectionPool {
    /// Create new connection pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create new connection pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(PoolStats::default())),
        }
    }

    /// Acquire a connection from the pool
    #[instrument(skip(self))]
    pub fn acquire(&self) -> Result<Connection> {
        let start_time = Instant::now();
        let deadline = start_time + self.config.acquire_timeout;

        debug!("Acquiring connection from pool");

        loop {
            // Check if we've exceeded acquire timeout
            if Instant::now() > deadline {
                warn!(
                    timeout_secs = self.config.acquire_timeout.as_secs(),
                    "Connection acquire timeout"
                );
                return Err(anyhow!(
                    "Connection acquire timeout ({} seconds)",
                    self.config.acquire_timeout.as_secs()
                ));
            }

            let mut connections = self
                .connections
                .lock()
                .map_err(|e| anyhow!("Connection pool mutex poisoned: {}", e))?;
            let mut stats = self
                .stats
                .lock()
                .map_err(|e| anyhow!("Stats mutex poisoned: {}", e))?;

            // Try to find an idle connection
            if let Some((id, metadata)) = connections
                .iter_mut()
                .find(|(_, m)| m.state == ConnectionState::Idle)
            {
                // Reuse idle connection
                metadata.state = ConnectionState::Active;
                metadata.last_active = Instant::now();

                stats.total_acquisitions += 1;
                stats.active_connections += 1;
                stats.idle_connections = stats.idle_connections.saturating_sub(1);

                debug!(
                    connection_id = *id,
                    active = stats.active_connections,
                    idle = stats.idle_connections,
                    "Reused idle connection"
                );

                let connection = Connection {
                    id: *id,
                    pool: Arc::new(self.clone_pool_handle()),
                    acquired_at: Instant::now(),
                };

                drop(connections);
                drop(stats);

                return Ok(connection);
            }

            // Check if we can create a new connection
            let total_connections = connections.len();
            if total_connections < self.config.max_connections {
                // Create new connection
                let id = {
                    let mut next_id = self
                        .next_id
                        .lock()
                        .map_err(|e| anyhow!("ID generator mutex poisoned: {}", e))?;
                    let id = *next_id;
                    *next_id += 1;
                    id
                };

                let metadata = ConnectionMetadata {
                    id,
                    created_at: Instant::now(),
                    last_active: Instant::now(),
                    query_count: 0,
                    bytes_processed: 0,
                    state: ConnectionState::Active,
                };

                connections.insert(id, metadata);

                stats.total_created += 1;
                stats.total_acquisitions += 1;
                stats.active_connections += 1;

                info!(
                    connection_id = id,
                    total_connections = total_connections + 1,
                    max_connections = self.config.max_connections,
                    "Created new connection"
                );

                let connection = Connection {
                    id,
                    pool: Arc::new(self.clone_pool_handle()),
                    acquired_at: Instant::now(),
                };

                drop(connections);
                drop(stats);

                return Ok(connection);
            }

            // Pool is full, record this
            stats.max_connections_reached += 1;
            warn!(
                max_connections = self.config.max_connections,
                active = stats.active_connections,
                "Connection pool at capacity, waiting"
            );

            drop(connections);
            drop(stats);

            // Wait a bit before retrying
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Release a connection back to the pool
    fn release(&self, connection_id: u64) {
        let mut connections = match self.connections.lock() {
            Ok(guard) => guard,
            Err(e) => {
                warn!("Connection pool mutex poisoned during release: {}", e);
                return;
            }
        };
        let mut stats = match self.stats.lock() {
            Ok(guard) => guard,
            Err(e) => {
                warn!("Stats mutex poisoned during release: {}", e);
                return;
            }
        };

        if let Some(metadata) = connections.get_mut(&connection_id) {
            metadata.state = ConnectionState::Idle;
            metadata.last_active = Instant::now();

            stats.total_releases += 1;
            stats.active_connections = stats.active_connections.saturating_sub(1);
            stats.idle_connections += 1;
        }
    }

    /// Clean up idle connections that have exceeded timeout
    #[instrument(skip(self))]
    pub fn cleanup_idle_connections(&self) -> Result<usize> {
        let mut connections = self
            .connections
            .lock()
            .map_err(|e| anyhow!("Connection pool mutex poisoned: {}", e))?;
        let mut stats = self
            .stats
            .lock()
            .map_err(|e| anyhow!("Stats mutex poisoned: {}", e))?;

        let now = Instant::now();
        let mut removed = 0;

        let to_remove: Vec<u64> = connections
            .iter()
            .filter(|(_, metadata)| {
                metadata.state == ConnectionState::Idle
                    && now.duration_since(metadata.last_active) > self.config.idle_timeout
            })
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            connections.remove(&id);
            removed += 1;
            stats.total_closed += 1;
            stats.idle_connections = stats.idle_connections.saturating_sub(1);
            stats.idle_timeouts += 1;
            debug!(connection_id = id, "Cleaned up idle connection");
        }

        if removed > 0 {
            info!(
                removed_count = removed,
                remaining = connections.len(),
                "Cleaned up idle connections"
            );
        }

        Ok(removed)
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_else(|e| {
                warn!("Stats mutex poisoned, returning default: {}", e);
                PoolStats::default()
            })
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.connections
            .lock()
            .map(|guard| guard.len())
            .unwrap_or_else(|e| {
                warn!("Connection pool mutex poisoned, returning 0: {}", e);
                0
            })
    }

    /// Get maximum connections allowed
    pub fn max_connections(&self) -> usize {
        self.config.max_connections
    }

    /// Close all connections and shut down the pool
    pub fn shutdown(&self) -> Result<()> {
        let mut connections = self
            .connections
            .lock()
            .map_err(|e| anyhow!("Connection pool mutex poisoned during shutdown: {}", e))?;
        let mut stats = self
            .stats
            .lock()
            .map_err(|e| anyhow!("Stats mutex poisoned during shutdown: {}", e))?;

        let count = connections.len();
        connections.clear();

        stats.total_closed += count as u64;
        stats.active_connections = 0;
        stats.idle_connections = 0;

        Ok(())
    }

    /// Clone pool handle for connection
    fn clone_pool_handle(&self) -> ConnectionPool {
        Self {
            config: self.config.clone(),
            connections: Arc::clone(&self.connections),
            next_id: Arc::clone(&self.next_id),
            stats: Arc::clone(&self.stats),
        }
    }

    /// Record query execution for a connection
    pub fn record_query(&self, connection_id: u64, bytes: u64) {
        let mut connections = match self.connections.lock() {
            Ok(guard) => guard,
            Err(e) => {
                warn!("Connection pool mutex poisoned during record_query: {}", e);
                return;
            }
        };
        if let Some(metadata) = connections.get_mut(&connection_id) {
            metadata.query_count += 1;
            metadata.bytes_processed += bytes;
            metadata.last_active = Instant::now();
        }
    }
}

/// Connection handle
pub struct Connection {
    id: u64,
    pool: Arc<ConnectionPool>,
    acquired_at: Instant,
}

impl Connection {
    /// Get connection ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get connection age (time since acquired)
    pub fn age(&self) -> Duration {
        self.acquired_at.elapsed()
    }

    /// Record query execution on this connection
    pub fn record_query(&self, bytes: u64) {
        self.pool.record_query(self.id, bytes);
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Return connection to pool when dropped
        self.pool.release(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool = ConnectionPool::new();
        assert_eq!(pool.connection_count(), 0);
        assert_eq!(pool.max_connections(), 100);
    }

    #[test]
    fn test_acquire_and_release() {
        let pool = ConnectionPool::new();

        let conn = pool.acquire().unwrap();
        assert_eq!(pool.connection_count(), 1);

        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.active_connections, 1);

        drop(conn); // Release connection

        let stats = pool.stats();
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 1);
    }

    #[test]
    fn test_connection_reuse() {
        let pool = ConnectionPool::new();

        let conn1 = pool.acquire().unwrap();
        let id1 = conn1.id();
        drop(conn1);

        let conn2 = pool.acquire().unwrap();
        let id2 = conn2.id();

        // Should reuse the same connection
        assert_eq!(id1, id2);
        assert_eq!(pool.connection_count(), 1);
    }

    #[test]
    fn test_max_connections() {
        let config = PoolConfig {
            max_connections: 3,
            acquire_timeout: Duration::from_millis(500),
            ..Default::default()
        };

        let pool = ConnectionPool::with_config(config);

        let conn1 = pool.acquire().unwrap();
        let conn2 = pool.acquire().unwrap();
        let conn3 = pool.acquire().unwrap();

        assert_eq!(pool.connection_count(), 3);

        // Fourth connection should timeout
        let start = Instant::now();
        let result = pool.acquire();
        assert!(result.is_err());
        assert!(start.elapsed() >= Duration::from_millis(500));

        drop(conn1); // Release one connection

        // Now should succeed
        let conn4 = pool.acquire().unwrap();
        assert!(conn4.id() < 4);
    }

    #[test]
    fn test_idle_cleanup() {
        let config = PoolConfig {
            max_connections: 10,
            idle_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let pool = ConnectionPool::with_config(config);

        // Create and release 3 connections
        {
            let _c1 = pool.acquire().unwrap();
            let _c2 = pool.acquire().unwrap();
            let _c3 = pool.acquire().unwrap();
        }

        assert_eq!(pool.connection_count(), 3);

        // Wait for idle timeout
        std::thread::sleep(Duration::from_millis(150));

        // Cleanup should remove all idle connections
        let removed = pool.cleanup_idle_connections().unwrap();
        assert_eq!(removed, 3);
        assert_eq!(pool.connection_count(), 0);
    }

    #[test]
    fn test_pool_stats() {
        let pool = ConnectionPool::new();

        let conn1 = pool.acquire().unwrap();
        let conn2 = pool.acquire().unwrap();

        let stats = pool.stats();
        assert_eq!(stats.total_created, 2);
        assert_eq!(stats.active_connections, 2);
        assert_eq!(stats.total_acquisitions, 2);

        drop(conn1);
        drop(conn2);

        let stats = pool.stats();
        assert_eq!(stats.total_releases, 2);
        assert_eq!(stats.active_connections, 0);
        assert_eq!(stats.idle_connections, 2);
    }

    #[test]
    fn test_shutdown() {
        let pool = ConnectionPool::new();

        let _conn1 = pool.acquire().unwrap();
        let _conn2 = pool.acquire().unwrap();

        assert_eq!(pool.connection_count(), 2);

        pool.shutdown().unwrap();

        assert_eq!(pool.connection_count(), 0);
    }

    #[test]
    fn test_query_recording() {
        let pool = ConnectionPool::new();

        let conn = pool.acquire().unwrap();
        let id = conn.id();

        conn.record_query(1024); // 1KB query

        pool.record_query(id, 2048); // Another 2KB

        // Connection metadata should be updated
        let connections = pool.connections.lock().unwrap();
        let metadata = connections.get(&id).unwrap();
        assert_eq!(metadata.query_count, 2);
        assert_eq!(metadata.bytes_processed, 3072);
    }
}
