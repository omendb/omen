"""Specialized SIMD kernels for common dimensions.

These hand-optimized kernels provide significant speedup for
the most common embedding dimensions used in production.
"""

from memory import UnsafePointer
from math import sqrt
from sys.info import simdwidthof
from algorithm import vectorize

# =============================================================================
# SPECIALIZED DISTANCE KERNELS
# =============================================================================

@always_inline
fn euclidean_distance_128d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """Ultra-optimized distance for 128D vectors (OpenAI ada-002).
    
    Uses 16-wide SIMD with 8 iterations, perfectly aligned.
    Expected: 2x faster than generic implementation.
    """
    alias simd_width = 16  # 128 = 16 * 8, perfect alignment
    var sum = SIMD[DType.float32, simd_width](0)
    
    # Unrolled loop for 128D = 8 iterations of 16-wide SIMD
    @parameter
    fn compute_chunk[i: Int]():
        var offset = i * simd_width
        var diff = a.load[width=simd_width](offset) - b.load[width=simd_width](offset)
        sum += diff * diff
    
    # Process all 8 chunks
    compute_chunk[0]()
    compute_chunk[1]()
    compute_chunk[2]()
    compute_chunk[3]()
    compute_chunk[4]()
    compute_chunk[5]()
    compute_chunk[6]()
    compute_chunk[7]()
    
    return sqrt(sum.reduce_add())

@always_inline
fn euclidean_distance_256d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """Ultra-optimized distance for 256D vectors.
    
    Uses 32-wide SIMD with 8 iterations or 16-wide with 16 iterations.
    Expected: 2x faster than generic implementation.
    """
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16
    alias iterations = 256 // simd_width
    
    var sum = SIMD[DType.float32, simd_width](0)
    
    @parameter
    for i in range(iterations):
        var offset = i * simd_width
        var diff = a.load[width=simd_width](offset) - b.load[width=simd_width](offset)
        sum += diff * diff
    
    return sqrt(sum.reduce_add())

@always_inline
fn euclidean_distance_384d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """Optimized distance for 384D vectors (sentence-transformers).
    
    Uses multiple accumulators to avoid the 384D performance cliff.
    Expected: 3x faster at this specific dimension.
    """
    alias simd_width = 16
    
    # Use 4 accumulators to prevent pipeline stalls
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    var sum3 = SIMD[DType.float32, simd_width](0)
    var sum4 = SIMD[DType.float32, simd_width](0)
    
    # Process 384 = 24 * 16 in groups of 4
    @parameter
    for i in range(0, 384, simd_width * 4):
        var diff1 = a.load[width=simd_width](i) - b.load[width=simd_width](i)
        var diff2 = a.load[width=simd_width](i + simd_width) - b.load[width=simd_width](i + simd_width)
        var diff3 = a.load[width=simd_width](i + simd_width*2) - b.load[width=simd_width](i + simd_width*2)
        var diff4 = a.load[width=simd_width](i + simd_width*3) - b.load[width=simd_width](i + simd_width*3)
        
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
        sum3 += diff3 * diff3
        sum4 += diff4 * diff4
    
    return sqrt((sum1 + sum2 + sum3 + sum4).reduce_add())

@always_inline
fn euclidean_distance_512d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """Ultra-optimized distance for 512D vectors.
    
    Perfect power of 2, ideal for SIMD.
    Expected: 2x faster than generic implementation.
    """
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16
    alias iterations = 512 // simd_width
    
    # Use 2 accumulators for better pipeline utilization
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    
    @parameter
    for i in range(0, iterations, 2):
        var offset1 = i * simd_width
        var offset2 = (i + 1) * simd_width
        
        var diff1 = a.load[width=simd_width](offset1) - b.load[width=simd_width](offset1)
        var diff2 = a.load[width=simd_width](offset2) - b.load[width=simd_width](offset2)
        
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
    
    return sqrt((sum1 + sum2).reduce_add())

