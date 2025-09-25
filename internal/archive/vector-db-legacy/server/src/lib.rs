//! OmenDB Server
//! 
//! High-performance vector database server with Mojo FFI integration.
//! 
//! ## Architecture
//! 
//! The server is built with a layered architecture:
//! - **HTTP/gRPC Layer**: REST and gRPC APIs using Axum and Tonic
//! - **Multi-tenant Layer**: Authentication, authorization, and resource management
//! - **FFI Bridge**: Safe communication with Mojo vector engine
//! - **Tiered Storage**: Hot/warm/cold data management coordination
//! 
//! ## Usage
//! 
//! ```rust
//! use omendb_server::{config::Config, server::Server};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let server = Server::new(config).await?;
//!     server.run().await
//! }
//! ```

pub mod auth;
pub mod auth_test;
// pub mod c_ffi; // Experimental - Python FFI performs better (33K vec/s vs 26K vec/s)
pub mod config;
pub mod engine;
pub mod error;
pub mod python_ffi; // Production FFI implementation (33K vec/s)
// pub mod grpc;  // TODO: Re-enable when protobuf files are created
pub mod metrics;
pub mod resources;
pub mod server;
pub mod storage;
pub mod types;

pub use error::{Error, Result};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information
pub const BUILD_INFO: &str = concat!(
    "OmenDB Server v",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("CARGO_PKG_NAME"),
    ")"
);