"""
Adjacency List Graph for DiskANN - supports edge removal for proper pruning.

This replaces the broken CSR graph that can't remove edges.
Key features:
- O(1) edge addition
- O(degree) edge removal  
- Bounded degree enforcement
- Memory efficient
"""

from collections import List, Dict, Optional
from memory import UnsafePointer
from utils import StringRef
from math import sqrt
from random import random_si64

@value
struct AdjacencyNode:
    """Node in adjacency list graph with mutable neighbors."""
    var id: String
    var vector: UnsafePointer[Float32]  # Points to vector data
    var neighbors: List[Int]            # List of neighbor indices
    var reverse_neighbors: List[Int]    # Nodes that point to this one
    
    fn __init__(out self, id: String, vector: UnsafePointer[Float32]):
        self.id = id
        self.vector = vector
        self.neighbors = List[Int]()
        self.reverse_neighbors = List[Int]()
    
    fn add_neighbor(mut self, neighbor_idx: Int) -> Bool:
        """Add a neighbor if not already present."""
        # Check if already exists
        for i in range(len(self.neighbors)):
            if self.neighbors[i] == neighbor_idx:
                return False
        self.neighbors.append(neighbor_idx)
        return True
    
    fn remove_neighbor(mut self, neighbor_idx: Int) -> Bool:
        """Remove a neighbor - CRITICAL for pruning."""
        for i in range(len(self.neighbors)):
            if self.neighbors[i] == neighbor_idx:
                # Swap with last and pop
                var last_idx = len(self.neighbors) - 1
                if i != last_idx:
                    self.neighbors[i] = self.neighbors[last_idx]
                _ = self.neighbors.pop()
                return True
        return False
    
    fn degree(self) -> Int:
        """Get current degree (number of outgoing edges)."""
        return len(self.neighbors)
    
    fn has_neighbor(self, neighbor_idx: Int) -> Bool:
        """Check if has specific neighbor."""
        for i in range(len(self.neighbors)):
            if self.neighbors[i] == neighbor_idx:
                return True
        return False

