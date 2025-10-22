//! PostgreSQL wire protocol server for OmenDB

use super::auth::OmenDbAuthSource;
use super::handlers::OmenDbHandlerFactory;
use crate::connection_pool::{ConnectionPool, PoolConfig};
use crate::metrics;
use datafusion::prelude::*;
use pgwire::tokio::process_socket;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// PostgreSQL wire protocol server
pub struct PostgresServer {
    /// Listen address (default: 127.0.0.1:5432)
    addr: String,

    /// Handler factory (contains query handlers and auth)
    factory: Arc<OmenDbHandlerFactory>,

    /// Connection pool for managing concurrent connections
    pool: Arc<ConnectionPool>,
}

impl PostgresServer {
    /// Create a new PostgreSQL server with default address (no authentication)
    pub fn new(ctx: SessionContext) -> Self {
        let ctx = Arc::new(RwLock::new(ctx));
        let factory = Arc::new(OmenDbHandlerFactory::new(ctx));
        let pool = Arc::new(ConnectionPool::new());
        Self {
            addr: "127.0.0.1:5432".to_string(),
            factory,
            pool,
        }
    }

    /// Create a new PostgreSQL server with custom address (no authentication)
    pub fn with_addr(addr: impl Into<String>, ctx: SessionContext) -> Self {
        let ctx = Arc::new(RwLock::new(ctx));
        let factory = Arc::new(OmenDbHandlerFactory::new(ctx));
        let pool = Arc::new(ConnectionPool::new());
        Self {
            addr: addr.into(),
            factory,
            pool,
        }
    }

    /// Create a new PostgreSQL server with SCRAM-SHA-256 authentication
    pub fn with_auth(
        addr: impl Into<String>,
        ctx: Arc<RwLock<SessionContext>>,
        auth_source: Arc<OmenDbAuthSource>,
    ) -> Self {
        let factory = Arc::new(OmenDbHandlerFactory::new_with_auth(ctx, auth_source));
        let pool = Arc::new(ConnectionPool::new());
        Self {
            addr: addr.into(),
            factory,
            pool,
        }
    }

    /// Create a new PostgreSQL server with custom connection pool configuration
    pub fn with_pool_config(
        addr: impl Into<String>,
        ctx: SessionContext,
        pool_config: PoolConfig,
    ) -> Self {
        let ctx = Arc::new(RwLock::new(ctx));
        let factory = Arc::new(OmenDbHandlerFactory::new(ctx));
        let pool = Arc::new(ConnectionPool::with_config(pool_config));
        Self {
            addr: addr.into(),
            factory,
            pool,
        }
    }

    /// Get connection pool statistics
    pub fn pool_stats(&self) -> crate::connection_pool::PoolStats {
        self.pool.stats()
    }

    /// Get current connection count
    pub fn connection_count(&self) -> usize {
        self.pool.connection_count()
    }

    /// Start serving PostgreSQL wire protocol connections
    pub async fn serve(self) -> anyhow::Result<()> {
        info!("Starting PostgreSQL server on {}", self.addr);
        info!(
            "Connection pool configured: max_connections={}",
            self.pool.max_connections()
        );

        let listener = TcpListener::bind(&self.addr).await?;
        info!("PostgreSQL server listening on {}", self.addr);

        // Spawn idle connection cleanup task
        let pool_cleanup = self.pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match pool_cleanup.cleanup_idle_connections() {
                    Ok(count) if count > 0 => {
                        info!("Cleaned up {} idle connections", count);
                    }
                    Err(e) => {
                        error!("Error during connection cleanup: {}", e);
                    }
                    _ => {}
                }
            }
        });

        loop {
            match listener.accept().await {
                Ok((socket, client_addr)) => {
                    info!("New connection attempt from {}", client_addr);

                    // Try to acquire connection slot from pool
                    let pool = self.pool.clone();
                    match pool.acquire() {
                        Ok(connection) => {
                            let conn_id = connection.id();
                            let active_count = pool.connection_count();

                            // Update metrics
                            metrics::set_active_connections(active_count as i64);

                            info!(
                                connection_id = conn_id,
                                client_addr = %client_addr,
                                active_connections = active_count,
                                "Connection acquired"
                            );

                            let factory_ref = self.factory.clone();
                            let pool_for_cleanup = pool.clone();

                            tokio::spawn(async move {
                                // Connection is held for the duration of this task
                                if let Err(e) = process_socket(socket, None, factory_ref).await {
                                    error!(
                                        connection_id = conn_id,
                                        client_addr = %client_addr,
                                        "Error processing socket: {}", e
                                    );
                                }
                                info!(
                                    connection_id = conn_id,
                                    client_addr = %client_addr,
                                    "Connection closed"
                                );

                                // Update metrics after connection closes
                                let remaining = pool_for_cleanup.connection_count();
                                metrics::set_active_connections(remaining as i64);

                                // Connection automatically released when dropped
                            });
                        }
                        Err(e) => {
                            warn!(
                                client_addr = %client_addr,
                                max_connections = pool.max_connections(),
                                current_connections = pool.connection_count(),
                                "Connection rejected: {}", e
                            );
                            // Connection is dropped, socket will be closed
                        }
                    }
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let ctx = SessionContext::new();
        let server = PostgresServer::new(ctx);
        assert_eq!(server.addr, "127.0.0.1:5432");
    }

    #[tokio::test]
    async fn test_server_with_custom_addr() {
        let ctx = SessionContext::new();
        let server = PostgresServer::with_addr("0.0.0.0:15432", ctx);
        assert_eq!(server.addr, "0.0.0.0:15432");
    }
}
