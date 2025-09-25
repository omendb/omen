"""
EMERGENCY STABLE INDEX - Simple but reliable vector search.

The current HNSW has critical memory corruption issues causing segfaults 
after inserting just 1 vector. This provides a working alternative that:
- Uses safe memory management  
- Provides basic nearest neighbor search
- Actually works without crashing
- Can serve as foundation for better algorithms later

Performance: ~1000x slower than ideal, but 1000x faster than segfault.
"""

from memory import UnsafePointer, memcpy
from collections import List, Dict
from math import sqrt
from algorithm import vectorize
from sys.info import simdwidthof

# Safe constants 
alias MAX_VECTORS = 100000
alias SIMD_WIDTH = simdwidthof[DType.float32]()

struct StableVectorIndex(Movable):
    """
    Simple but stable vector index using linear search.
    
    No fancy graphs, no complex allocations, just safe arrays
    and basic distance calculations that actually work.
    """
    
    var vectors: UnsafePointer[Float32]
    var dimension: Int
    var capacity: Int
    var size: Int
    var initialized: Bool
    
    # Safe ID mapping using Dict
    var id_to_index: Dict[String, Int]
    var index_to_id: Dict[Int, String]
    
    fn __init__(out self, dimension: Int, capacity: Int):
        """Initialize with safe, pre-allocated memory."""
        self.dimension = dimension
        self.capacity = min(capacity, MAX_VECTORS)  # Safety limit
        self.size = 0
        self.initialized = False
        
        # Single contiguous allocation - much safer than complex pools
        var total_floats = self.capacity * self.dimension
        self.vectors = UnsafePointer[Float32].alloc(total_floats)
        
        # Initialize to zero for safety
        for i in range(total_floats):
            self.vectors[i] = 0.0
        
        # Initialize mappings
        self.id_to_index = Dict[String, Int]()
        self.index_to_id = Dict[Int, String]()
        
        self.initialized = True
        print("StableVectorIndex initialized: ", self.capacity, " vectors, ", dimension, "d")
    
    fn __moveinit__(out self, owned existing: Self):
        """Safe move constructor."""
        self.vectors = existing.vectors
        self.dimension = existing.dimension
        self.capacity = existing.capacity
        self.size = existing.size
        self.initialized = existing.initialized
        self.id_to_index = existing.id_to_index^
        self.index_to_id = existing.index_to_id^
        
        # Prevent double-free
        existing.vectors = UnsafePointer[Float32]()
        existing.initialized = False
    
    fn __del__(owned self):
        """Safe cleanup."""
        if self.initialized and self.vectors:
            self.vectors.free()
    
    @always_inline
    fn euclidean_distance(self, vec1: UnsafePointer[Float32], vec2: UnsafePointer[Float32]) -> Float32:
        """SIMD-optimized distance calculation."""
        var sum = Float32(0.0)
        
        # Use SIMD for the main portion
        @parameter
        fn simd_distance[width: Int](i: Int):
            var v1 = vec1.load[width=width](i)
            var v2 = vec2.load[width=width](i)
            var diff = v1 - v2
            var squared = diff * diff
            # Sum reduction
            for j in range(width):
                sum += squared[j]
        
        # Process SIMD-width chunks
        var simd_end = (self.dimension // SIMD_WIDTH) * SIMD_WIDTH
        vectorize[simd_distance, SIMD_WIDTH](simd_end)
        
        # Handle remaining elements
        for i in range(simd_end, self.dimension):
            var diff = vec1[i] - vec2[i]
            sum += diff * diff
        
        return sqrt(sum)
    
    fn insert(mut self, vector: UnsafePointer[Float32], id: String) -> Bool:
        """Safely insert a vector. No complex graph building."""
        if not self.initialized:
            return False
        
        if self.size >= self.capacity:
            print("StableIndex capacity exceeded:", self.size, "/", self.capacity)
            return False
        
        # Check for duplicate ID
        if id in self.id_to_index:
            print("Duplicate ID:", id)
            return False
        
        # Safe memory copy
        var index = self.size
        var dest = self.vectors.offset(index * self.dimension)
        memcpy(dest, vector, self.dimension * 4)  # Float32 = 4 bytes
        
        # Update mappings
        self.id_to_index[id] = index
        self.index_to_id[index] = id
        
        # Update size last (atomic-ish)
        self.size += 1
        
        if self.size % 1000 == 0:
            print("StableIndex: ", self.size, " vectors indexed")
        
        return True
    
    fn search(self, query: UnsafePointer[Float32], k: Int) -> List[Tuple[String, Float32]]:
        """Linear search - slow but guaranteed to work."""
        if not self.initialized or self.size == 0:
            return List[Tuple[String, Float32]]()
        
        var candidates = List[Tuple[Float32, Int]]()
        
        # Calculate distances to all vectors
        for i in range(self.size):
            var vector = self.vectors.offset(i * self.dimension)
            var distance = self.euclidean_distance(query, vector)
            candidates.append((distance, i))
        
        # Simple bubble sort for k smallest (inefficient but safe)
        var actual_k = min(k, self.size)
        for i in range(actual_k):
            for j in range(i + 1, len(candidates)):
                if candidates[j][0] < candidates[i][0]:
                    var temp = candidates[i]
                    candidates[i] = candidates[j] 
                    candidates[j] = temp
        
        # Build result with IDs
        var results = List[Tuple[String, Float32]]()
        for i in range(actual_k):
            var index = candidates[i][1]
            var distance = candidates[i][0]
            var id = self.index_to_id.get(index, "unknown")
            results.append((id, distance))
        
        return results
    
    fn insert_batch(mut self, vectors: UnsafePointer[Float32], ids: List[String], count: Int) -> Int:
        """Batch insert - just calls insert() in a loop for safety."""
        var successful = 0
        
        for i in range(min(count, len(ids))):
            var vector = vectors.offset(i * self.dimension)
            var id = ids[i]
            
            if self.insert(vector, id):
                successful += 1
            else:
                print("Batch insert failed at index", i)
                break
        
        return successful
    
    fn get_stats(self) -> String:
        """Get index statistics."""
        return "StableVectorIndex: " + String(self.size) + "/" + String(self.capacity) + " vectors, " + String(self.dimension) + "d"