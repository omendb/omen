//! HTTP metrics endpoint for Prometheus scraping

use crate::metrics;
use axum::{
    routing::get,
    Router,
    response::{IntoResponse, Response},
    http::StatusCode,
};
use tracing::info;

/// Start metrics HTTP server on specified address
///
/// This exposes a `/metrics` endpoint that Prometheus can scrape
/// Example: http://localhost:9090/metrics
pub async fn serve_metrics(addr: impl AsRef<str>) -> anyhow::Result<()> {
    let addr_str = addr.as_ref();
    info!("Starting metrics HTTP server on {}", addr_str);

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler));

    let listener = tokio::net::TcpListener::bind(addr_str).await?;
    info!("Metrics server listening on {}", addr_str);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for /metrics endpoint
async fn metrics_handler() -> Response {
    let metrics = metrics::get_metrics();
    (StatusCode::OK, metrics).into_response()
}

/// Handler for /health endpoint
async fn health_handler() -> Response {
    let health = metrics::health_check();
    (StatusCode::OK, health.to_json()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_handler() {
        let response = health_handler().await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
