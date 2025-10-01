//! PostgreSQL wire protocol server for OmenDB
//!
//! Provides psql compatibility using the pgwire crate and DataFusion execution engine.

pub mod encoding;
pub mod handlers;
pub mod server;

#[cfg(test)]
mod tests;

pub use server::PostgresServer;
