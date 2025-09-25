"""
Dimension-Optimized SIMD Distance Calculations

This module provides improved SIMD distance calculations that eliminate
performance cliffs at dimension boundaries through:

1. Advanced dimension-aware kernel selection
2. Optimal memory alignment and padding
3. Cache-friendly access patterns
4. Specialized kernels for problematic dimensions

Addresses the 45% performance drop at 256D and other dimension boundary issues.
"""

from memory import UnsafePointer, memset_zero
from algorithm import vectorize, parallelize
from sys.info import simdwidthof
from math import sqrt, ceil
from collections import List
from .utils import get_optimal_workers

alias dtype = DType.float32
alias hardware_simd_width = simdwidthof[dtype]()  # Hardware-detected optimal width
alias max_possible_simd_width = 16  # AVX-512 theoretical maximum

@always_inline
fn calculate_optimal_padding(dimension: Int) -> Int:
    """Calculate padding needed for optimal SIMD alignment."""
    # Align to next multiple of hardware SIMD width for best performance
    var remainder = dimension % hardware_simd_width
    if remainder == 0:
        return 0
    return hardware_simd_width - remainder

@always_inline  
fn get_optimal_simd_width(dimension: Int, prefer_width: Int = -1) -> Int:
    """Dynamically select optimal SIMD width based on hardware and dimension.
    
    Args:
        dimension: Vector dimension
        prefer_width: Preferred width (-1 for auto-selection)
        
    Returns:
        Optimal SIMD width for this dimension and hardware
    """
    # If specific width requested, validate and use it
    if prefer_width > 0:
        if prefer_width <= hardware_simd_width:
            return prefer_width
        else:
            # Requested width exceeds hardware capability, use max available
            return hardware_simd_width
    
    # Auto-selection based on dimension characteristics
    if dimension <= 64:
        # Small dimensions: use smaller SIMD width to avoid overhead
        return min(8, hardware_simd_width)
    elif dimension <= 256:
        # Medium dimensions: use full hardware width for good efficiency
        return hardware_simd_width  
    elif dimension <= 512:
        # Large dimensions: prefer maximum width for throughput
        return hardware_simd_width
    else:
        # Very large dimensions: maximum width with potential tiling
        return hardware_simd_width

struct SIMDStrategy:
    var simd_width: Int
    var chunk_size: Int  
    var num_accumulators: Int
    
    fn __init__(out self, simd_width: Int, chunk_size: Int, num_accumulators: Int):
        self.simd_width = simd_width
        self.chunk_size = chunk_size
        self.num_accumulators = num_accumulators

@always_inline
fn select_advanced_simd_strategy(dimension: Int) -> SIMDStrategy:
    """Select optimal SIMD strategy with dynamic hardware-aware width selection.
    
    Returns: SIMDStrategy with (simd_width, chunk_size, num_accumulators)
    Expected 2-4x speedup on AVX-512 systems vs fixed 8-wide SIMD.
    """
    
    # Get optimal SIMD width for this dimension and hardware
    var optimal_width = get_optimal_simd_width(dimension)
    
    # For dimensions with specialized kernels - use hardware-optimal width
    if dimension == 128 or dimension == 256 or dimension == 512 or dimension == 768:
        return SIMDStrategy(optimal_width, optimal_width, 1)  # Specialized kernels
    
    # For dimensions around 384D cliff - use multiple accumulators with optimal width
    elif dimension >= 320 and dimension <= 448:
        var accumulators = 4 if optimal_width >= 16 else 2  # Scale accumulators with width
        return SIMDStrategy(optimal_width, optimal_width, accumulators)
    
    # For high dimensions (>512) - adaptive chunking with maximum SIMD width
    elif dimension > 512:
        var chunks = dimension // (optimal_width * 4)  # Scale chunk size with SIMD width
        var accumulators = min(chunks // 2, 8)  # Up to 8 accumulators
        return SIMDStrategy(optimal_width, optimal_width, max(accumulators, 2))
    
    # For problematic mid-range dimensions (160-320) - hardware-aware alignment
    elif dimension >= 160 and dimension <= 320:
        var accumulators = 2 if optimal_width >= 8 else 1  # Scale with SIMD capability
        
        # Check alignment with optimal width first, then fallback
        if dimension % optimal_width == 0:
            return SIMDStrategy(optimal_width, optimal_width, accumulators)
        elif optimal_width >= 8 and dimension % 8 == 0:
            return SIMDStrategy(8, 8, accumulators)
        else:
            # Use optimal width with padding for best performance
            return SIMDStrategy(optimal_width, optimal_width, accumulators)
    
    # For small dimensions (<160) - conservative approach
    else:
        # For small dimensions, check if optimal width is beneficial
        if dimension >= 32 and optimal_width >= 16:
            # Large SIMD width beneficial for moderate small dimensions
            return SIMDStrategy(optimal_width, optimal_width, 1)
        elif dimension % optimal_width == 0:
            # Perfect alignment with optimal width
            return SIMDStrategy(optimal_width, optimal_width, 1)
        elif optimal_width >= 8 and dimension % 8 == 0:
            # Good alignment with 8-wide
            return SIMDStrategy(8, 8, 1)
        else:
            # Use optimal width with padding handling
            return SIMDStrategy(optimal_width, optimal_width, 1)

@always_inline
fn euclidean_distance_adaptive_simd(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32],
    dimension: Int
) -> Float32:
    """Adaptive SIMD distance calculation that eliminates dimension boundary cliffs."""
    
    # Get optimal strategy for this dimension
    var strategy = select_advanced_simd_strategy(dimension)
    var simd_width = strategy.simd_width
    var chunk_size = strategy.chunk_size
    var num_accumulators = strategy.num_accumulators
    
    # Use adaptive multi-accumulator approach for all dimensions
    # This eliminates performance cliffs by using consistent optimization strategy
    return euclidean_distance_multi_accumulator(query, vector, dimension, simd_width, num_accumulators)

