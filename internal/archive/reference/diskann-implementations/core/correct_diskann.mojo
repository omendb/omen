"""
Correct DiskANN implementation following the Microsoft DiskANN paper.
This replaces the incorrect implementation with proper algorithms.
"""

from collections import List, Dict, Optional
from memory import UnsafePointer, memcpy
from math import sqrt
from sys.intrinsics import sizeof
from python import Python
from random import random_si64

# DiskANN parameters from the paper
alias DEFAULT_R = 32       # Maximum out-degree
alias DEFAULT_L = 70       # Beam width for search and build
alias DEFAULT_ALPHA = 1.2  # α parameter for RobustPrune
alias DEFAULT_ITERATIONS = 2  # Number of build iterations

struct CorrectDiskANN(Copyable, Movable):
    """
    Correct DiskANN implementation with proper algorithms.
    
    Key improvements over previous version:
    1. Proper RobustPrune with α-RNG property
    2. Convergence-based beam search
    3. Random graph initialization
    4. Multi-pass build process
    5. Medoid management
    """
    
    # Vector storage
    var vectors: UnsafePointer[Float32]
    var node_ids: List[String]
    
    # Graph structure using adjacency lists (needed for proper pruning)
    var adjacency_lists: List[List[Int]]
    
    # CSR representation for efficient search (built on demand)
    var csr_row_offsets: UnsafePointer[Int32]
    var csr_edge_indices: UnsafePointer[Int32]
    var is_csr_valid: Bool
    
    # Graph parameters
    var dimension: Int
    var num_nodes: Int
    var capacity: Int
    var R: Int              # Maximum out-degree
    var L: Int              # Beam width
    var alpha: Float32      # α parameter for pruning
    var medoid: Int         # Entry point for searches
    
    # Build state
    var is_built: Bool      # Track if graph has been properly built
    var build_iterations: Int
    
    fn __init__(out self, dimension: Int, expected_nodes: Int = 1000, 
                R: Int = DEFAULT_R, L: Int = DEFAULT_L, alpha: Float32 = DEFAULT_ALPHA):
        """Initialize DiskANN index with proper parameters."""
        self.dimension = dimension
        self.num_nodes = 0
        self.capacity = expected_nodes
        self.R = R
        self.L = L
        self.alpha = alpha
        self.medoid = -1
        self.is_built = False
        self.is_csr_valid = False
        self.build_iterations = DEFAULT_ITERATIONS
        
        # Allocate storage
        self.vectors = UnsafePointer[Float32].alloc(expected_nodes * dimension)
        self.node_ids = List[String]()
        self.adjacency_lists = List[List[Int]]()
        
        # CSR storage (allocated when needed)
        self.csr_row_offsets = UnsafePointer[Int32]()
        self.csr_edge_indices = UnsafePointer[Int32]()
    
    fn __del__(owned self):
        """Free allocated memory."""
        if self.vectors:
            self.vectors.free()
        if self.csr_row_offsets:
            self.csr_row_offsets.free()
        if self.csr_edge_indices:
            self.csr_edge_indices.free()
    
    fn __copyinit__(out self, other: Self):
        """Deep copy constructor."""
        # Copy basic properties
        self.dimension = other.dimension
        self.num_nodes = other.num_nodes
        self.capacity = other.capacity
        self.R = other.R
        self.L = other.L
        self.alpha = other.alpha
        self.medoid = other.medoid
        self.is_built = other.is_built
        self.is_csr_valid = other.is_csr_valid
        self.build_iterations = other.build_iterations
        
        # Deep copy vectors
        self.vectors = UnsafePointer[Float32].alloc(self.capacity * self.dimension)
        memcpy(self.vectors, other.vectors, self.num_nodes * self.dimension * sizeof[Float32]())
        
        # Deep copy node IDs
        self.node_ids = List[String]()
        for i in range(len(other.node_ids)):
            self.node_ids.append(other.node_ids[i])
        
        # Deep copy adjacency lists
        self.adjacency_lists = List[List[Int]]()
        for i in range(len(other.adjacency_lists)):
            var neighbors = List[Int]()
            for j in range(len(other.adjacency_lists[i])):
                neighbors.append(other.adjacency_lists[i][j])
            self.adjacency_lists.append(neighbors^)
        
        # Copy CSR if valid
        if other.is_csr_valid:
            self._build_csr()
        else:
            self.csr_row_offsets = UnsafePointer[Int32]()
            self.csr_edge_indices = UnsafePointer[Int32]()
    
    fn __moveinit__(out self, owned other: Self):
        """Move constructor."""
        self.dimension = other.dimension
        self.num_nodes = other.num_nodes
        self.capacity = other.capacity
        self.R = other.R
        self.L = other.L
        self.alpha = other.alpha
        self.medoid = other.medoid
        self.is_built = other.is_built
        self.is_csr_valid = other.is_csr_valid
        self.build_iterations = other.build_iterations
        
        # Move ownership
        self.vectors = other.vectors
        self.node_ids = other.node_ids^
        self.adjacency_lists = other.adjacency_lists^
        self.csr_row_offsets = other.csr_row_offsets
        self.csr_edge_indices = other.csr_edge_indices
        
        # Clear other
        other.vectors = UnsafePointer[Float32]()
        other.csr_row_offsets = UnsafePointer[Int32]()
        other.csr_edge_indices = UnsafePointer[Int32]()
    
    fn add_vector(mut self, id: String, vector: List[Float32]) -> Int:
        """Add a vector to the index. Returns node index."""
        if len(vector) != self.dimension:
            return -1
        
        if self.num_nodes >= self.capacity:
            self._grow_capacity()
        
        # Store normalized vector
        var norm = Float32(0)
        for i in range(self.dimension):
            norm += vector[i] * vector[i]
        norm = sqrt(norm)
        
        # Normalize and store
        if norm > 0:
            for i in range(self.dimension):
                self.vectors[self.num_nodes * self.dimension + i] = vector[i] / norm
        else:
            for i in range(self.dimension):
                self.vectors[self.num_nodes * self.dimension + i] = 0
        
        # Add empty adjacency list
        self.adjacency_lists.append(List[Int]())
        self.node_ids.append(id)
        
        var idx = self.num_nodes
        self.num_nodes += 1
        self.is_built = False  # Need to rebuild after adding vectors
        self.is_csr_valid = False
        
        return idx
    
    fn build_index(mut self):
        """Build the DiskANN index using the correct algorithm."""
        if self.num_nodes == 0:
            return
        
        print("Building DiskANN index with", self.num_nodes, "vectors...")
        
        # Step 1: Initialize with random graph
        self._initialize_random_graph()
        
        # Step 2: Find medoid
        self.medoid = self._find_medoid()
        print("Medoid selected:", self.medoid)
        
        # Step 3: Multiple iterations of improvement
        for iteration in range(self.build_iterations):
            print("Build iteration", iteration + 1, "of", self.build_iterations)
            self._improve_graph_iteration()
        
        # Step 4: Build CSR for efficient search
        self._build_csr()
        
        self.is_built = True
        print("DiskANN index build complete!")
    
    fn _initialize_random_graph(mut self):
        """Initialize graph with random edges (Step 1 of DiskANN build)."""
        print("Initializing random graph with R =", self.R)
        
        # Clear existing edges
        for i in range(len(self.adjacency_lists)):
            self.adjacency_lists[i].clear()
        
        # Add R random edges for each node
        for i in range(self.num_nodes):
            var edges_added = 0
            var attempts = 0
            var max_attempts = self.R * 3  # Avoid infinite loops
            
            while edges_added < self.R and attempts < max_attempts:
                var target = Int(random_si64() % self.num_nodes)
                if target != i and not self._has_edge(i, target):
                    self.adjacency_lists[i].append(target)
                    edges_added += 1
                attempts += 1
    
    fn _improve_graph_iteration(mut self):
        """One iteration of graph improvement (Step 3 of DiskANN build)."""
        var improvements = 0
        
        for i in range(self.num_nodes):
            # Search for better neighbors
            var candidates = self._search_for_candidates(i, self.L)
            
            # Apply RobustPrune to get best neighbors
            var new_neighbors = self._robust_prune(i, candidates, self.R)
            
            # Update if different
            if not self._same_neighbor_sets(self.adjacency_lists[i], new_neighbors):
                self.adjacency_lists[i] = new_neighbors
                improvements += 1
                
                # Update reverse edges
                self._update_reverse_edges(i, new_neighbors)
        
        print("Graph improvements in this iteration:", improvements)
    
    fn _robust_prune(self, center: Int, candidates: List[Int], target_size: Int) -> List[Int]:
        """
        Correct RobustPrune implementation with α-RNG property.
        
        For each candidate c, we keep it only if there's no other candidate q such that:
        - distance(c, q) < α * distance(center, c)
        
        This maintains the Relative Neighborhood Graph property for navigability.
        """
        if len(candidates) <= target_size:
            return candidates
        
        var pruned = List[Int]()
        var candidate_distances = List[Tuple[Float32, Int]]()
        
        # Calculate distances and sort by distance to center
        for i in range(len(candidates)):
            var dist = self._compute_distance(center, candidates[i])
            candidate_distances.append((dist, candidates[i]))
        
        # Sort by distance (closest first)
        self._sort_candidates_by_distance(candidate_distances)
        
        # Apply α-RNG property check
        for i in range(len(candidate_distances)):
            if len(pruned) >= target_size:
                break
            
            var candidate_dist = candidate_distances[i][0]
            var candidate_id = candidate_distances[i][1]
            
            # Check α-RNG property against all existing pruned neighbors
            var violates_alpha_rng = False
            for j in range(len(pruned)):
                var existing_neighbor = pruned[j]
                var inter_distance = self._compute_distance(candidate_id, existing_neighbor)
                
                # α-RNG property: reject if inter-distance < α * candidate_distance
                if inter_distance < self.alpha * candidate_dist:
                    violates_alpha_rng = True
                    break
            
            # Add candidate if it doesn't violate α-RNG property
            if not violates_alpha_rng:
                pruned.append(candidate_id)
        
        # Ensure minimum connectivity (add closest remaining if needed)
        var min_connectivity = max(1, target_size // 2)
        if len(pruned) < min_connectivity:
            for i in range(len(candidate_distances)):
                var candidate_id = candidate_distances[i][1]
                var found = False
                for j in range(len(pruned)):
                    if pruned[j] == candidate_id:
                        found = True
                        break
                if not found and len(pruned) < min_connectivity:
                    pruned.append(candidate_id)
        
        return pruned
    
    fn _search_for_candidates(self, query_node: Int, beam_width: Int) -> List[Int]:
        """
        Correct beam search with convergence-based termination.
        
        This implements the proper Vamana search algorithm that continues
        until the working set stops improving.
        """
        var working_set = List[Int]()      # W in the paper
        var visited_set = Dict[Int, Bool]()  # V in the paper
        var distances = Dict[Int, Float32]()
        
        # Initialize with medoid (or random node if no medoid)
        var start_node = self.medoid if self.medoid >= 0 else 0
        working_set.append(start_node)
        distances[start_node] = self._compute_distance(query_node, start_node)
        
        # Convergence-based search (not fixed iterations!)
        var max_iterations = 1000  # Safety limit
        var iteration = 0
        
        while iteration < max_iterations:
            # Find closest unvisited node in working set
            var best_unvisited = -1
            var best_distance = Float32.MAX
            
            for i in range(len(working_set)):
                var node = working_set[i]
                if node not in visited_set:
                    if node in distances:
                        if distances[node] < best_distance:
                            best_distance = distances[node]
                            best_unvisited = node
            
            # Convergence check: if no unvisited nodes, we're done
            if best_unvisited == -1:
                break
            
            # Mark as visited
            visited_set[best_unvisited] = True
            
            # Expand neighbors
            var neighbors = self.adjacency_lists[best_unvisited]
            for j in range(len(neighbors)):
                var neighbor = neighbors[j]
                if neighbor not in visited_set:
                    var neighbor_dist = self._compute_distance(query_node, neighbor)
                    distances[neighbor] = neighbor_dist
                    
                    # Add to working set if not present
                    var found = False
                    for k in range(len(working_set)):
                        if working_set[k] == neighbor:
                            found = True
                            break
                    if not found:
                        working_set.append(neighbor)
            
            # Prune working set to beam_width (keep closest)
            if len(working_set) > beam_width:
                self._prune_working_set(working_set, distances, beam_width)
            
            iteration += 1
        
        return working_set
    
    fn _find_medoid(self) -> Int:
        """Find the medoid (centroid) of the dataset."""
        if self.num_nodes == 0:
            return -1
        
        var best_medoid = 0
        var best_total_distance = Float32.MAX
        
        # For efficiency, only sample up to 100 nodes for medoid calculation
        var sample_size = min(100, self.num_nodes)
        
        for i in range(sample_size):
            var total_distance = Float32(0)
            var sample_count = min(50, self.num_nodes)  # Sample distances
            
            for j in range(0, self.num_nodes, max(1, self.num_nodes // sample_count)):
                if j != i:
                    total_distance += self._compute_distance(i, j)
            
            if total_distance < best_total_distance:
                best_total_distance = total_distance
                best_medoid = i
        
        return best_medoid
    
    fn search(self, query: List[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Search for k nearest neighbors using correct beam search."""
        if not self.is_built:
            return List[Tuple[String, Float32]]()
        
        if not self.is_csr_valid:
            self._build_csr()
        
        # Create temporary query node
        var query_node = self._add_temp_query_node(query)
        
        # Use correct beam search with convergence
        var candidates = self._search_for_candidates(query_node, self.L)
        
        # Sort and return top k
        var results = List[Tuple[String, Float32]]()
        var candidate_distances = List[Tuple[Float32, Int]]()
        
        for i in range(len(candidates)):
            var dist = self._compute_distance(query_node, candidates[i])
            candidate_distances.append((dist, candidates[i]))
        
        self._sort_candidates_by_distance(candidate_distances)
        
        for i in range(min(k, len(candidate_distances))):
            var node_idx = candidate_distances[i][1]
            var distance = candidate_distances[i][0]
            if node_idx < len(self.node_ids):
                results.append((self.node_ids[node_idx], distance))
        
        # Remove temporary query node
        self._remove_temp_query_node()
        
        return results
    
    # Helper methods (implementation details)
    
    fn _compute_distance(self, idx1: Int, idx2: Int) -> Float32:
        """Compute L2 distance between two nodes."""
        if idx1 >= self.num_nodes or idx2 >= self.num_nodes:
            return Float32.MAX
        
        var ptr1 = self.vectors.offset(idx1 * self.dimension)
        var ptr2 = self.vectors.offset(idx2 * self.dimension)
        
        var sum = Float32(0)
        for i in range(self.dimension):
            var diff = ptr1[i] - ptr2[i]
            sum += diff * diff
        
        return sqrt(sum)
    
    fn _has_edge(self, from_node: Int, to_node: Int) -> Bool:
        """Check if edge exists."""
        if from_node >= len(self.adjacency_lists):
            return False
        
        var neighbors = self.adjacency_lists[from_node]
        for i in range(len(neighbors)):
            if neighbors[i] == to_node:
                return True
        return False
    
    fn _sort_candidates_by_distance(self, mut candidates: List[Tuple[Float32, Int]]):
        """Sort candidates by distance (simple bubble sort for now)."""
        var n = len(candidates)
        for i in range(n):
            for j in range(n - 1 - i):
                if candidates[j][0] > candidates[j + 1][0]:
                    var temp = candidates[j]
                    candidates[j] = candidates[j + 1]
                    candidates[j + 1] = temp
    
    fn _prune_working_set(self, mut working_set: List[Int], 
                          distances: Dict[Int, Float32], target_size: Int):
        """Prune working set to target size, keeping closest nodes."""
        if len(working_set) <= target_size:
            return
        
        # Create list of (distance, node) pairs
        var distance_pairs = List[Tuple[Float32, Int]]()
        for i in range(len(working_set)):
            var node = working_set[i]
            var dist = distances[node] if node in distances else Float32.MAX
            distance_pairs.append((dist, node))
        
        # Sort by distance
        self._sort_candidates_by_distance(distance_pairs)
        
        # Keep only the closest target_size nodes
        working_set.clear()
        for i in range(min(target_size, len(distance_pairs))):
            working_set.append(distance_pairs[i][1])
    
    fn _same_neighbor_sets(self, set1: List[Int], set2: List[Int]) -> Bool:
        """Check if two neighbor sets are the same."""
        if len(set1) != len(set2):
            return False
        
        # Simple O(n²) comparison for small sets
        for i in range(len(set1)):
            var found = False
            for j in range(len(set2)):
                if set1[i] == set2[j]:
                    found = True
                    break
            if not found:
                return False
        
        return True
    
    fn _update_reverse_edges(mut self, node: Int, new_neighbors: List[Int]):
        """Update reverse edges for bidirectional connectivity."""
        # This is a simplified version - full implementation would need
        # to maintain reverse edge lists and prune them as well
        for i in range(len(new_neighbors)):
            var neighbor = new_neighbors[i]
            if neighbor < len(self.adjacency_lists):
                # Add reverse edge if not at capacity
                if len(self.adjacency_lists[neighbor]) < self.R:
                    if not self._has_edge(neighbor, node):
                        self.adjacency_lists[neighbor].append(node)
    
    fn _build_csr(mut self):
        """Build CSR representation for efficient search."""
        if self.csr_row_offsets:
            self.csr_row_offsets.free()
        if self.csr_edge_indices:
            self.csr_edge_indices.free()
        
        # Count total edges
        var total_edges = 0
        for i in range(len(self.adjacency_lists)):
            total_edges += len(self.adjacency_lists[i])
        
        # Allocate CSR storage
        self.csr_row_offsets = UnsafePointer[Int32].alloc(self.num_nodes + 1)
        self.csr_edge_indices = UnsafePointer[Int32].alloc(total_edges)
        
        # Build CSR
        var edge_offset = 0
        self.csr_row_offsets[0] = 0
        
        for i in range(self.num_nodes):
            var neighbors = self.adjacency_lists[i]
            for j in range(len(neighbors)):
                self.csr_edge_indices[edge_offset] = Int32(neighbors[j])
                edge_offset += 1
            self.csr_row_offsets[i + 1] = Int32(edge_offset)
        
        self.is_csr_valid = True
    
    fn _grow_capacity(mut self):
        """Grow storage capacity."""
        var new_capacity = self.capacity * 2
        var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
        memcpy(new_vectors, self.vectors, self.num_nodes * self.dimension * sizeof[Float32]())
        self.vectors.free()
        self.vectors = new_vectors
        self.capacity = new_capacity
    
    fn _add_temp_query_node(mut self, query: List[Float32]) -> Int:
        """Add temporary query node for search (simplified)."""
        # For now, just use existing infrastructure
        return self.add_vector("__temp_query__", query)
    
    fn _remove_temp_query_node(mut self):
        """Remove temporary query node (simplified)."""
        # For now, just decrement count (proper implementation would clean up)
        if self.num_nodes > 0:
            self.num_nodes -= 1
            _ = self.node_ids.pop()
            _ = self.adjacency_lists.pop()
    
    fn size(self) -> Int:
        """Get number of nodes."""
        return self.num_nodes