"""
Brute force search for small datasets (<10K vectors).
Simple, fast, and 100% accurate.
"""

from collections import List
from memory import UnsafePointer, memcpy
from math import sqrt
from ..utils.memory_pool import allocate_vector, free_vector
from ..core.sparse_map import SparseMap

@value
struct BruteForceIndex:
    """Simple brute force index for small datasets.
    
    Much faster than DiskANN for <10K vectors:
    - No graph maintenance overhead
    - 100% search accuracy
    - Better cache locality
    - Simpler code
    """
    
    var vectors: List[UnsafePointer[Float32]]
    var ids: List[String]
    var id_to_index: SparseMap  # O(1) ID lookup (180x more memory efficient)
    var dimension: Int
    var size: Int
    
    fn __init__(out self, dimension: Int):
        self.vectors = List[UnsafePointer[Float32]]()
        self.ids = List[String]()
        self.id_to_index = SparseMap(50000)  # Large capacity for production scale
        self.dimension = dimension
        self.size = 0
    
    fn __del__(owned self):
        """Free allocated memory."""
        for i in range(len(self.vectors)):
            self.vectors[i].free()
            # free_vector(self.vectors[i], self.dimension)
    
    fn add(mut self, id: String, vector: List[Float32]) -> Bool:
        """Add a vector - O(1) complexity."""
        if len(vector) != self.dimension:
            return False
        
        # Direct allocation (pool disabled due to invalid pointer issues)
        var vec_ptr = UnsafePointer[Float32].alloc(self.dimension)
        # var vec_ptr = allocate_vector(self.dimension)
        
        # Pre-normalize for cosine similarity
        var norm_sq = Float32(0)
        for i in range(self.dimension):
            norm_sq += vector[i] * vector[i]
        
        var norm = sqrt(norm_sq + Float32(1e-12))
        var inv_norm = 1.0 / norm
        
        # Store normalized vector
        for i in range(self.dimension):
            vec_ptr[i] = vector[i] * inv_norm
        
        self.vectors.append(vec_ptr)
        self.ids.append(id)
        self.id_to_index.insert(id, self.size)  # Map ID to its index
        self.size += 1
        
        return True
    
    fn search(self, query: List[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Brute force search - O(n) but fast for small n."""
        if self.size == 0:
            return List[Tuple[String, Float32]]()
        
        # Normalize query
        var query_norm_sq = Float32(0)
        for i in range(self.dimension):
            query_norm_sq += query[i] * query[i]
        
        var query_norm = sqrt(query_norm_sq + Float32(1e-12))
        var inv_query_norm = 1.0 / query_norm
        
        # Compute all distances
        var distances = List[Tuple[Int, Float32]]()
        
        for idx in range(self.size):
            var vec_ptr = self.vectors[idx]
            
            # Compute cosine distance (vectors are pre-normalized)
            var dot_product = Float32(0)
            
            # Unrolled loop for better performance
            var i = 0
            while i + 4 <= self.dimension:
                var q0 = query[i] * inv_query_norm
                var q1 = query[i+1] * inv_query_norm
                var q2 = query[i+2] * inv_query_norm
                var q3 = query[i+3] * inv_query_norm
                
                dot_product += q0 * vec_ptr[i]
                dot_product += q1 * vec_ptr[i+1]
                dot_product += q2 * vec_ptr[i+2]
                dot_product += q3 * vec_ptr[i+3]
                
                i += 4
            
            # Handle remainder
            while i < self.dimension:
                dot_product += (query[i] * inv_query_norm) * vec_ptr[i]
                i += 1
            
            # Cosine distance = 1 - similarity
            var distance = 1.0 - dot_product
            
            # Clamp to valid range
            if distance < 0.0:
                distance = 0.0
            elif distance > 2.0:
                distance = 2.0
            
            distances.append((idx, distance))
        
        # Sort by distance (simple sort for small k)
        for i in range(len(distances)):
            for j in range(i + 1, len(distances)):
                if distances[j][1] < distances[i][1]:
                    var temp = distances[i]
                    distances[i] = distances[j]
                    distances[j] = temp
        
        # Return top k results
        var results = List[Tuple[String, Float32]]()
        var count = min(k, len(distances))
        
        for i in range(count):
            var idx = distances[i][0]
            var dist = distances[i][1]
            results.append((self.ids[idx], dist))
        
        return results
    
    fn clear(mut self):
        """Clear all vectors."""
        for i in range(len(self.vectors)):
            self.vectors[i].free()
        
        self.vectors.clear()
        self.ids.clear()
        self.id_to_index = SparseMap(50000)  # Reset with large capacity
        self.size = 0
    
    fn remove(mut self, id: String) raises -> Bool:
        """Remove a vector by ID - O(1) lookup."""
        if not self.id_to_index.contains(id):
            return False  # ID not found
        
        var idx_opt = self.id_to_index.get(id)
        if not idx_opt:
            return False  # Shouldn't happen after contains check
        var i = idx_opt.value()
        _ = self.id_to_index.remove(id)
        
        # Free memory
        self.vectors[i].free()
        
        # Remove from lists (swap with last and pop)
        if i < len(self.ids) - 1:
            var last_id = self.ids[len(self.ids) - 1]
            self.vectors[i] = self.vectors[len(self.vectors) - 1]
            self.ids[i] = last_id
            # Update mapping for the moved element
            self.id_to_index.insert(last_id, i)
        
        _ = self.vectors.pop()
        _ = self.ids.pop()
        self.size -= 1
        
        return True