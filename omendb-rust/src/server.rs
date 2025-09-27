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

    #[tokio::test]
    async fn test_metrics_endpoint_request() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/plain; version=0.0.4");
    }

    #[tokio::test]
    async fn test_health_endpoint_request() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        // Should be OK unless error rate is too high
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE);

        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
    }

    #[tokio::test]
    async fn test_ready_endpoint_request() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/ready")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "text/plain");
    }

    #[tokio::test]
    async fn test_not_found_endpoint() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/unknown")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_post_method_not_found() {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_metrics_content_format() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Should contain Prometheus metrics
        assert!(body_str.contains("omendb_"));
        assert!(body_str.contains("TYPE"));
        assert!(body_str.contains("HELP"));
    }

    #[tokio::test]
    async fn test_health_json_format() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Should be valid JSON with expected fields
        assert!(body_str.contains("\"healthy\""));
        assert!(body_str.contains("\"version\""));
        assert!(body_str.contains("\"uptime_seconds\""));
        assert!(body_str.contains("\"total_operations\""));
        assert!(body_str.contains("\"error_rate\""));
    }

    #[tokio::test]
    async fn test_ready_response_content() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/ready")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_str, "ready");
    }

    #[tokio::test]
    async fn test_different_paths() {
        let test_cases = vec![
            ("/", StatusCode::NOT_FOUND),
            ("/healthz", StatusCode::NOT_FOUND),
            ("/status", StatusCode::NOT_FOUND),
            ("/ping", StatusCode::NOT_FOUND),
            ("/metrics/", StatusCode::NOT_FOUND),
            ("/health/check", StatusCode::NOT_FOUND),
        ];

        for (path, expected_status) in test_cases {
            let req = Request::builder()
                .method(Method::GET)
                .uri(path)
                .body(Body::empty())
                .unwrap();

            let response = handle_request(req).await.unwrap();
            assert_eq!(response.status(), expected_status, "Failed for path: {}", path);
        }
    }
}