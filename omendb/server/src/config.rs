//! Configuration management for OmenDB Server
//! 
//! Handles loading and validation of server configuration from files,
//! environment variables, and command-line arguments.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

/// Helper module for deserializing Duration from seconds
mod duration_seconds {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;
    
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(seconds))
    }
}

/// Main server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server-specific settings
    pub server: ServerConfig,
    /// Engine configuration
    pub engine: EngineConfig,
    /// Authentication settings
    pub auth: AuthConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Metrics and monitoring
    pub metrics: MetricsConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP server port
    pub http_port: u16,
    /// gRPC server port
    pub grpc_port: u16,
    /// Number of worker threads
    pub worker_threads: usize,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Request timeout duration
    #[serde(with = "duration_seconds")]
    pub request_timeout: Duration,
    /// Keep-alive timeout
    #[serde(with = "duration_seconds")]
    pub keep_alive_timeout: Duration,
    /// Maximum request body size in bytes
    pub max_request_size: usize,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Vector dimension
    pub dimension: i32,
    /// Engine pool size
    pub pool_size: usize,
    /// Engine idle timeout
    #[serde(with = "duration_seconds")]
    pub idle_timeout: Duration,
    /// Maximum vectors per engine
    pub max_vectors_per_engine: usize,
    /// Enable tiered storage by default
    pub enable_tiered_storage: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication (set to false for testing ONLY)
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT token expiration
    #[serde(with = "duration_seconds")]
    pub jwt_expiration: Duration,
    /// Enable API key authentication
    pub enable_api_keys: bool,
    /// Rate limiting settings
    pub rate_limit: RateLimitConfig,
}

fn default_auth_enabled() -> bool {
    true
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute per tenant
    pub requests_per_minute: u32,
    /// Burst capacity
    pub burst_capacity: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory
    pub data_dir: String,
    /// Enable hot tier
    pub enable_hot_tier: bool,
    /// Hot tier memory limit in MB
    pub hot_tier_memory_mb: usize,
    /// Enable warm tier
    pub enable_warm_tier: bool,
    /// Warm tier storage path
    pub warm_tier_path: String,
    /// Enable cold tier
    pub enable_cold_tier: bool,
    /// Cold tier storage path (can be S3 URL)
    pub cold_tier_path: String,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics server port
    pub port: u16,
    /// Collection interval
    #[serde(with = "duration_seconds")]
    pub collection_interval: Duration,
    /// Enable detailed engine metrics
    pub enable_engine_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                http_port: 8080,
                grpc_port: 9090,
                worker_threads: num_cpus::get(),
                max_connections: 1000,
                request_timeout: Duration::from_secs(30),
                keep_alive_timeout: Duration::from_secs(60),
                max_request_size: 10 * 1024 * 1024, // 10MB
            },
            engine: EngineConfig {
                dimension: 128,
                pool_size: 10,
                idle_timeout: Duration::from_secs(300),
                max_vectors_per_engine: 1_000_000,
                enable_tiered_storage: true,
            },
            auth: AuthConfig {
                enabled: true,
                jwt_secret: "your-secret-key".to_string(),
                jwt_expiration: Duration::from_secs(24 * 60 * 60),
                enable_api_keys: true,
                rate_limit: RateLimitConfig {
                    requests_per_minute: 1000,
                    burst_capacity: 100,
                },
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
                enable_hot_tier: true,
                hot_tier_memory_mb: 1024,
                enable_warm_tier: true,
                warm_tier_path: "./data/warm".to_string(),
                enable_cold_tier: false,
                cold_tier_path: "./data/cold".to_string(),
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9091,
                collection_interval: Duration::from_secs(60),
                enable_engine_metrics: true,
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Config::default();
        
        // Override with environment variables if present
        if let Ok(port) = std::env::var("OMENDB_HTTP_PORT") {
            config.server.http_port = port.parse()?;
        }
        
        if let Ok(port) = std::env::var("OMENDB_GRPC_PORT") {
            config.server.grpc_port = port.parse()?;
        }
        
        if let Ok(threads) = std::env::var("OMENDB_WORKER_THREADS") {
            config.server.worker_threads = threads.parse()?;
        }
        
        if let Ok(dimension) = std::env::var("OMENDB_DIMENSION") {
            config.engine.dimension = dimension.parse()?;
        }
        
        if let Ok(secret) = std::env::var("OMENDB_JWT_SECRET") {
            config.auth.jwt_secret = secret;
        }
        
        if let Ok(data_dir) = std::env::var("OMENDB_DATA_DIR") {
            config.storage.data_dir = data_dir;
        }
        
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.server.http_port == 0 {
            return Err(anyhow::anyhow!("HTTP port must be greater than 0"));
        }
        
        if self.server.grpc_port == 0 {
            return Err(anyhow::anyhow!("gRPC port must be greater than 0"));
        }
        
        if self.server.worker_threads == 0 {
            return Err(anyhow::anyhow!("Worker threads must be greater than 0"));
        }
        
        if self.engine.dimension <= 0 {
            return Err(anyhow::anyhow!("Vector dimension must be positive"));
        }
        
        if self.engine.pool_size == 0 {
            return Err(anyhow::anyhow!("Engine pool size must be greater than 0"));
        }
        
        if self.auth.jwt_secret.is_empty() {
            return Err(anyhow::anyhow!("JWT secret cannot be empty"));
        }
        
        Ok(())
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config.server.http_port, deserialized.server.http_port);
        assert_eq!(config.engine.dimension, deserialized.engine.dimension);
    }

    #[test]
    fn test_config_file_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");
        
        let config = Config::default();
        config.save_to_file(&file_path).unwrap();
        
        let loaded_config = Config::from_file(&file_path).unwrap();
        assert_eq!(config.server.http_port, loaded_config.server.http_port);
    }

    #[test]
    fn test_invalid_config() {
        let mut config = Config::default();
        config.server.http_port = 0;
        
        assert!(config.validate().is_err());
    }
}