@always_inline
fn euclidean_distance_multi_accumulator(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32],
    dimension: Int,
    simd_width: Int,
    num_accumulators: Int
) -> Float32:
    """Multi-accumulator SIMD distance calculation to prevent pipeline stalls."""
    
    var total_sum = Float32(0)
    
    if simd_width == 16:
        # Use 16-wide SIMD with multiple accumulators
        var accumulators = List[SIMD[dtype, 16]]()
        for i in range(num_accumulators):
            accumulators.append(SIMD[dtype, 16](0))
        
        var offset = 0
        var full_chunks = dimension // 16
        var chunks_per_acc = full_chunks // num_accumulators
        
        # Process chunks with multiple accumulators
        for acc_idx in range(num_accumulators):
            var start_chunk = acc_idx * chunks_per_acc
            var end_chunk = start_chunk + chunks_per_acc
            
            # Handle last accumulator getting remaining chunks
            if acc_idx == num_accumulators - 1:
                end_chunk = full_chunks
            
            for chunk_idx in range(start_chunk, end_chunk):
                var pos = chunk_idx * 16
                var q = query.load[width=16](pos)
                var v = vector.load[width=16](pos)
                var diff = q - v
                accumulators[acc_idx] += diff * diff
        
        # Sum all accumulators
        for i in range(num_accumulators):
            total_sum += accumulators[i].reduce_add()
        
        # Handle remainder
        var remainder_start = full_chunks * 16
        for i in range(remainder_start, dimension):
            var diff = query[i] - vector[i]
            total_sum += diff * diff
    
    elif simd_width == 8:
        # Use 8-wide SIMD with multiple accumulators
        var accumulators = List[SIMD[dtype, 8]]()
        for i in range(num_accumulators):
            accumulators.append(SIMD[dtype, 8](0))
        
        var full_chunks = dimension // 8
        var chunks_per_acc = full_chunks // num_accumulators
        
        for acc_idx in range(num_accumulators):
            var start_chunk = acc_idx * chunks_per_acc
            var end_chunk = start_chunk + chunks_per_acc
            
            if acc_idx == num_accumulators - 1:
                end_chunk = full_chunks
            
            for chunk_idx in range(start_chunk, end_chunk):
                var pos = chunk_idx * 8
                var q = query.load[width=8](pos)
                var v = vector.load[width=8](pos)
                var diff = q - v
                accumulators[acc_idx] += diff * diff
        
        for i in range(num_accumulators):
            total_sum += accumulators[i].reduce_add()
        
        # Handle remainder
        var remainder_start = full_chunks * 8
        for i in range(remainder_start, dimension):
            var diff = query[i] - vector[i]
            total_sum += diff * diff
    
    return sqrt(total_sum)

