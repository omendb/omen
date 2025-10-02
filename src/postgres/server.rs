//! PostgreSQL wire protocol server for OmenDB

use super::handlers::OmenDbHandlerFactory;
use datafusion::prelude::*;
use pgwire::tokio::process_socket;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info};

/// PostgreSQL wire protocol server
pub struct PostgresServer {
    /// Listen address (default: 127.0.0.1:5432)
    addr: String,

    /// DataFusion session context
    ctx: Arc<RwLock<SessionContext>>,
}

impl PostgresServer {
    /// Create a new PostgreSQL server with default address
    pub fn new(ctx: SessionContext) -> Self {
        Self {
            addr: "127.0.0.1:5432".to_string(),
            ctx: Arc::new(RwLock::new(ctx)),
        }
    }

    /// Create a new PostgreSQL server with custom address
    pub fn with_addr(addr: impl Into<String>, ctx: SessionContext) -> Self {
        Self {
            addr: addr.into(),
            ctx: Arc::new(RwLock::new(ctx)),
        }
    }

    /// Start serving PostgreSQL wire protocol connections
    pub async fn serve(self) -> anyhow::Result<()> {
        let factory = Arc::new(OmenDbHandlerFactory::new(self.ctx.clone()));

        info!("Starting PostgreSQL server on {}", self.addr);

        let listener = TcpListener::bind(&self.addr).await?;
        info!("PostgreSQL server listening on {}", self.addr);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New connection from {}", addr);
                    let factory_ref = factory.clone();

                    tokio::spawn(async move {
                        if let Err(e) = process_socket(socket, None, factory_ref).await {
                            error!("Error processing socket from {}: {}", addr, e);
                        }
                    });
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
