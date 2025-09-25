//! Error handling for OmenDB Server
//! 
//! Provides a unified error type that can represent various error conditions
//! that can occur during server operations.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use tonic::Status;

/// Result type alias for OmenDB operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for OmenDB Server
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Engine-related errors
    #[error("Engine error: {0}")]
    Engine(String),

    /// FFI communication errors
    #[error("FFI error: {0}")]
    Ffi(String),

    /// Python integration errors
    #[error("Python error: {0}")]
    Python(String),

    /// Engine not initialized
    #[error("Engine not initialized")]
    EngineNotInitialized,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Authorization errors
    #[error("Authorization error: {0}")]
    AuthZ(String),

    /// Tenant management errors
    #[error("Tenant error: {0}")]
    Tenant(String),

    /// Resource limit errors
    #[error("Resource limit error: {0}")]
    ResourceLimit(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// JWT token errors
    #[error("Token error: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Vector dimension mismatch
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    /// Vector not found
    #[error("Vector not found: {id}")]
    VectorNotFound { id: String },

    /// Collection not found
    #[error("Collection not found: {name}")]
    CollectionNotFound { name: String },

    /// Tenant not found
    #[error("Tenant not found: {id}")]
    TenantNotFound { id: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for tenant {tenant_id}")]
    RateLimitExceeded { tenant_id: String },

    /// Quota exceeded
    #[error("Quota exceeded for tenant {tenant_id}: {resource}")]
    QuotaExceeded { tenant_id: String, resource: String },

    /// Engine pool exhausted
    #[error("Engine pool exhausted")]
    EnginePoolExhausted,

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,
}

impl Error {
    /// Create an engine error
    pub fn engine<S: Into<String>>(msg: S) -> Self {
        Self::Engine(msg.into())
    }

    /// Create an FFI error
    pub fn ffi<S: Into<String>>(msg: S) -> Self {
        Self::Ffi(msg.into())
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::Auth(msg.into())
    }

    /// Create an authorization error
    pub fn authz<S: Into<String>>(msg: S) -> Self {
        Self::AuthZ(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::Auth(_) => StatusCode::UNAUTHORIZED,
            Error::AuthZ(_) => StatusCode::FORBIDDEN,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::VectorNotFound { .. } => StatusCode::NOT_FOUND,
            Error::CollectionNotFound { .. } => StatusCode::NOT_FOUND,
            Error::TenantNotFound { .. } => StatusCode::NOT_FOUND,
            Error::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            Error::QuotaExceeded { .. } => StatusCode::PAYMENT_REQUIRED,
            Error::DimensionMismatch { .. } => StatusCode::BAD_REQUEST,
            Error::Timeout => StatusCode::REQUEST_TIMEOUT,
            Error::EnginePoolExhausted => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error category for logging/metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Config(_) => "config",
            Error::Engine(_) => "engine",
            Error::Ffi(_) => "ffi",
            Error::Auth(_) => "auth",
            Error::AuthZ(_) => "authz",
            Error::Tenant(_) => "tenant",
            Error::ResourceLimit(_) => "resource_limit",
            Error::Storage(_) => "storage",
            Error::Serialization(_) => "serialization",
            Error::Validation(_) => "validation",
            Error::Network(_) => "network",
            Error::Database(_) => "database",
            Error::Token(_) => "token",
            Error::Internal(_) => "internal",
            Error::DimensionMismatch { .. } => "dimension_mismatch",
            Error::VectorNotFound { .. } => "vector_not_found",
            Error::CollectionNotFound { .. } => "collection_not_found",
            Error::TenantNotFound { .. } => "tenant_not_found",
            Error::RateLimitExceeded { .. } => "rate_limit",
            Error::QuotaExceeded { .. } => "quota",
            Error::EnginePoolExhausted => "engine_pool",
            Error::Timeout => "timeout",
            Error::Python(_) => "python",
            Error::EngineNotInitialized => "engine_not_initialized",
            Error::InvalidConfiguration(_) => "invalid_configuration",
        }
    }
}

// HTTP response conversion
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let category = self.category();
        let message = self.to_string();

        tracing::error!(
            error = %self,
            category = category,
            status = %status,
            "Request failed"
        );

        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "category": category,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

// gRPC response conversion
impl From<Error> for Status {
    fn from(error: Error) -> Self {
        let code = match error {
            Error::Auth(_) => tonic::Code::Unauthenticated,
            Error::AuthZ(_) => tonic::Code::PermissionDenied,
            Error::Validation(_) => tonic::Code::InvalidArgument,
            Error::VectorNotFound { .. } => tonic::Code::NotFound,
            Error::CollectionNotFound { .. } => tonic::Code::NotFound,
            Error::TenantNotFound { .. } => tonic::Code::NotFound,
            Error::RateLimitExceeded { .. } => tonic::Code::ResourceExhausted,
            Error::QuotaExceeded { .. } => tonic::Code::ResourceExhausted,
            Error::DimensionMismatch { .. } => tonic::Code::InvalidArgument,
            Error::Timeout => tonic::Code::DeadlineExceeded,
            Error::EnginePoolExhausted => tonic::Code::Unavailable,
            _ => tonic::Code::Internal,
        };

        Status::new(code, error.to_string())
    }
}

// Convert from anyhow errors
impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error::Internal(error.to_string())
    }
}

// Convert from PyO3 errors
impl From<pyo3::PyErr> for Error {
    fn from(error: pyo3::PyErr) -> Self {
        Error::Python(error.to_string())
    }
}

// Convert from tokio JoinError
impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Error::Internal(format!("Task join error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(Error::auth("test").status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(Error::authz("test").status_code(), StatusCode::FORBIDDEN);
        assert_eq!(Error::validation("test").status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(Error::engine("test").category(), "engine");
        assert_eq!(Error::ffi("test").category(), "ffi");
        assert_eq!(Error::auth("test").category(), "auth");
    }

    #[test]
    fn test_dimension_mismatch_error() {
        let error = Error::DimensionMismatch { expected: 128, actual: 256 };
        assert!(error.to_string().contains("expected 128"));
        assert!(error.to_string().contains("got 256"));
    }

    #[test]
    fn test_grpc_conversion() {
        let error = Error::auth("invalid token");
        let status = Status::from(error);
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }
}