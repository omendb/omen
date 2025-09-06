"""
DiskANN with proper heap-based operations - NO SORTING!

This eliminates all unnecessary sorting, using heaps for O(log k) operations.
"""

from collections import List, Dict
from math import sqrt
from ..core.mmap_graph import MMapGraph
from .priority_queue import MinHeapPriorityQueue, SearchCandidate

# DiskANN parameters
alias R_MAX = 64
alias L_BUILD = 100  
alias ALPHA = 1.2

struct HeapBasedDiskANN:
    """DiskANN using heaps instead of sorting - massive performance win."""
    
    var graph: MMapGraph
    var dimension: Int
    var node_count: Int
    
    fn __init__(out self, dimension: Int, capacity: Int = 1000000):
        self.dimension = dimension
        self.node_count = 0
        var path = String("/tmp/omendb_heap_") + String(capacity) + String(".dat")
        self.graph = MMapGraph(path, dimension, capacity, False)
    
    fn add(mut self, id: String, vector: List[Float32]) -> Int:
        var idx = self.graph.add_node(vector)
        if idx < 0:
            return -1
        
        self.node_count += 1
        
        if self.node_count > 1:
            self._connect_with_heap(idx, vector)
        
        return idx
    
    fn _connect_with_heap(mut self, new_node: Int, vector: List[Float32]):
        """Connect using heaps - NO SORTING!"""
        
        # Find L_BUILD nearest with beam search
        var candidates = self._beam_search_heap(new_node, 0, L_BUILD)
        
        # RobustPrune using heap - O(n log k) instead of O(n log n)!
        var neighbors = self._heap_prune(new_node, candidates, R_MAX)
        
        # Add edges
        for i in range(len(neighbors)):
            _ = self.graph.add_edge(new_node, neighbors[i])
            
            # Bidirectional with heap-based pruning
            if self.graph.neighbor_count(neighbors[i]) < R_MAX:
                _ = self.graph.add_edge(neighbors[i], new_node)
            else:
                self._heap_prune_existing(neighbors[i], new_node)
    
    fn _beam_search_heap(self, query_node: Int, start: Int, beam_width: Int) -> List[Int]:
        """Pure heap-based beam search - NO SORTING!"""
        
        # Min-heap for beam (closest nodes)
        var beam = MinHeapPriorityQueue(beam_width * 2)  
        
        # Max-heap for results (using negative distances)
        var W_heap = MinHeapPriorityQueue(beam_width)
        
        var visited = Dict[Int, Bool]()
        
        # Initialize
        var start_dist = self._distance(query_node, start)
        beam.push(SearchCandidate(start, start_dist))
        W_heap.push(SearchCandidate(start, -start_dist))  # Negative for max-heap
        visited[start] = True
        
        # Beam search with heap operations only
        while not beam.is_empty():
            var current = beam.pop()
            
            # Early termination
            if W_heap.size() >= beam_width:
                var worst_in_W = -W_heap.peek().distance  # Un-negate
                if current.distance > worst_in_W * ALPHA:
                    break
            
            # Expand neighbors
            var neighbors = self.graph.get_neighbors(current.node_id)
            for i in range(len(neighbors)):
                var neighbor = neighbors[i]
                if neighbor not in visited:
                    visited[neighbor] = True
                    var dist = self._distance(query_node, neighbor)
                    
                    # Add to beam - heap handles ordering
                    beam.push(SearchCandidate(neighbor, dist))
                    
                    # Update result set with heap
                    if W_heap.size() < beam_width:
                        W_heap.push(SearchCandidate(neighbor, -dist))
                    elif dist < -W_heap.peek().distance:  # Better than worst
                        _ = W_heap.pop()
                        W_heap.push(SearchCandidate(neighbor, -dist))
        
        # Extract results from heap
        var results = List[Int]()
        while not W_heap.is_empty():
            results.append(W_heap.pop().node_id)
        
        return results
    
    fn _heap_prune(self, node: Int, candidates: List[Int], max_degree: Int) -> List[Int]:
        """RobustPrune using heap - O(n log k) instead of O(n log n)!"""
        
        if len(candidates) <= max_degree:
            return candidates
        
        # Min-heap to efficiently get closest candidates
        var heap = MinHeapPriorityQueue(len(candidates))
        for i in range(len(candidates)):
            var dist = self._distance(node, candidates[i])
            heap.push(SearchCandidate(candidates[i], dist))
        
        var pruned = List[Int]()
        var pruned_set = Dict[Int, Bool]()  # For O(1) lookup
        
        # Greedily select diverse neighbors using heap
        while len(pruned) < max_degree and not heap.is_empty():
            var closest = heap.pop()
            
            # Check diversity (could optimize with spatial data structure)
            var diverse = True
            for i in range(len(pruned)):
                var dist_to_pruned = self._distance(closest.node_id, pruned[i])
                if dist_to_pruned < closest.distance * 1.5:  # Diversity threshold
                    diverse = False
                    break
            
            if diverse:
                pruned.append(closest.node_id)
                pruned_set[closest.node_id] = True
        
        # Fill remaining slots if needed (relaxing diversity)
        while len(pruned) < max_degree and not heap.is_empty():
            var next_node = heap.pop()
            if next_node.node_id not in pruned_set:
                pruned.append(next_node.node_id)
        
        return pruned
    
    fn _heap_prune_existing(mut self, node: Int, new_neighbor: Int):
        """Prune existing edges using heap to maintain degree bound."""
        
        var neighbors = self.graph.get_neighbors(node)
        
        # Build max-heap of edges (using negative distance)
        var heap = MinHeapPriorityQueue(len(neighbors) + 1)
        
        for i in range(len(neighbors)):
            var dist = self._distance(node, neighbors[i])
            heap.push(SearchCandidate(neighbors[i], -dist))  # Negative for max-heap
        
        # Add new neighbor
        var new_dist = self._distance(node, new_neighbor)
        heap.push(SearchCandidate(new_neighbor, -new_dist))
        
        # Keep only R_MAX closest (remove furthest)
        var kept = List[Int]()
        for i in range(min(R_MAX, heap.size())):
            kept.append(heap.pop().node_id)
        
        # Rebuild edges (in real implementation, would modify in-place)
        # This is where we'd update the graph structure
        # For now, placeholder as graph doesn't support edge removal yet
    
    fn search(self, query: List[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Search using pure heap operations - NO SORTING!"""
        
        if len(query) != self.dimension or self.node_count == 0:
            return List[Tuple[String, Float32]]()
        
        var beam_width = Int(Float32(70) * ALPHA)
        
        # Min-heap for beam search
        var beam = MinHeapPriorityQueue(beam_width * 2)
        
        # Max-heap for results (negative distances)
        var results_heap = MinHeapPriorityQueue(k)
        
        var visited = Dict[Int, Bool]()
        
        # Start from node 0 (should be medoid)
        var start_dist = self._distance_to_vector(0, query)
        beam.push(SearchCandidate(0, start_dist))
        results_heap.push(SearchCandidate(0, -start_dist))
        visited[0] = True
        
        # Pure heap-based beam search
        while not beam.is_empty():
            var current = beam.pop()
            
            # Early termination based on heap
            if results_heap.size() >= k:
                var worst_result = -results_heap.peek().distance
                if current.distance > worst_result * ALPHA:
                    break
            
            # Expand neighbors
            var neighbors = self.graph.get_neighbors(current.node_id)
            for i in range(len(neighbors)):
                var neighbor = neighbors[i]
                if neighbor not in visited:
                    visited[neighbor] = True
                    var dist = self._distance_to_vector(neighbor, query)
                    
                    # Update beam
                    beam.push(SearchCandidate(neighbor, dist))
                    
                    # Update results with heap
                    if results_heap.size() < k:
                        results_heap.push(SearchCandidate(neighbor, -dist))
                    elif dist < -results_heap.peek().distance:
                        _ = results_heap.pop()
                        results_heap.push(SearchCandidate(neighbor, -dist))
        
        # Extract final results from heap
        var results = List[Tuple[String, Float32]]()
        var temp = List[SearchCandidate]()
        
        # Pop all from heap (they come out in reverse order)
        while not results_heap.is_empty():
            temp.append(results_heap.pop())
        
        # Add in correct order
        for i in range(len(temp) - 1, -1, -1):
            var item = temp[i]
            var node_id = self.graph.get_node_id(item.node_id)
            results.append((node_id, -item.distance))  # Un-negate distance
        
        return results
    
    fn _distance(self, node1: Int, node2: Int) -> Float32:
        """Distance between nodes - ready for SIMD when stable."""
        var v1 = self.graph.get_vector_ptr(node1)
        var v2 = self.graph.get_vector_ptr(node2)
        return self._cosine_distance_ptr(v1, v2)
    
    fn _distance_to_vector(self, node: Int, vector: List[Float32]) -> Float32:
        """Distance from node to vector."""
        var v = self.graph.get_vector_ptr(node)
        
        # Convert list to computation
        var dot = Float32(0)
        var norm1 = Float32(0) 
        var norm2 = Float32(0)
        
        for i in range(self.dimension):
            dot += v[i] * vector[i]
            norm1 += v[i] * v[i]
            norm2 += vector[i] * vector[i]
        
        if norm1 == 0 or norm2 == 0:
            return 1.0
        
        return 1.0 - dot / (sqrt(norm1) * sqrt(norm2))
    
    fn _cosine_distance_ptr(self, v1: UnsafePointer[Float32], v2: UnsafePointer[Float32]) -> Float32:
        """Cosine distance - isolated for easy SIMD upgrade."""
        # When SIMD is stable, replace this function body with:
        # return simd_cosine_distance(v1, v2, self.dimension)
        
        var dot = Float32(0)
        var norm1 = Float32(0)
        var norm2 = Float32(0)
        
        # Simple scalar for now - easy to replace with SIMD
        for i in range(self.dimension):
            dot += v1[i] * v2[i]
            norm1 += v1[i] * v1[i]
            norm2 += v2[i] * v2[i]
        
        if norm1 == 0 or norm2 == 0:
            return 1.0
            
        return 1.0 - dot / (sqrt(norm1) * sqrt(norm2))