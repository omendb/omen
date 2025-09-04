//! Core types for OmenDB Server
//! 
//! Defines the main data structures used throughout the server,
//! including vectors, search results, and tenant information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Security constants to prevent protocol-level attacks
/// Maximum size for any input data (1GB) - prevents protocol overflow attacks
pub const MAX_INPUT_SIZE: usize = 1_073_741_824; // 1GB

/// Maximum vector dimension we support
pub const MAX_VECTOR_DIMENSION: usize = 2048;

/// Maximum metadata size (1MB)
pub const MAX_METADATA_SIZE: usize = 1_048_576; // 1MB

/// Maximum batch size for operations
pub const MAX_BATCH_SIZE: usize = 10_000;

/// Unique identifier for vectors
pub type VectorId = String;

/// Unique identifier for tenants
pub type TenantId = Uuid;

/// Unique identifier for collections
pub type CollectionId = String;

/// Vector data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Unique identifier for this vector
    pub id: VectorId,
    /// Vector data as array of floats
    pub data: Vec<f32>,
    /// Optional metadata associated with this vector
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Search result containing a vector and its distance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Vector ID
    pub id: VectorId,
    /// Distance/similarity score
    pub distance: f32,
    /// Vector metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Actual vector data (optional, for efficiency)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
}

/// Search request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    /// Query vector
    pub vector: Vec<f32>,
    /// Number of results to return
    #[serde(default = "default_top_k")]
    pub top_k: i32,
    /// Optional metadata filters
    #[serde(default)]
    pub filter: HashMap<String, serde_json::Value>,
    /// Include vector data in results
    #[serde(default)]
    pub include_vector: bool,
    /// Collection to search in (optional, defaults to default collection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionId>,
}

fn default_top_k() -> i32 {
    10
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Query execution time in milliseconds
    pub query_time_ms: f64,
    /// Number of vectors searched
    pub vectors_searched: usize,
}

/// Request to add a single vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVectorRequest {
    /// Vector to add
    pub vector: Vector,
    /// Collection to add to (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionId>,
}

/// Response for adding a vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVectorResponse {
    /// ID of the added vector
    pub id: VectorId,
    /// Whether this was a new vector or update
    pub created: bool,
}

/// Request to add multiple vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAddRequest {
    /// Vectors to add
    pub vectors: Vec<Vector>,
    /// Collection to add to (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<CollectionId>,
}

/// Response for batch add operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAddResponse {
    /// Number of vectors successfully added
    pub added: usize,
    /// Number of vectors updated
    pub updated: usize,
    /// Any errors that occurred
    pub errors: Vec<BatchError>,
}

/// Error in batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Vector ID that caused the error
    pub vector_id: VectorId,
    /// Error message
    pub error: String,
}

/// Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    /// Collection identifier
    pub id: CollectionId,
    /// Human-readable name
    pub name: String,
    /// Collection description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Vector dimension for this collection
    pub dimension: i32,
    /// Number of vectors in collection
    pub vector_count: usize,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Collection metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Request to create a collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    /// Collection identifier
    pub id: CollectionId,
    /// Human-readable name
    pub name: String,
    /// Collection description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Vector dimension
    pub dimension: i32,
    /// Collection metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Tenant context for request processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// Tenant name/organization
    pub name: String,
    /// Subscription tier
    pub tier: SubscriptionTier,
    /// Current usage statistics
    pub usage: TenantUsage,
    /// Permissions for this tenant
    pub permissions: Vec<Permission>,
}

/// Subscription tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    /// Free embedded tier
    Free,
    /// Platform tier ($99-999/month)
    Platform,
    /// Enterprise tier ($5-50K/month)
    Enterprise,
}

/// Tenant usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    /// Total vectors stored
    pub vectors_stored: usize,
    /// Queries this hour
    pub queries_this_hour: usize,
    /// Bandwidth used this month (bytes)
    pub bandwidth_this_month: u64,
    /// Storage used (bytes)
    pub storage_used_bytes: u64,
}

/// Permissions for tenant operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// Read vectors and collections
    Read,
    /// Write/update vectors
    Write,
    /// Delete vectors
    Delete,
    /// Manage collections
    ManageCollections,
    /// Access metrics and analytics
    Analytics,
    /// Administrative access
    Admin,
}

/// Engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    /// Algorithm currently in use
    pub algorithm: String,
    /// Current status
    pub status: String,
    /// Total number of vectors
    pub vector_count: usize,
    /// Hot tier vectors
    pub hot_vectors: usize,
    /// Warm tier vectors
    pub warm_vectors: usize,
    /// Cold tier vectors
    pub cold_vectors: usize,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
    /// Queries per second
    pub queries_per_second: f64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Server version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Engine statistics
    pub engine: EngineStats,
}

/// FFI-compatible search result for C interface
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FFISearchResult {
    /// Vector ID (null-terminated string)
    pub id: *mut std::os::raw::c_char,
    /// Distance score
    pub distance: f32,
    /// Metadata JSON (null-terminated string)
    pub metadata: *mut std::os::raw::c_char,
}

impl Default for FFISearchResult {
    fn default() -> Self {
        Self {
            id: std::ptr::null_mut(),
            distance: 0.0,
            metadata: std::ptr::null_mut(),
        }
    }
}

/// FFI-compatible engine statistics
#[repr(C)]
#[derive(Debug, Clone)]
pub struct FFIStats {
    /// Vector count
    pub vector_count: usize,
    /// Hot tier count
    pub hot_vectors: usize,
    /// Warm tier count
    pub warm_vectors: usize,
    /// Cold tier count
    pub cold_vectors: usize,
    /// Memory usage
    pub memory_usage_bytes: u64,
    /// Average query time in microseconds
    pub avg_query_time_us: f64,
}

impl Vector {
    /// Create a new vector
    pub fn new(id: VectorId, data: Vec<f32>) -> Self {
        Self {
            id,
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create a vector with metadata
    pub fn with_metadata(
        id: VectorId,
        data: Vec<f32>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self { id, data, metadata }
    }

    /// Get vector dimension
    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    /// Validate vector data
    pub fn validate(&self) -> crate::Result<()> {
        if self.id.is_empty() {
            return Err(crate::Error::validation("Vector ID cannot be empty"));
        }

        if self.data.is_empty() {
            return Err(crate::Error::validation("Vector data cannot be empty"));
        }

        // Security: Check dimension limits
        if self.data.len() > MAX_VECTOR_DIMENSION {
            return Err(crate::Error::validation(format!(
                "Vector dimension {} exceeds maximum allowed {}",
                self.data.len(),
                MAX_VECTOR_DIMENSION
            )));
        }

        // Security: Estimate serialized size to prevent protocol attacks
        // Each f32 is 4 bytes, plus overhead for JSON structure
        let estimated_size = self.data.len() * 4 + self.id.len() + 1024; // Conservative estimate
        if estimated_size > MAX_INPUT_SIZE {
            return Err(crate::Error::validation(
                "Vector data exceeds maximum allowed size"
            ));
        }

        // Security: Check metadata size
        let metadata_size = serde_json::to_vec(&self.metadata)
            .map(|v| v.len())
            .unwrap_or(0);
        if metadata_size > MAX_METADATA_SIZE {
            return Err(crate::Error::validation(format!(
                "Metadata size {} exceeds maximum allowed {}",
                metadata_size,
                MAX_METADATA_SIZE
            )));
        }

        // Check for NaN or infinite values
        for (i, &value) in self.data.iter().enumerate() {
            if !value.is_finite() {
                return Err(crate::Error::validation(format!(
                    "Vector contains invalid value at index {}: {}",
                    i, value
                )));
            }
        }

        Ok(())
    }
}

impl SearchRequest {
    /// Validate search request
    pub fn validate(&self, expected_dimension: usize) -> crate::Result<()> {
        if self.vector.len() != expected_dimension {
            return Err(crate::Error::DimensionMismatch {
                expected: expected_dimension,
                actual: self.vector.len(),
            });
        }

        if self.top_k <= 0 {
            return Err(crate::Error::validation("top_k must be positive"));
        }

        if self.top_k > 10000 {
            return Err(crate::Error::validation("top_k cannot exceed 10000"));
        }

        // Check for NaN or infinite values in query vector
        for (i, &value) in self.vector.iter().enumerate() {
            if !value.is_finite() {
                return Err(crate::Error::validation(format!(
                    "Query vector contains invalid value at index {}: {}",
                    i, value
                )));
            }
        }

        Ok(())
    }
}

impl TenantContext {
    /// Check if tenant has specific permission
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission) || self.permissions.contains(&Permission::Admin)
    }

