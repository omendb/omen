# OmenDB Code Style Guide

**Last Updated**: July 25, 2025

## Code Comment Guidelines

### ✅ Good Comment Patterns (Keep These)

#### Module Documentation
```mojo
"""
OmenDB native module with RoarGraph algorithm and SIMD optimizations.
"""
```
- Clear, concise module purpose
- Highlight key features

#### Function Documentation
```mojo
fn euclidean_distance(
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    """Compute Euclidean (L2) distance between two vectors.
    
    Hardware-optimized SIMD implementation with adaptive unrolling.
    
    Args:
        a: First vector data pointer
        b: Second vector data pointer  
        dim: Vector dimension
        
    Returns:
        Euclidean distance as Float32
    """
```
- Clear purpose statement
- Implementation notes when relevant
- Complete parameter documentation
- Return value description

#### Performance Documentation
```mojo
# DISABLED: Prefetch optimization shows 18.1% performance variance.
# Keeping function for potential future re-enable with better tuning.
```
- Document why optimizations are disabled
- Include performance measurements
- Explain trade-offs

#### Algorithm Explanations
```mojo
# LAZY INITIALIZATION - only create indexes when needed
# This avoids expensive RoarGraph construction for small datasets
```
- Explain "why" not "what"
- Document design decisions
- Note performance implications

### ❌ Comment Anti-Patterns (Avoid These)

#### Redundant Comments
```mojo
# Bad
i += 1  # Increment i

# Good
i += 1  # Process remaining elements after SIMD chunks
```

#### Obvious Comments
```mojo
# Bad
self.dimension = 0  # Initialize dimension to 0

# Good - no comment needed for obvious initialization
self.dimension = 0
```

#### Commented-Out Code Without Context
```mojo
# Bad
# result = old_algorithm(data)

# Good
# DEPRECATED: old_algorithm caused O(n²) scaling
# Replaced with new_algorithm in v0.1.0
# result = old_algorithm(data)
```

### 📋 When to Add Comments

1. **Complex algorithms**: Explain the approach and why it was chosen
2. **Performance decisions**: Document benchmarks and trade-offs  
3. **Non-obvious behavior**: Clarify edge cases or special handling
4. **TODO items**: Include context and priority
5. **Hardware-specific code**: Explain platform dependencies
6. **Temporary workarounds**: Document why and when to remove

### 📋 When NOT to Add Comments

1. **Self-documenting code**: Clear variable/function names
2. **Standard patterns**: Common idioms don't need explanation
3. **Type declarations**: Types are self-documenting
4. **Simple getters/setters**: Unless they have side effects

### 🎯 Comment Quality Checklist

- [ ] Does the comment explain "why" not "what"?
- [ ] Would a new developer understand the context?
- [ ] Is the comment still accurate after code changes?
- [ ] Does it add value beyond what the code shows?
- [ ] Is it useful for both humans and AI agents?

### 📝 Examples of Good Context Comments

```mojo
# Algorithm selection thresholds based on empirical testing
# BruteForce is faster for <10K vectors due to cache efficiency
alias BRUTE_FORCE_THRESHOLD = 10000

# Use middle of padding for better cache alignment
# Testing showed 15% better performance vs start alignment
var offset = cache_line_floats - (cache_line_floats // 2)

# Convert similarity to distance in range [0, 2]
# Standard transformation used by Faiss and Pinecone
return 1.0 - cosine_sim
```

### 🚫 Over-Commenting Example (Avoid)

```mojo
# BAD: Too many obvious comments
fn add_vectors(a: Float32, b: Float32) -> Float32:
    """Add two floating point numbers."""  # Redundant with function name
    # Store the result of addition
    var result = a + b  # Add a and b
    # Return the result
    return result  # Return statement
```

### ✅ Balanced Commenting Example

```mojo
fn add_vectors(a: List[Float32], b: List[Float32]) -> List[Float32]:
    """Element-wise vector addition with SIMD optimization."""
    # Process in SIMD-width chunks for 4x speedup
    alias width = simdwidthof[DType.float32]()
    
    # Handles dimension mismatch per NumPy broadcasting rules
    if len(a) != len(b):
        raise Error("Dimension mismatch")
    
    # Remaining implementation...
```

## Summary

Good comments provide context and explain decisions, not mechanics. They're equally useful for human developers and AI agents understanding the codebase. Focus on the "why" behind the code, not the "what" that's already visible.