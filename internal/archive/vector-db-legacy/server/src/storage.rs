//! Storage coordination for OmenDB Server
//! 
//! Coordinates tiered storage operations and manages data placement
//! across hot, warm, and cold storage tiers.

use crate::config::StorageConfig;
use crate::types::{TenantContext, Vector};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Storage coordinator that manages tiered storage operations
pub struct StorageCoordinator {
    /// Configuration
    config: StorageConfig,
    /// Hot tier storage (in-memory, fastest access)
    hot_tier: Arc<HotTierStorage>,
    /// Warm tier storage (SSD, medium latency)
    warm_tier: Arc<WarmTierStorage>,
    /// Cold tier storage (disk/S3, highest latency)
    cold_tier: Arc<ColdTierStorage>,
    /// Access pattern tracker
    access_tracker: Arc<RwLock<AccessTracker>>,
    /// Migration coordinator
    migration_coordinator: Arc<MigrationCoordinator>,
}

/// Hot tier storage (in-memory)
pub struct HotTierStorage {
    /// Current vectors in hot tier
    vectors: Arc<RwLock<HashMap<String, Vector>>>,
    /// Memory usage in bytes
    memory_usage: Arc<RwLock<u64>>,
    /// Maximum memory limit
    memory_limit: u64,
}

/// Warm tier storage (SSD-based)
pub struct WarmTierStorage {
    /// Storage path
    storage_path: PathBuf,
    /// Cached metadata
    metadata_cache: Arc<RwLock<HashMap<String, VectorMetadata>>>,
}

/// Cold tier storage (disk/S3)
pub struct ColdTierStorage {
    /// Storage path (local or S3 URL)
    storage_path: String,
    /// Local cache for frequently accessed cold data
    cache: Arc<RwLock<HashMap<String, Vector>>>,
    /// Cache size limit
    cache_limit: usize,
}

