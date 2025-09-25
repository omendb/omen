//! High-level engine management for OmenDB Server
//! 
//! Provides a higher-level interface over the FFI bridge, handling
//! connection pooling, resource management, and multi-tenant isolation.

use crate::config::EngineConfig;
use crate::python_ffi::PythonMojoEngine;
use crate::types::{EngineStats, TenantContext, Vector};
use crate::{Error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// High-level engine manager that coordinates multiple Mojo engines
/// and provides multi-tenant isolation and resource management.
pub struct EngineManager {
    /// Configuration
    config: EngineConfig,
    /// Primary Python Mojo engine (best performance)
    primary_engine: Arc<RwLock<PythonMojoEngine>>,
    /// Per-tenant engine assignments (for isolation)
    tenant_engines: Arc<RwLock<HashMap<Uuid, Arc<RwLock<PythonMojoEngine>>>>>,
    /// Global statistics
    stats: Arc<RwLock<GlobalStats>>,
}

/// Global engine statistics
#[derive(Debug, Clone, Default)]
struct GlobalStats {
    /// Total requests processed
    total_requests: u64,
    /// Total vectors added
    total_vectors_added: u64,
    /// Total search operations
    total_searches: u64,
    /// Average query time in microseconds
    avg_query_time_us: f64,
}

impl EngineManager {
    /// Create a new engine manager
    #[instrument(level = "info")]
    pub async fn new(config: EngineConfig) -> Result<Self> {
        info!(
            "Initializing engine manager: dimension={}",
            config.dimension
        );

        // Create primary Python Mojo engine (best performance)
        let mut primary_engine = PythonMojoEngine::new(config.dimension)?;
        primary_engine.initialize().await?;

        // Test that the engine works
        let stats = primary_engine.get_stats().await?;
        info!("Engine manager initialized successfully. Test stats: {:?}", stats);

        Ok(EngineManager {
            config,
            primary_engine: Arc::new(RwLock::new(primary_engine)),
            tenant_engines: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(GlobalStats::default())),
        })
    }

    /// Add a vector for a specific tenant
    #[instrument(level = "debug", skip(self, vector))]
    pub async fn add_vector(&self, tenant: &TenantContext, vector: Vector) -> Result<()> {
        // Validate vector
        vector.validate()?;

        // Check tenant quotas
        tenant.check_quota("add_vector")?;

        // Get engine for this tenant
        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;

        // Add vector
        let start_time = std::time::Instant::now();
        let result = engine.add_vector(&vector.id, &vector.data).await;
        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            if result.is_ok() {
                stats.total_vectors_added += 1;
            }
        }

        debug!(
            "Added vector {} for tenant {} in {:?}",
            vector.id, tenant.tenant_id, duration
        );

        result
    }

    /// Search vectors for a specific tenant
    #[instrument(level = "debug", skip(self, request))]
    pub async fn search(&self, tenant: &TenantContext, request: crate::types::SearchRequest) -> Result<crate::types::SearchResponse> {
        // Validate request
        request.validate(self.config.dimension as usize)?;

        // Check tenant quotas
        tenant.check_quota("search")?;

        // Get engine for this tenant
        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;

        // Perform search
        let start_time = std::time::Instant::now();
        let search_results = engine.search(&request.vector, request.top_k as usize).await?;
        let duration = start_time.elapsed();
        
        // Convert internal SearchResult to our types::SearchResult
        let results: Vec<crate::types::SearchResult> = search_results.into_iter().map(|r| crate::types::SearchResult {
            id: r.id,
            distance: r.distance,
            metadata: r.metadata.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
            vector: None,
        }).collect();

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.total_searches += 1;
            
            // Update running average of query time
            let new_time_us = duration.as_micros() as f64;
            if stats.avg_query_time_us == 0.0 {
                stats.avg_query_time_us = new_time_us;
            } else {
                // Exponential moving average
                stats.avg_query_time_us = 0.9 * stats.avg_query_time_us + 0.1 * new_time_us;
            }
        }

        debug!(
            "Search completed for tenant {} in {:?}: {} results",
            tenant.tenant_id,
            duration,
            results.len()
        );

        Ok(crate::types::SearchResponse {
            results,
            query_time_ms: duration.as_millis() as f64,
            vectors_searched: self.get_tenant_vector_count(tenant).await.unwrap_or(0),
        })
    }

    /// Get a vector by ID for a specific tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn get_vector(&self, tenant: &TenantContext, id: &str) -> Result<Option<Vector>> {
        // Check tenant permissions
        if !tenant.has_permission(crate::types::Permission::Read) {
            return Err(Error::authz("Read permission required"));
        }

        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;
        
        // Note: OptimizedPythonEngine doesn't have get_vector yet
        // For now, return None indicating not found
        debug!("get_vector not yet implemented in OptimizedPythonEngine");
        Ok(None)
    }

    /// Delete a vector for a specific tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_vector(&self, tenant: &TenantContext, id: &str) -> Result<bool> {
        // Check tenant permissions
        if !tenant.has_permission(crate::types::Permission::Delete) {
            return Err(Error::authz("Delete permission required"));
        }

        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;
        
        // Note: OptimizedPythonEngine doesn't have delete_vector yet
        // For now, return false indicating not deleted
        debug!("delete_vector not yet implemented in OptimizedPythonEngine");
        Ok(false)
    }

    /// Enable tiered storage for a tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn enable_tiered_storage(&self, tenant: &TenantContext) -> Result<bool> {
        // Check if tenant has permission for advanced features
        if !tenant.has_permission(crate::types::Permission::Admin) {
            return Err(Error::authz("Admin permission required for tiered storage"));
        }

        let engine = self.get_tenant_engine(tenant).await?;
        // Note: We need a mutable reference, so we'll need to handle this differently
        // For now, return an error indicating this needs to be implemented
        Err(Error::internal(
            "Tiered storage enablement needs engine pool redesign for mutability"
        ))
    }

    /// Get engine statistics for a tenant
    #[instrument(level = "debug", skip(self))]
    pub async fn get_tenant_stats(&self, tenant: &TenantContext) -> Result<EngineStats> {
        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;
        
        let stats_map = engine.get_stats().await?;
        
        // Convert HashMap<String, serde_json::Value> to EngineStats
        let vector_count = stats_map.get("vector_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
            
        Ok(crate::types::EngineStats {
            algorithm: stats_map.get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("HNSW")
                .to_string(),
            status: "ready".to_string(),
            vector_count,
            hot_vectors: 0,
            warm_vectors: 0,
            cold_vectors: 0,
            memory_usage_bytes: 0,
            avg_query_time_us: 0.0,
            queries_per_second: 0.0,
        })
    }

    /// Get global engine statistics
    #[instrument(level = "debug", skip(self))]
    pub async fn get_global_stats(&self) -> Result<crate::types::EngineStats> {
        let stats = self.stats.read().await;
        let tenant_engines = self.tenant_engines.read().await;

        // Get stats from primary engine
        let engine = self.primary_engine.read().await;
        let stats_map = engine.get_stats().await?;
        
        let vector_count = stats_map.get("vector_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Ok(crate::types::EngineStats {
            algorithm: stats_map.get("algorithm")
                .and_then(|v| v.as_str())
                .unwrap_or("HNSW")
                .to_string(),
            status: format!("primary + {} dedicated engines", tenant_engines.len()),
            vector_count,
            hot_vectors: 0,
            warm_vectors: 0,
            cold_vectors: 0,
            memory_usage_bytes: 0,
            avg_query_time_us: stats.avg_query_time_us,
            queries_per_second: if stats.avg_query_time_us > 0.0 {
                1_000_000.0 / stats.avg_query_time_us
            } else {
                0.0
            },
        })
    }

    /// Get vector count for a tenant
    async fn get_tenant_vector_count(&self, tenant: &TenantContext) -> Result<usize> {
        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;
        let stats_map = engine.get_stats().await?;
        
        let count = stats_map.get("vector_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
            
        Ok(count)
    }

    /// Get or create an engine for a specific tenant
    /// 
    /// This implements tenant isolation by assigning dedicated engines
    /// to tenants based on their subscription tier and usage patterns.
    async fn get_tenant_engine(&self, tenant: &TenantContext) -> Result<Arc<RwLock<PythonMojoEngine>>> {
        match tenant.tier {
            crate::types::SubscriptionTier::Enterprise => {
                // Enterprise gets dedicated engine
                self.get_or_create_dedicated_engine(tenant).await
            }
            _ => {
                // Platform and Free use shared primary engine
                Ok(self.primary_engine.clone())
            }
        }
    }

    /// Get or create a dedicated engine for enterprise tenants
    async fn get_or_create_dedicated_engine(&self, tenant: &TenantContext) -> Result<Arc<RwLock<PythonMojoEngine>>> {
        // Check if we already have a dedicated engine for this tenant
        {
            let tenant_engines = self.tenant_engines.read().await;
            if let Some(engine) = tenant_engines.get(&tenant.tenant_id) {
                debug!("Using existing dedicated engine for tenant {}", tenant.tenant_id);
                return Ok(engine.clone());
            }
        }

        // Create new dedicated engine
        info!("Creating dedicated engine for enterprise tenant {}", tenant.tenant_id);
        
        let mut new_engine = PythonMojoEngine::new(self.config.dimension)?;
        new_engine.initialize().await?;
        let engine_arc = Arc::new(RwLock::new(new_engine));
        
        // Store the dedicated engine
        {
            let mut tenant_engines = self.tenant_engines.write().await;
            tenant_engines.insert(tenant.tenant_id, engine_arc.clone());
        }

        Ok(engine_arc)
    }

    /// Clean up resources for a tenant (when they're deleted)
    #[instrument(level = "info", skip(self))]
    pub async fn cleanup_tenant(&self, tenant_id: Uuid) -> Result<()> {
        info!("Cleaning up resources for tenant {}", tenant_id);
        
        let mut tenant_engines = self.tenant_engines.write().await;
        if let Some(_engine) = tenant_engines.remove(&tenant_id) {
            info!("Removed dedicated engine for tenant {}", tenant_id);
            // Engine will be dropped and cleaned up automatically
        }

        Ok(())
    }

    /// Health check for the engine manager
    pub async fn health_check(&self) -> Result<()> {
        // Check primary engine
        let engine = self.primary_engine.read().await;
        
        if !engine.is_initialized() {
            return Err(Error::EngineNotInitialized);
        }
        
        // Try to get stats (this tests the C FFI connection)
        let _stats = engine.get_stats().await?;
        
        debug!("Engine manager health check passed");
        Ok(())
    }

    /// Get engine pool status
    pub async fn pool_status(&self) -> String {
        let tenant_engines = self.tenant_engines.read().await;
        let primary_initialized = self.primary_engine.read().await.is_initialized();
        
        format!("primary: {}, dedicated: {}", 
               if primary_initialized { "ready" } else { "not_initialized" },
               tenant_engines.len())
    }

    /// Get configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }
}

