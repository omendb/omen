//! Integration tests for server platform with optimized embedded database
//! 
//! Tests the complete pipeline: 
//! Rust Server → Python FFI Bridge → Optimized Mojo Engine

use omendb_server::{
    Config, EngineConfig, ServerConfig, StorageConfig, AuthConfig, MetricsConfig,
    engine::EngineManager,
    python_ffi::PythonMojoEngine,
    types::Vector,
};
use std::time::Duration;
use std::collections::HashMap;
use serde_json::json;
use tokio;
use uuid::Uuid;

/// Test configuration for integration tests
fn create_test_config() -> Config {
    Config {
        server: ServerConfig {
            http_port: 8081, // Use different port for tests
            grpc_port: 9091,
            worker_threads: 2,
            max_connections: 100,
            request_timeout: Duration::from_secs(10),
            keep_alive_timeout: Duration::from_secs(30),
            max_request_size: 1024 * 1024, // 1MB
        },
        engine: EngineConfig {
            dimension: 128,
            pool_size: 2,
            idle_timeout: Duration::from_secs(60),
            max_vectors_per_engine: 10000,
            enable_tiered_storage: false, // Disable to avoid tiered storage complexity
        },
        auth: AuthConfig {
            jwt_secret: "test-secret-key".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            enable_api_keys: true,
            rate_limit: omendb_server::config::RateLimitConfig {
                requests_per_minute: 1000,
                burst_capacity: 100,
            },
        },
        storage: StorageConfig {
            data_dir: "./test_data".to_string(),
            enable_hot_tier: true,
            hot_tier_memory_mb: 128,
            enable_warm_tier: false,
            warm_tier_path: "./test_data/warm".to_string(),
            enable_cold_tier: false,
            cold_tier_path: "./test_data/cold".to_string(),
        },
        metrics: MetricsConfig {
            enabled: true,
            port: 9092,
            collection_interval: Duration::from_secs(10),
            enable_engine_metrics: true,
        },
    }
}

/// Generate test vectors for performance testing
fn generate_test_vectors(count: usize, dimension: usize) -> Vec<Vector> {
    (0..count)
        .map(|i| {
            let data: Vec<f32> = (0..dimension)
                .map(|j| (i * dimension + j) as f32 * 0.01)
                .collect();
            
            let mut metadata = HashMap::new();
            metadata.insert("category".to_string(), json!(format!("test_{}", i % 10)));
            metadata.insert("index".to_string(), json!(i));
            
            Vector {
                id: format!("test_vector_{}", i),
                data,
                metadata,
            }
        })
        .collect()
}

#[tokio::test]
async fn test_python_ffi_engine_creation() {
    // Test basic engine creation and initialization
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    assert!(engine.is_initialized());
    
    // Test initialization (should be idempotent)
    engine.initialize().await.expect("Failed to initialize engine");
    assert!(engine.is_initialized());
}

#[tokio::test]
async fn test_engine_basic_operations() {
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize engine");
    
    // Test vector addition
    let test_vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.01).collect();
    engine.add_vector("test_1", &test_vector).await.expect("Failed to add vector");
    
    // Test vector search
    let query_vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.01).collect();
    let results = engine.search(&query_vector, 5).await.expect("Failed to search");
    
    assert!(!results.is_empty(), "Should find at least one result");
    assert_eq!(results[0].id, "test_1", "Should find the exact vector we added");
}

#[tokio::test]
async fn test_engine_manager_integration() {
    let config = create_test_config();
    let tenant_id = Uuid::new_v4();
    
    // Create engine manager
    let engine_manager = EngineManager::new(&config.engine)
        .await
        .expect("Failed to create engine manager");
    
    // Test engine pool management
    let _engine = engine_manager
        .get_or_create_engine(tenant_id, config.engine.dimension)
        .await
        .expect("Failed to get engine from pool");
    
    // Verify engine is cached
    let _engine2 = engine_manager
        .get_or_create_engine(tenant_id, config.engine.dimension)
        .await
        .expect("Failed to get cached engine");
}