/// Vector metadata for storage management
#[derive(Debug, Clone)]
struct VectorMetadata {
    /// Vector ID
    id: String,
    /// Size in bytes
    size: u64,
    /// Current storage tier
    tier: StorageTier,
    /// Access count
    access_count: u64,
    /// Last access time
    last_access: chrono::DateTime<chrono::Utc>,
    /// Creation time
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Storage tier enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StorageTier {
    Hot,
    Warm,
    Cold,
}

/// Access pattern tracking
struct AccessTracker {
    /// Access patterns by vector ID
    patterns: HashMap<String, AccessPattern>,
    /// Global access statistics
    global_stats: AccessStats,
}

/// Access pattern for a specific vector
#[derive(Debug, Clone)]
struct AccessPattern {
    /// Vector ID
    vector_id: String,
    /// Total access count
    access_count: u64,
    /// Recent access timestamps
    recent_accesses: Vec<chrono::DateTime<chrono::Utc>>,
    /// Average time between accesses
    avg_interval: Option<chrono::Duration>,
    /// Predicted next access time
    predicted_next_access: Option<chrono::DateTime<chrono::Utc>>,
}

/// Global access statistics
#[derive(Debug, Clone, Default)]
struct AccessStats {
    /// Total accesses across all vectors
    total_accesses: u64,
    /// Hot tier hit rate
    hot_hit_rate: f64,
    /// Warm tier hit rate
    warm_hit_rate: f64,
    /// Average access latency by tier
    avg_latency_by_tier: HashMap<StorageTier, f64>,
}

/// Migration coordinator for moving data between tiers
pub struct MigrationCoordinator {
    /// Pending migrations
    pending_migrations: Arc<RwLock<Vec<MigrationTask>>>,
    /// Migration statistics
    migration_stats: Arc<RwLock<MigrationStats>>,
}

/// Migration task
#[derive(Debug, Clone)]
struct MigrationTask {
    /// Vector ID to migrate
    vector_id: String,
    /// Source tier
    from_tier: StorageTier,
    /// Destination tier
    to_tier: StorageTier,
    /// Priority (higher = more urgent)
    priority: u32,
    /// Scheduled time
    scheduled_at: chrono::DateTime<chrono::Utc>,
}

/// Migration statistics
#[derive(Debug, Clone, Default)]
struct MigrationStats {
    /// Total migrations performed
    total_migrations: u64,
    /// Migrations by direction
    migrations_by_direction: HashMap<(StorageTier, StorageTier), u64>,
    /// Average migration time
    avg_migration_time: f64,
    /// Failed migrations
    failed_migrations: u64,
}

impl StorageCoordinator {
    /// Create a new storage coordinator
    #[instrument(level = "info")]
    pub async fn new(config: StorageConfig) -> Result<Self> {
        info!("Initializing storage coordinator");

        // Initialize hot tier
        let hot_memory_limit = config.hot_tier_memory_mb * 1024 * 1024; // Convert MB to bytes
        let hot_tier = Arc::new(HotTierStorage {
            vectors: Arc::new(RwLock::new(HashMap::new())),
            memory_usage: Arc::new(RwLock::new(0)),
            memory_limit: hot_memory_limit as u64,
        });

        // Initialize warm tier
        let warm_path = PathBuf::from(&config.warm_tier_path);
        if config.enable_warm_tier {
            tokio::fs::create_dir_all(&warm_path).await
                .map_err(|e| Error::Storage(e))?;
        }
        let warm_tier = Arc::new(WarmTierStorage {
            storage_path: warm_path,
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
        });

        // Initialize cold tier
        let cold_tier = Arc::new(ColdTierStorage {
            storage_path: config.cold_tier_path.clone(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_limit: 1000, // 1000 vectors in cold cache
        });

        // Initialize access tracker
        let access_tracker = Arc::new(RwLock::new(AccessTracker {
            patterns: HashMap::new(),
            global_stats: AccessStats::default(),
        }));

        // Initialize migration coordinator
        let migration_coordinator = Arc::new(MigrationCoordinator {
            pending_migrations: Arc::new(RwLock::new(Vec::new())),
            migration_stats: Arc::new(RwLock::new(MigrationStats::default())),
        });

        info!("Storage coordinator initialized with tiers: hot={}, warm={}, cold={}",
              config.enable_hot_tier, config.enable_warm_tier, config.enable_cold_tier);

        Ok(StorageCoordinator {
            config,
            hot_tier,
            warm_tier,
            cold_tier,
            access_tracker,
            migration_coordinator,
        })
    }

    /// Store a vector, choosing the appropriate tier
    #[instrument(level = "debug", skip(self, vector))]
    pub async fn store_vector(&self, tenant: &TenantContext, vector: Vector) -> Result<()> {
        let vector_size = self.estimate_vector_size(&vector);
        let vector_id = vector.id.clone();
        
        // For new vectors, start in hot tier if there's space
        if self.config.enable_hot_tier && self.hot_tier_has_space(vector_size).await {
            self.store_in_hot_tier(vector).await?;
            debug!("Stored vector {} in hot tier", vector_id);
        } else if self.config.enable_warm_tier {
            self.store_in_warm_tier(vector).await?;
            debug!("Stored vector {} in warm tier", vector_id);
        } else {
            self.store_in_cold_tier(vector).await?;
            debug!("Stored vector {} in cold tier", vector_id);
        }

        // Schedule background optimization
        self.schedule_tier_optimization().await;

        Ok(())
    }

    /// Retrieve a vector from any tier
    #[instrument(level = "debug", skip(self))]
    pub async fn get_vector(&self, tenant: &TenantContext, vector_id: &str) -> Result<Option<Vector>> {
        // Track access for migration decisions
        self.record_access(vector_id).await;

        // Try hot tier first
        if let Some(vector) = self.get_from_hot_tier(vector_id).await? {
            debug!("Retrieved vector {} from hot tier", vector_id);
            return Ok(Some(vector));
        }

        // Try warm tier
        if let Some(vector) = self.get_from_warm_tier(vector_id).await? {
            debug!("Retrieved vector {} from warm tier", vector_id);
            
            // Consider promoting to hot tier if frequently accessed
            self.consider_hot_promotion(vector_id).await;
            
            return Ok(Some(vector));
        }

        // Try cold tier
        if let Some(vector) = self.get_from_cold_tier(vector_id).await? {
            debug!("Retrieved vector {} from cold tier", vector_id);
            
            // Consider promoting to warm tier
            self.consider_warm_promotion(vector_id).await;
            
            return Ok(Some(vector));
        }

        debug!("Vector {} not found in any tier", vector_id);
        Ok(None)
    }

    /// Delete a vector from all tiers
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_vector(&self, tenant: &TenantContext, vector_id: &str) -> Result<bool> {
        let mut deleted = false;

        // Delete from hot tier
        if self.delete_from_hot_tier(vector_id).await? {
            deleted = true;
        }

        // Delete from warm tier
        if self.delete_from_warm_tier(vector_id).await? {
            deleted = true;
        }

        // Delete from cold tier
        if self.delete_from_cold_tier(vector_id).await? {
            deleted = true;
        }

        // Remove from access tracking
        {
            let mut tracker = self.access_tracker.write().await;
            tracker.patterns.remove(vector_id);
        }

        debug!("Deleted vector {} from storage (found: {})", vector_id, deleted);
        Ok(deleted)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> StorageStats {
        let hot_count = self.hot_tier.vectors.read().await.len();
        let warm_count = self.warm_tier.metadata_cache.read().await.len();
        let cold_count = self.cold_tier.cache.read().await.len(); // This is just cache, not actual count

        let hot_memory = *self.hot_tier.memory_usage.read().await;
        let access_stats = self.access_tracker.read().await.global_stats.clone();

        StorageStats {
            hot_tier_vectors: hot_count,
            warm_tier_vectors: warm_count,
            cold_tier_vectors: cold_count,
            hot_tier_memory_bytes: hot_memory,
            total_accesses: access_stats.total_accesses,
            hot_hit_rate: access_stats.hot_hit_rate,
            warm_hit_rate: access_stats.warm_hit_rate,
        }
    }

    /// Start background migration tasks
    pub async fn start_background_tasks(&self) {
        let migration_coordinator = Arc::clone(&self.migration_coordinator);
        let access_tracker = Arc::clone(&self.access_tracker);
        let hot_tier = Arc::clone(&self.hot_tier);
        let warm_tier = Arc::clone(&self.warm_tier);
        let cold_tier = Arc::clone(&self.cold_tier);

        // Migration task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                Self::process_migrations(
                    &migration_coordinator,
                    &hot_tier,
                    &warm_tier,
                    &cold_tier,
                ).await;
            }
        });

