//! HTTP server for metrics and health endpoints
//! Essential for production monitoring

use crate::metrics::{get_metrics, health_check};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Result as HyperResult, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio;

/// Handle HTTP requests for metrics and health
async fn handle_request(req: Request<Body>) -> HyperResult<Response<Body>> {
    let response = match (req.method(), req.uri().path()) {
        // Prometheus metrics endpoint
        (&Method::GET, "/metrics") => {
            let metrics = get_metrics();
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4")
                .body(Body::from(metrics))
                .unwrap()
        }

        // Health check endpoint
        (&Method::GET, "/health") => {
            let health = health_check();
            let status = if health.healthy {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };

            Response::builder()
                .status(status)
                .header("Content-Type", "application/json")
                .body(Body::from(health.to_json()))
                .unwrap()
        }

        // Ready check endpoint (simpler health check)
        (&Method::GET, "/ready") => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from("ready"))
                .unwrap()
        }

        // Default 404
        _ => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    };

    Ok(response)
}

/// Start the monitoring HTTP server
pub async fn start_monitoring_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Monitoring server running on http://0.0.0.0:{}", port);
    println!("Metrics: http://0.0.0.0:{}/metrics", port);
    println!("Health:  http://0.0.0.0:{}/health", port);

    server.await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        // Can't easily test the actual server without starting it
        // But we can test the metrics function
        let metrics = get_metrics();
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_health_check() {
        let health = health_check();
        let json = health.to_json();
        assert!(json.contains("healthy"));
        assert!(json.contains("version"));
    }
}