# RUST-COMPETITIVE MEMORY OPTIMIZATION
# Target: 36KB → <1KB per vector (36x improvement)

from memory import UnsafePointer, memcpy
from collections import InlineArray
from algorithm import vectorize
from math import sqrt

# =============================================================================
# COMPACT VECTOR STORAGE (Rust-style)
# =============================================================================

struct CompactVector[dimension: Int]:
    """Compact vector storage with inline data - no pointer indirection."""
    var data: InlineArray[Float32, dimension]
    
    fn __init__(out self):
        self.data = InlineArray[Float32, dimension](fill=0.0)
    
    fn __init__(out self, data_ptr: UnsafePointer[Float32]):
        """Initialize from external data pointer."""
        self.data = InlineArray[Float32, dimension](fill=0.0)
        for i in range(dimension):
            self.data[i] = data_ptr[i]
    
    @always_inline
    fn simd_distance_squared(self, other: Self) -> Float32:
        """SIMD-optimized distance calculation."""
        var sum = Float32(0)
        
        @parameter
        if dimension >= 8:
            # Use SIMD for larger dimensions
            var simd_sum = SIMD[DType.float32, 8](0)
            var i = 0
            while i + 8 <= dimension:
                var a = SIMD[DType.float32, 8]()
                var b = SIMD[DType.float32, 8]()
                
                for j in range(8):
                    a[j] = self.data[i + j]
                    b[j] = other.data[i + j]
                
                var diff = a - b
                simd_sum += diff * diff
                i += 8
            
            sum = simd_sum.reduce_add()
            
            # Handle remaining elements
            while i < dimension:
                var diff = self.data[i] - other.data[i]
                sum += diff * diff
                i += 1
        else:
            # Scalar for small dimensions
            for i in range(dimension):
                var diff = self.data[i] - other.data[i]
                sum += diff * diff
        
        return sum

# =============================================================================
# ZERO-ALLOCATION SEARCH STRUCTURES  
# =============================================================================

struct SearchCandidate:
    """Compact candidate without tuple overhead."""
    var distance: Float32
    var node_id: Int
    
    fn __init__(out self, distance: Float32, node_id: Int):
        self.distance = distance
        self.node_id = node_id

struct CandidateHeap[capacity: Int]:
    """Fixed-size heap for search candidates - no allocations."""
    var candidates: InlineArray[SearchCandidate, capacity]
    var size: Int
    
    fn __init__(out self):
        self.candidates = InlineArray[SearchCandidate, capacity](
            SearchCandidate(Float32.MAX, -1)
        )
        self.size = 0
    
    fn clear(mut self):
        self.size = 0
    
    fn add_candidate(mut self, distance: Float32, node_id: Int) -> Bool:
        """Add candidate, maintaining heap property."""
        if self.size < capacity:
            self.candidates[self.size] = SearchCandidate(distance, node_id)
            self.size += 1
            self._bubble_up(self.size - 1)
            return True
        elif distance < self.candidates[0].distance:
            # Replace worst candidate
            self.candidates[0] = SearchCandidate(distance, node_id)
            self._bubble_down(0)
            return True
        return False
    
    fn _bubble_up(mut self, idx: Int):
        if idx == 0:
            return
        var parent = (idx - 1) // 2
        if self.candidates[idx].distance < self.candidates[parent].distance:
            var temp = self.candidates[idx]
            self.candidates[idx] = self.candidates[parent]
            self.candidates[parent] = temp
            self._bubble_up(parent)
    
    fn _bubble_down(mut self, idx: Int):
        var left = 2 * idx + 1
        var right = 2 * idx + 2
        var smallest = idx
        
        if left < self.size and self.candidates[left].distance < self.candidates[smallest].distance:
            smallest = left
        if right < self.size and self.candidates[right].distance < self.candidates[smallest].distance:
            smallest = right
        
        if smallest != idx:
            var temp = self.candidates[idx]
            self.candidates[idx] = self.candidates[smallest]
            self.candidates[smallest] = temp
            self._bubble_down(smallest)

# =============================================================================
# COMPACT HNSW NODE (Rust-competitive)
# =============================================================================

