//! Vector database module
//!
//! Provides vector storage with HNSW (Hierarchical Navigable Small World) indexing
//! for approximate nearest neighbor search.
//!
//! Week 2 Goal: Production-ready HNSW implementation
//! - Recall@10: >95%
//! - Latency: <10ms p95
//! - Memory: <200 bytes/vector overhead

pub mod types;
pub mod store;
pub mod hnsw_index;

// Re-export main types
pub use types::Vector;
pub use store::VectorStore;
pub use hnsw_index::HNSWIndex;
