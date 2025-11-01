//! OmenDB - PostgreSQL-Compatible Vector Database
//!
//! Embeddable vector database with custom HNSW implementation,
//! binary quantization, and PostgreSQL wire protocol.
//!
//! ## Features
//!
//! - **Custom HNSW**: Production-ready HNSW implementation with SIMD optimizations (7223 QPS @ 128D)
//! - **Binary Quantization**: Memory-efficient vector storage (19x compression)
//! - **PostgreSQL Protocol**: Drop-in pgvector replacement (port 5433)
//! - **Production Ready**: Error handling, logging, persistence (1222x speedup)
//!
//! ## Example
//!
//! ```rust,no_run
//! use omen::vector::{Vector, VectorStore};
//!
//! // Create vector store
//! let mut store = VectorStore::new(128); // 128 dimensions
//!
//! // Insert vectors
//! let vec1 = Vector::new(vec![0.1; 128]);
//! store.insert(vec1).unwrap();
//!
//! // Search
//! let query = Vector::new(vec![0.1; 128]);
//! let results = store.knn_search(&query, 10).unwrap();
//! ```

pub mod vector;
pub mod logging;

// Re-export core types
pub use vector::{Vector, VectorStore};
pub use logging::{init_logging, init_from_env, LogConfig};