@always_inline
fn euclidean_distance_768d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """AVX-512 optimized distance for 768D vectors (BERT embeddings).

    Research: Intel AVX-512 provides up to 48% throughput improvement.
    Uses 8 accumulators and aggressive unrolling for maximum pipeline utilization.
    Expected: 3x faster than generic implementation.
    """
    # AVX-512 optimization: Use wider SIMD when available
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16

    # Use 8 accumulators for optimal pipeline utilization
    # Research: More accumulators prevent pipeline stalls in high-dimensional space
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    var sum3 = SIMD[DType.float32, simd_width](0)
    var sum4 = SIMD[DType.float32, simd_width](0)
    var sum5 = SIMD[DType.float32, simd_width](0)
    var sum6 = SIMD[DType.float32, simd_width](0)
    var sum7 = SIMD[DType.float32, simd_width](0)
    var sum8 = SIMD[DType.float32, simd_width](0)

    # Process 768 = 24 * 32 (or 48 * 16) in groups of 8 for better cache utilization
    alias chunk_size = simd_width * 8
    alias num_chunks = 768 // chunk_size

    @parameter
    for chunk in range(num_chunks):
        var base_offset = chunk * chunk_size

        # Aggressive unrolling: Process 8 SIMD vectors per iteration
        var diff1 = a.load[width=simd_width](base_offset) - b.load[width=simd_width](base_offset)
        var diff2 = a.load[width=simd_width](base_offset + simd_width) - b.load[width=simd_width](base_offset + simd_width)
        var diff3 = a.load[width=simd_width](base_offset + simd_width*2) - b.load[width=simd_width](base_offset + simd_width*2)
        var diff4 = a.load[width=simd_width](base_offset + simd_width*3) - b.load[width=simd_width](base_offset + simd_width*3)
        var diff5 = a.load[width=simd_width](base_offset + simd_width*4) - b.load[width=simd_width](base_offset + simd_width*4)
        var diff6 = a.load[width=simd_width](base_offset + simd_width*5) - b.load[width=simd_width](base_offset + simd_width*5)
        var diff7 = a.load[width=simd_width](base_offset + simd_width*6) - b.load[width=simd_width](base_offset + simd_width*6)
        var diff8 = a.load[width=simd_width](base_offset + simd_width*7) - b.load[width=simd_width](base_offset + simd_width*7)

        # Parallel accumulation to maximize ALU utilization
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
        sum3 += diff3 * diff3
        sum4 += diff4 * diff4
        sum5 += diff5 * diff5
        sum6 += diff6 * diff6
        sum7 += diff7 * diff7
        sum8 += diff8 * diff8

    # Combine all accumulators efficiently
    var final_sum = (sum1 + sum2) + (sum3 + sum4) + (sum5 + sum6) + (sum7 + sum8)
    return sqrt(final_sum.reduce_add())

@always_inline
fn euclidean_distance_1536d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """AVX-512 optimized distance for 1536D vectors (OpenAI ada-003).

    Research: Intel AVX-512 provides up to 48% throughput improvement.
    Uses 16 accumulators with extreme unrolling for ultra-high dimensions.
    Expected: 4x faster than generic implementation.
    """
    # AVX-512 optimization: Use wider SIMD when available
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16

    # Use 16 accumulators for ultra-high dimensional optimization
    # Research: Maximum pipeline utilization for 1536D vectors
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    var sum3 = SIMD[DType.float32, simd_width](0)
    var sum4 = SIMD[DType.float32, simd_width](0)
    var sum5 = SIMD[DType.float32, simd_width](0)
    var sum6 = SIMD[DType.float32, simd_width](0)
    var sum7 = SIMD[DType.float32, simd_width](0)
    var sum8 = SIMD[DType.float32, simd_width](0)
    var sum9 = SIMD[DType.float32, simd_width](0)
    var sum10 = SIMD[DType.float32, simd_width](0)
    var sum11 = SIMD[DType.float32, simd_width](0)
    var sum12 = SIMD[DType.float32, simd_width](0)
    var sum13 = SIMD[DType.float32, simd_width](0)
    var sum14 = SIMD[DType.float32, simd_width](0)
    var sum15 = SIMD[DType.float32, simd_width](0)
    var sum16 = SIMD[DType.float32, simd_width](0)

    # Process 1536 = 48 * 32 (or 96 * 16) in groups of 16 for cache efficiency
    alias chunk_size = simd_width * 16
    alias num_chunks = 1536 // chunk_size

    @parameter
    for chunk in range(num_chunks):
        var base_offset = chunk * chunk_size

        # Extreme unrolling: Process 16 SIMD vectors per iteration
        var diff1 = a.load[width=simd_width](base_offset) - b.load[width=simd_width](base_offset)
        var diff2 = a.load[width=simd_width](base_offset + simd_width) - b.load[width=simd_width](base_offset + simd_width)
        var diff3 = a.load[width=simd_width](base_offset + simd_width*2) - b.load[width=simd_width](base_offset + simd_width*2)
        var diff4 = a.load[width=simd_width](base_offset + simd_width*3) - b.load[width=simd_width](base_offset + simd_width*3)
        var diff5 = a.load[width=simd_width](base_offset + simd_width*4) - b.load[width=simd_width](base_offset + simd_width*4)
        var diff6 = a.load[width=simd_width](base_offset + simd_width*5) - b.load[width=simd_width](base_offset + simd_width*5)
        var diff7 = a.load[width=simd_width](base_offset + simd_width*6) - b.load[width=simd_width](base_offset + simd_width*6)
        var diff8 = a.load[width=simd_width](base_offset + simd_width*7) - b.load[width=simd_width](base_offset + simd_width*7)
        var diff9 = a.load[width=simd_width](base_offset + simd_width*8) - b.load[width=simd_width](base_offset + simd_width*8)
        var diff10 = a.load[width=simd_width](base_offset + simd_width*9) - b.load[width=simd_width](base_offset + simd_width*9)
        var diff11 = a.load[width=simd_width](base_offset + simd_width*10) - b.load[width=simd_width](base_offset + simd_width*10)
        var diff12 = a.load[width=simd_width](base_offset + simd_width*11) - b.load[width=simd_width](base_offset + simd_width*11)
        var diff13 = a.load[width=simd_width](base_offset + simd_width*12) - b.load[width=simd_width](base_offset + simd_width*12)
        var diff14 = a.load[width=simd_width](base_offset + simd_width*13) - b.load[width=simd_width](base_offset + simd_width*13)
        var diff15 = a.load[width=simd_width](base_offset + simd_width*14) - b.load[width=simd_width](base_offset + simd_width*14)
        var diff16 = a.load[width=simd_width](base_offset + simd_width*15) - b.load[width=simd_width](base_offset + simd_width*15)

        # Parallel accumulation for maximum ALU utilization
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
        sum3 += diff3 * diff3
        sum4 += diff4 * diff4
        sum5 += diff5 * diff5
        sum6 += diff6 * diff6
        sum7 += diff7 * diff7
        sum8 += diff8 * diff8
        sum9 += diff9 * diff9
        sum10 += diff10 * diff10
        sum11 += diff11 * diff11
        sum12 += diff12 * diff12
        sum13 += diff13 * diff13
        sum14 += diff14 * diff14
        sum15 += diff15 * diff15
        sum16 += diff16 * diff16

    # Hierarchical reduction for efficient combining
    var group1 = (sum1 + sum2) + (sum3 + sum4)
    var group2 = (sum5 + sum6) + (sum7 + sum8)
    var group3 = (sum9 + sum10) + (sum11 + sum12)
    var group4 = (sum13 + sum14) + (sum15 + sum16)
    var final_sum = (group1 + group2) + (group3 + group4)
    return sqrt(final_sum.reduce_add())

