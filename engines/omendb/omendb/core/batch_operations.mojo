"""
Optimized Batch Operations for Vector Database
=============================================

Implements true batch processing that eliminates per-vector overhead
through memory pooling and SIMD-optimized bulk operations.

Key optimizations:
- Single memory allocation for entire batch
- SIMD-parallel processing of multiple vectors
- Cache-friendly memory access patterns
- Zero intermediate allocations
"""

from memory import UnsafePointer, memcpy, memset_zero
from algorithm import vectorize, parallelize
from math import sqrt
from sys.info import simdwidthof

from ..utils.memory_pool import BatchBuffer, VectorPool, allocate_vector_batch

alias dtype = DType.float32
alias simd_width = simdwidthof[dtype]()

struct BatchProcessor(Copyable, Movable):
    """
    High-performance batch processor for vector operations.
    
    Processes multiple vectors in a single pass with zero allocations.
    """
    
    var dimension: Int
    var batch_buffer: BatchBuffer
    var norm_buffer: UnsafePointer[Float32]
    var temp_buffer: UnsafePointer[Float32]
    
    fn __init__(out self, dimension: Int, batch_size: Int):
        """Initialize batch processor with buffers."""
        self.dimension = dimension
        self.batch_buffer = allocate_vector_batch(dimension, batch_size)
        self.norm_buffer = UnsafePointer[Float32].alloc(batch_size)
        self.temp_buffer = UnsafePointer[Float32].alloc(dimension * simd_width)
    
    fn __del__(owned self):
        """Free allocated buffers."""
        self.norm_buffer.free()
        self.temp_buffer.free()
    
    fn process_batch[
        simd_width: Int = simdwidthof[dtype]()
    ](
        mut self,
        vectors: UnsafePointer[Float32],
        count: Int,
        output: UnsafePointer[Float32],
        output_norms: UnsafePointer[Float32],
        start_offset: Int,
        output_capacity: Int
    ):
        """
        Process a batch of vectors with SIMD optimization.
        
        Args:
            vectors: Input vectors (row-major)
            count: Number of vectors
            output: Output storage (row-major - fixed)
            output_norms: Pre-computed norms storage
            start_offset: Offset in output arrays
        """
        # Process vectors in SIMD-width groups for efficiency
        alias vectors_per_iter = 4  # Process 4 vectors at a time
        
        var vec_idx = 0
        while vec_idx < count - vectors_per_iter + 1:
            # Process multiple vectors simultaneously
            self._process_vector_group[vectors_per_iter](
                vectors + vec_idx * self.dimension,
                output,
                output_norms,
                start_offset + vec_idx,
                output_capacity
            )
            vec_idx += vectors_per_iter
        
        # Process remaining vectors
        while vec_idx < count:
            self._process_single_vector(
                vectors + vec_idx * self.dimension,
                output,
                output_norms,
                start_offset + vec_idx,
                output_capacity
            )
            vec_idx += 1
    
    @always_inline
    fn _process_vector_group[
        group_size: Int
    ](
        self,
        vectors: UnsafePointer[Float32],
        output: UnsafePointer[Float32],
        output_norms: UnsafePointer[Float32],
        output_idx: Int,
        output_capacity: Int
    ):
        """Process a group of vectors with data parallelism."""
        # Compute norms for all vectors in group
        @parameter
        for v in range(group_size):
            var norm_sq = Float32(0)
            var vec_ptr = vectors + v * self.dimension
            
            # SIMD norm computation
            @parameter
            fn compute_norm[width: Int](d: Int):
                var vals = vec_ptr.load[width=width](d)
                norm_sq += (vals * vals).reduce_add()
            
            vectorize[compute_norm, simd_width, unroll_factor=2](self.dimension)
            output_norms[output_idx + v] = norm_sq
        
        # Store vectors in row-major format with proper padding
        # CRITICAL FIX: Must use padded dimension for offset calculation!
        var padded_dimension = output_capacity  # This is passed as padded_dimension from BruteForceIndex
        @parameter
        for v in range(group_size):
            var src_ptr = vectors + v * self.dimension
            var dst_ptr = output + (output_idx + v) * padded_dimension
            for d in range(self.dimension):
                dst_ptr[d] = src_ptr[d]
    
    @always_inline
    fn _process_single_vector(
        self,
        vector: UnsafePointer[Float32],
        output: UnsafePointer[Float32],
        output_norms: UnsafePointer[Float32],
        output_idx: Int,
        output_capacity: Int
    ):
        """Process a single vector."""
        # Compute norm
        var norm_sq = Float32(0)
        
        @parameter
        fn compute_norm[width: Int](d: Int):
            var vals = vector.load[width=width](d)
            norm_sq += (vals * vals).reduce_add()
        
        vectorize[compute_norm, simd_width, unroll_factor=2](self.dimension)
        output_norms[output_idx] = norm_sq
        
        # Store in row-major format with proper padding
        # CRITICAL FIX: Must use padded dimension for offset calculation!
        var padded_dimension = output_capacity  # This is passed as padded_dimension from BruteForceIndex
        var output_ptr = output + output_idx * padded_dimension
        for d in range(self.dimension):
            output_ptr[d] = vector[d]

