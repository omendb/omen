use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Binary quantized vector (1 bit per dimension)
///
/// Stores vector as packed u64 array for SIMD efficiency.
/// For 1536D: 1536 bits = 24 × u64 (64 bits each) = 192 bytes
/// vs original float32: 1536 × 4 bytes = 6,144 bytes (96% reduction)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantizedVector {
    /// Packed binary representation (u64 array for fast XOR operations)
    bits: Vec<u64>,

    /// Dimension count (for validation)
    dimensions: usize,
}

impl QuantizedVector {
    /// Create from float32 vector using per-dimension thresholds
    ///
    /// Quantization rule: bit[i] = 1 if value[i] >= threshold[i], else 0
    ///
    /// # Arguments
    /// * `vector` - Input float32 vector
    /// * `thresholds` - Per-dimension thresholds (same length as vector)
    pub fn from_f32(vector: &[f32], thresholds: &[f32]) -> Self {
        assert_eq!(
            vector.len(),
            thresholds.len(),
            "Vector and thresholds must have same length"
        );

        let dimensions = vector.len();
        let num_words = (dimensions + 63) / 64; // Ceiling division
        let mut bits = vec![0u64; num_words];

        for (i, (&value, &threshold)) in vector.iter().zip(thresholds.iter()).enumerate() {
            if value >= threshold {
                let word_idx = i / 64;
                let bit_idx = i % 64;
                bits[word_idx] |= 1u64 << bit_idx;
            }
        }

        QuantizedVector { bits, dimensions }
    }

    /// Hamming distance: count of differing bits
    ///
    /// Uses XOR + popcount (hardware-accelerated POPCNT instruction).
    /// For 1536D: Only 24 XOR + POPCNT operations vs 1536 float subtractions for L2.
    ///
    /// # Performance
    /// - XOR: Single CPU instruction per u64
    /// - count_ones(): POPCNT instruction (1-3 cycles)
    /// - Target: <0.01ms per distance computation
    pub fn hamming_distance(&self, other: &QuantizedVector) -> u32 {
        assert_eq!(
            self.dimensions, other.dimensions,
            "Vectors must have same dimensions"
        );

        self.bits
            .iter()
            .zip(other.bits.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum()
    }

    /// Convert to bytes for storage
    ///
    /// Format: [dimensions: 4 bytes][bits: N × 8 bytes]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + self.bits.len() * 8);

        // Store dimensions (4 bytes, little-endian)
        bytes.extend_from_slice(&(self.dimensions as u32).to_le_bytes());

        // Store bits (8 bytes per u64)
        for &word in &self.bits {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        bytes
    }

    /// Load from bytes
    pub fn from_bytes(bytes: &[u8], dimensions: usize) -> Result<Self> {
        anyhow::ensure!(
            bytes.len() >= 4,
            "Invalid quantized vector: too short (need at least 4 bytes for dimensions)"
        );

        // Read dimensions
        let stored_dims = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        anyhow::ensure!(
            stored_dims == dimensions,
            "Dimension mismatch: stored {} but expected {}",
            stored_dims,
            dimensions
        );

        // Read bits
        let num_words = (dimensions + 63) / 64;
        anyhow::ensure!(
            bytes.len() == 4 + num_words * 8,
            "Invalid quantized vector: expected {} bytes but got {}",
            4 + num_words * 8,
            bytes.len()
        );

        let mut bits = Vec::with_capacity(num_words);
        for i in 0..num_words {
            let offset = 4 + i * 8;
            let word = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            bits.push(word);
        }

        Ok(QuantizedVector { bits, dimensions })
    }

    /// Get dimensions
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get memory size in bytes
    pub fn memory_size(&self) -> usize {
        std::mem::size_of::<usize>() + // dimensions field
        std::mem::size_of::<usize>() + // Vec capacity/len
        self.bits.len() * std::mem::size_of::<u64>() // bit data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_simple() {
        let vector = vec![1.0, -1.0, 2.0, -2.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let quantized = QuantizedVector::from_f32(&vector, &thresholds);

        // bits[0] should have bits 0 and 2 set (values >= 0.0)
        assert_eq!(quantized.dimensions, 4);
        assert_eq!(quantized.bits.len(), 1); // 4 dims fits in 1 u64
        assert_eq!(quantized.bits[0], 0b0101); // bits 0 and 2 set
    }

    #[test]
    fn test_quantize_all_zeros() {
        let vector = vec![-1.0, -2.0, -3.0, -4.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let quantized = QuantizedVector::from_f32(&vector, &thresholds);

        assert_eq!(quantized.bits[0], 0); // All bits should be 0
    }

    #[test]
    fn test_quantize_all_ones() {
        let vector = vec![1.0, 2.0, 3.0, 4.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let quantized = QuantizedVector::from_f32(&vector, &thresholds);

        assert_eq!(quantized.bits[0], 0b1111); // All 4 bits set
    }

    #[test]
    fn test_hamming_distance_identical() {
        let vector = vec![1.0, -1.0, 2.0, -2.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let q1 = QuantizedVector::from_f32(&vector, &thresholds);
        let q2 = QuantizedVector::from_f32(&vector, &thresholds);

        assert_eq!(q1.hamming_distance(&q2), 0);
    }

    #[test]
    fn test_hamming_distance_different() {
        let v1 = vec![1.0, 1.0, 1.0, 1.0];
        let v2 = vec![-1.0, -1.0, -1.0, -1.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let q1 = QuantizedVector::from_f32(&v1, &thresholds);
        let q2 = QuantizedVector::from_f32(&v2, &thresholds);

        assert_eq!(q1.hamming_distance(&q2), 4); // All 4 bits differ
    }

    #[test]
    fn test_hamming_distance_symmetry() {
        let v1 = vec![1.0, -1.0, 2.0, -2.0];
        let v2 = vec![-1.0, 1.0, -2.0, 2.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0];

        let q1 = QuantizedVector::from_f32(&v1, &thresholds);
        let q2 = QuantizedVector::from_f32(&v2, &thresholds);

        assert_eq!(q1.hamming_distance(&q2), q2.hamming_distance(&q1));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let vector = vec![1.0, -1.0, 2.0, -2.0, 3.0, -3.0];
        let thresholds = vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let original = QuantizedVector::from_f32(&vector, &thresholds);
        let bytes = original.to_bytes();
        let restored = QuantizedVector::from_bytes(&bytes, 6).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_large_vector_1536d() {
        // Test 1536D (OpenAI embedding size)
        let vector: Vec<f32> = (0..1536).map(|i| (i as f32) - 768.0).collect();
        let thresholds = vec![0.0; 1536];

        let quantized = QuantizedVector::from_f32(&vector, &thresholds);

        assert_eq!(quantized.dimensions, 1536);
        assert_eq!(quantized.bits.len(), 24); // 1536 / 64 = 24 u64 words

        // Memory check: 24 × 8 bytes = 192 bytes (vs 6,144 for float32)
        let memory = quantized.bits.len() * 8;
        assert_eq!(memory, 192);
    }

    #[test]
    fn test_memory_size() {
        let vector = vec![1.0; 1536];
        let thresholds = vec![0.0; 1536];

        let quantized = QuantizedVector::from_f32(&vector, &thresholds);
        let size = quantized.memory_size();

        // Should be approximately 192 bytes for bits + overhead
        assert!(size >= 192 && size <= 256);
    }
}