    /// Check if operation is within quotas
    pub fn check_quota(&self, _operation: &str) -> crate::Result<()> {
        match self.tier {
            SubscriptionTier::Free => {
                if self.usage.vectors_stored >= 100_000 {
                    return Err(crate::Error::QuotaExceeded {
                        tenant_id: self.tenant_id.to_string(),
                        resource: "vectors".to_string(),
                    });
                }
                if self.usage.queries_this_hour >= 1_000 {
                    return Err(crate::Error::QuotaExceeded {
                        tenant_id: self.tenant_id.to_string(),
                        resource: "queries".to_string(),
                    });
                }
            }
            SubscriptionTier::Platform => {
                if self.usage.vectors_stored >= 10_000_000 {
                    return Err(crate::Error::QuotaExceeded {
                        tenant_id: self.tenant_id.to_string(),
                        resource: "vectors".to_string(),
                    });
                }
            }
            SubscriptionTier::Enterprise => {
                // No hard limits for enterprise
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_validation() {
        let vector = Vector::new("test".to_string(), vec![1.0, 2.0, 3.0]);
        assert!(vector.validate().is_ok());

        let empty_id = Vector::new("".to_string(), vec![1.0]);
        assert!(empty_id.validate().is_err());

        let empty_data = Vector::new("test".to_string(), vec![]);
        assert!(empty_data.validate().is_err());

        let nan_vector = Vector::new("test".to_string(), vec![1.0, f32::NAN]);
        assert!(nan_vector.validate().is_err());
    }

    #[test]
    fn test_vector_security_validation() {
        // Test dimension limit
        let large_dim = vec![1.0_f32; MAX_VECTOR_DIMENSION + 1];
        let oversized_vector = Vector::new("test".to_string(), large_dim);
        assert!(oversized_vector.validate().is_err());

        // Test large metadata
        let mut huge_metadata = HashMap::new();
        let big_string = "x".repeat(MAX_METADATA_SIZE + 1);
        huge_metadata.insert("key".to_string(), serde_json::Value::String(big_string));
        let vector_with_huge_metadata = Vector::with_metadata(
            "test".to_string(),
            vec![1.0, 2.0, 3.0],
            huge_metadata,
        );
        assert!(vector_with_huge_metadata.validate().is_err());
    }

    #[test]
    fn test_search_request_validation() {
        let request = SearchRequest {
            vector: vec![1.0, 2.0, 3.0],
            top_k: 10,
            filter: HashMap::new(),
            include_vector: false,
            collection: None,
        };

        assert!(request.validate(3).is_ok());
        assert!(request.validate(4).is_err()); // Wrong dimension

        let bad_k = SearchRequest {
            vector: vec![1.0, 2.0, 3.0],
            top_k: 0,
            filter: HashMap::new(),
            include_vector: false,
            collection: None,
        };
        assert!(bad_k.validate(3).is_err());
    }

    #[test]
    fn test_tenant_permissions() {
        let context = TenantContext {
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
        };

        assert!(context.has_permission(Permission::Read));
        assert!(context.has_permission(Permission::Write));
        assert!(!context.has_permission(Permission::Admin));
    }

    #[test]
    fn test_quota_checking() {
        let mut context = TenantContext {
            tenant_id: Uuid::new_v4(),
            name: "Free Tenant".to_string(),
            tier: SubscriptionTier::Free,
            usage: TenantUsage {
                vectors_stored: 50_000,
                queries_this_hour: 500,
                bandwidth_this_month: 0,
                storage_used_bytes: 0,
            },
            permissions: vec![Permission::Read, Permission::Write],
        };

        assert!(context.check_quota("search").is_ok());

        context.usage.vectors_stored = 150_000;
        assert!(context.check_quota("add").is_err());
    }
}