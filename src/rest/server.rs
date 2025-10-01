//! REST API server implementation

use super::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use datafusion::prelude::SessionContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tracing::info;

/// REST API server for OmenDB
pub struct RestServer {
    addr: String,
    ctx: Arc<RwLock<SessionContext>>,
}

impl RestServer {
    /// Create a new REST server with default address (0.0.0.0:8080)
    pub fn new(ctx: SessionContext) -> Self {
        Self {
            addr: "0.0.0.0:8080".to_string(),
            ctx: Arc::new(RwLock::new(ctx)),
        }
    }

    /// Create a new REST server with custom address
    pub fn with_addr(addr: &str, ctx: SessionContext) -> Self {
        Self {
            addr: addr.to_string(),
            ctx: Arc::new(RwLock::new(ctx)),
        }
    }

    /// Start the REST API server
    pub async fn serve(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(handlers::health))
            .route("/metrics", get(handlers::metrics))
            .route("/query", post(handlers::query))
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .with_state(self.ctx);

        info!("REST API server listening on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
