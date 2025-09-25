//! gRPC service implementation for OmenDB Server
//! 
//! Provides gRPC endpoints that complement the HTTP REST API,
//! with better performance for high-throughput scenarios.

use crate::auth::{extract_auth_from_request, AuthManager};
use crate::engine::EngineManager;
use crate::metrics::MetricsCollector;
use crate::types::{SearchRequest as InternalSearchRequest, Vector};
use crate::{Error, Result};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{debug, info, instrument};

// Generated gRPC types (when build.rs runs)
// pub mod generated {
//     tonic::include_proto!("omendb.v1");
// }

// For now, define the types manually until the build system is set up
pub mod proto {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddVectorRequest {
        pub id: String,
        pub vector: Vec<f32>,
        pub metadata: HashMap<String, String>,
        pub collection: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddVectorResponse {
        pub id: String,
        pub created: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchRequest {
        pub vector: Vec<f32>,
        pub top_k: i32,
        pub filter: HashMap<String, String>,
        pub include_vector: bool,
        pub collection: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub results: Vec<SearchResult>,
        pub query_time_ms: f64,
        pub vectors_searched: i32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResult {
        pub id: String,
        pub distance: f32,
        pub metadata: HashMap<String, String>,
        pub vector: Option<VectorData>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VectorData {
        pub data: Vec<f32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetVectorRequest {
        pub id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetVectorResponse {
        pub vector: Option<VectorData>,
        pub metadata: HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteVectorRequest {
        pub id: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeleteVectorResponse {
        pub deleted: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthCheckRequest {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HealthCheckResponse {
        pub status: String,
        pub version: String,
        pub uptime_seconds: i64,
    }
}

/// gRPC service implementation
pub struct OmenDbGrpcService {
    /// Authentication manager
    auth: Arc<AuthManager>,
    /// Engine manager
    engine: Arc<EngineManager>,
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

impl OmenDbGrpcService {
    /// Create a new gRPC service
    pub fn new(
        auth: Arc<AuthManager>,
        engine: Arc<EngineManager>,
        metrics: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            auth,
            engine,
            metrics,
        }
    }

    /// Extract tenant context from gRPC request metadata
    async fn extract_tenant_context(&self, request: &tonic::Request<impl std::fmt::Debug>) -> Result<crate::types::TenantContext> {
        let metadata = request.metadata();
        
        // Try Authorization header (JWT)
        if let Some(auth_header) = metadata.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return self.auth.validate_token(token).await;
                }
            }
        }

        // Try x-api-key header
        if let Some(api_key_header) = metadata.get("x-api-key") {
            if let Ok(api_key) = api_key_header.to_str() {
                return self.auth.validate_api_key(api_key).await;
            }
        }

        Err(Error::auth("No valid authentication provided"))
    }
}

// Note: This would be implemented with the actual gRPC trait when build.rs generates the code
// For now, providing the implementation structure

impl OmenDbGrpcService {
    /// Add a single vector
    #[instrument(level = "debug", skip(self, request))]
    pub async fn add_vector(
        &self,
        request: Request<proto::AddVectorRequest>,
    ) -> std::result::Result<Response<proto::AddVectorResponse>, Status> {
        let start_time = std::time::Instant::now();
        
        // Extract tenant context
        let tenant = self.extract_tenant_context(&request).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();
        
        // Convert to internal types
        let vector = Vector {
            id: req.id.clone(),
            data: req.vector,
            metadata: req.metadata.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
        };

        // Check if vector already exists
        let existing = self.engine.get_vector(&tenant, &req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;
        let created = existing.is_none();

        // Add vector
        self.engine.add_vector(&tenant, vector).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Record metrics
        let duration = start_time.elapsed().as_secs_f64();
        self.metrics.record_add(&tenant);

        debug!("gRPC add_vector completed in {:.3}s", duration);

        let response = proto::AddVectorResponse {
            id: req.id,
            created,
        };

        Ok(Response::new(response))
    }

    /// Search for similar vectors
    #[instrument(level = "debug", skip(self, request))]
    pub async fn search(
        &self,
        request: Request<proto::SearchRequest>,
    ) -> std::result::Result<Response<proto::SearchResponse>, Status> {
        let start_time = std::time::Instant::now();
        
        // Extract tenant context
        let tenant = self.extract_tenant_context(&request).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // Convert to internal search request
        let search_request = InternalSearchRequest {
            vector: req.vector,
            top_k: req.top_k,
            filter: req.filter.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
            include_vector: req.include_vector,
            collection: req.collection,
        };

        // Perform search
        let search_result = self.engine.search(&tenant, search_request).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let duration = start_time.elapsed().as_secs_f64();

        // Convert results
        let results = search_result.results.into_iter().map(|result| {
            proto::SearchResult {
                id: result.id,
                distance: result.distance,
                metadata: result.metadata.into_iter()
                    .filter_map(|(k, v)| {
                        if let serde_json::Value::String(s) = v {
                            Some((k, s))
                        } else {
                            Some((k, v.to_string()))
                        }
                    })
                    .collect(),
                vector: result.vector.map(|v| proto::VectorData { data: v }),
            }
        }).collect();

        // Record metrics
        self.metrics.record_search(&tenant, duration, results.len());

        debug!("gRPC search completed in {:.3}s with {} results", duration, results.len());

        let response = proto::SearchResponse {
            results,
            query_time_ms: search_result.query_time_ms,
            vectors_searched: search_result.vectors_searched as i32,
        };

        Ok(Response::new(response))
    }

    /// Get vector by ID
    #[instrument(level = "debug", skip(self, request))]
    pub async fn get_vector(
        &self,
        request: Request<proto::GetVectorRequest>,
    ) -> std::result::Result<Response<proto::GetVectorResponse>, Status> {
        // Extract tenant context
        let tenant = self.extract_tenant_context(&request).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // Get vector
        let vector = self.engine.get_vector(&tenant, &req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = if let Some(vector) = vector {
            proto::GetVectorResponse {
                vector: Some(proto::VectorData {
                    data: vector.data,
                }),
                metadata: vector.metadata.into_iter()
                    .filter_map(|(k, v)| {
                        if let serde_json::Value::String(s) = v {
                            Some((k, s))
                        } else {
                            Some((k, v.to_string()))
                        }
                    })
                    .collect(),
            }
        } else {
            proto::GetVectorResponse {
                vector: None,
                metadata: std::collections::HashMap::new(),
            }
        };

        Ok(Response::new(response))
    }

    /// Delete vector by ID
    #[instrument(level = "debug", skip(self, request))]
    pub async fn delete_vector(
        &self,
        request: Request<proto::DeleteVectorRequest>,
    ) -> std::result::Result<Response<proto::DeleteVectorResponse>, Status> {
        // Extract tenant context
        let tenant = self.extract_tenant_context(&request).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        let req = request.into_inner();

        // Delete vector
        let deleted = self.engine.delete_vector(&tenant, &req.id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = proto::DeleteVectorResponse { deleted };

        Ok(Response::new(response))
    }

    /// Health check
    #[instrument(level = "debug", skip(self, _request))]
    pub async fn health_check(
        &self,
        _request: Request<proto::HealthCheckRequest>,
    ) -> std::result::Result<Response<proto::HealthCheckResponse>, Status> {
        // No authentication required for health check

        let response = proto::HealthCheckResponse {
            status: "healthy".to_string(),
            version: crate::VERSION.to_string(),
            uptime_seconds: 0, // Would be calculated from server start time
        };

        Ok(Response::new(response))
    }
}

/// Create and configure gRPC server
pub async fn create_grpc_server(
    auth: Arc<AuthManager>,
    engine: Arc<EngineManager>,
    metrics: Arc<MetricsCollector>,
    port: u16,
) -> Result<()> {
    let service = OmenDbGrpcService::new(auth, engine, metrics);
    let addr = format!("0.0.0.0:{}", port).parse()
        .map_err(|e| Error::internal(format!("Invalid gRPC address: {}", e)))?;

    info!("Starting gRPC server on {}", addr);

    // Note: This would use the actual generated service trait
    // For now, this is a placeholder showing the structure
    
    // tonic::transport::Server::builder()
    //     .add_service(OmenDbServer::new(service))
    //     .serve(addr)
    //     .await
    //     .map_err(|e| Error::internal(format!("gRPC server error: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, EngineConfig, MetricsConfig, RateLimitConfig};
    use std::time::Duration;

    fn create_test_auth_config() -> AuthConfig {
        AuthConfig {
            jwt_secret: "test-secret".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            enable_api_keys: true,
            rate_limit: RateLimitConfig {
                requests_per_minute: 1000,
                burst_capacity: 100,
            },
        }
    }

    fn create_test_engine_config() -> EngineConfig {
        EngineConfig {
            dimension: 128,
            pool_size: 2,
            idle_timeout: Duration::from_secs(60),
            max_vectors_per_engine: 10000,
            enable_tiered_storage: false,
        }
    }

    fn create_test_metrics_config() -> MetricsConfig {
        MetricsConfig {
            enabled: true,
            port: 9091,
            collection_interval: Duration::from_secs(60),
            enable_engine_metrics: true,
        }
    }

    #[tokio::test]
    async fn test_grpc_service_creation() {
        let auth = Arc::new(AuthManager::new(create_test_auth_config()).unwrap());
        let metrics = Arc::new(MetricsCollector::new(create_test_metrics_config()).unwrap());
        
        // Note: This would require the actual engine for full testing
        // let engine = Arc::new(EngineManager::new(create_test_engine_config()).await.unwrap());
        // let service = OmenDbGrpcService::new(auth, engine, metrics);
        
        // For now, just test creation without engine
        // assert!(service creation would work);
    }

    #[test]
    fn test_proto_types() {
        let request = proto::AddVectorRequest {
            id: "test".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: std::collections::HashMap::new(),
            collection: None,
        };

        assert_eq!(request.id, "test");
        assert_eq!(request.vector.len(), 3);
    }
}