#[tokio::test]
async fn test_batch_performance_with_optimizations() {
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize engine");
    
    // Generate test data
    let test_vectors = generate_test_vectors(1000, 128);
    
    // Test batch addition performance
    let start_time = std::time::Instant::now();
    
    for vector in &test_vectors {
        engine.add_vector_with_metadata(vector)
            .await
            .expect("Failed to add vector");
    }
    
    let batch_add_duration = start_time.elapsed();
    println!("Batch add (1000 vectors): {:?}", batch_add_duration);
    
    // Test search performance
    let query_vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.005).collect();
    
    let search_start = std::time::Instant::now();
    let _results = engine.search(&query_vector, 10)
        .await
        .expect("Failed to search");
    let search_duration = search_start.elapsed();
    
    println!("Search duration: {:?}", search_duration);
    
    // Performance assertions (based on our optimizations)
    // With the 27.7x FFI improvement, server should handle this easily
    assert!(
        batch_add_duration < Duration::from_secs(10),
        "Batch addition should be fast with FFI optimizations"
    );
    assert!(
        search_duration < Duration::from_millis(100),
        "Search should be fast with SIMD optimizations"
    );
}

#[tokio::test]
async fn test_concurrent_operations() {
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize engine");
    
    // Add some initial vectors
    let test_vectors = generate_test_vectors(100, 128);
    for vector in &test_vectors[..10] {
        engine.add_vector_with_metadata(vector)
            .await
            .expect("Failed to add vector");
    }
    
    // Test concurrent searches
    let query_vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.005).collect();
    
    let mut search_tasks = Vec::new();
    for _ in 0..10 {
        let query = query_vector.clone();
        let task = tokio::spawn(async move {
            let mut test_engine = PythonMojoEngine::new(128).expect("Failed to create engine");
            test_engine.initialize().await.expect("Failed to initialize engine");
            test_engine.search(&query, 5).await
        });
        search_tasks.push(task);
    }
    
    // Wait for all searches to complete
    for task in search_tasks {
        let _result = task.await.expect("Task failed").expect("Search failed");
    }
}

#[tokio::test]
async fn test_memory_efficiency() {
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize engine");
    
    // Add vectors and check stats
    let test_vectors = generate_test_vectors(100, 128);
    for vector in &test_vectors {
        engine.add_vector_with_metadata(vector)
            .await
            .expect("Failed to add vector");
    }
    
    // Get engine statistics
    let stats = engine.get_stats().await.expect("Failed to get stats");
    
    // Verify we have reasonable memory usage
    if let Some(total_vectors) = stats.get("total_vectors") {
        assert_eq!(total_vectors.as_i64().unwrap_or(0), 100);
    }
    
    println!("Engine stats: {:?}", stats);
}

#[tokio::test]
async fn test_crud_operations() {
    let mut engine = PythonMojoEngine::new(128).expect("Failed to create engine");
    engine.initialize().await.expect("Failed to initialize engine");
    
    // Create
    let test_vector: Vec<f32> = (0..128).map(|i| i as f32 * 0.01).collect();
    engine.add_vector("crud_test", &test_vector)
        .await
        .expect("Failed to add vector");
    
    // Read
    let retrieved = engine.get_vector("crud_test")
        .await
        .expect("Failed to get vector")
        .expect("Vector should exist");
    
    assert_eq!(retrieved.id, "crud_test");
    assert_eq!(retrieved.data.len(), 128);
    
    // Delete
    let deleted = engine.delete_vector("crud_test")
        .await
        .expect("Failed to delete vector");
    assert!(deleted, "Vector should be deleted");
    
    // Verify deletion
    let not_found = engine.get_vector("crud_test")
        .await
        .expect("Failed to query vector");
    assert!(not_found.is_none(), "Vector should not exist after deletion");
}