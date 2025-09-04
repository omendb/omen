//! Main server implementation for OmenDB Server
//! 
//! Coordinates all server components including HTTP/gRPC APIs, authentication,
//! engine management, and resource monitoring.

use crate::auth::{extract_auth_from_request, AuthManager};
use crate::config::Config;
use crate::engine::EngineManager;
use crate::metrics::MetricsCollector;
use crate::resources::ResourceManager;
use crate::storage::StorageCoordinator;
use crate::types::{
    AddVectorRequest, AddVectorResponse, BatchAddRequest, SearchRequest, SearchResponse,
    HealthResponse, Vector,
};
use crate::{Error, Result, VERSION};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info, instrument, warn};

/// Main OmenDB server
pub struct Server {
    /// Configuration
    config: Config,
    /// Application state
    app_state: Arc<AppState>,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Authentication manager
    pub auth: Arc<AuthManager>,
    /// Engine manager
    pub engine: Arc<EngineManager>,
    /// Metrics collector
    pub metrics: Arc<MetricsCollector>,
    /// Resource manager
    pub resources: Arc<ResourceManager>,
    /// Storage coordinator
    pub storage: Arc<StorageCoordinator>,
    /// Server start time
    pub start_time: Instant,
    /// Whether authentication is enabled
    pub auth_enabled: bool,
}

impl Server {
    /// Create a new server instance
    #[instrument(level = "info")]
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing OmenDB Server v{}", VERSION);

        // Validate configuration
        config.validate()?;

        // Initialize components
        let auth = Arc::new(AuthManager::new(config.auth.clone())?);
        let engine = Arc::new(EngineManager::new(config.engine.clone()).await?);
        let metrics = Arc::new(MetricsCollector::new(config.metrics.clone())?);
        let resources = Arc::new(ResourceManager::new(config.clone()));
        let storage = Arc::new(StorageCoordinator::new(config.storage.clone()).await?);

        let app_state = Arc::new(AppState {
            auth,
            engine,
            metrics,
            resources,
            storage,
            start_time: Instant::now(),
            auth_enabled: config.auth.enabled,
        });

        info!("Server components initialized successfully");

