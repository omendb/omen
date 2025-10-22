//! PostgreSQL wire protocol server for OmenDB

use super::auth::OmenDbAuthSource;
use super::handlers::OmenDbHandlerFactory;
use crate::connection_pool::{ConnectionPool, PoolConfig};
use crate::metrics;
use datafusion::prelude::*;
use pgwire::tokio::process_socket;
use rustls::pki_types::CertificateDer;
use rustls::ServerConfig as RustlsServerConfig;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info, warn};

/// PostgreSQL wire protocol server
pub struct PostgresServer {
    /// Listen address (default: 127.0.0.1:5432)
    addr: String,

    /// Handler factory (contains query handlers and auth)
    factory: Arc<OmenDbHandlerFactory>,

    /// Connection pool for managing concurrent connections
    pool: Arc<ConnectionPool>,

    /// Optional TLS acceptor
    tls_acceptor: Option<Arc<TlsAcceptor>>,
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
            tls_acceptor: None,
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
            tls_acceptor: None,
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
            tls_acceptor: None,
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
            tls_acceptor: None,
        }
    }

    /// Enable TLS/SSL for this server
    ///
    /// Loads certificate and private key from PEM files.
    /// Fails if files don't exist or are invalid.
    pub fn with_tls(
        mut self,
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let tls_config = Self::load_tls_config(cert_path, key_path)?;
        let acceptor = TlsAcceptor::from(Arc::new(tls_config));
        self.tls_acceptor = Some(Arc::new(acceptor));
        Ok(self)
    }

    /// Load TLS configuration from certificate and key files
    fn load_tls_config(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> anyhow::Result<RustlsServerConfig> {
        // Load certificates
        let cert_file = File::open(cert_path.as_ref())?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()?;

        if certs.is_empty() {
            anyhow::bail!("No certificates found in {:?}", cert_path.as_ref());
        }

        // Load private key
        let key_file = File::open(key_path.as_ref())?;
        let mut key_reader = BufReader::new(key_file);

        let key = rustls_pemfile::private_key(&mut key_reader)?
            .ok_or_else(|| anyhow::anyhow!("No private key found in {:?}", key_path.as_ref()))?;

        // Build TLS config
        let config = RustlsServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        Ok(config)
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_acceptor.is_some()
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

        if self.is_tls_enabled() {
            info!("TLS/SSL enabled - connections will be encrypted");
        } else {
            warn!("TLS/SSL not enabled - connections will be unencrypted");
            warn!("For production, enable TLS with --cert and --key flags");
        }

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

                            let tls_acceptor_ref = self.tls_acceptor.clone();
                            tokio::spawn(async move {
                                // Connection is held for the duration of this task
                                let result = process_socket(socket, tls_acceptor_ref, factory_ref).await;

                                if let Err(e) = result {
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
