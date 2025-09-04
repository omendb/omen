"""
High-performance SIMD distance calculations for OmenDB.
Provides 4-8x speedup over scalar implementations.
"""

from math import sqrt
from memory import UnsafePointer

@always_inline
fn simd_l2_distance_squared(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    """Calculate squared L2 distance using SIMD instructions."""
    var sum = Float32(0)
    var i = 0
    
    # SIMD path: process 8 floats at once
    while i + 8 <= dim:
        var vec_a = (a + i).load[width=8]()
        var vec_b = (b + i).load[width=8]()
        var diff = vec_a - vec_b
        sum += (diff * diff).reduce_add()
        i += 8
    
    # Handle remaining elements (scalar)
    while i < dim:
        var diff = a[i] - b[i]
        sum += diff * diff
        i += 1
    
    return sum

@always_inline  
fn simd_l2_distance(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    """Calculate L2 distance using SIMD instructions."""
    return sqrt(simd_l2_distance_squared(a, b, dim))

@always_inline
fn simd_cosine_distance(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    """Calculate cosine distance using SIMD instructions.
    
    Computes dot product and norms in parallel for maximum efficiency.
    """
    var dot_product = Float32(0)
    var norm_a_sq = Float32(0)
    var norm_b_sq = Float32(0)
    var i = 0
    
    # SIMD path: compute all three values in parallel
    while i + 8 <= dim:
        var vec_a = (a + i).load[width=8]()
        var vec_b = (b + i).load[width=8]()
        
        dot_product += (vec_a * vec_b).reduce_add()
        norm_a_sq += (vec_a * vec_a).reduce_add()
        norm_b_sq += (vec_b * vec_b).reduce_add()
        i += 8
    
    # Handle remaining elements  
    while i < dim:
        dot_product += a[i] * b[i]
        norm_a_sq += a[i] * a[i]
        norm_b_sq += b[i] * b[i]
        i += 1
    
    var norm_a = sqrt(norm_a_sq + 1e-12)
    var norm_b = sqrt(norm_b_sq + 1e-12)
    var similarity = dot_product / (norm_a * norm_b)
    
    return 1.0 - similarity

@always_inline
fn simd_dot_product(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    """Calculate dot product using SIMD instructions."""
    var sum = Float32(0)
    var i = 0
    
    # SIMD path
    while i + 8 <= dim:
        var vec_a = (a + i).load[width=8]()
        var vec_b = (b + i).load[width=8]()
        sum += (vec_a * vec_b).reduce_add()
        i += 8
    
    # Scalar remainder
    while i < dim:
        sum += a[i] * b[i]
        i += 1
    
    return sum

@always_inline
fn simd_vector_norm_squared(vec: UnsafePointer[Float32], dim: Int) -> Float32:
    """Calculate squared vector norm using SIMD instructions."""
    var sum = Float32(0)
    var i = 0
    
    # SIMD path
    while i + 8 <= dim:
        var v = (vec + i).load[width=8]()
        sum += (v * v).reduce_add()
        i += 8
    
    # Scalar remainder
    while i < dim:
        sum += vec[i] * vec[i]
        i += 1
    
    return sum