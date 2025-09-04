"""
Distance calculation API layer for OmenDB.

Provides unified distance metric enums and imports optimized implementations 
from distance_functions.mojo to avoid code duplication.
"""

from math import sqrt
from memory import UnsafePointer
from algorithm import vectorize
from sys.info import simdwidthof

# Import optimized implementations to avoid duplication
from .distance_functions import (
    euclidean_distance as _euclidean_distance_impl,
    cosine_distance as _cosine_distance_impl,
    dot_product as _dot_product_impl
)


# ========================================
# Distance Metric Enum
# ========================================

struct DistanceMetric(Copyable, Movable):
    """Enumeration of supported distance metrics."""

    alias EUCLIDEAN = 0
    alias COSINE = 1
    alias MANHATTAN = 2
    alias DOT_PRODUCT = 3
    alias L2 = 0  # Alias for EUCLIDEAN

    var value: Int

    fn __init__(out self, value: Int = 1):  # Default to COSINE
        self.value = value

    fn __eq__(self, other: Self) -> Bool:
        """Compare two distance metrics."""
        return self.value == other.value


# ========================================
# Core Distance Functions
# ========================================

fn euclidean_distance(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute Euclidean (L2) distance between two vectors.
    
    Delegates to optimized implementation in distance_functions.mojo.
    """
    return _euclidean_distance_impl(a, b, dim)


fn cosine_distance(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute cosine distance between two vectors.
    
    Delegates to optimized implementation in distance_functions.mojo.
    """
    return _cosine_distance_impl(a, b, dim)


fn manhattan_distance(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute Manhattan (L1) distance between two vectors.
    
    Args:
        a: First vector data pointer
        b: Second vector data pointer
        dim: Vector dimension
        
    Returns:
        Manhattan distance as Float32
    """
    alias simd_width = simdwidthof[DType.float32]()
    var sum_abs = Float32(0)

    # Simple scalar implementation for now
    # TODO: Optimize with SIMD abs when available
    for i in range(dim):
        var diff = a[i] - b[i]
        sum_abs += diff if diff >= 0 else -diff

    return sum_abs


fn dot_product(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute dot product between two vectors.
    
    Delegates to optimized implementation in distance_functions.mojo.
    """
    return _dot_product_impl(a, b, dim)


# ========================================
# Legacy Compatibility Functions
# ========================================

fn cosine_similarity(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute cosine similarity (for backward compatibility).
    
    Args:
        a: First vector data pointer
        b: Second vector data pointer
        dim: Vector dimension
        
    Returns:
        Cosine similarity in range [-1, 1]
    """
    return 1.0 - cosine_distance(a, b, dim)


fn l2_distance(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Alias for euclidean_distance (for backward compatibility)."""
    return euclidean_distance(a, b, dim)


# ========================================
# Batch Operations (Placeholder)
# ========================================

struct BatchDistanceOperations:
    """Batch distance operations for performance optimization."""
    
    @staticmethod
    fn l2_distance_batch_simd[dtype: DType](
        query: UnsafePointer[Scalar[dtype]],
        vectors: List[UnsafePointer[Scalar[dtype]]],
        dim: Int,
        results: UnsafePointer[Float32]
    ):
        """Compute L2 distances for multiple vectors using SIMD.
        
        Args:
            query: Query vector pointer
            vectors: List of vector pointers to compare against
            dim: Vector dimension
            results: Output buffer for distances
        """
        # Simple implementation using single vector distance function
        @parameter
        if dtype == DType.float32:
            # Cast to Float32 pointers for compatibility
            var query_f32 = query.bitcast[Float32]()
            
            for i in range(len(vectors)):
                var vector_f32 = vectors[i].bitcast[Float32]()
                results[i] = euclidean_distance(query_f32, vector_f32, dim)
        else:
            # Fallback for other types - convert each element
            for i in range(len(vectors)):
                var sum = Float32(0)
                for j in range(dim):
                    var diff = Float32(query[j]) - Float32(vectors[i][j])
                    sum += diff * diff
                results[i] = sqrt(sum)