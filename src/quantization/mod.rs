/// Binary quantization for vector compression
///
/// Implements RaBitQ-inspired binary quantization:
/// - 1 bit per dimension (96% memory reduction)
/// - Hamming distance for fast approximate search
/// - Reranking with original vectors for accuracy
///
/// Note: quantized_store.rs uses old hnsw_rs integration.
/// Binary quantization is now built into custom_hnsw module.
/// TODO: Port benchmark_bq_* benchmarks to use custom_hnsw

pub mod quantized_vector;
pub mod quantization_model;
// pub mod quantized_store; // Temporarily disabled - uses old hnsw_rs

pub use quantized_vector::QuantizedVector;
pub use quantization_model::QuantizationModel;
// pub use quantized_store::{HammingDistance, MemoryUsage, QuantizedVectorStore};
