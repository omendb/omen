// Custom HNSW implementation for OmenDB
//
// Design goals:
// - Cache-optimized (64-byte aligned hot data)
// - Memory-efficient (flattened index with u32 node IDs)
// - SIMD-ready (AVX2/AVX512 distance calculations)
// - SOTA features support (Extended RaBitQ, delta encoding)

mod types;
mod storage;
mod index;
mod simd_distance;
mod error;

// Public API exports
pub use types::{
    HNSWParams, HNSWNode, DistanceFunction, Candidate, SearchResult,
};

// Re-export SIMD-enabled distance functions
pub use simd_distance::{l2_distance, cosine_distance, dot_product};

pub use storage::{NeighborLists, VectorStorage};

pub use index::HNSWIndex;

// Re-export error types
pub use error::{HNSWError, Result};
