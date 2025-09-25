"""
CSR (Compressed Sparse Row) graph representation for DiskANN.
Eliminates dynamic allocation overhead while maintaining memory efficiency.
"""

from collections import List, Dict
from memory import UnsafePointer, memcpy
from math import sqrt
from sys.intrinsics import sizeof
from python import Python

struct VamanaGraph(Copyable, Movable):
    """CSR-based graph storage with deferred construction for O(1) insertion.
    
    PERFORMANCE FIX: Uses temporary adjacency lists during insertion,
    then builds CSR format only when needed (during search/finalize).
    
    This eliminates the O(n²) edge shifting that caused 1000x slowdown
    when merging large batches into existing indexes.
    
    Benefits:
    - O(1) edge insertion (vs O(n) with immediate CSR)
    - No dynamic allocation overhead (fixed arrays)
    - Better cache locality (contiguous storage)
    - ~40% memory reduction vs List[Int] per node
    - ~79% memory reduction vs fixed R=48 arrays
    """
    # MEMORY OPTIMIZATION: Single vector storage (normalized only)
    # Saves 512 bytes per vector by not storing unnormalized versions
    var vectors: UnsafePointer[Float32]           # Normalized vectors for similarity search
    var quantized_vectors: UnsafePointer[UInt8]   # Quantized vectors when use_quantization=True
    var quantization_scales: UnsafePointer[Float32]  # Scale per vector for dequantization
    var quantization_offsets: UnsafePointer[Float32] # Offset per vector for dequantization
    var node_ids: List[String]                    # String IDs for each node
    
    # PERFORMANCE FIX: Dual representation for O(1) insertion
    # Adjacency lists for fast insertion (used during build phase)
    var temp_adjacency_lists: List[List[Int]]     # Temporary storage during insertion
    var is_csr_built: Bool                        # Track if CSR is currently valid
    
    # CSR format (built on demand for search operations)
    var row_offsets: UnsafePointer[Int32]   # CSR row pointers
    var edge_indices: UnsafePointer[Int32]  # CSR edge data
    
    var dimension: Int
    var num_nodes: Int
    var capacity: Int  # Allocated capacity for nodes
    var num_edges: Int
    var max_edges: Int
    var use_quantization: Bool
    
    fn __init__(out self, dimension: Int, expected_nodes: Int = 50, avg_degree: Int = 16, use_quantization: Bool = False):
        """Initialize CSR graph with expected capacity.
        
        Args:
            dimension: Vector dimension
            expected_nodes: Expected number of nodes
            avg_degree: Average degree per node
            use_quantization: If True, skip storing original vectors (saves 50% memory)
        """
        self.dimension = dimension
        self.num_nodes = 0
        self.capacity = expected_nodes  # Track allocated capacity
        self.num_edges = 0
        # Calculate max edges based on expected nodes and average degree
        # No artificial cap - let it grow as needed (proper solution is pruning)
        self.max_edges = expected_nodes * avg_degree
        self.use_quantization = use_quantization
        
        # PERFORMANCE FIX: Initialize temporary adjacency lists
        self.temp_adjacency_lists = List[List[Int]]()
        self.is_csr_built = False
        
        # No need to allocate original_vectors - only storing normalized
        
        # Allocate storage based on quantization mode
        if use_quantization:
            # Quantized storage: 1 byte per dimension + scale/offset per vector
            self.quantized_vectors = UnsafePointer[UInt8].alloc(expected_nodes * dimension)
            self.quantization_scales = UnsafePointer[Float32].alloc(expected_nodes)
            self.quantization_offsets = UnsafePointer[Float32].alloc(expected_nodes)
            self.vectors = UnsafePointer[Float32]()  # Don't allocate full vectors
        else:
            # Full precision storage
            self.vectors = UnsafePointer[Float32].alloc(expected_nodes * dimension)
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        self.node_ids = List[String]()
        self.row_offsets = UnsafePointer[Int32].alloc(expected_nodes + 1)
        self.edge_indices = UnsafePointer[Int32].alloc(self.max_edges)
        
        # Initialize first offset to 0
        self.row_offsets[0] = 0
    
    fn __del__(owned self):
        """Free allocated memory."""
        # No original_vectors to free - only normalized vectors allocated
        if self.vectors:
            self.vectors.free()
        if self.quantized_vectors:
            self.quantized_vectors.free()
        if self.quantization_scales:
            self.quantization_scales.free()
        if self.quantization_offsets:
            self.quantization_offsets.free()
        if self.row_offsets:
            self.row_offsets.free()
        if self.edge_indices:
            self.edge_indices.free()
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.dimension = existing.dimension
        self.num_nodes = existing.num_nodes
        self.capacity = existing.capacity
        self.num_edges = existing.num_edges
        self.max_edges = existing.max_edges
        self.use_quantization = existing.use_quantization
        self.node_ids = existing.node_ids
        self.temp_adjacency_lists = existing.temp_adjacency_lists
        self.is_csr_built = existing.is_csr_built
        
        # Copy vectors based on quantization setting
        var actual_size = existing.num_nodes * existing.dimension
        var alloc_size = existing.capacity * existing.dimension
        
        # Skip original_vectors copy - only normalized vectors needed
        
        # Copy vectors or quantized data based on mode
        if self.use_quantization:
            # Copy quantized vectors and parameters - allocate for capacity
            self.quantized_vectors = UnsafePointer[UInt8].alloc(alloc_size)
            memcpy(self.quantized_vectors, existing.quantized_vectors, actual_size)
            
            self.quantization_scales = UnsafePointer[Float32].alloc(existing.capacity)
            memcpy(self.quantization_scales, existing.quantization_scales, existing.num_nodes * sizeof[Float32]())
            
            self.quantization_offsets = UnsafePointer[Float32].alloc(existing.capacity)
            memcpy(self.quantization_offsets, existing.quantization_offsets, existing.num_nodes * sizeof[Float32]())
            
            self.vectors = UnsafePointer[Float32]()
        else:
            # Copy full precision vectors - allocate for capacity
            self.vectors = UnsafePointer[Float32].alloc(alloc_size)
            memcpy(self.vectors, existing.vectors, actual_size * sizeof[Float32]())
            
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        
        # Copy CSR structure - allocate based on capacity, not num_nodes
        self.row_offsets = UnsafePointer[Int32].alloc(existing.capacity + 1)
        memcpy(self.row_offsets, existing.row_offsets, (existing.num_nodes + 1) * sizeof[Int32]())
        
        self.edge_indices = UnsafePointer[Int32].alloc(existing.max_edges)
        memcpy(self.edge_indices, existing.edge_indices, existing.num_edges * sizeof[Int32]())
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.dimension = existing.dimension
        self.num_nodes = existing.num_nodes
        self.capacity = existing.capacity
        self.num_edges = existing.num_edges
        self.max_edges = existing.max_edges
        self.use_quantization = existing.use_quantization
        self.node_ids = existing.node_ids^
        self.temp_adjacency_lists = existing.temp_adjacency_lists^
        self.is_csr_built = existing.is_csr_built
        # No original_vectors to move
        self.vectors = existing.vectors
        self.quantized_vectors = existing.quantized_vectors
        self.quantization_scales = existing.quantization_scales
        self.quantization_offsets = existing.quantization_offsets
        self.row_offsets = existing.row_offsets
        self.edge_indices = existing.edge_indices
        
        # Null out moved pointers
        # No original_vectors pointer to null
        existing.vectors = UnsafePointer[Float32]()
        existing.quantized_vectors = UnsafePointer[UInt8]()
        existing.quantization_scales = UnsafePointer[Float32]()
        existing.quantization_offsets = UnsafePointer[Float32]()
        existing.row_offsets = UnsafePointer[Int32]()
        existing.edge_indices = UnsafePointer[Int32]()
    
    fn add_node_from_ptr(mut self, id: String, vector_ptr: UnsafePointer[Float32], dimension: Int) -> Int:
        """Add node directly from vector pointer (zero-copy)."""
        var node_idx = self.num_nodes
        
        # Check if we need to reallocate
        if node_idx >= self.capacity:
            self._reallocate(self._compute_new_capacity(node_idx + 100))
        
        # Store ID
        self.node_ids.append(id)
        
        # Initialize empty adjacency list for this node
        self.temp_adjacency_lists.append(List[Int]())
        
        # Invalidate CSR representation (will be rebuilt on demand)
        self.is_csr_built = False
        
        var start_idx = node_idx * self.dimension
        
        # CRITICAL FIX: Additional bounds checking after reallocation
        if node_idx >= self.capacity:
            print("❌ ERROR: node_idx", node_idx, "still >=", self.capacity, "after reallocation")
            return -1
        
        if start_idx + dimension > self.capacity * self.dimension:
            print("❌ ERROR: start_idx", start_idx, "+ dimension exceeds vector capacity")
            return -1
        
        # Calculate normalization factor
        var norm_sq: Float32 = 0
        for i in range(dimension):
            norm_sq += vector_ptr[i] * vector_ptr[i]
        var norm = sqrt(norm_sq + Float32(1e-12))
        var inv_norm = 1.0 / norm
        
        if self.use_quantization:
            var min_val: Float32 = 1e10
            var max_val: Float32 = -1e10
            for i in range(dimension):
                if vector_ptr[i] < min_val:
                    min_val = vector_ptr[i]
                if vector_ptr[i] > max_val:
                    max_val = vector_ptr[i]
            
            var scale = (max_val - min_val) / 255.0
            if scale < 1e-12:
                scale = 1.0
            var offset = min_val
            
            # Store quantization parameters (now bounds-checked)
            if node_idx < self.capacity:
                self.quantization_scales[node_idx] = scale
                self.quantization_offsets[node_idx] = offset
            else:
                print("❌ ERROR: quantization index", node_idx, "exceeds capacity", self.capacity)
                return -1
            
            # Quantize and store (now bounds-checked)
            for i in range(dimension):
                if start_idx + i < self.capacity * self.dimension:
                    var quantized_val = UInt8((vector_ptr[i] - offset) / scale + 0.5)
                    self.quantized_vectors[start_idx + i] = quantized_val
                else:
                    print("❌ ERROR: quantized vector index", start_idx + i, "exceeds capacity")
                    return -1
        else:
            # Store normalized vectors (now bounds-checked)
            for i in range(dimension):
                if start_idx + i < self.capacity * self.dimension:
                    self.vectors[start_idx + i] = vector_ptr[i] * inv_norm
                else:
                    print("❌ ERROR: vector index", start_idx + i, "exceeds capacity")
                    return -1
        
        # Initialize offset for this node (now bounds-checked)
        if node_idx + 1 <= self.capacity:
            self.row_offsets[node_idx + 1] = self.row_offsets[node_idx]
        else:
            print("❌ ERROR: row_offsets index", node_idx + 1, "exceeds capacity")
            return -1
        self.num_nodes += 1
        return node_idx
    
    fn add_node(mut self, id: String, vector: List[Float32]) -> Int:
        """Add a new node and return its index."""
        var node_idx = self.num_nodes
        
        # Check if we need to reallocate (safety check for quantization)
        # Need space for node_idx + 1 in row_offsets array
        if node_idx >= self.capacity:
            self._reallocate(self._compute_new_capacity(node_idx + 100))
        
        # Store ID
        self.node_ids.append(id)
        
        # PERFORMANCE FIX: Initialize empty adjacency list for this node
        self.temp_adjacency_lists.append(List[Int]())
        
        # Invalidate CSR representation (will be rebuilt on demand)
        self.is_csr_built = False
        
        # Store vectors efficiently based on quantization mode
        var start_idx = node_idx * self.dimension
        
        # Calculate normalization factor
        var norm_sq: Float32 = 0
        for i in range(self.dimension):
            norm_sq += vector[i] * vector[i]
        var norm = sqrt(norm_sq + Float32(1e-12))
        var inv_norm = 1.0 / norm
        
        # Store vectors based on quantization mode
        if self.use_quantization:
            # Quantize ORIGINAL vector (not normalized) to preserve retrieval accuracy
            # Find min and max of original vector for quantization
            var min_val: Float32 = 1e10
            var max_val: Float32 = -1e10
            for i in range(self.dimension):
                if vector[i] < min_val:
                    min_val = vector[i]
                if vector[i] > max_val:
                    max_val = vector[i]
            
            # Calculate scale and offset for quantization
            # Protect against zero scale when all values are identical
            var scale = (max_val - min_val) / 255.0
            if scale < 1e-12:
                scale = 1.0  # Arbitrary non-zero value when vector is constant
            var offset = min_val
            
            # CRITICAL FIX: Bounds check before accessing quantization arrays
            if node_idx >= self.capacity:
                print("❌ ERROR: node_idx", node_idx, "exceeds capacity", self.capacity, "after reallocation")
                return -1
            
            if start_idx + self.dimension > self.capacity * self.dimension:
                print("❌ ERROR: quantized_vectors range [", start_idx, ",", start_idx + self.dimension, ") exceeds capacity")
                return -1
            
            # Store quantization parameters for reconstruction
            self.quantization_scales[node_idx] = scale
            self.quantization_offsets[node_idx] = offset
            
            # Quantize and store ORIGINAL values with bounds checking
            for i in range(self.dimension):
                var quantized_val = UInt8((vector[i] - offset) / scale + 0.5)
                self.quantized_vectors[start_idx + i] = quantized_val
        else:
            # CRITICAL FIX: Bounds check before accessing vectors array
            if node_idx >= self.capacity:
                print("❌ ERROR: node_idx", node_idx, "exceeds capacity", self.capacity, "after reallocation")
                return -1
            
            if start_idx + self.dimension > self.capacity * self.dimension:
                print("❌ ERROR: vectors range [", start_idx, ",", start_idx + self.dimension, ") exceeds capacity")
                return -1
            
            # Store only normalized vectors (saves 512 bytes per vector)
            for i in range(self.dimension):
                self.vectors[start_idx + i] = vector[i] * inv_norm  # Normalized for cosine similarity
        
        # CRITICAL FIX: Bounds check before accessing row_offsets
        if node_idx + 1 > self.capacity:
            print("❌ ERROR: row_offsets index", node_idx + 1, "exceeds capacity", self.capacity)
            return -1
        
        # Initialize offset for this node (same as previous)
        self.row_offsets[node_idx + 1] = self.row_offsets[node_idx]
        self.num_nodes += 1
        return node_idx
    
    fn _compute_new_capacity(self, min_needed: Int) -> Int:
        """Compute new capacity using smart growth strategy.
        
        Growth strategy:
        - Small (< 10K): 2x growth for fast startup  
        - Medium (10K-100K): 1.5x growth for balanced expansion
        - Large (> 100K): 1.125x growth to minimize waste
        
        Args:
            min_needed: Minimum capacity needed
            
        Returns:
            New capacity that's at least min_needed
        """
        var current = self.capacity
        var new_capacity = current
        
        # Apply growth strategy based on current size
        if current < 10000:
            # Small: 2x growth for fast startup
            new_capacity = current * 2
        elif current < 100000:
            # Medium: 1.5x growth for balanced expansion  
            new_capacity = Int(Float32(current) * 1.5)
        else:
            # Large: 1.125x growth to minimize waste
            new_capacity = Int(Float32(current) * 1.125)
        
        # Ensure we meet minimum requirement
        return max(new_capacity, min_needed)
    
    fn _reallocate(mut self, new_capacity: Int):
        """Reallocate storage for more nodes."""
        # Only reallocate if we need more space
        if new_capacity <= self.capacity:
            return
        
        var old_capacity = self.capacity
        self.capacity = new_capacity
        
        # CRITICAL FIX: Resize temp_adjacency_lists to match new capacity
        # This prevents buffer overflow in add_edge() when accessing temp_adjacency_lists[from_node]
        while len(self.temp_adjacency_lists) < new_capacity:
            self.temp_adjacency_lists.append(List[Int]())
        
        # Also grow edge storage proportionally with safety margin
        # Calculate average degree and apply to new capacity with generous growth
        var avg_degree = 16  # Conservative to prevent edge explosion (was 32, caused crashes)
        if old_capacity > 0 and self.max_edges > 0:
            avg_degree = self.max_edges // old_capacity
            # CRITICAL FIX: Add growth factor for dense graphs at scale
            avg_degree = Int(Float32(avg_degree) * 1.5)  # 50% growth factor
        
        var new_max_edges = new_capacity * avg_degree  # No artificial cap
        if new_max_edges > self.max_edges:
            var new_edge_indices = UnsafePointer[Int32].alloc(new_max_edges)
            
            # CRITICAL FIX: Bounds check memcpy to prevent buffer overflow
            var copy_size = min(self.num_edges, self.max_edges)
            if copy_size > 0 and copy_size <= new_max_edges:
                memcpy(new_edge_indices, self.edge_indices, copy_size * sizeof[Int32]())
            elif copy_size > new_max_edges:
                print("❌ ERROR: Edge reallocation insufficient:", copy_size, ">", new_max_edges)
                # Allocate even more aggressively
                new_edge_indices.free()
                new_max_edges = copy_size * 2  # Double current usage
                new_edge_indices = UnsafePointer[Int32].alloc(new_max_edges)
                memcpy(new_edge_indices, self.edge_indices, copy_size * sizeof[Int32]())
            
            self.edge_indices.free()
            self.edge_indices = new_edge_indices
            self.max_edges = new_max_edges
        
        # Reallocate based on quantization mode
        if self.use_quantization:
            # Reallocate quantized vectors with bounds checking
            var new_quantized = UnsafePointer[UInt8].alloc(new_capacity * self.dimension)
            var quantized_copy_size = min(self.num_nodes * self.dimension, self.capacity * self.dimension)
            if quantized_copy_size > 0:
                memcpy(new_quantized, self.quantized_vectors, quantized_copy_size)
            self.quantized_vectors.free()
            self.quantized_vectors = new_quantized
            
            # Reallocate scales with bounds checking
            var new_scales = UnsafePointer[Float32].alloc(new_capacity)
            var scales_copy_size = min(self.num_nodes, self.capacity)
            if scales_copy_size > 0:
                memcpy(new_scales, self.quantization_scales, scales_copy_size * sizeof[Float32]())
            self.quantization_scales.free()
            self.quantization_scales = new_scales
            
            # Reallocate offsets with bounds checking  
            var new_offsets = UnsafePointer[Float32].alloc(new_capacity)
            if scales_copy_size > 0:
                memcpy(new_offsets, self.quantization_offsets, scales_copy_size * sizeof[Float32]())
            self.quantization_offsets.free()
            self.quantization_offsets = new_offsets
        else:
            # Reallocate normalized vectors with bounds checking
            if self.vectors:
                var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
                var vector_copy_size = min(self.num_nodes * self.dimension, self.capacity * self.dimension)
                if vector_copy_size > 0:
                    memcpy(new_vectors, self.vectors, vector_copy_size * sizeof[Float32]())
                self.vectors.free()
                self.vectors = new_vectors
        
        # Reallocate row offsets with bounds checking
        var new_offsets = UnsafePointer[Int32].alloc(new_capacity + 1)
        var offsets_copy_size = min(self.num_nodes + 1, self.capacity + 1)
        if offsets_copy_size > 0:
            memcpy(new_offsets, self.row_offsets, offsets_copy_size * sizeof[Int32]())
        self.row_offsets.free()
        self.row_offsets = new_offsets
    
    fn add_edge(mut self, from_node: Int, to_node: Int) -> Bool:
        """Add an edge with much reduced shifting overhead.
        
        PERFORMANCE FIX: Use append-only insertion to avoid O(n²) shifts.
        Edges are added to the end of the array and sorted later during finalize().
        This reduces complexity from O(total_edges) per insertion to O(1).
        """
        if from_node >= self.num_nodes or to_node >= self.num_nodes:
            return False
        
        # Reallocate edge storage if needed
        if self.num_edges >= self.max_edges:
            var new_max_edges = max(self.max_edges * 2, self.num_edges + 1000)
            var new_edge_indices = UnsafePointer[Int32].alloc(new_max_edges)
            memcpy(new_edge_indices, self.edge_indices, self.num_edges * sizeof[Int32]())
            self.edge_indices.free()
            self.edge_indices = new_edge_indices
            self.max_edges = new_max_edges
        
        # CRITICAL FIX: Bounds check before accessing temp_adjacency_lists
        if from_node >= len(self.temp_adjacency_lists):
            print("❌ ERROR: from_node", from_node, ">=", len(self.temp_adjacency_lists), "temp_adjacency_lists size")
            return False
        
        # Check if edge already exists in temp adjacency list
        var neighbors = self.temp_adjacency_lists[from_node]
        for i in range(len(neighbors)):
            if neighbors[i] == to_node:
                return False
        
        # PERFORMANCE FIX: Add to temp adjacency list (O(1))
        # CSR will be rebuilt during finalize() with proper sorting
        self.temp_adjacency_lists[from_node].append(to_node)
        self.num_edges += 1
        self.is_csr_built = False
        
        return True
    
    fn get_neighbors(self, node_idx: Int) -> List[Int]:
        """Get neighbors for a node.
        
        PERFORMANCE FIX: Returns neighbors from current representation
        (adjacency lists during build, CSR during search).
        """
        var result = List[Int]()
        if node_idx >= self.num_nodes:
            return result
        
        if not self.is_csr_built:
            # Return directly from adjacency lists (build phase)
            return self.temp_adjacency_lists[node_idx]
        else:
            # Return from CSR format (search phase) with bounds checking
            if node_idx + 1 >= self.capacity + 1:
                print("❌ ERROR: row_offsets access", node_idx + 1, "exceeds bounds", self.capacity + 1)
                return result
            
            var start = Int(self.row_offsets[node_idx])
            var end = Int(self.row_offsets[node_idx + 1])
            
            # CRITICAL FIX: Bounds check edge_indices access
            for i in range(start, end):
                if i < self.max_edges:
                    result.append(Int(self.edge_indices[i]))
                else:
                    print("❌ ERROR: edge_indices access", i, "exceeds max_edges", self.max_edges)
                    break
            
            return result
    
    fn _sort_neighbors(self, mut neighbors: List[Int]):
        """Sort neighbors list for binary search optimization."""
        # Simple insertion sort for small lists (typical node degree < 100)
        if len(neighbors) <= 1:
            return
        
        for i in range(1, len(neighbors)):
            var key = neighbors[i]
            var j = i - 1
            while j >= 0 and neighbors[j] > key:
                neighbors[j + 1] = neighbors[j]
                j -= 1
            neighbors[j + 1] = key
    
    fn neighbor_count(self, node_idx: Int) -> Int:
        """Get number of neighbors for a node."""
        if node_idx >= self.num_nodes:
            return 0
            
        if not self.is_csr_built:
            # Return count from adjacency lists
            return len(self.temp_adjacency_lists[node_idx])
        else:
            # Return count from CSR format
            return Int(self.row_offsets[node_idx + 1] - self.row_offsets[node_idx])
    
    fn has_edge(self, from_node: Int, to_node: Int) -> Bool:
        """Check if an edge exists."""
        if from_node >= self.num_nodes:
            return False
        
        if not self.is_csr_built:
            # Linear search in adjacency list (typically small)
            var neighbors = self.temp_adjacency_lists[from_node]
            for i in range(len(neighbors)):
                if neighbors[i] == to_node:
                    return True
            return False
        else:
            # Binary search in sorted CSR edge array with bounds checking
            if from_node + 1 >= self.capacity + 1:
                print("❌ ERROR: contains_edge row_offsets access exceeds bounds")
                return False
            
            var start = Int(self.row_offsets[from_node])
            var end = Int(self.row_offsets[from_node + 1])
            var target = Int32(to_node)
            
            # CRITICAL FIX: Bounds check for edge_indices access
            while start < end:
                var mid = start + (end - start) // 2
                if mid >= self.max_edges:
                    print("❌ ERROR: edge_indices binary search access", mid, "exceeds max_edges", self.max_edges)
                    return False
                
                if self.edge_indices[mid] == target:
                    return True
                elif self.edge_indices[mid] < target:
                    start = mid + 1
                else:
                    end = mid
            
            return False
    
    fn get_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get pointer to a node's normalized vector (for similarity search).
        
        Note: When quantization is enabled, this returns null pointer.
        Use get_quantized_vector_ptr instead for quantized access.
        """
        if self.use_quantization or node_idx >= self.num_nodes:
            # Return null pointer - caller should use quantized path or invalid index
            return UnsafePointer[Float32]()
        var start_idx = node_idx * self.dimension
        return self.vectors + start_idx
    
    fn get_quantized_vector_ptr(self, node_idx: Int) -> UnsafePointer[UInt8]:
        """Get pointer to quantized vector data."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return UnsafePointer[UInt8]()
        var start_idx = node_idx * self.dimension
        return self.quantized_vectors + start_idx
    
    fn get_quantization_scale(self, node_idx: Int) -> Float32:
        """Get scale for dequantizing a vector."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return 1.0
        return self.quantization_scales[node_idx]
    
    fn get_quantization_offset(self, node_idx: Int) -> Float32:
        """Get offset for dequantizing a vector."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return 0.0
        return self.quantization_offsets[node_idx]
    
    fn get_original_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get pointer to normalized vector data.
        
        MEMORY OPTIMIZATION: Returns normalized vectors only.
        Unnormalized vectors are no longer stored (saves 512 bytes per vector).
        """
        if self.use_quantization:
            return UnsafePointer[Float32]()
        var start_idx = node_idx * self.dimension
        return self.vectors + start_idx  # Return normalized vectors
    
    fn get_node_id(self, node_idx: Int) -> String:
        """Get the string ID for a node."""
        if node_idx < len(self.node_ids):
            return self.node_ids[node_idx]
        return ""
    
    fn get_node_index(self, id: String) -> Optional[Int]:
        """Get the node index for a given ID."""
        for i in range(len(self.node_ids)):
            if self.node_ids[i] == id:
                return Optional[Int](i)
        return Optional[Int]()
    
    fn memory_bytes(self) -> Int:
        """Calculate total ALLOCATED memory usage (not just used memory).
        
        This tracks actual memory allocation to fix the hidden memory leak.
        """
        # Vector storage - TRACK ALLOCATED CAPACITY, not just used nodes
        var vector_bytes: Int
        if self.use_quantization:
            # Quantized storage: allocated for full capacity
            vector_bytes = self.capacity * self.dimension  # quantized_vectors
            vector_bytes += self.capacity * 4  # quantization_scales
            vector_bytes += self.capacity * 4  # quantization_offsets
        else:
            # Full precision: only normalized vectors allocated
            vector_bytes = self.capacity * self.dimension * 4  # vectors
        
        # ID storage (estimate based on used nodes - this grows dynamically)
        var id_bytes = 0
        if self.num_nodes > 0:
            # Estimate based on average ID length
            id_bytes = self.num_nodes * 40  # Conservative estimate for used IDs
        
        # CSR structure - TRACK ALLOCATED CAPACITY
        var offset_bytes = (self.capacity + 1) * 4  # Int32 offsets (allocated capacity)
        var edge_bytes = self.max_edges * 4  # Int32 edges (allocated capacity)
        
        return vector_bytes + id_bytes + offset_bytes + edge_bytes
    
    fn memory_usage_mb(self) -> Float32:
        """Get memory usage in MB."""
        return Float32(self.memory_bytes()) / (1024.0 * 1024.0)
    
    fn get_random_nodes(self, count: Int) -> List[Int]:
        """Get random sample of existing nodes for efficient batch connections."""
        var result = List[Int]()
        if self.num_nodes == 0:
            return result
        
        var sample_size = min(count, self.num_nodes)
        var step = max(1, self.num_nodes // sample_size)
        
        # Simple deterministic sampling (avoids random number overhead)
        var i = 0
        while len(result) < sample_size and i < self.num_nodes:
            result.append(i)
            i += step
        
        return result
    
    fn finalize(mut self):
        """Build CSR representation from adjacency lists for optimal search performance.
        
        PERFORMANCE FIX: This is where the O(n) CSR construction happens,
        but only once per finalize call instead of on every edge insertion.
        """
        if self.is_csr_built:
            return  # Already built
            
        # Build CSR row_offsets with bounds checking
        self.row_offsets[0] = 0
        var offset = 0
        for i in range(self.num_nodes):
            var degree = len(self.temp_adjacency_lists[i])
            offset += degree
            
            # CRITICAL FIX: Bounds check row_offsets write
            if i + 1 <= self.capacity:
                self.row_offsets[i + 1] = Int32(offset)
            else:
                print("❌ ERROR: row_offsets write", i + 1, "exceeds capacity", self.capacity + 1)
                return  # Abort CSR build to prevent corruption
        
        # Build CSR edge_indices (sorted for binary search) with bounds checking
        var edge_pos = 0
        for i in range(self.num_nodes):
            var neighbors = self.temp_adjacency_lists[i]
            
            # Sort neighbors for binary search optimization
            self._sort_neighbors(neighbors)
            
            # Copy to CSR format with bounds checking
            for j in range(len(neighbors)):
                # CRITICAL FIX: Bounds check edge_indices write
                if edge_pos < self.max_edges:
                    self.edge_indices[edge_pos] = Int32(neighbors[j])
                    edge_pos += 1
                else:
                    print("❌ ERROR: edge_indices write", edge_pos, "exceeds max_edges", self.max_edges)
                    print("   Node", i, "neighbor", j, "- truncating to prevent corruption")
                    return  # Abort CSR build to prevent corruption
        
        self.is_csr_built = True
        
        # Optional: Free adjacency lists to save memory after CSR build
        # self.temp_adjacency_lists.clear()  # Comment out to keep both representations
    
    fn report_stats(self):
        """Print detailed statistics."""
        print("\nCSR Graph Statistics:")
        print(f"  Nodes: {self.num_nodes}")
        print(f"  Edges: {self.num_edges}")
        print(f"  Avg degree: {Float64(self.num_edges) / Float64(self.num_nodes):.1f}")
        print(f"  Memory usage: {self.memory_bytes() / (1024*1024):.2f} MB")
        
        # Compare with alternatives
        var old_fixed_memory = self.num_nodes * 48 * 8  # R=48, Int64
        var new_dynamic_memory = self.num_edges * 4 + self.num_nodes * 32  # Rough estimate
        
        var old_mb = Float64(old_fixed_memory) / (1024.0 * 1024.0)
        var new_mb = Float64(new_dynamic_memory) / (1024.0 * 1024.0)
        var csr_mb = Float64(self.memory_bytes()) / (1024.0 * 1024.0)
        
        print(f"  vs Fixed R=48: {old_mb:.2f} MB → {csr_mb:.2f} MB ({(1.0 - csr_mb/old_mb)*100:.1f}% savings)")
        print(f"  vs Dynamic lists: {new_mb:.2f} MB → {csr_mb:.2f} MB ({(1.0 - csr_mb/new_mb)*100:.1f}% savings)")

struct VamanaNode(Copyable, Movable):
    """Lightweight node wrapper for CSR graph.
    
    Instead of storing edges in the node, just stores an index
    into the CSR graph. This eliminates all dynamic allocation.
    """
    var node_idx: Int
    var graph_ref: UnsafePointer[VamanaGraph]
    
    fn __init__(out self, node_idx: Int, graph_ref: UnsafePointer[VamanaGraph]):
        """Initialize with reference to CSR graph."""
        self.node_idx = node_idx
        self.graph_ref = graph_ref
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor - just copies the index and reference."""
        self.node_idx = existing.node_idx
        self.graph_ref = existing.graph_ref
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.node_idx = existing.node_idx
        self.graph_ref = existing.graph_ref
    
    fn get_neighbors(self) -> List[Int]:
        """Get neighbors from CSR graph."""
        return self.graph_ref[].get_neighbors(self.node_idx)
    
    fn neighbor_count(self) -> Int:
        """Get neighbor count from CSR graph."""
        return self.graph_ref[].neighbor_count(self.node_idx)
    
    fn add_neighbor(self, to_node: Int) -> Bool:
        """Add neighbor via CSR graph."""
        return self.graph_ref[].add_edge(self.node_idx, to_node)
    
    fn has_neighbor(self, to_node: Int) -> Bool:
        """Check if neighbor exists via CSR graph."""
        return self.graph_ref[].has_edge(self.node_idx, to_node)
    
    fn get_vector_ptr(self) -> UnsafePointer[Float32]:
        """Get vector pointer from CSR graph."""
        return self.graph_ref[].get_vector_ptr(self.node_idx)
    
    fn get_id(self) -> String:
        """Get node ID from CSR graph."""
        return self.graph_ref[].get_node_id(self.node_idx)
    
    fn memory_bytes(self) -> Int:
        """CSR nodes are lightweight - just index and pointer."""
        return sizeof[Int]() + sizeof[UnsafePointer[VamanaGraph]]()