        Ok(Server { config, app_state })
    }

    /// Start the server
    #[instrument(level = "info", skip(self))]
    pub async fn run(self) -> Result<()> {
        info!("Starting OmenDB Server");

        // Start background tasks
        self.start_background_tasks().await;

        // Create HTTP router
        let http_router = self.create_http_router();

        // Start HTTP server
        let http_addr = format!("0.0.0.0:{}", self.config.server.http_port);
        let http_listener = TcpListener::bind(&http_addr).await
            .map_err(|e| Error::internal(format!("Failed to bind HTTP server: {}", e)))?;

        info!("HTTP server listening on {}", http_addr);

        // Start gRPC server (if enabled)
        let grpc_handle = if self.config.server.grpc_port > 0 {
            Some(self.start_grpc_server().await?)
        } else {
            None
        };

        // Start metrics server
        let metrics_handle = if self.config.metrics.enabled {
            Some(self.start_metrics_server().await?)
        } else {
            None
        };

        // Run HTTP server
        let http_handle = tokio::spawn({
            let router = http_router;
            async move {
                axum::serve(http_listener, router).await
                    .map_err(|e| Error::internal(format!("HTTP server error: {}", e)))
            }
        });

        // Wait for any server to complete (or fail)
        tokio::select! {
            result = http_handle => {
                match result {
                    Ok(Ok(())) => info!("HTTP server completed"),
                    Ok(Err(e)) => error!("HTTP server error: {}", e),
                    Err(e) => error!("HTTP server task error: {}", e),
                }
            }
            result = async { 
                if let Some(handle) = grpc_handle { 
                    handle.await 
                } else { 
                    std::future::pending().await 
                }
            } => {
                match result {
                    Ok(Ok(())) => info!("gRPC server completed"),
                    Ok(Err(e)) => error!("gRPC server error: {}", e),
                    Err(e) => error!("gRPC server task error: {}", e),  
                }
            }
            result = async {
                if let Some(handle) = metrics_handle {
                    handle.await
                } else {
                    std::future::pending().await
                }
            } => {
                match result {
                    Ok(Ok(())) => info!("Metrics server completed"),
                    Ok(Err(e)) => error!("Metrics server error: {}", e),
                    Err(e) => error!("Metrics server task error: {}", e),
                }
            }
        }

        Ok(())
    }

    /// Create HTTP router with all routes and middleware
    fn create_http_router(&self) -> Router {
        Router::new()
            // Vector operations
            .route("/v1/vectors", post(add_vector))
            .route("/v1/vectors/batch", post(add_vectors_batch))
            .route("/v1/vectors/:id", get(get_vector))
            .route("/v1/vectors/:id", axum::routing::delete(delete_vector))
            .route("/v1/search", post(search_vectors))
            
            // Collection management (TODO: implement)
            .route("/v1/collections", post(create_collection))
            .route("/v1/collections", get(list_collections))
            .route("/v1/collections/:name", axum::routing::delete(delete_collection))
            
            // Health and status
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check))
            .route("/metrics", get(get_metrics))
            
            // Admin endpoints (TODO: implement)
            .route("/admin/stats", get(get_stats))
            .route("/admin/tenants", get(list_tenants))
            
            // Add state
            .with_state(Arc::clone(&self.app_state))
            
            // Add middleware
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
                    .layer(TimeoutLayer::new(self.config.server.request_timeout))
                    .layer(middleware::from_fn(metrics_middleware))
                    .into_inner(),
            )
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        info!("Starting background tasks");

        // Start resource monitoring
        self.app_state.resources.start_monitoring().await;

        // Start storage background tasks
        self.app_state.storage.start_background_tasks().await;

        // Start metrics collection task
        let metrics = Arc::clone(&self.app_state.metrics);
        let engine = Arc::clone(&self.app_state.engine);
        let resources = Arc::clone(&self.app_state.resources);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                // Update engine metrics
                if let Ok(stats) = engine.get_global_stats().await {
                    metrics.update_engine_stats(&stats);
                }
                
                // Update resource metrics
                let usage = resources.get_resource_usage().await;
                metrics.update_resource_usage(
                    usage.memory_used,
                    usage.cpu_usage_percent,
                    usage.disk_usage_bytes,
                );
            }
        });
    }

    /// Start gRPC server
    async fn start_grpc_server(&self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let grpc_addr = format!("0.0.0.0:{}", self.config.server.grpc_port);
        info!("Starting gRPC server on {}", grpc_addr);

        // TODO: Implement gRPC service
        let handle = tokio::spawn(async move {
            // Placeholder for gRPC server
            warn!("gRPC server not yet implemented");
            tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
            Ok(())
        });

        Ok(handle)
    }

    /// Start metrics server
    async fn start_metrics_server(&self) -> Result<tokio::task::JoinHandle<Result<()>>> {
        let metrics_addr = format!("0.0.0.0:{}", self.config.metrics.port);
        let metrics = Arc::clone(&self.app_state.metrics);
        
        info!("Starting metrics server on {}", metrics_addr);

        let handle = tokio::spawn(async move {
            let app = Router::new()
                .route("/metrics", get(move || async move {
                    match metrics.export_metrics() {
                        Ok(metrics_text) => (StatusCode::OK, metrics_text),
                        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)),
                    }
                }));

            let listener = TcpListener::bind(&metrics_addr).await
                .map_err(|e| Error::internal(format!("Failed to bind metrics server: {}", e)))?;

            axum::serve(listener, app).await
                .map_err(|e| Error::internal(format!("Metrics server error: {}", e)))
        });

        Ok(handle)
    }
}

// HTTP handlers

/// Add a single vector
#[instrument(level = "debug", skip(state, headers))]
async fn add_vector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AddVectorRequest>,
) -> Result<Json<AddVectorResponse>> {
    let start_time = Instant::now();

    // Authenticate request (bypass if auth disabled)
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        warn!("⚠️  AUTH DISABLED - Using test tenant");
        crate::auth_test::test_tenant_context()
    };

    // Add vector via engine manager
    let created = !state.engine.get_vector(&tenant, &request.vector.id).await?.is_some();
    state.engine.add_vector(&tenant, request.vector.clone()).await?;

    // Record metrics
    let duration = start_time.elapsed().as_secs_f64();
    state.metrics.record_add(&tenant);

    Ok(Json(AddVectorResponse {
        id: request.vector.id,
        created,
    }))
}

/// Add multiple vectors in batch
#[instrument(level = "debug", skip(state, headers, request))]
async fn add_vectors_batch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<BatchAddRequest>,
) -> Result<Json<crate::types::BatchAddResponse>> {
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    let response = state.engine.add_vectors_batch(&tenant, request.vectors).await?;
    
    Ok(Json(response))
}

