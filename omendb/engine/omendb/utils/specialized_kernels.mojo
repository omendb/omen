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
    """Optimized distance for 768D vectors (BERT embeddings).
    
    Common dimension for transformer models.
    Expected: 2x faster than generic implementation.
    """
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16
    alias iterations = 768 // simd_width
    
    # Use 3 accumulators for 768 = 3 * 256
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    var sum3 = SIMD[DType.float32, simd_width](0)
    
    @parameter
    for i in range(0, iterations, 3):
        var offset1 = i * simd_width
        var offset2 = (i + 1) * simd_width
        var offset3 = (i + 2) * simd_width
        
        var diff1 = a.load[width=simd_width](offset1) - b.load[width=simd_width](offset1)
        var diff2 = a.load[width=simd_width](offset2) - b.load[width=simd_width](offset2)
        var diff3 = a.load[width=simd_width](offset3) - b.load[width=simd_width](offset3)
        
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
        sum3 += diff3 * diff3
    
    return sqrt((sum1 + sum2 + sum3).reduce_add())

@always_inline
fn euclidean_distance_1536d(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    """Optimized distance for 1536D vectors (OpenAI ada-003).
    
    Latest OpenAI embedding dimension.
    Expected: 2x faster than generic implementation.
    """
    alias simd_width = 32 if simdwidthof[DType.float32]() >= 32 else 16
    alias iterations = 1536 // simd_width
    
    # Use 4 accumulators for maximum throughput
    var sum1 = SIMD[DType.float32, simd_width](0)
    var sum2 = SIMD[DType.float32, simd_width](0)
    var sum3 = SIMD[DType.float32, simd_width](0)
    var sum4 = SIMD[DType.float32, simd_width](0)
    
    @parameter
    for i in range(0, iterations, 4):
        var offset1 = i * simd_width
        var offset2 = (i + 1) * simd_width
        var offset3 = (i + 2) * simd_width
        var offset4 = (i + 3) * simd_width
        
        var diff1 = a.load[width=simd_width](offset1) - b.load[width=simd_width](offset1)
        var diff2 = a.load[width=simd_width](offset2) - b.load[width=simd_width](offset2)
        var diff3 = a.load[width=simd_width](offset3) - b.load[width=simd_width](offset3)
        var diff4 = a.load[width=simd_width](offset4) - b.load[width=simd_width](offset4)
        
        sum1 += diff1 * diff1
        sum2 += diff2 * diff2
        sum3 += diff3 * diff3
        sum4 += diff4 * diff4
    
    return sqrt((sum1 + sum2 + sum3 + sum4).reduce_add())

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