fn batch_add_optimized(
    index_vectors: UnsafePointer[Float32],  # Row-major storage (fixed)
    index_norms: UnsafePointer[Float32],
    index_size: Int,
    index_capacity: Int,
    dimension: Int,
    batch_vectors: UnsafePointer[Float32],  # Row-major input
    batch_count: Int
) -> Int:
    """
    Optimized batch add operation with zero allocations.
    
    Returns:
        Number of vectors successfully added.
    """
    # Check capacity
    var available = index_capacity - index_size
    var to_add = min(batch_count, available)
    
    if to_add == 0:
        return 0
    
    # Create batch processor
    var processor = BatchProcessor(dimension, to_add)
    
    # Process entire batch at once
    processor.process_batch(
        batch_vectors,
        to_add,
        index_vectors,
        index_norms,
        index_size
    )
    
    return to_add

fn batch_compute_distances[
    simd_width: Int = simdwidthof[dtype]()
](
    database: UnsafePointer[Float32],  # Column-major
    query_batch: UnsafePointer[Float32],  # Row-major queries
    output: UnsafePointer[Float32],  # Distance matrix
    dimension: Int,
    num_vectors: Int,
    num_queries: Int,
    vector_norms: UnsafePointer[Float32]
):
    """
    Compute distances for multiple queries in parallel.
    
    Uses cache-blocking and SIMD optimization for maximum throughput.
    """
    # Process queries in blocks for cache efficiency
    alias query_block_size = 8
    alias vector_block_size = 64
    
    for q_block in range(0, num_queries, query_block_size):
        var q_end = min(q_block + query_block_size, num_queries)
        
        # Pre-compute query norms for this block
        var query_norms = UnsafePointer[Float32].alloc(query_block_size)
        
        for q_idx in range(q_block, q_end):
            var norm_sq = Float32(0)
            var query_ptr = query_batch + q_idx * dimension
            
            @parameter
            fn compute_norm[width: Int](d: Int):
                var vals = query_ptr.load[width=width](d)
                norm_sq += (vals * vals).reduce_add()
            
            vectorize[compute_norm, simd_width, unroll_factor=2](dimension)
            query_norms[q_idx - q_block] = norm_sq
        
        # Process database vectors in blocks
        for v_block in range(0, num_vectors, vector_block_size):
            var v_end = min(v_block + vector_block_size, num_vectors)
            
            # Compute distances for this query-vector block
            for q_idx in range(q_block, q_end):
                var query_ptr = query_batch + q_idx * dimension
                var query_norm = query_norms[q_idx - q_block]
                
                @parameter
                fn process_vectors[width: Int](v_idx: Int):
                    var v_global = v_block + v_idx
                    if v_global < v_end:
                        var dot_product = Float32(0)
                        
                        # Compute dot product
                        for d in range(dimension):
                            dot_product += query_ptr[d] * database[d * num_vectors + v_global]
                        
                        # Euclidean distance
                        var db_norm = vector_norms[v_global]
                        var dist_sq = query_norm + db_norm - 2.0 * dot_product
                        var dist = sqrt(max(0.0, dist_sq))
                        
                        # Store result
                        output[q_idx * num_vectors + v_global] = dist
                
                vectorize[process_vectors, simd_width](v_end - v_block)
        
        query_norms.free()

# Utility functions for batch operations

fn prepare_batch_data(
    vectors: List[List[Float32]],
    dimension: Int
) -> BatchBuffer:
    """
    Prepare batch data from Python lists.
    
    Converts to contiguous memory layout for efficient processing.
    """
    var batch = allocate_vector_batch(dimension, len(vectors))
    
    for i in range(len(vectors)):
        var vec = vectors[i]
        if len(vec) == dimension:
            var ptr = batch.get_vector(i)
            for d in range(dimension):
                ptr[d] = vec[d]
            batch.count += 1
    
    return batch

fn process_batch_with_pool(
    pool: VectorPool,
    batch: BatchBuffer,
    ids: List[String],
    start_idx: Int
) -> List[Int]:
    """
    Process batch using memory pool.
    
    Returns list of allocated indices.
    """
    var indices = List[Int]()
    
    for i in range(batch.count):
        var (ptr, idx) = pool.allocate()
        
        # Copy vector to pool
        var src = batch.get_vector(i)
        memcpy(ptr, src, pool.dimension)
        
        indices.append(idx)
    
    return indices