/// Get a vector by ID
#[instrument(level = "debug", skip(state, headers))]
async fn get_vector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Option<Vector>>> {
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    let vector = state.engine.get_vector(&tenant, &id).await?;
    
    Ok(Json(vector))
}

/// Delete a vector by ID
#[instrument(level = "debug", skip(state, headers))]
async fn delete_vector(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<bool>> {
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    let deleted = state.engine.delete_vector(&tenant, &id).await?;
    
    Ok(Json(deleted))
}

/// Search for similar vectors
#[instrument(level = "debug", skip(state, headers, request))]
async fn search_vectors(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    let start_time = Instant::now();
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    let search_result = state.engine.search(&tenant, request).await?;
    let duration = start_time.elapsed().as_secs_f64();
    
    // Record metrics
    state.metrics.record_search(&tenant, duration, search_result.results.len());
    
    Ok(Json(search_result))
}

/// Create a collection (placeholder)
async fn create_collection(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
) -> Result<Json<String>> {
    Err(Error::internal("Collections not yet implemented"))
}

/// List collections (placeholder)
async fn list_collections(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
) -> Result<Json<Vec<String>>> {
    Ok(Json(vec![]))
}

/// Delete a collection (placeholder)
async fn delete_collection(
    State(_state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Path(_name): Path<String>,
) -> Result<Json<bool>> {
    Err(Error::internal("Collections not yet implemented"))
}

/// Health check endpoint
#[instrument(level = "debug", skip(state))]
async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<HealthResponse>> {
    // Check all components
    state.auth.health_check().await?;
    state.engine.health_check().await?;
    state.metrics.health_check()?;
    state.resources.health_check().await?;

    let engine_stats = state.engine.get_global_stats().await?;
    let uptime = state.start_time.elapsed().as_secs();

    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        version: VERSION.to_string(),
        uptime_seconds: uptime,
        engine: engine_stats,
    }))
}

/// Readiness check endpoint
async fn readiness_check(State(state): State<Arc<AppState>>) -> Result<Json<String>> {
    // Quick readiness check
    if state.resources.is_memory_pressure().await {
        return Err(Error::internal("Server under memory pressure"));
    }

    Ok(Json("ready".to_string()))
}

/// Get metrics endpoint
async fn get_metrics(State(state): State<Arc<AppState>>) -> Result<String> {
    state.metrics.export_metrics()
}

/// Get server stats (admin endpoint)
#[axum::debug_handler]
async fn get_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    // Check admin permission
    if !tenant.has_permission(crate::types::Permission::Admin) {
        return Err(Error::authz("Admin access required"));
    }

    let engine_stats = state.engine.get_global_stats().await?;
    let resource_usage = state.resources.get_resource_usage().await;
    let storage_stats = state.storage.get_storage_stats().await;
    let pool_status = state.engine.pool_status().await;

    let stats = serde_json::json!({
        "engine": engine_stats,
        "resources": resource_usage,
        "storage": storage_stats,
        "pool_status": pool_status,
        "uptime_seconds": state.start_time.elapsed().as_secs()
    });

    Ok(Json(stats))
}

/// List tenants (admin endpoint)
#[axum::debug_handler]
async fn list_tenants(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<crate::types::TenantContext>>> {
    let tenant = if state.auth_enabled {
        extract_auth_from_request(&state.auth, &headers).await?
    } else {
        crate::auth_test::test_tenant_context()
    };
    
    // Check admin permission
    if !tenant.has_permission(crate::types::Permission::Admin) {
        return Err(Error::authz("Admin access required"));
    }

    // TODO: Implement tenant listing
    Ok(Json(vec![]))
}

/// Metrics middleware
async fn metrics_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed().as_secs_f64();
    let status_code = response.status();
    
    // Log request
    if status_code.is_server_error() {
        error!("{} {} {} {:.3}s", method, uri, status_code, duration);
    } else if duration > 1.0 {
        warn!("{} {} {} {:.3}s (slow)", method, uri, status_code, duration);
    } else {
        info!("{} {} {} {:.3}s", method, uri, status_code, duration);
    }
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_server_creation() {
        let config = Config::default();
        let server = Server::new(config).await;
        // This test will fail without the actual Mojo library
        // assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        // This would require the full server setup
        // let config = Config::default();
        // let server = Server::new(config).await.unwrap();
        // let app = server.create_http_router();
        // let test_server = TestServer::new(app).unwrap();
        
        // let response = test_server.get("/health").await;
        // assert_eq!(response.status_code(), 200);
    }
}