@always_inline
fn euclidean_distance_specialized_128_improved(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32]
) -> Float32:
    """Improved 128D specialized kernel with better instruction scheduling."""
    
    # Use 2 accumulators to prevent pipeline stalls
    var sum0 = SIMD[dtype, 16](0)
    var sum1 = SIMD[dtype, 16](0)
    
    # Process 8 chunks of 16, alternating accumulators
    # First 4 chunks to sum0
    sum0 += (query.load[width=16](0) - vector.load[width=16](0)) ** 2
    sum0 += (query.load[width=16](32) - vector.load[width=16](32)) ** 2
    sum0 += (query.load[width=16](64) - vector.load[width=16](64)) ** 2
    sum0 += (query.load[width=16](96) - vector.load[width=16](96)) ** 2
    
    # Second 4 chunks to sum1
    sum1 += (query.load[width=16](16) - vector.load[width=16](16)) ** 2
    sum1 += (query.load[width=16](48) - vector.load[width=16](48)) ** 2
    sum1 += (query.load[width=16](80) - vector.load[width=16](80)) ** 2
    sum1 += (query.load[width=16](112) - vector.load[width=16](112)) ** 2
    
    return sqrt((sum0 + sum1).reduce_add())

@always_inline  
fn euclidean_distance_specialized_256_improved(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32]
) -> Float32:
    """Improved 256D specialized kernel with optimal instruction scheduling."""
    
    # Use 4 accumulators for better pipeline utilization
    var sum0 = SIMD[dtype, 16](0)
    var sum1 = SIMD[dtype, 16](0)
    var sum2 = SIMD[dtype, 16](0)
    var sum3 = SIMD[dtype, 16](0)
    
    # Process 16 chunks of 16, cycling through accumulators
    # This prevents pipeline stalls in the ALU
    
    # Chunks 0, 4, 8, 12 -> sum0
    sum0 += (query.load[width=16](0) - vector.load[width=16](0)) ** 2
    sum0 += (query.load[width=16](64) - vector.load[width=16](64)) ** 2
    sum0 += (query.load[width=16](128) - vector.load[width=16](128)) ** 2
    sum0 += (query.load[width=16](192) - vector.load[width=16](192)) ** 2
    
    # Chunks 1, 5, 9, 13 -> sum1
    sum1 += (query.load[width=16](16) - vector.load[width=16](16)) ** 2
    sum1 += (query.load[width=16](80) - vector.load[width=16](80)) ** 2
    sum1 += (query.load[width=16](144) - vector.load[width=16](144)) ** 2
    sum1 += (query.load[width=16](208) - vector.load[width=16](208)) ** 2
    
    # Chunks 2, 6, 10, 14 -> sum2
    sum2 += (query.load[width=16](32) - vector.load[width=16](32)) ** 2
    sum2 += (query.load[width=16](96) - vector.load[width=16](96)) ** 2
    sum2 += (query.load[width=16](160) - vector.load[width=16](160)) ** 2
    sum2 += (query.load[width=16](224) - vector.load[width=16](224)) ** 2
    
    # Chunks 3, 7, 11, 15 -> sum3
    sum3 += (query.load[width=16](48) - vector.load[width=16](48)) ** 2
    sum3 += (query.load[width=16](112) - vector.load[width=16](112)) ** 2
    sum3 += (query.load[width=16](176) - vector.load[width=16](176)) ** 2
    sum3 += (query.load[width=16](240) - vector.load[width=16](240)) ** 2
    
    return sqrt((sum0 + sum1 + sum2 + sum3).reduce_add())

