"""
Core traits for modular OmenDB architecture.
Defines interfaces that components must implement.
"""

from collections import List, Dict

# Result types for clean API boundaries
@value
struct SearchResult:
    var id: String
    var distance: Float32

@value
struct IndexStats:
    var node_count: Int
    var edge_count: Int
    var avg_degree: Float32
    var memory_mb: Float64

@value
struct MemoryStats:
    var total_allocated: Int
    var total_freed: Int
    var current_usage: Int
    var peak_usage: Int
    var components: Dict[String, Int]

# Core traits for swappable components
trait VectorIndex:
    """Interface for vector indexing algorithms."""
    
    fn add(mut self, id: String, vector: List[Float32]) raises -> Bool:
        """Add a vector to the index."""
        ...
    
    fn search(mut self, query: List[Float32], k: Int) -> List[SearchResult]:
        """Search for k nearest neighbors."""
        ...
    
    fn delete(mut self, id: String) raises -> Bool:
        """Remove a vector from the index."""
        ...
    
    fn size(self) -> Int:
        """Get number of vectors in index."""
        ...
    
    fn get_stats(self) -> IndexStats:
        """Get index statistics."""
        ...

trait VectorStorage:
    """Interface for vector storage backends."""
    
    fn store(mut self, id: String, vector: List[Float32]) raises:
        """Store a vector."""
        ...
    
    fn retrieve(self, id: String) raises -> List[Float32]:
        """Retrieve a vector by ID."""
        ...
    
    fn delete(mut self, id: String) raises -> Bool:
        """Delete a vector."""
        ...
    
    fn checkpoint(mut self) raises -> Bool:
        """Persist data to storage."""
        ...

trait MemoryTracker:
    """Interface for memory tracking."""
    
    fn track_allocation(mut self, component: String, bytes: Int):
        """Track memory allocation."""
        ...
    
    fn track_deallocation(mut self, component: String, bytes: Int):
        """Track memory deallocation."""
        ...
    
    fn get_stats(self) -> MemoryStats:
        """Get current memory statistics."""
        ...
    
    fn get_component_usage(self, component: String) -> Int:
        """Get memory usage for specific component."""
        ...

# Quantization types for configuration
@value
struct QuantizationType:
    alias NONE = 0
    alias SCALAR_INT8 = 1
    alias BINARY = 2
    var value: Int

@value 
struct IndexType:
    alias SPARSE_DISKANN = 0
    alias DISKANN = 1
    alias BRUTE_FORCE = 2
    var value: Int

@value
struct StorageType:
    alias MEMORY_MAPPED = 0
    alias IN_MEMORY = 1
    alias PERSISTENT = 2
    var value: Int