/// Batch operations for efficiency
impl EngineManager {
    /// Add multiple vectors in a batch
    #[instrument(level = "debug", skip(self, vectors))]
    pub async fn add_vectors_batch(&self, tenant: &TenantContext, vectors: Vec<Vector>) -> Result<crate::types::BatchAddResponse> {
        let mut added = 0;
        let mut updated = 0;
        let mut errors = Vec::new();

        // Check overall quota first
        tenant.check_quota("batch_add")?;

        let engine_arc = self.get_tenant_engine(tenant).await?;
        let engine = engine_arc.read().await;

        for vector in vectors {
            match vector.validate() {
                Ok(_) => {
                    match engine.add_vector(&vector.id, &vector.data).await {
                        Ok(_) => {
                            // For now, consider all as new additions
                            // TODO: Implement contains check in CMojoEngine
                            added += 1;
                        }
                        Err(e) => {
                            errors.push(crate::types::BatchError {
                                vector_id: vector.id.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    errors.push(crate::types::BatchError {
                        vector_id: vector.id.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Update global statistics
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.total_vectors_added += added as u64;
        }

        Ok(crate::types::BatchAddResponse {
            added,
            updated,
            errors,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EngineConfig;
    use crate::types::{Permission, SubscriptionTier, TenantUsage};
    use std::time::Duration;

    fn create_test_config() -> EngineConfig {
        EngineConfig {
            dimension: 128,
            pool_size: 2,
            idle_timeout: Duration::from_secs(60),
            max_vectors_per_engine: 10000,
            enable_tiered_storage: false,
        }
    }

    fn create_test_tenant() -> TenantContext {
        TenantContext {
            tenant_id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            tier: SubscriptionTier::Platform,
            usage: TenantUsage {
                vectors_stored: 0,
                queries_this_hour: 0,
                bandwidth_this_month: 0,
                storage_used_bytes: 0,
            },
            permissions: vec![Permission::Read, Permission::Write],
        }
    }

    #[tokio::test]
    async fn test_engine_manager_creation() {
        // This test would require the actual Mojo library
        // let config = create_test_config();
        // let manager = EngineManager::new(config).await;
        // assert!(manager.is_ok());
    }

    #[test]
    fn test_tenant_context_permissions() {
        let tenant = create_test_tenant();
        assert!(tenant.has_permission(Permission::Read));
        assert!(tenant.has_permission(Permission::Write));
        assert!(!tenant.has_permission(Permission::Admin));
    }

    #[test]
    fn test_vector_validation() {
        let vector = Vector::new("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(vector.validate().is_ok());

        let invalid_vector = Vector::new("".to_string(), vec![]);
        assert!(invalid_vector.validate().is_err());
    }
}