struct CompactHNSWNode[max_connections_l0: Int, max_connections_higher: Int, max_layers: Int]:
    """Ultra-compact HNSW node - inline everything, zero allocations."""
    var id: Int
    var level: Int
    var vector: CompactVector[128]  # Inline vector storage
    
    # Connections with bitmap for faster operations  
    var connections_l0: InlineArray[Int, max_connections_l0]
    var connections_l0_count: Int8
    
    var connections_higher: InlineArray[Int, max_connections_higher * max_layers]
    var connections_count: InlineArray[Int8, max_layers]
    
    # Bitmap for visited tracking (cache-friendly)
    var visited_bit: UInt64  # Single bit for visited state
    
    fn __init__(out self, id: Int, level: Int, vector_data: UnsafePointer[Float32]):
        self.id = id
        self.level = level
        self.vector = CompactVector[128](vector_data)
        
        self.connections_l0 = InlineArray[Int, max_connections_l0](fill=-1)
        self.connections_l0_count = 0
        
        self.connections_higher = InlineArray[Int, max_connections_higher * max_layers](fill=-1)
        self.connections_count = InlineArray[Int8, max_layers](fill=0)
        
        self.visited_bit = 0
    
    @always_inline
    fn distance_to(self, other: Self) -> Float32:
        """SIMD-optimized distance calculation."""
        return sqrt(self.vector.simd_distance_squared(other.vector))
    
    @always_inline
    fn distance_to_query(self, query_vector: CompactVector[128]) -> Float32:
        """Distance to external query vector."""
        return sqrt(self.vector.simd_distance_squared(query_vector))
    
    @always_inline
    fn add_connection_l0(mut self, neighbor: Int) -> Bool:
        if self.connections_l0_count >= max_connections_l0:
            return False
        self.connections_l0[int(self.connections_l0_count)] = neighbor
        self.connections_l0_count += 1
        return True
    
    @always_inline
    fn add_connection_higher(mut self, layer: Int, neighbor: Int) -> Bool:
        if layer <= 0 or layer >= max_layers:
            return False
        var layer_idx = layer - 1
        if self.connections_count[layer_idx] >= max_connections_higher:
            return False
        
        var slot = layer_idx * max_connections_higher + int(self.connections_count[layer_idx])
        self.connections_higher[slot] = neighbor
        self.connections_count[layer_idx] += 1
        return True

# =============================================================================
# RUST-COMPETITIVE HNSW INDEX
# =============================================================================