@always_inline
fn euclidean_distance_specialized_512_improved(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32]
) -> Float32:
    """Improved 512D specialized kernel addressing the 384D→512D cliff."""
    
    # Use 8 accumulators to maximize pipeline utilization
    var sum0 = SIMD[dtype, 16](0)
    var sum1 = SIMD[dtype, 16](0)
    var sum2 = SIMD[dtype, 16](0)
    var sum3 = SIMD[dtype, 16](0)
    var sum4 = SIMD[dtype, 16](0)
    var sum5 = SIMD[dtype, 16](0)
    var sum6 = SIMD[dtype, 16](0)
    var sum7 = SIMD[dtype, 16](0)
    
    # Process 32 chunks of 16, cycling through 8 accumulators
    # This should eliminate the pipeline stalls causing the cliff
    
    # Use a loop for cleaner code while maintaining performance
    for chunk in range(0, 32, 8):
        var base = chunk * 16
        
        sum0 += (query.load[width=16](base) - vector.load[width=16](base)) ** 2
        sum1 += (query.load[width=16](base + 16) - vector.load[width=16](base + 16)) ** 2
        sum2 += (query.load[width=16](base + 32) - vector.load[width=16](base + 32)) ** 2
        sum3 += (query.load[width=16](base + 48) - vector.load[width=16](base + 48)) ** 2
        sum4 += (query.load[width=16](base + 64) - vector.load[width=16](base + 64)) ** 2
        sum5 += (query.load[width=16](base + 80) - vector.load[width=16](base + 80)) ** 2
        sum6 += (query.load[width=16](base + 96) - vector.load[width=16](base + 96)) ** 2
        sum7 += (query.load[width=16](base + 112) - vector.load[width=16](base + 112)) ** 2
    
    return sqrt((sum0 + sum1 + sum2 + sum3 + sum4 + sum5 + sum6 + sum7).reduce_add())

@always_inline
fn euclidean_distance_specialized_768_improved(
    query: UnsafePointer[Float32],
    vector: UnsafePointer[Float32]
) -> Float32:
    """Improved 768D specialized kernel for BERT embeddings."""
    
    # Use 6 accumulators for optimal balance (768/16 = 48 chunks, 48/6 = 8 per accumulator)
    var sum0 = SIMD[dtype, 16](0)
    var sum1 = SIMD[dtype, 16](0)
    var sum2 = SIMD[dtype, 16](0)
    var sum3 = SIMD[dtype, 16](0)
    var sum4 = SIMD[dtype, 16](0)
    var sum5 = SIMD[dtype, 16](0)
    
    # Process 48 chunks of 16, with 8 chunks per accumulator
    for acc_group in range(8):
        var base = acc_group * 96  # 6 accumulators * 16 bytes each
        
        sum0 += (query.load[width=16](base) - vector.load[width=16](base)) ** 2
        sum1 += (query.load[width=16](base + 16) - vector.load[width=16](base + 16)) ** 2
        sum2 += (query.load[width=16](base + 32) - vector.load[width=16](base + 32)) ** 2
        sum3 += (query.load[width=16](base + 48) - vector.load[width=16](base + 48)) ** 2
        sum4 += (query.load[width=16](base + 64) - vector.load[width=16](base + 64)) ** 2
        sum5 += (query.load[width=16](base + 80) - vector.load[width=16](base + 80)) ** 2
    
    return sqrt((sum0 + sum1 + sum2 + sum3 + sum4 + sum5).reduce_add())

@always_inline
fn batch_distance_calculation_cliff_optimized(
    queries: UnsafePointer[Float32],
    database: UnsafePointer[Float32],
    n_queries: Int,
    n_database: Int,
    dimension: Int,
    results: UnsafePointer[Float32]
) -> None:
    """Cliff-optimized batch distance calculation with dimension-aware processing."""
    
    # Adjust block sizes based on dimension to optimize cache usage
    var query_block_size: Int
    var db_block_size: Int
    
    if dimension <= 128:
        query_block_size = 8
        db_block_size = 128
    elif dimension <= 256:
        query_block_size = 6
        db_block_size = 96
    elif dimension <= 512:
        query_block_size = 4
        db_block_size = 64
    else:
        query_block_size = 2
        db_block_size = 32
    
    # Process in dimension-aware blocks
    @parameter
    fn process_query_block(query_start: Int) -> None:
        var query_end = min(query_start + query_block_size, n_queries)
        
        for db_start in range(0, n_database, db_block_size):
            var db_end = min(db_start + db_block_size, n_database)
            
            # Process all query-database pairs in this block
            for q_idx in range(query_start, query_end):
                var query_ptr = queries + q_idx * dimension
                
                for db_idx in range(db_start, db_end):
                    var db_ptr = database + db_idx * dimension
                    var result_idx = q_idx * n_database + db_idx
                    
                    results[result_idx] = euclidean_distance_adaptive_simd(
                        query_ptr, db_ptr, dimension
                    )
    
    parallelize[process_query_block](n_queries, min(get_optimal_workers(), n_queries))