//! PostgreSQL wire protocol server for OmenDB
//!
//! Provides psql compatibility using the pgwire crate and DataFusion execution engine.

pub mod auth;
pub mod encoding;
pub mod extended;
pub mod handlers;
pub mod metrics_endpoint;
pub mod server;

#[cfg(test)]
mod tests;

pub use auth::OmenDbAuthSource;
pub use metrics_endpoint::serve_metrics;
pub use server::PostgresServer;
