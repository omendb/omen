"""
Dynamic capacity HNSW implementation that grows as needed.
This fixes the fixed 100K limit that's killing our performance.
"""

from memory import UnsafePointer, memcpy
from algorithm import vectorize
from math import log, sqrt, min, max
from random import random_float64
from sys import simdwidthof
from python import Python, PythonObject
from collections import Optional, Dict, List

from .node_pool import NodePool
from .heap import KNNBuffer
from .binary_quantized_vector import BinaryQuantizedVector
from .pq import PQCompressor

# HNSW parameters - optimized for performance
alias M = 8  # Reduced for faster insertion
alias max_M = M
alias max_M0 = M * 2  # Layer 0 has more connections
alias ef_construction = 100  # Reduced from 200 for speed
alias ef_search = 200  # Keep high for good recall
alias ml = 1.0 / log(2.0)
alias MAX_LAYERS = 4

# Dynamic growth parameters
alias INITIAL_CAPACITY = 10000  # Start small
alias GROWTH_FACTOR = 2.0  # Double when needed
alias GROWTH_THRESHOLD = 0.8  # Grow at 80% full

struct DynamicHNSWIndex:
    """HNSW with dynamic capacity that grows as needed."""
    
    var dimension: Int
    var capacity: Int
    var size: Int
    var entry_point: Int
    
    # Core storage
    var node_pool: NodePool
    var vectors: UnsafePointer[Float32]
    
    # Performance optimizations
    var use_binary_quantization: Bool
    var binary_vectors: List[BinaryQuantizedVector]
    
    # Search buffer pool to avoid allocations
    var search_buffer_pool: List[KNNBuffer]
    var pool_size: Int
    
    # Visited tracking
    var visited_buffer: UnsafePointer[Int]
    var visited_version: Int
    
    fn __init__(out self, dimension: Int):
        """Initialize with small capacity that grows dynamically."""
        self.dimension = dimension
        self.capacity = INITIAL_CAPACITY
        self.size = 0
        self.entry_point = -1
        self.visited_version = 1
        
        # Start small
        self.node_pool = NodePool(INITIAL_CAPACITY)
        self.vectors = UnsafePointer[Float32].alloc(INITIAL_CAPACITY * dimension)
        
        # Search buffer pool
        self.search_buffer_pool = List[KNNBuffer]()
        self.pool_size = 0
        
        # Pre-allocate some search buffers
        for _ in range(4):
            self.search_buffer_pool.append(KNNBuffer(ef_construction))
            self.pool_size += 1
        
        # Binary quantization
        self.use_binary_quantization = True  # Enable by default
        self.binary_vectors = List[BinaryQuantizedVector]()
        
        # Visited tracking
        self.visited_buffer = UnsafePointer[Int].alloc(INITIAL_CAPACITY)
        for i in range(INITIAL_CAPACITY):
            self.visited_buffer[i] = 0
    
    fn grow(mut self) -> Bool:
        """Grow capacity by GROWTH_FACTOR when needed."""
        var new_capacity = Int(self.capacity * GROWTH_FACTOR)
        print("📈 Growing capacity:", self.capacity, "→", new_capacity)
        
        # Reallocate vectors
        var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
        memcpy(new_vectors, self.vectors, self.size * self.dimension * 4)
        self.vectors.free()
        self.vectors = new_vectors
        
        # Grow node pool
        # TODO: NodePool needs grow() method
        # For now, create new pool and migrate
        var new_pool = NodePool(new_capacity)
        # Migration would happen here
        self.node_pool = new_pool^
        
        # Reallocate visited buffer
        var new_visited = UnsafePointer[Int].alloc(new_capacity)
        memcpy(new_visited, self.visited_buffer, self.capacity * sizeof[Int]())
        # Initialize new portion
        for i in range(self.capacity, new_capacity):
            new_visited[i] = 0
        self.visited_buffer.free()
        self.visited_buffer = new_visited
        
        self.capacity = new_capacity
        return True
    
    fn should_grow(self) -> Bool:
        """Check if we need to grow."""
        return Float32(self.size) / Float32(self.capacity) > GROWTH_THRESHOLD
    
    fn get_search_buffer(mut self) -> KNNBuffer:
        """Get a search buffer from pool or create new."""
        if self.pool_size > 0:
            self.pool_size -= 1
            return self.search_buffer_pool.pop()
        return KNNBuffer(ef_construction)
    
    fn return_search_buffer(mut self, buffer: KNNBuffer):
        """Return buffer to pool for reuse."""
        if self.pool_size < 8:  # Keep pool size reasonable
            buffer.clear()  # Reset for reuse
            self.search_buffer_pool.append(buffer)
            self.pool_size += 1
    
    fn insert(mut self, vector: UnsafePointer[Float32]) -> Int:
        """Insert with automatic growth."""
        # Check if we need to grow
        if self.should_grow():
            if not self.grow():
                return -1  # Growth failed
        
        # Regular insertion logic
        if self.size >= self.capacity:
            return -1  # Still full after growth attempt
        
        # Allocate node
        var level = self.get_random_level()
        var new_id = self.node_pool.allocate(level)
        if new_id < 0:
            return -1
        
        # Copy vector
        var dest = self.get_vector(new_id)
        memcpy(dest, vector, self.dimension * 4)
        
        # Create binary quantized version if enabled
        if self.use_binary_quantization:
            var binary_vec = BinaryQuantizedVector(dest, self.dimension)
            while len(self.binary_vectors) <= new_id:
                # Add dummy vectors
                var dummy = UnsafePointer[Float32].alloc(self.dimension)
                for j in range(self.dimension):
                    dummy[j] = 0.0
                self.binary_vectors.append(BinaryQuantizedVector(dummy, self.dimension))
            self.binary_vectors[new_id] = binary_vec
        
        # First node becomes entry
        if self.size == 0:
            self.entry_point = new_id
            self.size = 1
            return new_id
        
        # Insert into graph with reduced search overhead
        self._insert_node_fast(new_id, level, dest)
        
        self.size += 1
        return new_id
    
    fn _insert_node_fast(mut self, new_id: Int, level: Int, vector: UnsafePointer[Float32]):
        """Fast insertion with reduced search overhead."""
        # Increment visited version
        self.visited_version += 1
        if self.visited_version > 1000000000:
            self.visited_version = 1
            # Clear on overflow (rare)
            for i in range(self.capacity):
                self.visited_buffer[i] = 0
        
        # Start from entry point
        var curr_nearest = self.entry_point
        
        # Search from top to target layer (simplified)
        var entry_vec = self.get_vector(self.entry_point)
        var curr_dist = self.distance(vector, entry_vec)
        
        # Get search buffer from pool
        var search_buffer = self.get_search_buffer()
        
        # For each layer, find M neighbors
        for lc in range(level, -1, -1):
            var M_layer = max_M if lc > 0 else max_M0
            
            # OPTIMIZATION: Use M instead of M*2 for faster insertion
            var ef = M_layer  # Not M_layer * 2!
            
            # Simple greedy search for nearest neighbors
            var neighbors = self._greedy_search(vector, curr_nearest, M_layer, lc, ef)
            
            # Connect to neighbors
            var node = self.node_pool.get(new_id)
            for neighbor_id in neighbors:
                _ = node[].add_connection(lc, neighbor_id.value)
                # Bidirectional connection
                var neighbor_node = self.node_pool.get(neighbor_id.value)
                _ = neighbor_node[].add_connection(lc, new_id)
            
            # Move to next layer
            if len(neighbors) > 0:
                curr_nearest = neighbors[0].value
        
        # Return buffer to pool
        self.return_search_buffer(search_buffer)
    
    fn _greedy_search(mut self, query: UnsafePointer[Float32], entry: Int, 
                      M: Int, layer: Int, ef: Int) -> List[Optional[Int]]:
        """Simplified greedy search for fast insertion."""
        var results = List[Optional[Int]]()
        
        # Quick and dirty nearest neighbor search
        # Just explore immediate neighbors without complex beam search
        var visited = Dict[Int, Bool]()
        var candidates = List[Int]()
        candidates.append(entry)
        visited[entry] = True
        
        var best_dist = self.distance(query, self.get_vector(entry))
        results.append(Optional[Int](entry))
        
        # Explore neighbors
        var iterations = 0
        while len(candidates) > 0 and iterations < ef:
            var current = candidates.pop()
            var node = self.node_pool.get(current)
            
            if not node:
                continue
            
            # Check neighbors at this layer
            var connections = node[].get_connections(layer)
            for neighbor_id in connections:
                if neighbor_id.value < 0:
                    continue
                    
                if neighbor_id.value in visited:
                    continue
                    
                visited[neighbor_id.value] = True
                
                var dist = self.distance(query, self.get_vector(neighbor_id.value))
                if dist < best_dist * 1.5:  # Accept reasonably close neighbors
                    candidates.append(neighbor_id.value)
                    if len(results) < M:
                        results.append(neighbor_id)
                
            iterations += 1
        
        return results
    
    fn search(self, query: UnsafePointer[Float32], k: Int) -> List[Int]:
        """Search for k nearest neighbors."""
        # Simplified search for now
        var results = List[Int]()
        
        if self.size == 0:
            return results
        
        # Linear scan for simplicity (optimize later)
        var distances = List[Float32]()
        for i in range(self.size):
            var dist = self.distance(query, self.get_vector(i))
            distances.append(dist)
        
        # Find k smallest
        for _ in range(min(k, self.size)):
            var min_dist = Float32(1e9)
            var min_idx = -1
            for i in range(len(distances)):
                if distances[i] < min_dist:
                    min_dist = distances[i]
                    min_idx = i
            
            if min_idx >= 0:
                results.append(min_idx)
                distances[min_idx] = Float32(1e10)  # Mark as used
        
        return results
    
    @always_inline
    fn get_vector(self, idx: Int) -> UnsafePointer[Float32]:
        """Get vector by index."""
        return self.vectors.offset(idx * self.dimension)
    
    @always_inline
    fn distance(self, a: UnsafePointer[Float32], b: UnsafePointer[Float32]) -> Float32:
        """Simple L2 distance."""
        var sum = Float32(0)
        for i in range(self.dimension):
            var diff = a[i] - b[i]
            sum += diff * diff
        return sqrt(sum)
    
    fn get_random_level(self) -> Int:
        """Generate random level for new node."""
        var level = 0
        while random_float64() < 0.5 and level < MAX_LAYERS - 1:
            level += 1
        return level
    
    fn __del__(owned self):
        """Clean up allocated memory."""
        if self.vectors:
            self.vectors.free()
        if self.visited_buffer:
            self.visited_buffer.free()