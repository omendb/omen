/// Binary quantization for vector compression
///
/// Implements RaBitQ-inspired binary quantization:
/// - 1 bit per dimension (96% memory reduction)
/// - Hamming distance for fast approximate search
/// - Reranking with original vectors for accuracy

pub mod quantized_vector;
pub mod quantization_model;

pub use quantized_vector::QuantizedVector;
pub use quantization_model::QuantizationModel;
