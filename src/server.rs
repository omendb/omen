//! HTTP server for metrics and health endpoints
//! Essential for production monitoring with authentication

use crate::metrics::{get_metrics, health_check};
use crate::security::SecurityContext;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Result as HyperResult, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio;

/// Handle HTTP requests for metrics and health with authentication
async fn handle_request_with_auth(req: Request<Body>, security_ctx: Arc<SecurityContext>) -> HyperResult<Response<Body>> {
    let response = match (req.method(), req.uri().path()) {
        // Prometheus metrics endpoint (requires authentication)
        (&Method::GET, "/metrics") => {
            match security_ctx.authenticate_request(req.headers()) {
                Ok(true) => {
                    let metrics = get_metrics();
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "text/plain; version=0.0.4")
                        .body(Body::from(metrics))
                        .unwrap()
                }
                Ok(false) | Err(_) => {
                    let (header_name, header_value) = security_ctx.auth_challenge_header();
                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header_name, header_value)
                        .body(Body::from("Authentication required"))
                        .unwrap()
                }
            }
        }

        // Health check endpoint (requires authentication for detailed info)
        (&Method::GET, "/health") => {
            match security_ctx.authenticate_request(req.headers()) {
                Ok(true) => {
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
                Ok(false) | Err(_) => {
                    let (header_name, header_value) = security_ctx.auth_challenge_header();
                    Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header(header_name, header_value)
                        .body(Body::from("Authentication required"))
                        .unwrap()
                }
            }
        }

        // Ready check endpoint (public, no auth required for load balancers)
        (&Method::GET, "/ready") => {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from("ready"))
                .unwrap()
        }

        // Status endpoint (public, basic status for monitoring)
        (&Method::GET, "/status") => {
            let health = health_check();
            let status_text = if health.healthy { "healthy" } else { "unhealthy" };
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain")
                .body(Body::from(status_text))
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

/// Legacy handle_request function for backward compatibility
async fn handle_request(req: Request<Body>) -> HyperResult<Response<Body>> {
    let mut security_ctx = SecurityContext::default();
    security_ctx.auth.enabled = false; // Disable auth for backward compatibility
    let security_ctx = Arc::new(security_ctx);
    handle_request_with_auth(req, security_ctx).await
}

/// Start the monitoring HTTP server with authentication
pub async fn start_secure_monitoring_server(port: u16, security_ctx: SecurityContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Print configuration before moving into Arc
    println!("🔒 Secure monitoring server running on http://0.0.0.0:{}", port);
    if security_ctx.auth.enabled {
        println!("🔐 Authentication: ENABLED");
        println!("   Default credentials: admin:admin123 (CHANGE IN PRODUCTION!)");
    } else {
        println!("⚠️  Authentication: DISABLED");
    }
    println!("📊 Metrics:  http://0.0.0.0:{}/metrics", port);
    println!("❤️  Health:   http://0.0.0.0:{}/health", port);
    println!("✅ Ready:    http://0.0.0.0:{}/ready (public)", port);
    println!("📈 Status:   http://0.0.0.0:{}/status (public)", port);

    let security_ctx = Arc::new(security_ctx);

    let make_svc = make_service_fn(move |_conn| {
        let security_ctx = Arc::clone(&security_ctx);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let security_ctx = Arc::clone(&security_ctx);
                handle_request_with_auth(req, security_ctx)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    server.await?;
    Ok(())
}

/// Start the monitoring HTTP server (legacy, no authentication)
pub async fn start_monitoring_server(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut security_ctx = SecurityContext::default();
    security_ctx.auth.enabled = false; // Disable auth for legacy compatibility

    start_secure_monitoring_server(port, security_ctx).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        use crate::metrics::record_search;

        // Initialize some metrics first
        record_search(0.001);

        // Test with auth disabled (legacy mode)
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let metrics = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(!metrics.is_empty());
    }

    #[tokio::test]
    async fn test_health_check() {
        // Test with auth disabled (legacy mode)
        let req = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::SERVICE_UNAVAILABLE);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let json = String::from_utf8(body_bytes.to_vec()).unwrap();
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
        use crate::metrics::record_search;

        // Initialize some metrics first
        record_search(0.001);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        // Should contain Prometheus metrics format
        assert!(!body_str.is_empty());
        // Basic Prometheus format check
        assert!(body_str.contains("TYPE") || body_str.contains("HELP") || body_str.contains("omendb_"));
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

    #[tokio::test]
    async fn test_status_endpoint() {
        let req = Request::builder()
            .method(Method::GET)
            .uri("/status")
            .body(Body::empty())
            .unwrap();

        let response = handle_request(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str == "healthy" || body_str == "unhealthy");
    }

    #[tokio::test]
    async fn test_authentication_required() {
        let security_ctx = Arc::new(SecurityContext::default()); // Auth enabled by default

        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        let response = handle_request_with_auth(req, security_ctx).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_valid_authentication() {
        use base64::{Engine as _, engine::general_purpose};

        let security_ctx = Arc::new(SecurityContext::default());

        let credentials = general_purpose::STANDARD.encode("admin:admin123");
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .header("authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = handle_request_with_auth(req, security_ctx).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_invalid_credentials() {
        use base64::{Engine as _, engine::general_purpose};

        let security_ctx = Arc::new(SecurityContext::default());

        let credentials = general_purpose::STANDARD.encode("admin:wrongpass");
        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .header("authorization", format!("Basic {}", credentials))
            .body(Body::empty())
            .unwrap();

        let response = handle_request_with_auth(req, security_ctx).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_ready_endpoint_no_auth() {
        let security_ctx = Arc::new(SecurityContext::default()); // Auth enabled

        let req = Request::builder()
            .method(Method::GET)
            .uri("/ready")
            .body(Body::empty())
            .unwrap();

        // Ready endpoint should work without auth
        let response = handle_request_with_auth(req, security_ctx).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_disabled() {
        let mut security_ctx = SecurityContext::default();
        security_ctx.auth.enabled = false;
        let security_ctx = Arc::new(security_ctx);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();

        // Should work without auth when disabled
        let response = handle_request_with_auth(req, security_ctx).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}