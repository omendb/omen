//! REST API server for OmenDB
//!
//! Provides HTTP endpoints for health checks, metrics, and query execution.

pub mod handlers;
pub mod server;

pub use server::RestServer;
