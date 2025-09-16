#include <metal_stdlib>
using namespace metal;

/**
 * Metal Compute Shaders for OmenDB GPU Acceleration
 *
 * Target: M3 Max 40-core GPU (5120 threads)
 * Expected: 10-100x distance computation speedup
 * Architecture: Unified memory with zero-copy access
 */

// =============================================================================
// DISTANCE COMPUTATION KERNELS
// =============================================================================

/**
 * Parallel Euclidean Distance Computation
 *
 * Computes distances between a query vector and batch of candidate vectors.
 * Thread Strategy: 1 thread per distance calculation
 * Expected Speedup: 20-100x vs CPU implementation
 */
kernel void euclidean_distance_batch(
    constant float* query_vector        [[buffer(0)]],  // Query vector [dimension]
    constant float* candidate_vectors   [[buffer(1)]],  // Candidate vectors [num_candidates * dimension]
    device float* distances            [[buffer(2)]],  // Output distances [num_candidates]
    constant uint& dimension           [[buffer(3)]],  // Vector dimension
    constant uint& num_candidates      [[buffer(4)]],  // Number of candidates
    uint thread_id                     [[thread_position_in_grid]]
) {
    // Ensure we don't exceed bounds
    if (thread_id >= num_candidates) {
        return;
    }

    // Calculate starting position for this candidate vector
    constant float* candidate = candidate_vectors + (thread_id * dimension);

    // Compute squared euclidean distance
    float sum = 0.0f;
    for (uint i = 0; i < dimension; i++) {
        float diff = query_vector[i] - candidate[i];
        sum += diff * diff;
    }

    // Store the distance (sqrt will be computed on CPU if needed)
    distances[thread_id] = sqrt(sum);
}

/**
 * Optimized Euclidean Distance for Common Dimensions
 *
 * Specialized kernel for 128D vectors (most common embedding size).
 * Uses vectorized operations for maximum performance.
 */
kernel void euclidean_distance_128d_optimized(
    constant float* query_vector        [[buffer(0)]],  // Query vector [128]
    constant float* candidate_vectors   [[buffer(1)]],  // Candidate vectors [num_candidates * 128]
    device float* distances            [[buffer(2)]],  // Output distances [num_candidates]
    constant uint& num_candidates      [[buffer(3)]],  // Number of candidates
    uint thread_id                     [[thread_position_in_grid]]
) {
    if (thread_id >= num_candidates) {
        return;
    }

    constant float* candidate = candidate_vectors + (thread_id * 128);

    // Use float4 vectorization for 4x parallel operations
    float sum = 0.0f;
    for (uint i = 0; i < 128; i += 4) {
        float4 q = float4(query_vector[i], query_vector[i+1], query_vector[i+2], query_vector[i+3]);
        float4 c = float4(candidate[i], candidate[i+1], candidate[i+2], candidate[i+3]);
        float4 diff = q - c;
        sum += dot(diff, diff);
    }

    distances[thread_id] = sqrt(sum);
}

// =============================================================================
// QUANTIZATION KERNELS
// =============================================================================

/**
 * Parallel Binary Quantization
 *
 * Converts float vectors to binary representation for 32x compression.
 * Thread Strategy: 1 thread per vector quantization
 * Expected Speedup: 50-200x vs CPU implementation
 */
kernel void binary_quantization_batch(
    constant float* float_vectors      [[buffer(0)]],  // Input float vectors [num_vectors * dimension]
    device uint* binary_vectors       [[buffer(1)]],  // Output binary vectors [num_vectors * (dimension/32)]
    constant uint& dimension          [[buffer(2)]],  // Vector dimension
    constant uint& num_vectors        [[buffer(3)]],  // Number of vectors
    constant float& threshold         [[buffer(4)]],  // Quantization threshold (typically 0.0)
    uint thread_id                    [[thread_position_in_grid]]
) {
    if (thread_id >= num_vectors) {
        return;
    }

    constant float* input_vector = float_vectors + (thread_id * dimension);
    device uint* output_vector = binary_vectors + (thread_id * ((dimension + 31) / 32));

    // Process 32 dimensions at a time into uint32
    uint num_uints = (dimension + 31) / 32;
    for (uint i = 0; i < num_uints; i++) {
        uint binary_chunk = 0;
        uint chunk_start = i * 32;
        uint chunk_end = min(chunk_start + 32, dimension);

        for (uint j = chunk_start; j < chunk_end; j++) {
            if (input_vector[j] > threshold) {
                binary_chunk |= (1u << (j - chunk_start));
            }
        }

        output_vector[i] = binary_chunk;
    }
}

/**
 * Binary Hamming Distance Computation
 *
 * Computes Hamming distances between binary vectors.
 * Ultra-fast bitwise operations on GPU.
 */