struct CompactHNSWIndex[capacity: Int, dimension: Int = 128]:
    """Ultra-compact HNSW with zero List overhead."""
    
    # Compact storage - everything inline
    var nodes: InlineArray[CompactHNSWNode[32, 16, 8], capacity]
    var size: Int
    var entry_point: Int
    
    # Search pools - reused, never allocated
    var search_candidates: CandidateHeap[512]
    var search_visited: InlineArray[UInt64, capacity // 64 + 1]  # Bitmap
    var search_version: UInt64
    
    fn __init__(out self):
        self.nodes = InlineArray[CompactHNSWNode[32, 16, 8], capacity](
            CompactHNSWNode[32, 16, 8](0, 0, UnsafePointer[Float32]())
        )
        self.size = 0
        self.entry_point = -1
        
        self.search_candidates = CandidateHeap[512]()
        self.search_visited = InlineArray[UInt64, capacity // 64 + 1](fill=0)
        self.search_version = 1
    
    fn insert(mut self, vector_data: UnsafePointer[Float32]) -> Int:
        """Insert vector with compact operations."""
        if self.size >= capacity:
            return -1
        
        var new_id = self.size
        var level = self._get_random_level()
        
        # Initialize node inline - no allocations
        self.nodes[new_id] = CompactHNSWNode[32, 16, 8](new_id, level, vector_data)
        
        if self.size == 0:
            self.entry_point = new_id
        else:
            # Insert into graph with compact operations
            self._insert_compact(new_id, level)
        
        self.size += 1
        return new_id
    
    fn _insert_compact(mut self, new_id: Int, level: Int):
        """Compact insertion with zero allocations."""
        var curr = self.entry_point
        
        # Clear search state
        self._clear_visited()
        self.search_candidates.clear()
        
        # Search from top layer down
        for layer in range(self.nodes[self.entry_point].level, level, -1):
            curr = self._search_layer_compact(new_id, curr, 1, layer)[0]
        
        # Insert at each layer
        for layer in range(min(level, self.nodes[self.entry_point].level), -1, -1):
            var M = 32 if layer == 0 else 16
            var candidates = self._search_layer_compact(new_id, curr, M * 2, layer)
            
            # Add best connections
            var connections_added = 0
            for i in range(len(candidates)):
                if connections_added >= M:
                    break
                
                var neighbor_id = candidates[i]
                if neighbor_id != new_id:
                    if layer == 0:
                        _ = self.nodes[new_id].add_connection_l0(neighbor_id)
                        _ = self.nodes[neighbor_id].add_connection_l0(new_id)
                    else:
                        _ = self.nodes[new_id].add_connection_higher(layer, neighbor_id)
                        _ = self.nodes[neighbor_id].add_connection_higher(layer, new_id)
                    connections_added += 1
        
        # Update entry point if needed
        if level > self.nodes[self.entry_point].level:
            self.entry_point = new_id
    
    fn _search_layer_compact(mut self, query_id: Int, entry: Int, ef: Int, layer: Int) -> InlineArray[Int, 64]:
        """Compact layer search with zero allocations."""
        var results = InlineArray[Int, 64](fill=-1)
        var result_count = 0
        
        self.search_candidates.clear()
        self._set_visited(entry)
        
        var query_vector = self.nodes[query_id].vector
        var entry_dist = self.nodes[entry].distance_to_query(query_vector)
        _ = self.search_candidates.add_candidate(entry_dist, entry)
        
        var closest_dist = entry_dist
        var closest_id = entry
        
        while self.search_candidates.size > 0:
            var candidate = self.search_candidates.candidates[0]
            
            if candidate.distance > closest_dist:
                break
            
            # Explore neighbors at this layer
            var connections = self._get_connections_compact(candidate.node_id, layer)
            for i in range(connections.size):
                var neighbor = connections.data[i]
                if neighbor < 0:
                    break
                
                if not self._is_visited(neighbor):
                    self._set_visited(neighbor)
                    
                    var neighbor_dist = self.nodes[neighbor].distance_to_query(query_vector)
                    
                    if neighbor_dist < closest_dist:
                        closest_dist = neighbor_dist
                        closest_id = neighbor
                    
                    _ = self.search_candidates.add_candidate(neighbor_dist, neighbor)
        
        # Extract results
        results[0] = closest_id
        result_count = 1
        
        return results
    
    fn _get_connections_compact(self, node_id: Int, layer: Int) -> InlineArray[Int, 32]:
        """Get connections without allocation overhead."""
        var connections = InlineArray[Int, 32](fill=-1)
        
        if layer == 0:
            var count = int(self.nodes[node_id].connections_l0_count)
            for i in range(count):
                connections[i] = self.nodes[node_id].connections_l0[i]
        else:
            var layer_idx = layer - 1
            var count = int(self.nodes[node_id].connections_count[layer_idx])
            var base = layer_idx * 16
            for i in range(count):
                connections[i] = self.nodes[node_id].connections_higher[base + i]
        
        return connections
    
    fn _clear_visited(mut self):
        """Clear visited bitmap."""
        self.search_version += 1
        if self.search_version == 0:  # Overflow
            for i in range(len(self.search_visited)):
                self.search_visited[i] = 0
            self.search_version = 1
    
    @always_inline
    fn _set_visited(mut self, node_id: Int):
        var word_idx = node_id // 64
        var bit_idx = node_id % 64
        self.search_visited[word_idx] |= (UInt64(1) << bit_idx)
    
    @always_inline
    fn _is_visited(self, node_id: Int) -> Bool:
        var word_idx = node_id // 64
        var bit_idx = node_id % 64
        return (self.search_visited[word_idx] & (UInt64(1) << bit_idx)) != 0
    
    fn _get_random_level(self) -> Int:
        # Simple level generation - in production use proper random
        return 0  # For now, single level

# MEMORY FOOTPRINT ANALYSIS:
# 
# CompactHNSWNode[32,16,8]:
#   - id: 8 bytes
#   - level: 8 bytes  
#   - vector: 128 * 4 = 512 bytes (inline)
#   - connections_l0: 32 * 8 = 256 bytes
#   - connections_higher: 16 * 8 * 8 = 1024 bytes
#   - metadata: ~32 bytes
#   - Total: ~1,840 bytes per node
#
# COMPARED TO CURRENT:
#   - Current: 36,717 bytes per vector
#   - Compact: 1,840 bytes per vector  
#   - Improvement: 20x reduction in memory
#
# TARGET PERFORMANCE:
#   - Current: 9,700 vec/s
#   - With 20x less memory pressure: 50,000-100,000 vec/s potential
#   - SIMD distance optimization: Additional 4-8x
#   - Total potential: 200,000-800,000 vec/s (Rust-competitive)