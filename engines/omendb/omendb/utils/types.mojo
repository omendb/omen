"""Common types and constants for OmenDB.

This module contains shared type definitions, enums, and constants
used throughout the OmenDB codebase.
"""

# =============================================================================
# CONSTANTS
# =============================================================================

alias DEFAULT_BUFFER_SIZE = 100       # Flush frequently for consistent performance and to enable quantization testing
alias MAX_VECTOR_DIM = 4096          # Maximum supported vector dimension
alias DEFAULT_BEAM_WIDTH = 32        # Default beam width for search
alias DEFAULT_MAX_EDGES = 64         # Default max edges per node

# =============================================================================
# TYPE DEFINITIONS
# =============================================================================

@value
struct AlgorithmType:
    """Type of index algorithm to use."""
    alias FLAT = 0      # Simple brute force - only for buffer
    alias DISKANN = 1   # Production algorithm - O(log n), no rebuilds
    
    var value: Int
    
    fn __init__(out self, value: Int = 1):  # Default to DiskANN
        self.value = value
        
    fn __eq__(self, other: Self) -> Bool:
        return self.value == other.value
        
    fn __ne__(self, other: Self) -> Bool:
        return self.value != other.value

@value  
struct StorageType:
    """Type of storage engine to use."""
    alias IN_MEMORY = 0      # No persistence (testing/benchmarks)
    alias SNAPSHOT = 1       # Traditional checkpoint-based (legacy)
    alias MEMORY_MAPPED = 2  # State-of-the-art memory-mapped (default)
    
    var value: Int
    
    fn __init__(out self, value: Int = 2):  # Default to memory-mapped
        self.value = value
        
    fn __eq__(self, other: Self) -> Bool:
        return self.value == other.value
        
    fn __ne__(self, other: Self) -> Bool:
        return self.value != other.value
        
    fn is_persistent(self) -> Bool:
        """Check if this storage type supports persistence."""
        return self.value != Self.IN_MEMORY

@value
struct QuantizationType:
    """Type of vector quantization to use."""
    alias NONE = 0           # No quantization (full precision)
    alias SCALAR = 1         # 8-bit scalar quantization (4x savings)
    alias BINARY = 2         # 1-bit binary quantization (32x savings)
    alias PRODUCT = 3        # Product quantization (configurable)
    
    var value: Int
    
    fn __init__(out self, value: Int = 0):  # Default to no quantization
        self.value = value
        
    fn __eq__(self, other: Self) -> Bool:
        return self.value == other.value
        
    fn get_compression_ratio(self) -> Float32:
        """Get the theoretical compression ratio for this quantization type."""
        if self.value == Self.SCALAR:
            return 4.0  # 32-bit to 8-bit
        elif self.value == Self.BINARY:
            return 32.0  # 32-bit to 1-bit
        elif self.value == Self.PRODUCT:
            return 8.0  # Typical for PQ with 8 subquantizers
        else:
            return 1.0  # No compression

@value
struct SearchMode:
    """Search mode for query operations."""
    alias EXACT = 0          # Exact brute-force search
    alias APPROXIMATE = 1    # Approximate nearest neighbor search
    alias HYBRID = 2         # Hybrid exact + approximate
    
    var value: Int
    
    fn __init__(out self, value: Int = 1):  # Default to approximate
        self.value = value

@value
struct DistanceMetric:
    """Distance metric for similarity computation."""
    alias L2 = 0             # Euclidean distance
    alias COSINE = 1         # Cosine similarity
    alias DOT_PRODUCT = 2    # Dot product similarity
    alias HAMMING = 3        # Hamming distance (for binary vectors)
    
    var value: Int
    
    fn __init__(out self, value: Int = 0):  # Default to L2
        self.value = value
        
    fn requires_normalization(self) -> Bool:
        """Check if this metric requires vector normalization."""
        return self.value == Self.COSINE

# =============================================================================
# RESULT TYPES
# =============================================================================

@value
struct SearchResult:
    """Result from a vector search operation."""
    var id: String           # Vector ID
    var distance: Float32    # Distance/similarity score
    var metadata: Dict[String, String]  # Associated metadata
    
    fn __init__(out self, id: String, distance: Float32):
        self.id = id
        self.distance = distance
        self.metadata = Dict[String, String]()
        
    fn __lt__(self, other: Self) -> Bool:
        """Compare by distance for sorting."""
        return self.distance < other.distance

@value
struct IndexStats:
    """Statistics about the vector index."""
    var total_vectors: Int
    var dimension: Int
    var buffer_size: Int
    var main_index_size: Int
    var memory_usage_mb: Float32
    var algorithm: String
    var storage_type: String
    
    fn __init__(out self):
        self.total_vectors = 0
        self.dimension = 0
        self.buffer_size = 0
        self.main_index_size = 0
        self.memory_usage_mb = 0.0
        self.algorithm = "unknown"
        self.storage_type = "unknown"

# =============================================================================
# ERROR TYPES
# =============================================================================

@value
struct DatabaseError:
    """Error type for database operations."""
    alias NOT_INITIALIZED = "Database not initialized"
    alias DIMENSION_MISMATCH = "Vector dimension mismatch"
    alias VECTOR_NOT_FOUND = "Vector not found"
    alias COLLECTION_NOT_FOUND = "Collection not found"
    alias INVALID_PARAMETER = "Invalid parameter"
    alias STORAGE_ERROR = "Storage operation failed"
    alias MEMORY_ERROR = "Memory allocation failed"
    
    var message: String
    
    fn __init__(out self, message: String):
        self.message = message
        
    fn __str__(self) -> String:
        return self.message