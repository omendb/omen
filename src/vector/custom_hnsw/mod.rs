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

// Public API exports
pub use types::{
    HNSWParams, HNSWNode, DistanceFunction, Candidate, SearchResult,
    l2_distance, cosine_distance, dot_product,
};

pub use storage::{NeighborLists, VectorStorage};

pub use index::HNSWIndex;