        // Access pattern analysis task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600)); // 10 minutes
            loop {
                interval.tick().await;
                Self::analyze_access_patterns(&access_tracker).await;
            }
        });
    }

    // Private helper methods

    async fn hot_tier_has_space(&self, vector_size: u64) -> bool {
        let current_usage = *self.hot_tier.memory_usage.read().await;
        current_usage + vector_size <= self.hot_tier.memory_limit
    }

    async fn store_in_hot_tier(&self, vector: Vector) -> Result<()> {
        let vector_size = self.estimate_vector_size(&vector);
        let vector_id = vector.id.clone();

        {
            let mut vectors = self.hot_tier.vectors.write().await;
            let mut memory_usage = self.hot_tier.memory_usage.write().await;
            
            vectors.insert(vector_id, vector);
            *memory_usage += vector_size;
        }

        Ok(())
    }

    async fn store_in_warm_tier(&self, vector: Vector) -> Result<()> {
        // In a real implementation, this would write to disk
        let metadata = VectorMetadata {
            id: vector.id.clone(),
            size: self.estimate_vector_size(&vector),
            tier: StorageTier::Warm,
            access_count: 0,
            last_access: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        let mut cache = self.warm_tier.metadata_cache.write().await;
        cache.insert(vector.id.clone(), metadata);

        // TODO: Actually write vector to disk
        Ok(())
    }

    async fn store_in_cold_tier(&self, vector: Vector) -> Result<()> {
        // In a real implementation, this would write to S3 or cold storage
        // For now, just add to cache
        let mut cache = self.cold_tier.cache.write().await;
        
        // Evict oldest if at capacity
        if cache.len() >= self.cold_tier.cache_limit {
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }
        
        cache.insert(vector.id.clone(), vector);
        Ok(())
    }

    async fn get_from_hot_tier(&self, vector_id: &str) -> Result<Option<Vector>> {
        let vectors = self.hot_tier.vectors.read().await;
        Ok(vectors.get(vector_id).cloned())
    }

    async fn get_from_warm_tier(&self, vector_id: &str) -> Result<Option<Vector>> {
        // Check metadata cache first
        {
            let cache = self.warm_tier.metadata_cache.read().await;
            if !cache.contains_key(vector_id) {
                return Ok(None);
            }
        }

        // TODO: Actually read from disk
        // For now, return None
        Ok(None)
    }

    async fn get_from_cold_tier(&self, vector_id: &str) -> Result<Option<Vector>> {
        let cache = self.cold_tier.cache.read().await;
        Ok(cache.get(vector_id).cloned())
    }

    async fn delete_from_hot_tier(&self, vector_id: &str) -> Result<bool> {
        let mut vectors = self.hot_tier.vectors.write().await;
        let removed = vectors.remove(vector_id);
        
        if let Some(vector) = removed {
            let mut memory_usage = self.hot_tier.memory_usage.write().await;
            *memory_usage = memory_usage.saturating_sub(self.estimate_vector_size(&vector));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_from_warm_tier(&self, vector_id: &str) -> Result<bool> {
        let mut cache = self.warm_tier.metadata_cache.write().await;
        Ok(cache.remove(vector_id).is_some())
    }

    async fn delete_from_cold_tier(&self, vector_id: &str) -> Result<bool> {
        let mut cache = self.cold_tier.cache.write().await;
        Ok(cache.remove(vector_id).is_some())
    }

    fn estimate_vector_size(&self, vector: &Vector) -> u64 {
        // Rough estimate: vector data + metadata + overhead
        let vector_bytes = vector.data.len() * 4; // 4 bytes per f32
        let metadata_bytes = 1024; // Rough estimate for metadata
        let overhead_bytes = 256; // Object overhead
        
        (vector_bytes + metadata_bytes + overhead_bytes) as u64
    }

    async fn record_access(&self, vector_id: &str) {
        let mut tracker = self.access_tracker.write().await;
        tracker.global_stats.total_accesses += 1;

        let pattern = tracker.patterns.entry(vector_id.to_string()).or_insert_with(|| {
            AccessPattern {
                vector_id: vector_id.to_string(),
                access_count: 0,
                recent_accesses: Vec::new(),
                avg_interval: None,
                predicted_next_access: None,
            }
        });

        pattern.access_count += 1;
        pattern.recent_accesses.push(chrono::Utc::now());

        // Keep only recent accesses (last 100)
        if pattern.recent_accesses.len() > 100 {
            pattern.recent_accesses.drain(0..pattern.recent_accesses.len() - 100);
        }
    }

    async fn consider_hot_promotion(&self, vector_id: &str) {
        // Simple promotion logic: if accessed more than 10 times recently
        let tracker = self.access_tracker.read().await;
        if let Some(pattern) = tracker.patterns.get(vector_id) {
            if pattern.access_count > 10 && self.hot_tier_has_space(4096).await {
                // Schedule migration to hot tier
                drop(tracker);
                self.schedule_migration(vector_id, StorageTier::Warm, StorageTier::Hot, 100).await;
            }
        }
    }

    async fn consider_warm_promotion(&self, vector_id: &str) {
        // Simple promotion logic: if accessed more than 3 times recently
        let tracker = self.access_tracker.read().await;
        if let Some(pattern) = tracker.patterns.get(vector_id) {
            if pattern.access_count > 3 {
                drop(tracker);
                self.schedule_migration(vector_id, StorageTier::Cold, StorageTier::Warm, 50).await;
            }
        }
    }

    async fn schedule_migration(&self, vector_id: &str, from: StorageTier, to: StorageTier, priority: u32) {
        let task = MigrationTask {
            vector_id: vector_id.to_string(),
            from_tier: from,
            to_tier: to,
            priority,
            scheduled_at: chrono::Utc::now(),
        };

        let mut migrations = self.migration_coordinator.pending_migrations.write().await;
        migrations.push(task);
        migrations.sort_by(|a, b| b.priority.cmp(&a.priority)); // Sort by priority descending
    }

    async fn schedule_tier_optimization(&self) {
        // This would analyze current tier distribution and schedule optimizations
        debug!("Scheduled tier optimization");
    }

    async fn process_migrations(
        migration_coordinator: &MigrationCoordinator,
        _hot_tier: &HotTierStorage,
        _warm_tier: &WarmTierStorage,
        _cold_tier: &ColdTierStorage,
    ) {
        let mut migrations = migration_coordinator.pending_migrations.write().await;
        if migrations.is_empty() {
            return;
        }

        // Process up to 10 migrations per batch
        let batch_size = std::cmp::min(10, migrations.len());
        let batch: Vec<_> = migrations.drain(0..batch_size).collect();
        drop(migrations);

        for task in batch {
            debug!("Processing migration: {} from {:?} to {:?}", 
                   task.vector_id, task.from_tier, task.to_tier);
            
            // TODO: Implement actual migration logic
            // This would involve moving vectors between storage tiers
        }
    }

    async fn analyze_access_patterns(access_tracker: &RwLock<AccessTracker>) {
        let tracker = access_tracker.write().await;
        
        // Calculate hit rates and other statistics
        let total_accesses = tracker.global_stats.total_accesses;
        if total_accesses > 0 {
            // Update global statistics based on access patterns
            debug!("Analyzed access patterns: {} total accesses", total_accesses);
        }
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub hot_tier_vectors: usize,
    pub warm_tier_vectors: usize,
    pub cold_tier_vectors: usize,
    pub hot_tier_memory_bytes: u64,
    pub total_accesses: u64,
    pub hot_hit_rate: f64,
    pub warm_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;
    use crate::types::{Permission, SubscriptionTier, TenantUsage};

    fn create_test_config() -> StorageConfig {
        StorageConfig {
            data_dir: "./test_data".to_string(),
            enable_hot_tier: true,
            hot_tier_memory_mb: 100,
            enable_warm_tier: true,
            warm_tier_path: "./test_data/warm".to_string(),
            enable_cold_tier: true,
            cold_tier_path: "./test_data/cold".to_string(),
        }
    }

    fn create_test_tenant() -> TenantContext {
        TenantContext {
            tenant_id: uuid::Uuid::new_v4(),
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
    async fn test_storage_coordinator_creation() {
        let config = create_test_config();
        let coordinator = StorageCoordinator::new(config).await;
        assert!(coordinator.is_ok());
    }

    #[tokio::test]
    async fn test_vector_storage_and_retrieval() {
        let config = create_test_config();
        let coordinator = StorageCoordinator::new(config).await.unwrap();
        let tenant = create_test_tenant();

        let vector = Vector::new("test_vector".to_string(), vec![1.0, 2.0, 3.0, 4.0]);
        
        // Store vector
        let result = coordinator.store_vector(&tenant, vector.clone()).await;
        assert!(result.is_ok());

        // Retrieve vector
        let retrieved = coordinator.get_vector(&tenant, "test_vector").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test_vector");
    }

    #[tokio::test]
    async fn test_vector_deletion() {
        let config = create_test_config();
        let coordinator = StorageCoordinator::new(config).await.unwrap();
        let tenant = create_test_tenant();

        let vector = Vector::new("delete_test".to_string(), vec![1.0, 2.0]);
        
        // Store and then delete
        coordinator.store_vector(&tenant, vector).await.unwrap();
        let deleted = coordinator.delete_vector(&tenant, "delete_test").await.unwrap();
        assert!(deleted);

        // Should not be retrievable
        let retrieved = coordinator.get_vector(&tenant, "delete_test").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let config = create_test_config();
        let coordinator = StorageCoordinator::new(config).await.unwrap();
        
        let stats = coordinator.get_storage_stats().await;
        assert_eq!(stats.hot_tier_vectors, 0);
        assert_eq!(stats.hot_tier_memory_bytes, 0);
    }
}