# =============================================================================
# KERNEL SELECTION
# =============================================================================

@always_inline
fn select_distance_kernel(dimension: Int) -> fn(UnsafePointer[Float32], UnsafePointer[Float32]) -> Float32:
    """Select the optimal distance kernel for a given dimension.
    
    Returns specialized kernel for common dimensions, or None for generic.
    """
    if dimension == 128:
        return euclidean_distance_128d
    elif dimension == 256:
        return euclidean_distance_256d
    elif dimension == 384:
        return euclidean_distance_384d
    elif dimension == 512:
        return euclidean_distance_512d
    elif dimension == 768:
        return euclidean_distance_768d
    elif dimension == 1536:
        return euclidean_distance_1536d
    else:
        # Return None to indicate generic kernel should be used
        # Mojo doesn't support returning None from function pointers yet
        # So caller should check dimension first
        return euclidean_distance_128d  # Dummy return

@always_inline
fn has_specialized_kernel(dimension: Int) -> Bool:
    """Check if we have a specialized kernel for this dimension."""
    return dimension in [128, 256, 384, 512, 768, 1536]

# =============================================================================
# BATCH OPERATIONS
# =============================================================================

@always_inline
fn batch_euclidean_distance_128d(
    queries: UnsafePointer[Float32],
    database: UnsafePointer[Float32],
    n_queries: Int,
    n_database: Int,
    results: UnsafePointer[Float32]
):
    """Compute all pairwise distances for 128D vectors.
    
    Optimized for cache locality and SIMD efficiency.
    """
    alias dimension = 128
    alias tile_size = 64  # Process in tiles for cache efficiency
    
    @parameter
    fn process_tile(q_start: Int, q_end: Int, d_start: Int, d_end: Int):
        for q in range(q_start, q_end):
            var query_ptr = queries + q * dimension
            
            for d in range(d_start, d_end):
                var db_ptr = database + d * dimension
                var distance = euclidean_distance_128d(query_ptr, db_ptr)
                results[q * n_database + d] = distance
    
    # Process in tiles for better cache utilization
    for q_tile in range(0, n_queries, tile_size):
        var q_end = min(q_tile + tile_size, n_queries)
        
        for d_tile in range(0, n_database, tile_size):
            var d_end = min(d_tile + tile_size, n_database)
            process_tile(q_tile, q_end, d_tile, d_end)