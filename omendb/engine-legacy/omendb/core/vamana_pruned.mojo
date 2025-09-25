"""
Production-ready Vamana graph with proper edge pruning support.
Solves the 20K vector crash by implementing bounded degree maintenance.
"""

from collections import List, Dict, Optional
from memory import UnsafePointer, memcpy
from math import sqrt
from sys.intrinsics import sizeof
from python import Python

struct PrunedVamanaGraph(Copyable, Movable):
    """Vamana graph that properly maintains bounded degree through pruning.
    
    Key improvements:
    - Maintains adjacency lists for efficient edge modifications
    - Implements RobustPrune algorithm from DiskANN paper
    - Prevents unbounded edge growth that causes crashes
    - Scales to millions of vectors
    """
    
    # Vector storage (normalized for similarity search)
    var vectors: UnsafePointer[Float32]
    var quantized_vectors: UnsafePointer[UInt8]
    var quantization_scales: UnsafePointer[Float32]
    var quantization_offsets: UnsafePointer[Float32]
    var node_ids: List[String]
    
    # Graph structure - using adjacency lists for O(1) edge operations
    var adjacency_lists: List[List[Int]]  # Neighbors for each node
    var edge_distances: List[List[Float32]]  # Cached distances for pruning
    
    # CSR format for efficient search (built on demand)
    var csr_row_offsets: UnsafePointer[Int32]
    var csr_edge_indices: UnsafePointer[Int32]
    var is_csr_valid: Bool
    
    var dimension: Int
    var num_nodes: Int
    var capacity: Int
    var max_degree: Int  # R parameter - hard limit on edges per node
    var use_quantization: Bool
    var total_edges: Int
    
    fn __init__(out self, dimension: Int, expected_nodes: Int = 50, 
                max_degree: Int = 24, use_quantization: Bool = False):
        """Initialize with bounded degree support.
        
        Args:
            dimension: Vector dimension
            expected_nodes: Expected number of nodes
            max_degree: Maximum edges per node (24 is conservative for stability)
            use_quantization: If True, use quantized storage
        """
        self.dimension = dimension
        self.num_nodes = 0
        self.capacity = expected_nodes
        self.max_degree = max_degree
        self.use_quantization = use_quantization
        self.total_edges = 0
        self.is_csr_valid = False
        
        # Initialize adjacency lists
        self.adjacency_lists = List[List[Int]]()
        self.edge_distances = List[List[Float32]]()
        self.adjacency_lists.reserve(expected_nodes)
        self.edge_distances.reserve(expected_nodes)
        
        # Allocate vector storage
        if use_quantization:
            self.quantized_vectors = UnsafePointer[UInt8].alloc(expected_nodes * dimension)
            self.quantization_scales = UnsafePointer[Float32].alloc(expected_nodes)
            self.quantization_offsets = UnsafePointer[Float32].alloc(expected_nodes)
            self.vectors = UnsafePointer[Float32]()
        else:
            self.vectors = UnsafePointer[Float32].alloc(expected_nodes * dimension)
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        
        self.node_ids = List[String]()
        
        # CSR storage (allocated on first finalize)
        self.csr_row_offsets = UnsafePointer[Int32]()
        self.csr_edge_indices = UnsafePointer[Int32]()
    
    fn __del__(owned self):
        """Free allocated memory."""
        if self.vectors:
            self.vectors.free()
        if self.quantized_vectors:
            self.quantized_vectors.free()
        if self.quantization_scales:
            self.quantization_scales.free()
        if self.quantization_offsets:
            self.quantization_offsets.free()
        if self.csr_row_offsets:
            self.csr_row_offsets.free()
        if self.csr_edge_indices:
            self.csr_edge_indices.free()
    
    fn __copyinit__(out self, other: Self):
        """Deep copy constructor."""
        self.dimension = other.dimension
        self.num_nodes = other.num_nodes
        self.capacity = other.capacity
        self.max_degree = other.max_degree
        self.use_quantization = other.use_quantization
        self.total_edges = other.total_edges
        self.is_csr_valid = other.is_csr_valid
        
        # Deep copy adjacency lists
        self.adjacency_lists = List[List[Int]]()
        self.edge_distances = List[List[Float32]]()
        for i in range(len(other.adjacency_lists)):
            var neighbors = List[Int]()
            var distances = List[Float32]()
            for j in range(len(other.adjacency_lists[i])):
                neighbors.append(other.adjacency_lists[i][j])
                distances.append(other.edge_distances[i][j])
            self.adjacency_lists.append(neighbors^)
            self.edge_distances.append(distances^)
        
        # Copy vector storage
        if self.use_quantization:
            self.quantized_vectors = UnsafePointer[UInt8].alloc(self.capacity * self.dimension)
            self.quantization_scales = UnsafePointer[Float32].alloc(self.capacity)
            self.quantization_offsets = UnsafePointer[Float32].alloc(self.capacity)
            memcpy(self.quantized_vectors, other.quantized_vectors, 
                   self.num_nodes * self.dimension)
            memcpy(self.quantization_scales, other.quantization_scales, 
                   self.num_nodes * sizeof[Float32]())
            memcpy(self.quantization_offsets, other.quantization_offsets, 
                   self.num_nodes * sizeof[Float32]())
            self.vectors = UnsafePointer[Float32]()
        else:
            self.vectors = UnsafePointer[Float32].alloc(self.capacity * self.dimension)
            memcpy(self.vectors, other.vectors, 
                   self.num_nodes * self.dimension * sizeof[Float32]())
            self.quantized_vectors = UnsafePointer[UInt8]()
            self.quantization_scales = UnsafePointer[Float32]()
            self.quantization_offsets = UnsafePointer[Float32]()
        
        # Copy node IDs
        self.node_ids = List[String]()
        for i in range(len(other.node_ids)):
            self.node_ids.append(other.node_ids[i])
        
        # Copy CSR if valid
        if other.is_csr_valid and other.total_edges > 0:
            self.csr_row_offsets = UnsafePointer[Int32].alloc(self.num_nodes + 1)
            self.csr_edge_indices = UnsafePointer[Int32].alloc(other.total_edges)
            memcpy(self.csr_row_offsets, other.csr_row_offsets, 
                   (self.num_nodes + 1) * sizeof[Int32]())
            memcpy(self.csr_edge_indices, other.csr_edge_indices, 
                   other.total_edges * sizeof[Int32]())
        else:
            self.csr_row_offsets = UnsafePointer[Int32]()
            self.csr_edge_indices = UnsafePointer[Int32]()
    
    fn __moveinit__(out self, owned other: Self):
        """Move constructor."""
        self.dimension = other.dimension
        self.num_nodes = other.num_nodes
        self.capacity = other.capacity
        self.max_degree = other.max_degree
        self.use_quantization = other.use_quantization
        self.total_edges = other.total_edges
        self.is_csr_valid = other.is_csr_valid
        
        # Move ownership
        self.adjacency_lists = other.adjacency_lists^
        self.edge_distances = other.edge_distances^
        self.vectors = other.vectors
        self.quantized_vectors = other.quantized_vectors
        self.quantization_scales = other.quantization_scales
        self.quantization_offsets = other.quantization_offsets
        self.node_ids = other.node_ids^
        self.csr_row_offsets = other.csr_row_offsets
        self.csr_edge_indices = other.csr_edge_indices
        
        # Clear other
        other.vectors = UnsafePointer[Float32]()
        other.quantized_vectors = UnsafePointer[UInt8]()
        other.quantization_scales = UnsafePointer[Float32]()
        other.quantization_offsets = UnsafePointer[Float32]()
        other.csr_row_offsets = UnsafePointer[Int32]()
        other.csr_edge_indices = UnsafePointer[Int32]()
    
    fn add_node(mut self, id: String, vector: List[Float32]) -> Int:
        """Add a new node with its vector."""
        if self.num_nodes >= self.capacity:
            self._grow_capacity()
        
        # Store vector (normalized)
        var norm = Float32(0)
        for i in range(self.dimension):
            norm += vector[i] * vector[i]
        norm = sqrt(norm)
        
        if self.use_quantization:
            self._quantize_and_store(vector, self.num_nodes, norm)
        else:
            # Store normalized vector
            if norm > 0:
                for i in range(self.dimension):
                    self.vectors[self.num_nodes * self.dimension + i] = vector[i] / norm
            else:
                for i in range(self.dimension):
                    self.vectors[self.num_nodes * self.dimension + i] = 0
        
        # Add empty adjacency list for new node
        self.adjacency_lists.append(List[Int]())
        self.edge_distances.append(List[Float32]())
        self.node_ids.append(id)
        
        var idx = self.num_nodes
        self.num_nodes += 1
        self.is_csr_valid = False
        return idx
    
    fn add_node_from_ptr(mut self, id: String, vector_ptr: UnsafePointer[Float32], 
                         dimension: Int) -> Int:
        """Add node from pointer (for compatibility)."""
        var vector = List[Float32]()
        for i in range(dimension):
            vector.append(vector_ptr[i])
        return self.add_node(id, vector)
    
    fn add_edge(mut self, from_node: Int, to_node: Int) -> Bool:
        """Add edge with automatic pruning if degree exceeded."""
        if from_node >= self.num_nodes or to_node >= self.num_nodes:
            return False
        
        # Check if edge already exists
        var neighbors = self.adjacency_lists[from_node]
        for i in range(len(neighbors)):
            if neighbors[i] == to_node:
                return False  # Edge already exists
        
        # Calculate distance for pruning decisions
        var distance = self._compute_distance(from_node, to_node)
        
        # If under degree limit, just add
        if len(neighbors) < self.max_degree:
            self.adjacency_lists[from_node].append(to_node)
            self.edge_distances[from_node].append(distance)
            self.total_edges += 1
            self.is_csr_valid = False
            return True
        
        # Node is at capacity - need to prune
        # Add new candidate and run RobustPrune
        var all_candidates = List[Int]()
        var all_distances = List[Float32]()
        
        for i in range(len(neighbors)):
            all_candidates.append(neighbors[i])
            all_distances.append(self.edge_distances[from_node][i])
        all_candidates.append(to_node)
        all_distances.append(distance)
        
        # Apply RobustPrune to select best neighbors
        var pruned = self._robust_prune(from_node, all_candidates, 
                                        all_distances, self.max_degree)
        
        # Update adjacency list with pruned neighbors
        self.adjacency_lists[from_node].clear()
        self.edge_distances[from_node].clear()
        
        var edge_added = False
        for i in range(len(pruned)):
            var neighbor_idx = pruned[i]
            self.adjacency_lists[from_node].append(neighbor_idx)
            # Recalculate distance if needed
            if neighbor_idx == to_node:
                self.edge_distances[from_node].append(distance)
                edge_added = True
            else:
                # Find existing distance
                for j in range(len(all_candidates)):
                    if all_candidates[j] == neighbor_idx:
                        self.edge_distances[from_node].append(all_distances[j])
                        break
        
        self.is_csr_valid = False
        return edge_added
    
    fn _robust_prune(self, node_idx: Int, candidates: List[Int], 
                     distances: List[Float32], max_degree: Int) -> List[Int]:
        """RobustPrune algorithm for diverse neighbor selection.
        
        Selects neighbors that are both close and diverse to maintain
        graph connectivity and search quality.
        """
        if len(candidates) <= max_degree:
            return candidates
        
        var pruned = List[Int]()
        var used = List[Bool]()
        for _ in range(len(candidates)):
            used.append(False)
        
        # Sort candidates by distance (simple bubble sort for small lists)
        var sorted_indices = List[Int]()
        for i in range(len(candidates)):
            sorted_indices.append(i)
        
        for i in range(len(sorted_indices)):
            for j in range(i + 1, len(sorted_indices)):
                var idx_i = sorted_indices[i]
                var idx_j = sorted_indices[j]
                if distances[idx_j] < distances[idx_i]:
                    sorted_indices[i] = idx_j
                    sorted_indices[j] = idx_i
        
        # Greedily select diverse neighbors
        for i in range(len(sorted_indices)):
            if len(pruned) >= max_degree:
                break
            
            var idx = sorted_indices[i]
            if used[idx]:
                continue
            
            var candidate = candidates[idx]
            var candidate_dist = distances[idx]
            
            # Check diversity against already selected neighbors
            var is_diverse = True
            for j in range(len(pruned)):
                var pruned_neighbor = pruned[j]
                var dist_between = self._compute_distance(candidate, pruned_neighbor)
                
                # Reject if too close to an already selected neighbor
                # (closer than the candidate is to the node)
                if dist_between < candidate_dist * 1.2:  # Alpha = 1.2
                    is_diverse = False
                    break
            
            if is_diverse:
                pruned.append(candidate)
                used[idx] = True
        
        # If we don't have enough diverse neighbors, add closest remaining
        if len(pruned) < max_degree:
            for i in range(len(sorted_indices)):
                var idx = sorted_indices[i]
                if not used[idx] and len(pruned) < max_degree:
                    pruned.append(candidates[idx])
                    used[idx] = True
        
        return pruned
    
    fn neighbor_count(self, node_idx: Int) -> Int:
        """Get number of neighbors for a node."""
        if node_idx >= self.num_nodes:
            return 0
        return len(self.adjacency_lists[node_idx])
    
    fn get_neighbors(self, node_idx: Int) -> List[Int]:
        """Get neighbors of a node."""
        if node_idx >= self.num_nodes:
            return List[Int]()
        return self.adjacency_lists[node_idx]
    
    fn finalize(mut self):
        """Build CSR format for efficient search operations."""
        if self.is_csr_valid:
            return
        
        # Allocate CSR storage
        if self.csr_row_offsets:
            self.csr_row_offsets.free()
        if self.csr_edge_indices:
            self.csr_edge_indices.free()
        
        self.csr_row_offsets = UnsafePointer[Int32].alloc(self.num_nodes + 1)
        self.csr_edge_indices = UnsafePointer[Int32].alloc(self.total_edges)
        
        # Build CSR from adjacency lists
        var edge_offset = 0
        self.csr_row_offsets[0] = 0
        
        for i in range(self.num_nodes):
            var neighbors = self.adjacency_lists[i]
            for j in range(len(neighbors)):
                self.csr_edge_indices[edge_offset] = Int32(neighbors[j])
                edge_offset += 1
            self.csr_row_offsets[i + 1] = Int32(edge_offset)
        
        self.is_csr_valid = True
    
    fn get_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get vector pointer for a node."""
        if self.use_quantization:
            return UnsafePointer[Float32]()
        return self.vectors.offset(node_idx * self.dimension)
    
    fn get_original_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get original vector pointer (same as get_vector_ptr for compatibility)."""
        return self.get_vector_ptr(node_idx)
    
    fn get_quantized_vector_ptr(self, node_idx: Int) -> UnsafePointer[UInt8]:
        """Get quantized vector pointer."""
        if not self.use_quantization:
            return UnsafePointer[UInt8]()
        return self.quantized_vectors.offset(node_idx * self.dimension)
    
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
    
    fn get_node_id(self, node_idx: Int) -> String:
        """Get string ID for a node."""
        if node_idx >= len(self.node_ids):
            return ""
        return self.node_ids[node_idx]
    
    fn get_node_index(self, id: String) raises -> Optional[Int]:
        """Get node index for a given ID."""
        for i in range(len(self.node_ids)):
            if self.node_ids[i] == id:
                return Optional[Int](i)
        return Optional[Int]()
    
    fn _compute_distance(self, idx1: Int, idx2: Int) -> Float32:
        """Compute L2 distance between two nodes."""
        if idx1 >= self.num_nodes or idx2 >= self.num_nodes:
            return Float32(1e10)
        
        if self.use_quantization:
            return self._compute_distance_quantized(idx1, idx2)
        else:
            var ptr1 = self.vectors.offset(idx1 * self.dimension)
            var ptr2 = self.vectors.offset(idx2 * self.dimension)
            
            var sum = Float32(0)
            for i in range(self.dimension):
                var diff = ptr1[i] - ptr2[i]
                sum += diff * diff
            return sqrt(sum)
    
    fn _compute_distance_quantized(self, idx1: Int, idx2: Int) -> Float32:
        """Compute distance between quantized vectors."""
        var scale1 = self.quantization_scales[idx1]
        var offset1 = self.quantization_offsets[idx1]
        var scale2 = self.quantization_scales[idx2]
        var offset2 = self.quantization_offsets[idx2]
        
        var ptr1 = self.quantized_vectors.offset(idx1 * self.dimension)
        var ptr2 = self.quantized_vectors.offset(idx2 * self.dimension)
        
        var sum = Float32(0)
        for i in range(self.dimension):
            var val1 = Float32(ptr1[i]) * scale1 + offset1
            var val2 = Float32(ptr2[i]) * scale2 + offset2
            var diff = val1 - val2
            sum += diff * diff
        
        return sqrt(sum)
    
    fn _quantize_and_store(mut self, vector: List[Float32], idx: Int, norm: Float32):
        """Quantize and store a vector."""
        # Normalize first
        var normalized = List[Float32]()
        if norm > 0:
            for i in range(self.dimension):
                normalized.append(vector[i] / norm)
        else:
            for i in range(self.dimension):
                normalized.append(0)
        
        # Find min and max for quantization
        var min_val = normalized[0]
        var max_val = normalized[0]
        for i in range(1, self.dimension):
            if normalized[i] < min_val:
                min_val = normalized[i]
            if normalized[i] > max_val:
                max_val = normalized[i]
        
        # Calculate scale and offset
        var range = max_val - min_val
        var scale: Float32 = 1.0
        if range > 0:
            scale = range / 255.0
        var offset = min_val
        
        self.quantization_scales[idx] = scale
        self.quantization_offsets[idx] = offset
        
        # Quantize
        var i = 0
        while i < self.dimension:
            var normalized_val = (normalized[i] - offset) / scale
            var quantized_int = Int(normalized_val + 0.5)
            if quantized_int < 0:
                quantized_int = 0
            elif quantized_int > 255:
                quantized_int = 255
            self.quantized_vectors[idx * self.dimension + i] = UInt8(quantized_int)
            i += 1
    
    fn _grow_capacity(mut self):
        """Grow storage capacity when needed."""
        var new_capacity = max(self.capacity * 2, self.capacity + 100)
        
        if self.use_quantization:
            # Reallocate quantized storage
            var new_quantized = UnsafePointer[UInt8].alloc(new_capacity * self.dimension)
            var new_scales = UnsafePointer[Float32].alloc(new_capacity)
            var new_offsets = UnsafePointer[Float32].alloc(new_capacity)
            
            memcpy(new_quantized, self.quantized_vectors, 
                   self.num_nodes * self.dimension)
            memcpy(new_scales, self.quantization_scales, 
                   self.num_nodes * sizeof[Float32]())
            memcpy(new_offsets, self.quantization_offsets, 
                   self.num_nodes * sizeof[Float32]())
            
            self.quantized_vectors.free()
            self.quantization_scales.free()
            self.quantization_offsets.free()
            
            self.quantized_vectors = new_quantized
            self.quantization_scales = new_scales
            self.quantization_offsets = new_offsets
        else:
            # Reallocate vector storage
            var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
            memcpy(new_vectors, self.vectors, 
                   self.num_nodes * self.dimension * sizeof[Float32]())
            self.vectors.free()
            self.vectors = new_vectors
        
        self.capacity = new_capacity
    
    fn clear(mut self):
        """Clear all nodes and edges (for compatibility)."""
        self.adjacency_lists.clear()
        self.edge_distances.clear()
        self.node_ids.clear()
        self.num_nodes = 0
        self.total_edges = 0
        self.is_csr_valid = False
    
    fn memory_bytes(self) -> Int:
        """Calculate total memory usage."""
        var total = 0
        
        # Vector storage
        if self.use_quantization:
            total += self.capacity * self.dimension  # UInt8 per dimension
            total += self.capacity * 2 * sizeof[Float32]()  # scales and offsets
        else:
            total += self.capacity * self.dimension * sizeof[Float32]()
        
        # Graph structure
        total += self.total_edges * sizeof[Int32]() * 2  # edges and distances
        
        # CSR if built
        if self.is_csr_valid:
            total += (self.num_nodes + 1) * sizeof[Int32]()  # row offsets
            total += self.total_edges * sizeof[Int32]()  # edge indices
        
        # Node IDs (estimate)
        for i in range(len(self.node_ids)):
            total += len(self.node_ids[i])
        
        return total