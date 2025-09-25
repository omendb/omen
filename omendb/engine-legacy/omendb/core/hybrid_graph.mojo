"""
Hybrid graph structure for DiskANN that uses adjacency lists during construction
and converts to CSR format for efficient search operations.

This solves the O(E) insertion problem of pure CSR graphs.
"""

from collections import List, Dict, Optional
from memory import UnsafePointer, memcpy, memset_zero
from math import sqrt
from sys.intrinsics import sizeof
from python import Python

# Maximum edges per node (keep reasonable to avoid huge allocations)
alias MAX_EDGES_PER_NODE = 32

struct HybridGraph(Copyable, Movable):
    """Hybrid graph using adjacency lists for construction, CSR for search.
    
    During construction:
    - Uses List[List[Int]] for O(1) edge insertion
    - No edge shifting required
    - Can handle millions of nodes
    
    After finalization:
    - Converts to CSR format
    - Provides O(1) neighbor access
    - Memory efficient for search
    """
    
    # Vector storage (same as CSRGraph)
    var original_vectors: UnsafePointer[Float32]
    var vectors: UnsafePointer[Float32]
    var quantized_vectors: UnsafePointer[UInt8]
    var quantization_scales: UnsafePointer[Float32]
    var quantization_offsets: UnsafePointer[Float32]
    var node_ids: List[String]
    
    # Fixed edge storage to avoid Mojo's List[List[Int]] crash at 26K nodes
    var edge_storage: UnsafePointer[Int]     # Flat array indexed by node*MAX_EDGES+edge
    var edge_counts: UnsafePointer[Int]      # Number of edges per node
    var edge_capacity: Int                    # Total edge storage size
    var row_offsets: UnsafePointer[Int32]    # CSR format after finalization
    var edge_indices: UnsafePointer[Int32]   # CSR format after finalization
    
    var dimension: Int
    var num_nodes: Int
    var capacity: Int
    var num_edges: Int
    var use_quantization: Bool
    var is_finalized: Bool  # Track if converted to CSR
    
    fn __init__(out self, dimension: Int, expected_nodes: Int = 10000, use_quantization: Bool = False):
        """Initialize hybrid graph with expected capacity.
        
        Args:
            dimension: Vector dimension
            expected_nodes: Expected number of nodes (default 1M)
            use_quantization: Whether to use 8-bit quantization
        """
        self.dimension = dimension
        self.num_nodes = 0
        self.num_edges = 0
        self.use_quantization = use_quantization
        self.is_finalized = False
        
        # Pre-allocate conservatively to avoid huge upfront memory usage
        # Start with smaller allocation and grow as needed
        var initial_nodes = min(expected_nodes, 100000)  # Cap at 100K initially
        self.capacity = initial_nodes  # Set capacity to match allocation
        self.edge_capacity = initial_nodes * MAX_EDGES_PER_NODE
        self.edge_storage = UnsafePointer[Int].alloc(self.edge_capacity)
        self.edge_counts = UnsafePointer[Int].alloc(initial_nodes)
        
        # Initialize memory manually to avoid memset_zero issues
        for i in range(self.edge_capacity):
            self.edge_storage[i] = -1  # Use -1 to indicate empty slot
        for i in range(initial_nodes):
            self.edge_counts[i] = 0
        
        # CSR arrays start null, allocated on finalization
        self.row_offsets = UnsafePointer[Int32]()
        self.edge_indices = UnsafePointer[Int32]()
        
        # Vector storage - use initial_nodes to avoid huge allocations
        if not use_quantization:
            self.original_vectors = UnsafePointer[Float32].alloc(initial_nodes * dimension)
        else:
            self.original_vectors = UnsafePointer[Float32]()
        
        if use_quantization:
            self.quantized_vectors = UnsafePointer[UInt8].alloc(initial_nodes * dimension)
            self.quantization_scales = UnsafePointer[Float32].alloc(initial_nodes)
            self.quantization_offsets = UnsafePointer[Float32].alloc(initial_nodes)
            self.vectors = UnsafePointer[Float32]()
        else:
            self.vectors = UnsafePointer[Float32].alloc(initial_nodes * dimension)
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        
        self.node_ids = List[String]()
    
    fn __del__(owned self):
        """Free allocated memory."""
        if self.original_vectors:
            self.original_vectors.free()
        if self.vectors:
            self.vectors.free()
        if self.quantized_vectors:
            self.quantized_vectors.free()
        if self.quantization_scales:
            self.quantization_scales.free()
        if self.quantization_offsets:
            self.quantization_offsets.free()
        if self.edge_storage:
            self.edge_storage.free()
        if self.edge_counts:
            self.edge_counts.free()
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
        self.use_quantization = existing.use_quantization
        self.is_finalized = existing.is_finalized
        
        # Copy edge storage
        self.edge_capacity = existing.edge_capacity
        self.edge_storage = UnsafePointer[Int].alloc(self.edge_capacity)
        self.edge_counts = UnsafePointer[Int].alloc(existing.capacity)
        for i in range(self.edge_capacity):
            self.edge_storage[i] = existing.edge_storage[i]
        for i in range(existing.capacity):
            self.edge_counts[i] = existing.edge_counts[i]
        
        # Copy node IDs
        self.node_ids = List[String]()
        for i in range(len(existing.node_ids)):
            self.node_ids.append(existing.node_ids[i])
        
        # Allocate and copy vectors
        if existing.use_quantization:
            self.quantized_vectors = UnsafePointer[UInt8].alloc(existing.capacity * existing.dimension)
            for i in range(existing.num_nodes * existing.dimension):
                self.quantized_vectors[i] = existing.quantized_vectors[i]
            self.quantization_scales = UnsafePointer[Float32].alloc(existing.capacity)
            for i in range(existing.num_nodes):
                self.quantization_scales[i] = existing.quantization_scales[i]
            self.quantization_offsets = UnsafePointer[Float32].alloc(existing.capacity)
            for i in range(existing.num_nodes):
                self.quantization_offsets[i] = existing.quantization_offsets[i]
            self.vectors = UnsafePointer[Float32]()
            self.original_vectors = UnsafePointer[Float32]()
        else:
            if existing.original_vectors:
                self.original_vectors = UnsafePointer[Float32].alloc(existing.capacity * existing.dimension)
                for i in range(existing.num_nodes * existing.dimension):
                    self.original_vectors[i] = existing.original_vectors[i]
            else:
                self.original_vectors = UnsafePointer[Float32]()
            
            if existing.vectors:
                self.vectors = UnsafePointer[Float32].alloc(existing.capacity * existing.dimension)
                for i in range(existing.num_nodes * existing.dimension):
                    self.vectors[i] = existing.vectors[i]
            else:
                self.vectors = UnsafePointer[Float32]()
            
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        
        # Copy CSR arrays if finalized
        if existing.is_finalized and existing.row_offsets:
            self.row_offsets = UnsafePointer[Int32].alloc(existing.num_nodes + 1)
            for i in range(existing.num_nodes + 1):
                self.row_offsets[i] = existing.row_offsets[i]
            self.edge_indices = UnsafePointer[Int32].alloc(existing.num_edges)
            for i in range(existing.num_edges):
                self.edge_indices[i] = existing.edge_indices[i]
        else:
            self.row_offsets = UnsafePointer[Int32]()
            self.edge_indices = UnsafePointer[Int32]()
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.dimension = existing.dimension
        self.num_nodes = existing.num_nodes
        self.capacity = existing.capacity
        self.num_edges = existing.num_edges
        self.use_quantization = existing.use_quantization
        self.is_finalized = existing.is_finalized
        
        self.edge_capacity = existing.edge_capacity
        self.edge_storage = existing.edge_storage
        self.edge_counts = existing.edge_counts
        existing.edge_storage = UnsafePointer[Int]()
        existing.edge_counts = UnsafePointer[Int]()
        self.node_ids = existing.node_ids^
        
        self.original_vectors = existing.original_vectors
        self.vectors = existing.vectors
        self.quantized_vectors = existing.quantized_vectors
        self.quantization_scales = existing.quantization_scales
        self.quantization_offsets = existing.quantization_offsets
        self.row_offsets = existing.row_offsets
        self.edge_indices = existing.edge_indices
        
        # Clear existing pointers
        existing.original_vectors = UnsafePointer[Float32]()
        existing.vectors = UnsafePointer[Float32]()
        existing.quantized_vectors = UnsafePointer[UInt8]()
        existing.quantization_scales = UnsafePointer[Float32]()
        existing.quantization_offsets = UnsafePointer[Float32]()
        existing.row_offsets = UnsafePointer[Int32]()
        existing.edge_indices = UnsafePointer[Int32]()
    
    fn add_node(mut self, id: String, vector: List[Float32]) -> Int:
        """Add a new node with O(1) complexity.
        
        Uses adjacency list for fast insertion during construction.
        """
        var node_idx = self.num_nodes
        
        # Check capacity for vectors
        if node_idx >= self.capacity:
            self._reallocate_vectors(max(self.capacity * 2, node_idx + 1000))
        
        # Store ID
        self.node_ids.append(id)
        
        # Initialize edge count for this node (no List allocation!)
        self.edge_counts[node_idx] = 0
        
        # Store vectors (same as CSRGraph)
        var start_idx = node_idx * self.dimension
        
        # Calculate normalization
        var norm_sq: Float32 = 0
        for i in range(self.dimension):
            norm_sq += vector[i] * vector[i]
        var norm = sqrt(norm_sq + Float32(1e-12))
        var inv_norm = 1.0 / norm
        
        # Store based on quantization mode
        if self.use_quantization:
            # Find min/max for quantization
            var min_val: Float32 = 1e10
            var max_val: Float32 = -1e10
            for i in range(self.dimension):
                if vector[i] < min_val:
                    min_val = vector[i]
                if vector[i] > max_val:
                    max_val = vector[i]
            
            # Calculate scale and offset
            var scale = (max_val - min_val) / 255.0
            if scale < 1e-12:
                scale = 1.0
            var offset = min_val
            
            self.quantization_scales[node_idx] = scale
            self.quantization_offsets[node_idx] = offset
            
            # Quantize and store
            for i in range(self.dimension):
                var quantized_val = UInt8((vector[i] - offset) / scale + 0.5)
                self.quantized_vectors[start_idx + i] = quantized_val
        else:
            # Store both original and normalized
            for i in range(self.dimension):
                self.original_vectors[start_idx + i] = vector[i]
                self.vectors[start_idx + i] = vector[i] * inv_norm
        
        self.num_nodes += 1
        return node_idx
    
    fn add_edge(mut self, from_node: Int, to_node: Int) -> Bool:
        """Add an edge with O(1) complexity during construction.
        
        This is the key improvement over CSR - no edge shifting required!
        """
        if self.is_finalized:
            # Can't modify after finalization
            return False
        
        if from_node >= self.num_nodes or to_node >= self.num_nodes:
            return False
        
        # Check edge count
        var edge_count = self.edge_counts[from_node]
        if edge_count >= MAX_EDGES_PER_NODE:
            return False  # Node at max capacity
        
        # Check if edge already exists
        var base_idx = from_node * MAX_EDGES_PER_NODE
        for i in range(edge_count):
            if self.edge_storage[base_idx + i] == to_node:
                return False  # Edge already exists
        
        # Add edge - O(1) with no List allocation!
        self.edge_storage[base_idx + edge_count] = to_node
        self.edge_counts[from_node] = edge_count + 1
        self.num_edges += 1
        
        return True
    
    fn finalize(mut self):
        """Convert adjacency lists to CSR format for efficient search.
        
        This is called once after all nodes and edges are added.
        O(E) complexity but only done once, not per insertion.
        """
        if self.is_finalized:
            return
        
        # Allocate CSR arrays
        self.row_offsets = UnsafePointer[Int32].alloc(self.num_nodes + 1)
        self.edge_indices = UnsafePointer[Int32].alloc(self.num_edges)
        
        # Build row offsets
        self.row_offsets[0] = 0
        var edge_idx = 0
        
        for node_idx in range(self.num_nodes):
            var edge_count = self.edge_counts[node_idx]
            var base_idx = node_idx * MAX_EDGES_PER_NODE
            
            if edge_count > 0:
                # Copy edges to temporary list for sorting
                var neighbors = List[Int]()
                for i in range(edge_count):
                    neighbors.append(self.edge_storage[base_idx + i])
                
                # Sort neighbors (insertion sort for small lists)
                for i in range(1, edge_count):
                    var key = neighbors[i]
                    var j = i - 1
                    while j >= 0 and neighbors[j] > key:
                        neighbors[j + 1] = neighbors[j]
                        j -= 1
                    neighbors[j + 1] = key
                
                # Copy sorted neighbors to CSR edge array
                for i in range(edge_count):
                    self.edge_indices[edge_idx] = Int32(neighbors[i])
                    edge_idx += 1
            
            # Update offset for next node
            self.row_offsets[node_idx + 1] = Int32(edge_idx)
        
        self.is_finalized = True
        
        # Edge storage still needed for potential future operations
        # Keep it allocated but mark as finalized
    
    fn get_neighbors(self, node_idx: Int) -> List[Int]:
        """Get neighbors of a node.
        
        Works with both adjacency list (construction) and CSR (search) modes.
        """
        if node_idx >= self.num_nodes:
            return List[Int]()
        
        if not self.is_finalized:
            # Use edge storage during construction
            var result = List[Int]()
            var edge_count = self.edge_counts[node_idx]
            var base_idx = node_idx * MAX_EDGES_PER_NODE
            
            for i in range(edge_count):
                result.append(self.edge_storage[base_idx + i])
            return result
        else:
            # Use CSR after finalization
            var result = List[Int]()
            var start = Int(self.row_offsets[node_idx])
            var end = Int(self.row_offsets[node_idx + 1])
            
            for i in range(start, end):
                result.append(Int(self.edge_indices[i]))
            
            return result
    
    fn neighbor_count(self, node_idx: Int) -> Int:
        """Get number of neighbors for a node."""
        if node_idx >= self.num_nodes:
            return 0
        
        if not self.is_finalized:
            return self.edge_counts[node_idx]
        else:
            return Int(self.row_offsets[node_idx + 1] - self.row_offsets[node_idx])
    
    fn has_edge(self, from_node: Int, to_node: Int) -> Bool:
        """Check if an edge exists."""
        if from_node >= self.num_nodes:
            return False
        
        if not self.is_finalized:
            # Linear search in edge storage
            var edge_count = self.edge_counts[from_node]
            var base_idx = from_node * MAX_EDGES_PER_NODE
            
            for i in range(edge_count):
                if self.edge_storage[base_idx + i] == to_node:
                    return True
            return False
        else:
            # Binary search in sorted CSR edges
            var start = Int(self.row_offsets[from_node])
            var end = Int(self.row_offsets[from_node + 1])
            
            var left = start
            var right = end - 1
            
            while left <= right:
                var mid = (left + right) // 2
                var mid_val = Int(self.edge_indices[mid])
                
                if mid_val == to_node:
                    return True
                elif mid_val < to_node:
                    left = mid + 1
                else:
                    right = mid - 1
            
            return False
    
    fn _reallocate_vectors(mut self, new_capacity: Int):
        """Reallocate vector storage when capacity exceeded."""
        if new_capacity <= self.capacity:
            return
        
        var old_capacity = self.capacity
        self.capacity = new_capacity
        
        # Reallocate edge storage
        var new_edge_capacity = new_capacity * MAX_EDGES_PER_NODE
        var new_edge_storage = UnsafePointer[Int].alloc(new_edge_capacity)
        var new_edge_counts = UnsafePointer[Int].alloc(new_capacity)
        
        # Copy existing edge data
        for i in range(self.edge_capacity):
            new_edge_storage[i] = self.edge_storage[i]
        for i in range(old_capacity):
            new_edge_counts[i] = self.edge_counts[i]
        # Initialize new portion
        for i in range(old_capacity, new_capacity):
            new_edge_counts[i] = 0
        
        # Free old storage
        self.edge_storage.free()
        self.edge_counts.free()
        self.edge_storage = new_edge_storage
        self.edge_counts = new_edge_counts
        self.edge_capacity = new_edge_capacity
        
        # Reallocate based on quantization mode
        if self.use_quantization:
            var new_quantized = UnsafePointer[UInt8].alloc(new_capacity * self.dimension)
            for i in range(self.num_nodes * self.dimension):
                new_quantized[i] = self.quantized_vectors[i]
            self.quantized_vectors.free()
            self.quantized_vectors = new_quantized
            
            var new_scales = UnsafePointer[Float32].alloc(new_capacity)
            for i in range(self.num_nodes):
                new_scales[i] = self.quantization_scales[i]
            self.quantization_scales.free()
            self.quantization_scales = new_scales
            
            var new_offsets = UnsafePointer[Float32].alloc(new_capacity)
            for i in range(self.num_nodes):
                new_offsets[i] = self.quantization_offsets[i]
            self.quantization_offsets.free()
            self.quantization_offsets = new_offsets
        else:
            if self.original_vectors:
                var new_original = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
                for i in range(self.num_nodes * self.dimension):
                    new_original[i] = self.original_vectors[i]
                self.original_vectors.free()
                self.original_vectors = new_original
            
            if self.vectors:
                var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
                for i in range(self.num_nodes * self.dimension):
                    new_vectors[i] = self.vectors[i]
                self.vectors.free()
                self.vectors = new_vectors
    
    fn get_node_index(self, id: String) -> Optional[Int]:
        """Get node index by ID."""
        for i in range(len(self.node_ids)):
            if self.node_ids[i] == id:
                return Optional[Int](i)
        return Optional[Int]()
    
    fn get_vector(self, node_idx: Int) -> List[Float32]:
        """Get original vector for a node."""
        var result = List[Float32]()
        if node_idx >= self.num_nodes:
            return result
        
        var start_idx = node_idx * self.dimension
        
        if self.use_quantization:
            # Dequantize on the fly
            var scale = self.quantization_scales[node_idx]
            var offset = self.quantization_offsets[node_idx]
            
            for i in range(self.dimension):
                var quantized = Float32(self.quantized_vectors[start_idx + i])
                result.append(quantized * scale + offset)
        else:
            # Return original vector
            for i in range(self.dimension):
                result.append(self.original_vectors[start_idx + i])
        
        return result
    
    fn get_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get pointer to normalized vector for distance computation."""
        if node_idx >= self.num_nodes:
            return UnsafePointer[Float32]()
        
        if self.use_quantization:
            # Would need temporary buffer for dequantized vector
            # For now, return null - caller should use quantized distance
            return UnsafePointer[Float32]()
        else:
            var start_idx = node_idx * self.dimension
            return self.vectors.offset(start_idx)
    
    fn get_original_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get pointer to original (non-normalized) vector."""
        if node_idx >= self.num_nodes or self.use_quantization:
            return UnsafePointer[Float32]()
        
        var start_idx = node_idx * self.dimension
        return self.original_vectors.offset(start_idx)
    
    fn get_node_id(self, node_idx: Int) -> String:
        """Get node ID by index."""
        if node_idx < len(self.node_ids):
            return self.node_ids[node_idx]
        return String("")
    
    fn get_quantized_vector_ptr(self, node_idx: Int) -> UnsafePointer[UInt8]:
        """Get pointer to quantized vector."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return UnsafePointer[UInt8]()
        var start_idx = node_idx * self.dimension
        return self.quantized_vectors.offset(start_idx)
    
    fn get_quantization_scale(self, node_idx: Int) -> Float32:
        """Get quantization scale for a node."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return 1.0
        return self.quantization_scales[node_idx]
    
    fn get_quantization_offset(self, node_idx: Int) -> Float32:
        """Get quantization offset for a node."""
        if not self.use_quantization or node_idx >= self.num_nodes:
            return 0.0
        return self.quantization_offsets[node_idx]
    
    fn memory_bytes(self) -> Int:
        """Calculate total memory usage in bytes."""
        var total_bytes = 0
        
        # Vector storage
        if self.use_quantization:
            total_bytes += self.capacity * self.dimension  # UInt8 vectors
            total_bytes += self.capacity * 8  # Scales and offsets
        else:
            total_bytes += self.capacity * self.dimension * 8  # Both original and normalized
        
        # Graph structure
        if self.is_finalized:
            total_bytes += (self.num_nodes + 1) * 4  # Row offsets
            total_bytes += self.num_edges * 4  # Edge indices
        else:
            # Estimate adjacency list overhead
            total_bytes += self.num_nodes * 32  # List overhead
            total_bytes += self.num_edges * 4  # Edge data
        
        return total_bytes
    
    fn memory_usage_mb(self) -> Float32:
        """Calculate total memory usage in MB."""
        var total_bytes = 0
        
        # Vector storage
        if self.use_quantization:
            total_bytes += self.capacity * self.dimension  # UInt8 vectors
            total_bytes += self.capacity * 8  # Scales and offsets
        else:
            total_bytes += self.capacity * self.dimension * 8  # Both original and normalized
        
        # Graph structure
        if self.is_finalized:
            total_bytes += (self.num_nodes + 1) * 4  # Row offsets
            total_bytes += self.num_edges * 4  # Edge indices
        else:
            # Estimate adjacency list overhead
            total_bytes += self.num_nodes * 32  # List overhead
            total_bytes += self.num_edges * 4  # Edge data
        
        return Float32(total_bytes) / (1024.0 * 1024.0)