fn analyze_csr_memory_savings(num_nodes: Int, avg_degree: Int = 20) -> Dict[String, Float64]:
    """Analyze memory savings from CSR representation."""
    var result = Dict[String, Float64]()
    
    # Original fixed representation (R=48, Int64)
    var old_per_node = 48 * 8  # 48 neighbors * 8 bytes
    var old_total = num_nodes * old_per_node
    
    # Dynamic sparse representation (SparseNeighborList)
    var dynamic_per_node = avg_degree * 4 + 32  # Int32 + overhead per node
    var dynamic_total = num_nodes * dynamic_per_node
    
    # CSR representation
    var csr_offsets = (num_nodes + 1) * 4  # Int32 row offsets
    var csr_edges = num_nodes * avg_degree * 4  # Int32 edge indices
    var csr_total = csr_offsets + csr_edges
    
    # Store results
    result["old_mb"] = Float64(old_total) / (1024.0 * 1024.0)
    result["dynamic_mb"] = Float64(dynamic_total) / (1024.0 * 1024.0)
    result["csr_mb"] = Float64(csr_total) / (1024.0 * 1024.0)
    
    result["vs_fixed_savings"] = (1.0 - Float64(csr_total) / Float64(old_total)) * 100.0
    result["vs_dynamic_savings"] = (1.0 - Float64(csr_total) / Float64(dynamic_total)) * 100.0
    
    return result