kernel void binary_hamming_distance_batch(
    constant uint* query_binary        [[buffer(0)]],  // Query binary vector [binary_dimension]
    constant uint* candidate_binaries  [[buffer(1)]],  // Candidate binary vectors [num_candidates * binary_dimension]
    device float* distances           [[buffer(2)]],  // Output distances [num_candidates]
    constant uint& binary_dimension   [[buffer(3)]],  // Binary vector dimension (in uint32s)
    constant uint& num_candidates     [[buffer(4)]],  // Number of candidates
    constant uint& original_dimension [[buffer(5)]],  // Original float dimension for normalization
    uint thread_id                    [[thread_position_in_grid]]
) {
    if (thread_id >= num_candidates) {
        return;
    }

    constant uint* candidate = candidate_binaries + (thread_id * binary_dimension);

    // Compute Hamming distance using XOR + popcount
    uint hamming_distance = 0;
    for (uint i = 0; i < binary_dimension; i++) {
        uint xor_result = query_binary[i] ^ candidate[i];
        hamming_distance += popcount(xor_result);
    }

    // Convert to normalized distance [0, 2]
    distances[thread_id] = (float(hamming_distance) / float(original_dimension)) * 2.0f;
}

// =============================================================================
// MATRIX OPERATIONS
// =============================================================================

/**
 * All-Pairs Similarity Matrix Computation
 *
 * Computes similarity matrix between two sets of vectors.
 * Thread Strategy: 2D grid with (i,j) threads per matrix element
 * Expected Speedup: 100-1000x vs CPU implementation
 */
kernel void similarity_matrix_compute(
    constant float* vectors_a         [[buffer(0)]],  // First set of vectors [num_a * dimension]
    constant float* vectors_b         [[buffer(1)]],  // Second set of vectors [num_b * dimension]
    device float* similarity_matrix   [[buffer(2)]],  // Output matrix [num_a * num_b]
    constant uint& dimension          [[buffer(3)]],  // Vector dimension
    constant uint& num_a              [[buffer(4)]],  // Number of vectors in set A
    constant uint& num_b              [[buffer(5)]],  // Number of vectors in set B
    uint2 thread_id                   [[thread_position_in_grid]]
) {
    uint i = thread_id.x;
    uint j = thread_id.y;

    if (i >= num_a || j >= num_b) {
        return;
    }

    constant float* vec_a = vectors_a + (i * dimension);
    constant float* vec_b = vectors_b + (j * dimension);

    // Compute dot product (cosine similarity if vectors are normalized)
    float dot_product = 0.0f;
    for (uint k = 0; k < dimension; k++) {
        dot_product += vec_a[k] * vec_b[k];
    }

    similarity_matrix[i * num_b + j] = dot_product;
}

// =============================================================================
// UTILITY KERNELS
// =============================================================================

/**
 * Batch Vector Normalization
 *
 * Normalizes vectors to unit length for cosine similarity.
 * Thread Strategy: 1 thread per vector
 */
kernel void batch_vector_normalize(
    device float* vectors             [[buffer(0)]],  // Input/output vectors [num_vectors * dimension]
    constant uint& dimension          [[buffer(1)]],  // Vector dimension
    constant uint& num_vectors        [[buffer(2)]],  // Number of vectors
    uint thread_id                    [[thread_position_in_grid]]
) {
    if (thread_id >= num_vectors) {
        return;
    }

    device float* vector = vectors + (thread_id * dimension);

    // Compute norm
    float norm_squared = 0.0f;
    for (uint i = 0; i < dimension; i++) {
        norm_squared += vector[i] * vector[i];
    }

    float norm = sqrt(norm_squared);
    if (norm > 0.0f) {
        // Normalize in-place
        for (uint i = 0; i < dimension; i++) {
            vector[i] /= norm;
        }
    }
}

/**
 * Top-K Selection Kernel
 *
 * Selects top-k smallest distances and their indices.
 * Useful for k-NN search results.
 */
kernel void select_top_k_distances(
    constant float* distances         [[buffer(0)]],  // Input distances [num_candidates]
    constant uint* indices           [[buffer(1)]],  // Input indices [num_candidates]
    device float* top_k_distances    [[buffer(2)]],  // Output top-k distances [k]
    device uint* top_k_indices       [[buffer(3)]],  // Output top-k indices [k]
    constant uint& num_candidates    [[buffer(4)]],  // Number of input candidates
    constant uint& k                 [[buffer(5)]],  // Number of top results to select
    uint thread_id                   [[thread_position_in_grid]]
) {
    // Simple parallel selection (could be optimized with parallel sorting)
    if (thread_id >= k) {
        return;
    }

    // Find the thread_id-th smallest distance
    float min_distance = INFINITY;
    uint min_index = 0;
    uint count_smaller = 0;

    for (uint i = 0; i < num_candidates; i++) {
        if (distances[i] < min_distance) {
            // Count how many distances are smaller than current candidate
            uint smaller_count = 0;
            for (uint j = 0; j < num_candidates; j++) {
                if (distances[j] < distances[i]) {
                    smaller_count++;
                }
            }

            if (smaller_count == thread_id) {
                min_distance = distances[i];
                min_index = indices[i];
            }
        }
    }

    top_k_distances[thread_id] = min_distance;
    top_k_indices[thread_id] = min_index;
}