struct AdjacencyGraph(Copyable, Movable):
    """Adjacency list graph that supports proper DiskANN operations."""
    var nodes: List[AdjacencyNode]
    var id_to_idx: Dict[String, Int]
    var dimension: Int
    var max_degree: Int  # R parameter - hard limit on edges per node
    var num_edges: Int
    var use_quantization: Bool  # For compatibility (always False for now)
    
    # Vector storage (owned by graph)
    var vectors: UnsafePointer[Float32]
    var vector_capacity: Int
    
    fn __init__(out self, dimension: Int, expected_nodes: Int = 1000, max_degree: Int = 32):
        """Initialize graph with proper degree bounds."""
        self.nodes = List[AdjacencyNode]()
        self.id_to_idx = Dict[String, Int]()
        self.dimension = dimension
        self.max_degree = max_degree
        self.num_edges = 0
        self.use_quantization = False  # Always False for now
        
        # Pre-allocate vector storage
        self.vector_capacity = expected_nodes
        self.vectors = UnsafePointer[Float32].alloc(expected_nodes * dimension)
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.nodes = existing.nodes
        self.id_to_idx = existing.id_to_idx
        self.dimension = existing.dimension
        self.max_degree = existing.max_degree
        self.num_edges = existing.num_edges
        self.use_quantization = existing.use_quantization
        self.vector_capacity = existing.vector_capacity
        
        # Deep copy vector storage
        self.vectors = UnsafePointer[Float32].alloc(self.vector_capacity * self.dimension)
        for i in range(len(self.nodes) * self.dimension):
            self.vectors[i] = existing.vectors[i]
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.nodes = existing.nodes^
        self.id_to_idx = existing.id_to_idx^
        self.dimension = existing.dimension
        self.max_degree = existing.max_degree
        self.num_edges = existing.num_edges
        self.use_quantization = existing.use_quantization
        self.vector_capacity = existing.vector_capacity
        self.vectors = existing.vectors
        existing.vectors = UnsafePointer[Float32]()  # Null out moved pointer
    
    fn __del__(owned self):
        """Clean up allocated memory."""
        if self.vectors:
            self.vectors.free()
    
    fn add_node(mut self, id: String, vector: List[Float32]) -> Int:
        """Add a new node with vector data."""
        var node_idx = len(self.nodes)
        
        # Check if ID already exists
        try:
            if id in self.id_to_idx:
                return self.id_to_idx[id]
        except:
            pass  # ID not found, continue
        
        # Grow vector storage if needed
        if node_idx >= self.vector_capacity:
            self._grow_vector_storage()
        
        # Copy vector data
        var vector_offset = node_idx * self.dimension
        for i in range(self.dimension):
            self.vectors[vector_offset + i] = vector[i]
        
        # Create node pointing to vector data
        var vector_ptr = self.vectors.offset(vector_offset)
        var node = AdjacencyNode(id, vector_ptr)
        self.nodes.append(node)
        self.id_to_idx[id] = node_idx
        
        return node_idx
    
    fn add_node_from_ptr(mut self, id: String, vector_ptr: UnsafePointer[Float32], dimension: Int) -> Int:
        """Add node directly from vector pointer (zero-copy for compatibility)."""
        var node_idx = len(self.nodes)
        
        # Check if ID already exists
        try:
            if id in self.id_to_idx:
                return self.id_to_idx[id]
        except:
            pass  # ID not found, continue
        
        # Grow vector storage if needed
        if node_idx >= self.vector_capacity:
            self._grow_vector_storage()
        
        # Copy vector data from pointer
        var vector_offset = node_idx * self.dimension
        for i in range(dimension):
            self.vectors[vector_offset + i] = vector_ptr[i]
        
        # Create node pointing to vector data
        var own_vector_ptr = self.vectors.offset(vector_offset)
        var node = AdjacencyNode(id, own_vector_ptr)
        self.nodes.append(node)
        self.id_to_idx[id] = node_idx
        
        return node_idx
    
    fn add_edge(mut self, from_idx: Int, to_idx: Int) -> Bool:
        """Add edge with degree bound enforcement."""
        if from_idx >= len(self.nodes) or to_idx >= len(self.nodes):
            return False
        
        # Check degree bound
        if self.nodes[from_idx].degree() >= self.max_degree:
            return False  # Caller must prune first
        
        # Add forward edge
        if self.nodes[from_idx].add_neighbor(to_idx):
            # Track reverse edge for bidirectional operations
            self.nodes[to_idx].reverse_neighbors.append(from_idx)
            self.num_edges += 1
            return True
        
        return False
    
    fn remove_edge(mut self, from_idx: Int, to_idx: Int) -> Bool:
        """Remove edge - ESSENTIAL for RobustPrune."""
        if from_idx >= len(self.nodes) or to_idx >= len(self.nodes):
            return False
        
        # Remove forward edge
        if self.nodes[from_idx].remove_neighbor(to_idx):
            # Remove from reverse neighbors
            var reverse_neighbors = self.nodes[to_idx].reverse_neighbors
            for i in range(len(reverse_neighbors)):
                if reverse_neighbors[i] == from_idx:
                    var last_idx = len(reverse_neighbors) - 1
                    if i != last_idx:
                        self.nodes[to_idx].reverse_neighbors[i] = reverse_neighbors[last_idx]
                    _ = self.nodes[to_idx].reverse_neighbors.pop()
                    break
            
            self.num_edges -= 1
            return True
        
        return False
    
    fn get_neighbors(self, node_idx: Int) -> List[Int]:
        """Get neighbors of a node."""
        if node_idx >= len(self.nodes):
            return List[Int]()
        return self.nodes[node_idx].neighbors
    
    fn prune_edges(mut self, node_idx: Int, keep_neighbors: List[Int]):
        """Prune edges to only keep specified neighbors - CORE OPERATION."""
        if node_idx >= len(self.nodes):
            return
        
        # Get current neighbors
        var current = self.nodes[node_idx].neighbors
        
        # Remove edges not in keep list
        var i = 0
        while i < len(current):
            var neighbor = current[i]
            var keep = False
            
            # Check if in keep list
            for j in range(len(keep_neighbors)):
                if keep_neighbors[j] == neighbor:
                    keep = True
                    break
            
            if not keep:
                # Remove this edge
                _ = self.remove_edge(node_idx, neighbor)
                # Don't increment i since we removed an element
            else:
                i += 1
    
    fn degree(self, node_idx: Int) -> Int:
        """Get degree of a node."""
        if node_idx >= len(self.nodes):
            return 0
        return self.nodes[node_idx].degree()
    
    fn neighbor_count(self, node_idx: Int) -> Int:
        """Get neighbor count (alias for degree, for compatibility)."""
        return self.degree(node_idx)
    
    fn distance(self, idx1: Int, idx2: Int) -> Float32:
        """Compute L2 distance between two nodes."""
        if idx1 >= len(self.nodes) or idx2 >= len(self.nodes):
            return Float32.MAX
        
        var v1 = self.nodes[idx1].vector
        var v2 = self.nodes[idx2].vector
        
        var sum: Float32 = 0
        for i in range(self.dimension):
            var diff = v1[i] - v2[i]
            sum += diff * diff
        
        return sqrt(sum)
    
    fn _grow_vector_storage(mut self):
        """Grow vector storage when capacity exceeded."""
        var new_capacity = self.vector_capacity * 2
        var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
        
        # Copy existing vectors
        var copy_size = len(self.nodes) * self.dimension
        for i in range(copy_size):
            new_vectors[i] = self.vectors[i]
        
        # Update all node pointers
        for i in range(len(self.nodes)):
            var offset = i * self.dimension
            self.nodes[i].vector = new_vectors.offset(offset)
        
        # Free old storage and update
        self.vectors.free()
        self.vectors = new_vectors
        self.vector_capacity = new_capacity
    
    fn size(self) -> Int:
        """Get number of nodes."""
        return len(self.nodes)
    
    fn num_nodes(self) -> Int:
        """Get number of nodes (for compatibility with VamanaGraph)."""
        return len(self.nodes)
    
    fn get_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get vector pointer for a node (for compatibility with VamanaGraph)."""
        if node_idx >= len(self.nodes):
            return UnsafePointer[Float32]()
        return self.nodes[node_idx].vector
    
    fn get_node_index(self, id: String) raises -> Optional[Int]:
        """Get the node index for a given ID."""
        if id in self.id_to_idx:
            return Optional[Int](self.id_to_idx[id])
        return Optional[Int]()
    
    fn memory_bytes(self) -> Int:
        """Estimate memory usage."""
        var node_memory = len(self.nodes) * 64  # Rough estimate per node
        var edge_memory = self.num_edges * 8     # Two ints per edge
        var vector_memory = self.vector_capacity * self.dimension * 4
        return node_memory + edge_memory + vector_memory
    
    fn get_node_id(self, node_idx: Int) -> String:
        """Get string ID for a node."""
        if node_idx >= len(self.nodes):
            return ""
        return self.nodes[node_idx].id
    
    fn finalize(mut self):
        """No-op for compatibility - adjacency list doesn't need finalization."""
        # Unlike CSR, adjacency list is always ready for use
        pass
    
    fn get_quantized_vector_ptr(self, node_idx: Int) -> UnsafePointer[UInt8]:
        """Stub for quantization compatibility."""
        return UnsafePointer[UInt8]()
    
    fn get_quantization_scale(self, node_idx: Int) -> Float32:
        """Stub for quantization compatibility."""
        return 1.0
    
    fn get_quantization_offset(self, node_idx: Int) -> Float32:
        """Stub for quantization compatibility."""
        return 0.0
    
    fn get_original_vector_ptr(self, node_idx: Int) -> UnsafePointer[Float32]:
        """Get original vector pointer (alias for get_vector_ptr)."""
        return self.get_vector_ptr(node_idx)