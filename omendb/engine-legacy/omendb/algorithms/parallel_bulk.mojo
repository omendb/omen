"""
Parallel bulk insertion implementation for HNSW.
Uses Mojo's parallelize for multi-threaded vector insertion.
Target: 3-4x speedup over sequential insertion.
"""

from algorithm import parallelize
from memory import UnsafePointer, memcpy
from python import Python, PythonObject

fn parallel_vector_copy(
    dest: UnsafePointer[Float32],
    src: UnsafePointer[Float32], 
    n_vectors: Int,
    dimension: Int,
    num_threads: Int = 8
):
    """
    Parallel copy of vector data using multiple threads.
    Each thread copies a chunk of vectors.
    """
    
    @parameter
    fn copy_chunk(thread_id: Int):
        # Calculate chunk boundaries
        var chunk_size = n_vectors // num_threads
        var start_idx = thread_id * chunk_size
        var end_idx = start_idx + chunk_size
        
        # Last thread handles remainder
        if thread_id == num_threads - 1:
            end_idx = n_vectors
        
        # Copy this thread's chunk
        for i in range(start_idx, end_idx):
            var src_offset = i * dimension
            var dest_offset = i * dimension
            
            # Use memcpy for each vector
            memcpy(
                dest.offset(dest_offset),
                src.offset(src_offset),
                dimension * sizeof[Float32]()
            )
    
    # Execute parallel copy
    parallelize[copy_chunk](num_threads, num_threads)


fn parallel_distance_batch(
    query: UnsafePointer[Float32],
    vectors: UnsafePointer[Float32],
    n_vectors: Int,
    dimension: Int,
    distances: UnsafePointer[Float32],
    num_threads: Int = 8
):
    """
    Calculate distances from query to all vectors in parallel.
    Each thread processes a chunk of vectors.
    """
    
    @parameter
    fn calc_distances_chunk(thread_id: Int):
        # Calculate chunk boundaries
        var chunk_size = n_vectors // num_threads
        var start_idx = thread_id * chunk_size
        var end_idx = start_idx + chunk_size
        
        # Last thread handles remainder
        if thread_id == num_threads - 1:
            end_idx = n_vectors
        
        # Calculate distances for this chunk
        for i in range(start_idx, end_idx):
            var vec_offset = i * dimension
            var vec_ptr = vectors.offset(vec_offset)
            
            # L2 distance calculation
            var sum = Float32(0)
            for d in range(dimension):
                var diff = query[d] - vec_ptr[d]
                sum += diff * diff
            
            distances[i] = sum  # Store squared distance (sqrt later if needed)
    
    # Execute parallel distance calculation
    parallelize[calc_distances_chunk](num_threads, num_threads)


fn parallel_graph_construction(
    node_ids: List[Int],
    vectors: UnsafePointer[Float32],
    dimension: Int,
    M: Int,
    num_threads: Int = 8
) -> Bool:
    """
    Build HNSW graph connections in parallel.
    Each thread handles a subset of nodes.
    
    Note: This is a simplified version. Full implementation would need:
    - Thread-safe neighbor updates
    - Proper locking for shared edges
    - Layer management
    """
    
    var n_nodes = len(node_ids)
    
    @parameter
    fn build_connections_chunk(thread_id: Int):
        # Calculate chunk boundaries
        var chunk_size = n_nodes // num_threads
        var start_idx = thread_id * chunk_size
        var end_idx = start_idx + chunk_size
        
        # Last thread handles remainder
        if thread_id == num_threads - 1:
            end_idx = n_nodes
        
        # Build connections for nodes in this chunk
        for i in range(start_idx, end_idx):
            var node_id = node_ids[i]
            var node_vec = vectors.offset(node_id * dimension)
            
            # Find M nearest neighbors (simplified)
            # In production, this would use the full HNSW search
            
            # For now, just mark as processed
            # Real implementation would update graph edges
            pass
    
    # Execute parallel graph construction
    parallelize[build_connections_chunk](num_threads, num_threads)
    
    return True


struct ParallelBulkInserter:
    """
    Manages parallel bulk insertion with thread coordination.
    """
    
    var num_threads: Int
    var dimension: Int
    
    fn __init__(inout self, dimension: Int, num_threads: Int = 8):
        """Initialize with dimension and thread count."""
        self.dimension = dimension
        self.num_threads = num_threads
    
    fn insert_parallel(
        self,
        vectors: UnsafePointer[Float32],
        n_vectors: Int,
        dest_storage: UnsafePointer[Float32],
        base_idx: Int
    ) -> Int:
        """
        Perform parallel insertion of vectors.
        
        Returns: Number of vectors successfully inserted
        """
        
        # Phase 1: Parallel vector copy
        var dest_offset = dest_storage.offset(base_idx * self.dimension)
        parallel_vector_copy(
            dest_offset,
            vectors,
            n_vectors,
            self.dimension,
            self.num_threads
        )
        
        # Phase 2: Would do parallel graph construction here
        # For now, just return success
        
        return n_vectors
    
    fn search_parallel(
        self,
        query: UnsafePointer[Float32],
        vectors: UnsafePointer[Float32],
        n_vectors: Int,
        k: Int
    ) -> List[Tuple[Int, Float32]]:
        """
        Parallel k-NN search across all vectors.
        """
        
        # Allocate distance buffer
        var distances = UnsafePointer[Float32].alloc(n_vectors)
        
        # Calculate all distances in parallel
        parallel_distance_batch(
            query,
            vectors,
            n_vectors,
            self.dimension,
            distances,
            self.num_threads
        )
        
        # Find top-k (sequential for now, could parallelize with reduction)
        var results = List[Tuple[Int, Float32]]()
        
        for _ in range(min(k, n_vectors)):
            var min_dist = Float32(1e9)
            var min_idx = -1
            
            for i in range(n_vectors):
                if distances[i] < min_dist:
                    min_dist = distances[i]
                    min_idx = i
            
            if min_idx >= 0:
                results.append((min_idx, min_dist))
                distances[min_idx] = Float32(1e9)  # Mark as used
        
        distances.free()
        return results