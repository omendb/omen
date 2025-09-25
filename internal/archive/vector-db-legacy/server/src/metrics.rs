//! Metrics collection and monitoring for OmenDB Server
//! 
//! Provides Prometheus-style metrics for monitoring server performance,
//! resource usage, and business metrics.

use crate::config::MetricsConfig;
use crate::types::{EngineStats, TenantContext};
use crate::{Error, Result};
use prometheus::{
    CounterVec, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Metrics collector for OmenDB Server
pub struct MetricsCollector {
    /// Configuration
    config: MetricsConfig,
    /// Prometheus registry
    registry: Registry,
    /// Request metrics
    request_metrics: RequestMetrics,
    /// Engine metrics
    engine_metrics: EngineMetrics,
    /// Business metrics
    business_metrics: BusinessMetrics,
    /// Resource metrics
    resource_metrics: ResourceMetrics,
    /// Per-tenant metrics
    tenant_metrics: Arc<RwLock<HashMap<uuid::Uuid, TenantMetrics>>>,
}

/// HTTP request metrics
struct RequestMetrics {
    /// Total requests counter
    requests_total: IntCounter,
    /// Request duration histogram
    request_duration: Histogram,
    /// Active connections gauge
    active_connections: IntGauge,
    /// Errors by type
    errors_total: CounterVec,
}

/// Engine performance metrics
struct EngineMetrics {
    /// Vectors stored
    vectors_total: IntGauge,
    /// Search operations
    searches_total: IntCounter,
    /// Add operations
    adds_total: IntCounter,
    /// Query latency
    query_latency: Histogram,
    /// Engine memory usage
    engine_memory_bytes: Gauge,
    /// Hot tier vectors
    hot_tier_vectors: IntGauge,
    /// Warm tier vectors
    warm_tier_vectors: IntGauge,
    /// Cold tier vectors
    cold_tier_vectors: IntGauge,
}

/// Business metrics
struct BusinessMetrics {
    /// Active tenants
    tenants_active: IntGauge,
    /// Tenant requests by tier
    tenant_requests_by_tier: CounterVec,
    /// Revenue metrics (queries that count toward billing)
    billable_queries: IntCounter,
    /// API key usage
    api_key_usage: IntCounter,
}

/// System resource metrics
struct ResourceMetrics {
    /// Memory usage
    memory_usage_bytes: Gauge,
    /// CPU usage percentage
    cpu_usage_percent: Gauge,
    /// Disk usage
    disk_usage_bytes: Gauge,
    /// Network bytes
    network_bytes_total: CounterVec,
}

/// Per-tenant metrics
#[derive(Debug, Clone)]
struct TenantMetrics {
    /// Tenant ID
    tenant_id: uuid::Uuid,
    /// Request count
    requests: u64,
    /// Vector count
    vectors: u64,
    /// Query count
    queries: u64,
    /// Errors
    errors: u64,
    /// Last activity timestamp
    last_activity: chrono::DateTime<chrono::Utc>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[instrument(level = "info")]
    pub fn new(config: MetricsConfig) -> Result<Self> {
        info!("Initializing metrics collector");

        let registry = Registry::new();

        // Initialize request metrics
        let requests_total = IntCounter::with_opts(Opts::new(
            "omendb_requests_total",
            "Total number of HTTP requests",
        ))
        .map_err(|e| Error::internal(format!("Failed to create requests_total metric: {}", e)))?;

        let request_duration = Histogram::with_opts(HistogramOpts::new(
            "omendb_request_duration_seconds",
            "Request duration in seconds",
        ))
        .map_err(|e| Error::internal(format!("Failed to create request_duration metric: {}", e)))?;

        let active_connections = IntGauge::with_opts(Opts::new(
            "omendb_active_connections",
            "Number of active connections",
        ))
        .map_err(|e| Error::internal(format!("Failed to create active_connections metric: {}", e)))?;

        let errors_total = CounterVec::new(
            Opts::new("omendb_errors_total", "Total number of errors by type"),
            &["error_type"],
        )
        .map_err(|e| Error::internal(format!("Failed to create errors_total metric: {}", e)))?;

        // Initialize engine metrics
        let vectors_total = IntGauge::with_opts(Opts::new(
            "omendb_vectors_total",
            "Total number of vectors stored",
        ))
        .map_err(|e| Error::internal(format!("Failed to create vectors_total metric: {}", e)))?;

        let searches_total = IntCounter::with_opts(Opts::new(
            "omendb_searches_total",
            "Total number of search operations",
        ))
        .map_err(|e| Error::internal(format!("Failed to create searches_total metric: {}", e)))?;

        let adds_total = IntCounter::with_opts(Opts::new(
            "omendb_adds_total",
            "Total number of add operations",
        ))
        .map_err(|e| Error::internal(format!("Failed to create adds_total metric: {}", e)))?;

        let query_latency = Histogram::with_opts(HistogramOpts::new(
            "omendb_query_latency_seconds",
            "Query latency in seconds",
        ))
        .map_err(|e| Error::internal(format!("Failed to create query_latency metric: {}", e)))?;

        let engine_memory_bytes = Gauge::with_opts(Opts::new(
            "omendb_engine_memory_bytes",
            "Engine memory usage in bytes",
        ))
        .map_err(|e| Error::internal(format!("Failed to create engine_memory_bytes metric: {}", e)))?;

        let hot_tier_vectors = IntGauge::with_opts(Opts::new(
            "omendb_hot_tier_vectors",
            "Number of vectors in hot tier",
        ))
        .map_err(|e| Error::internal(format!("Failed to create hot_tier_vectors metric: {}", e)))?;

        let warm_tier_vectors = IntGauge::with_opts(Opts::new(
            "omendb_warm_tier_vectors",
            "Number of vectors in warm tier",
        ))
        .map_err(|e| Error::internal(format!("Failed to create warm_tier_vectors metric: {}", e)))?;

        let cold_tier_vectors = IntGauge::with_opts(Opts::new(
            "omendb_cold_tier_vectors",
            "Number of vectors in cold tier",
        ))
        .map_err(|e| Error::internal(format!("Failed to create cold_tier_vectors metric: {}", e)))?;

        // Initialize business metrics
        let tenants_active = IntGauge::with_opts(Opts::new(
            "omendb_tenants_active",
            "Number of active tenants",
        ))
        .map_err(|e| Error::internal(format!("Failed to create tenants_active metric: {}", e)))?;

        let tenant_requests_by_tier = CounterVec::new(
            Opts::new("omendb_tenant_requests_by_tier", "Requests by subscription tier"),
            &["tier", "operation"],
        )
        .map_err(|e| Error::internal(format!("Failed to create tenant_requests_by_tier metric: {}", e)))?;

        let billable_queries = IntCounter::with_opts(Opts::new(
            "omendb_billable_queries_total",
            "Total billable queries",
        ))
        .map_err(|e| Error::internal(format!("Failed to create billable_queries metric: {}", e)))?;

        let api_key_usage = IntCounter::with_opts(Opts::new(
            "omendb_api_key_usage_total",
            "Total API key usage",
        ))
        .map_err(|e| Error::internal(format!("Failed to create api_key_usage metric: {}", e)))?;

        // Initialize resource metrics
        let memory_usage_bytes = Gauge::with_opts(Opts::new(
            "omendb_memory_usage_bytes",
            "Memory usage in bytes",
        ))
        .map_err(|e| Error::internal(format!("Failed to create memory_usage_bytes metric: {}", e)))?;

        let cpu_usage_percent = Gauge::with_opts(Opts::new(
            "omendb_cpu_usage_percent",
            "CPU usage percentage",
        ))
        .map_err(|e| Error::internal(format!("Failed to create cpu_usage_percent metric: {}", e)))?;

        let disk_usage_bytes = Gauge::with_opts(Opts::new(
            "omendb_disk_usage_bytes",
            "Disk usage in bytes",
        ))
        .map_err(|e| Error::internal(format!("Failed to create disk_usage_bytes metric: {}", e)))?;

        let network_bytes_total = CounterVec::new(
            Opts::new("omendb_network_bytes_total", "Total network bytes by direction"),
            &["direction"],
        )
        .map_err(|e| Error::internal(format!("Failed to create network_bytes_total metric: {}", e)))?;

        // Register all metrics
        registry.register(Box::new(requests_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register requests_total: {}", e)))?;
        registry.register(Box::new(request_duration.clone()))
            .map_err(|e| Error::internal(format!("Failed to register request_duration: {}", e)))?;
        registry.register(Box::new(active_connections.clone()))
            .map_err(|e| Error::internal(format!("Failed to register active_connections: {}", e)))?;
        registry.register(Box::new(errors_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register errors_total: {}", e)))?;
        registry.register(Box::new(vectors_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register vectors_total: {}", e)))?;
        registry.register(Box::new(searches_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register searches_total: {}", e)))?;
        registry.register(Box::new(adds_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register adds_total: {}", e)))?;
        registry.register(Box::new(query_latency.clone()))
            .map_err(|e| Error::internal(format!("Failed to register query_latency: {}", e)))?;
        registry.register(Box::new(engine_memory_bytes.clone()))
            .map_err(|e| Error::internal(format!("Failed to register engine_memory_bytes: {}", e)))?;
        registry.register(Box::new(hot_tier_vectors.clone()))
            .map_err(|e| Error::internal(format!("Failed to register hot_tier_vectors: {}", e)))?;
        registry.register(Box::new(warm_tier_vectors.clone()))
            .map_err(|e| Error::internal(format!("Failed to register warm_tier_vectors: {}", e)))?;
        registry.register(Box::new(cold_tier_vectors.clone()))
            .map_err(|e| Error::internal(format!("Failed to register cold_tier_vectors: {}", e)))?;
        registry.register(Box::new(tenants_active.clone()))
            .map_err(|e| Error::internal(format!("Failed to register tenants_active: {}", e)))?;
        registry.register(Box::new(tenant_requests_by_tier.clone()))
            .map_err(|e| Error::internal(format!("Failed to register tenant_requests_by_tier: {}", e)))?;
        registry.register(Box::new(billable_queries.clone()))
            .map_err(|e| Error::internal(format!("Failed to register billable_queries: {}", e)))?;
        registry.register(Box::new(api_key_usage.clone()))
            .map_err(|e| Error::internal(format!("Failed to register api_key_usage: {}", e)))?;
        registry.register(Box::new(memory_usage_bytes.clone()))
            .map_err(|e| Error::internal(format!("Failed to register memory_usage_bytes: {}", e)))?;
        registry.register(Box::new(cpu_usage_percent.clone()))
            .map_err(|e| Error::internal(format!("Failed to register cpu_usage_percent: {}", e)))?;
        registry.register(Box::new(disk_usage_bytes.clone()))
            .map_err(|e| Error::internal(format!("Failed to register disk_usage_bytes: {}", e)))?;
        registry.register(Box::new(network_bytes_total.clone()))
            .map_err(|e| Error::internal(format!("Failed to register network_bytes_total: {}", e)))?;

        Ok(MetricsCollector {
            config,
            registry,
            request_metrics: RequestMetrics {
                requests_total,
                request_duration,
                active_connections,
                errors_total,
            },
            engine_metrics: EngineMetrics {
                vectors_total,
                searches_total,
                adds_total,
                query_latency,
                engine_memory_bytes,
                hot_tier_vectors,
                warm_tier_vectors,
                cold_tier_vectors,
            },
            business_metrics: BusinessMetrics {
                tenants_active,
                tenant_requests_by_tier,
                billable_queries,
                api_key_usage,
            },
            resource_metrics: ResourceMetrics {
                memory_usage_bytes,
                cpu_usage_percent,
                disk_usage_bytes,
                network_bytes_total,
            },
            tenant_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Record an HTTP request
    #[instrument(level = "debug", skip(self))]
    pub fn record_request(&self, duration_seconds: f64, method: &str, status_code: u16) {
        self.request_metrics.requests_total.inc();
        self.request_metrics.request_duration.observe(duration_seconds);

        debug!(
            "Recorded request: {} {} - {:.3}s",
            method, status_code, duration_seconds
        );
    }

    /// Record an error
    #[instrument(level = "debug", skip(self))]
    pub fn record_error(&self, error_type: &str) {
        self.request_metrics
            .errors_total
            .with_label_values(&[error_type])
            .inc();
        debug!("Recorded error: {}", error_type);
    }

    /// Record a search operation
    #[instrument(level = "debug", skip(self))]
    pub fn record_search(&self, tenant: &TenantContext, latency_seconds: f64, results_count: usize) {
        self.engine_metrics.searches_total.inc();
        self.engine_metrics.query_latency.observe(latency_seconds);

        // Record billable query
        self.business_metrics.billable_queries.inc();

        // Record tenant-specific metrics
        self.record_tenant_activity(tenant, "search");

        // Record by tier
        let tier = match tenant.tier {
            crate::types::SubscriptionTier::Free => "free",
            crate::types::SubscriptionTier::Platform => "platform",
            crate::types::SubscriptionTier::Enterprise => "enterprise",
        };
        
        self.business_metrics
            .tenant_requests_by_tier
            .with_label_values(&[tier, "search"])
            .inc();

        debug!(
            "Recorded search: tenant={}, latency={:.3}s, results={}",
            tenant.tenant_id, latency_seconds, results_count
        );
    }

    /// Record an add operation
    #[instrument(level = "debug", skip(self))]
    pub fn record_add(&self, tenant: &TenantContext) {
        self.engine_metrics.adds_total.inc();
        self.record_tenant_activity(tenant, "add");

        let tier = match tenant.tier {
            crate::types::SubscriptionTier::Free => "free",
            crate::types::SubscriptionTier::Platform => "platform",
            crate::types::SubscriptionTier::Enterprise => "enterprise",
        };
        
        self.business_metrics
            .tenant_requests_by_tier
            .with_label_values(&[tier, "add"])
            .inc();

        debug!("Recorded add: tenant={}", tenant.tenant_id);
    }

    /// Update engine statistics
    #[instrument(level = "debug", skip(self))]
    pub fn update_engine_stats(&self, stats: &EngineStats) {
        self.engine_metrics.vectors_total.set(stats.vector_count as i64);
        self.engine_metrics.engine_memory_bytes.set(stats.memory_usage_bytes as f64);
        self.engine_metrics.hot_tier_vectors.set(stats.hot_vectors as i64);
        self.engine_metrics.warm_tier_vectors.set(stats.warm_vectors as i64);
        self.engine_metrics.cold_tier_vectors.set(stats.cold_vectors as i64);

        debug!("Updated engine stats: {} vectors, {} MB memory", 
               stats.vector_count, stats.memory_usage_bytes / 1024 / 1024);
    }

    /// Update resource usage
    #[instrument(level = "debug", skip(self))]
    pub fn update_resource_usage(&self, memory_bytes: u64, cpu_percent: f64, disk_bytes: u64) {
        self.resource_metrics.memory_usage_bytes.set(memory_bytes as f64);
        self.resource_metrics.cpu_usage_percent.set(cpu_percent);
        self.resource_metrics.disk_usage_bytes.set(disk_bytes as f64);

        debug!(
            "Updated resource usage: {} MB memory, {:.1}% CPU, {} MB disk",
            memory_bytes / 1024 / 1024,
            cpu_percent,
            disk_bytes / 1024 / 1024
        );
    }

    /// Record network traffic
    pub fn record_network_bytes(&self, bytes: u64, direction: &str) {
        self.resource_metrics
            .network_bytes_total
            .with_label_values(&[direction])
            .inc_by(bytes as f64);
    }

    /// Update active connections count
    pub fn set_active_connections(&self, count: i64) {
        self.request_metrics.active_connections.set(count);
    }

    /// Update active tenants count
    pub fn set_active_tenants(&self, count: i64) {
        self.business_metrics.tenants_active.set(count);
    }

    /// Record API key usage
    pub fn record_api_key_usage(&self, tenant: &TenantContext) {
        self.business_metrics.api_key_usage.inc();
        self.record_tenant_activity(tenant, "api_key");
    }

    /// Record tenant activity
    fn record_tenant_activity(&self, tenant: &TenantContext, activity_type: &str) {
        let tenant_id = tenant.tenant_id;
        let activity_type = activity_type.to_string();
        
        tokio::spawn({
            let tenant_metrics = Arc::clone(&self.tenant_metrics);
            async move {
                let mut metrics = tenant_metrics.write().await;
                let tenant_metrics = metrics.entry(tenant_id).or_insert_with(|| TenantMetrics {
                    tenant_id,
                    requests: 0,
                    vectors: 0,
                    queries: 0,
                    errors: 0,
                    last_activity: chrono::Utc::now(),
                });

                tenant_metrics.requests += 1;
                tenant_metrics.last_activity = chrono::Utc::now();

                match activity_type.as_str() {
                    "search" => tenant_metrics.queries += 1,
                    "add" => tenant_metrics.vectors += 1,
                    "error" => tenant_metrics.errors += 1,
                    _ => {}
                }
            }
        });
    }

    /// Get tenant metrics
    pub async fn get_tenant_metrics(&self, tenant_id: uuid::Uuid) -> Option<TenantMetrics> {
        let metrics = self.tenant_metrics.read().await;
        metrics.get(&tenant_id).cloned()
    }

    /// Get all tenant metrics
    pub async fn get_all_tenant_metrics(&self) -> Vec<TenantMetrics> {
        let metrics = self.tenant_metrics.read().await;
        metrics.values().cloned().collect()
    }

    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String> {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)
            .map_err(|e| Error::internal(format!("Failed to encode metrics: {}", e)))?;
        
        String::from_utf8(buffer)
            .map_err(|e| Error::internal(format!("Failed to convert metrics to string: {}", e)))
    }

    /// Get metrics registry for custom exporters
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Health check for metrics collector
    pub fn health_check(&self) -> Result<()> {
        // Try to export metrics to verify everything works
        let _metrics = self.export_metrics()?;
        debug!("Metrics collector health check passed");
        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &MetricsConfig {
        &self.config
    }
}

/// Metrics middleware for HTTP requests
pub async fn metrics_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let start_time = std::time::Instant::now();
    let method = request.method().to_string();
    
    let response = next.run(request).await;
    
    let duration = start_time.elapsed().as_secs_f64();
    let status_code = response.status().as_u16();
    
    // In a real implementation, you'd get the metrics collector from app state
    // metrics_collector.record_request(duration, &method, status_code);
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricsConfig;
    use crate::types::{Permission, SubscriptionTier, TenantUsage};
    use std::time::Duration;

    fn create_test_config() -> MetricsConfig {
        MetricsConfig {
            enabled: true,
            port: 9091,
            collection_interval: Duration::from_secs(60),
            enable_engine_metrics: true,
        }
    }

    fn create_test_tenant() -> TenantContext {
        TenantContext {
            tenant_id: uuid::Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            tier: SubscriptionTier::Platform,
            usage: TenantUsage {
                vectors_stored: 1000,
                queries_this_hour: 50,
                bandwidth_this_month: 1024 * 1024,
                storage_used_bytes: 100 * 1024 * 1024,
            },
            permissions: vec![Permission::Read, Permission::Write],
        }
    }

    #[test]
    fn test_metrics_collector_creation() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config);
        assert!(collector.is_ok());
    }

    #[test]
    fn test_metrics_export() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        
        // Record some metrics
        collector.record_request(0.1, "GET", 200);
        collector.record_error("validation");
        
        let metrics_text = collector.export_metrics().unwrap();
        assert!(metrics_text.contains("omendb_requests_total"));
        assert!(metrics_text.contains("omendb_errors_total"));
    }

    #[tokio::test]
    async fn test_tenant_metrics() {
        let config = create_test_config();
        let collector = MetricsCollector::new(config).unwrap();
        let tenant = create_test_tenant();
        
        // Record some tenant activity
        collector.record_search(&tenant, 0.05, 10);
        collector.record_add(&tenant);
        
        // Give time for async operations
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let tenant_metrics = collector.get_tenant_metrics(tenant.tenant_id).await;
        assert!(tenant_metrics.is_some());
        
        let metrics = tenant_metrics.unwrap();
        assert_eq!(metrics.tenant_id, tenant.tenant_id);
        assert!(metrics.requests > 0);
    }
}