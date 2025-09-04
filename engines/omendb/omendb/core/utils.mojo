"""
Core utility functions and constants for OmenDB algorithms.

This module provides platform detection, mathematical constants, and 
utility functions needed by the research algorithms.
"""

from math import sqrt, log, exp, pi
from math import cos as math_cos, sin as math_sin
from sys import num_logical_cores

# ========================================
# Platform Detection
# ========================================


fn get_system_cores() -> Int:
    """Get the number of logical CPU cores available."""
    return num_logical_cores()


fn get_optimal_workers() -> Int:
    """Get optimal number of worker threads for parallel operations.
    
    Uses hardware-aware logic: leave 1 core for OS, cap at 16 for memory bandwidth.
    Common configs: 8-core dev machines -> 7 workers, 16-core servers -> 15 workers.
    """
    var cores = get_system_cores()
    return min(max(1, cores - 1), 16)


fn get_cache_line_size() -> Int:
    """Get the CPU cache line size in bytes."""
    return 64  # Common cache line size for most modern CPUs


# ========================================
# Mathematical Constants
# ========================================

alias STREAMING_WINDOW_SIZE = 1000
alias PERFORMANCE_MONITORING_INTERVAL = 100
alias AUTO_OPTIMIZATION_THRESHOLD = 0.7
alias DEFAULT_PROJECTION_LAYERS = 4
alias MAX_PROJECTION_DIM = 256
alias MIN_PROJECTION_DIM = 16

# Random number generation constants
alias RANDOM_SEED_MULTIPLIER = 1103515245
alias RANDOM_SEED_INCREMENT = 12345
alias RANDOM_SEED_MODULUS = 2147483648

# ========================================
# Trigonometric Functions
# ========================================


fn cos(x: Float32) -> Float32:
    """Cosine function for Float32."""
    return Float32(math_cos(Float64(x)))


fn sin(x: Float32) -> Float32:
    """Sine function for Float32."""
    return Float32(math_sin(Float64(x)))


# ========================================
# Random Number Generation
# ========================================


struct SimpleRNG:
    """Simple random number generator for algorithm testing."""

    var seed: Int

    fn __init__(out self, seed: Int = 12345):
        self.seed = seed

    fn random_float32(mut self) -> Float32:
        """Generate random float32 in [0, 1]."""
        self.seed = (
            self.seed * RANDOM_SEED_MULTIPLIER + RANDOM_SEED_INCREMENT
        ) % RANDOM_SEED_MODULUS
        return Float32(self.seed) / Float32(RANDOM_SEED_MODULUS)

    fn random_float64(mut self) -> Float64:
        """Generate random float64 in [0, 1]."""
        return Float64(self.random_float32())

    fn gaussian(mut self, mean: Float32 = 0.0, std: Float32 = 1.0) -> Float32:
        """Generate Gaussian random number using Box-Muller transform."""
        # Box-Muller transform
        var u1 = self.random_float32()
        var u2 = self.random_float32()

        # Ensure u1 is not zero to avoid log(0)
        if u1 < 1e-10:
            u1 = 1e-10

        var z0 = sqrt(-2.0 * log(u1)) * cos(2.0 * Float32(pi) * u2)
        return mean + std * z0


fn random_float32() -> Float32:
    """Global random float32 generator using static RNG."""
    var rng = SimpleRNG()
    return rng.random_float32()


fn random_float64() -> Float64:
    """Global random float64 generator using static RNG."""
    var rng = SimpleRNG()
    return rng.random_float64()


fn random_gaussian(mean: Float32 = 0.0, std: Float32 = 1.0) -> Float32:
    """Global Gaussian random number generator using static RNG."""
    var rng = SimpleRNG()
    return rng.gaussian(mean, std)


# ========================================
# Time and Performance Utilities
# ========================================


fn get_time_microseconds() -> Int:
    """Get current time in microseconds.

    This is a placeholder implementation. In production, this would
    use proper high-resolution timing APIs.
    """
    # TODO: Implement proper high-resolution timing
    return 0


fn get_time_seconds() -> Int:
    """Get current time in seconds."""
    return get_time_microseconds() // 1000000


# ========================================
# Memory Utilities
# ========================================


fn calculate_memory_usage(
    vectors: Int, dimension: Int, bytes_per_element: Int = 4
) -> Int:
    """Calculate memory usage for vector storage."""
    return vectors * dimension * bytes_per_element


fn bytes_to_mb(bytes: Int) -> Float32:
    """Convert bytes to megabytes."""
    return Float32(bytes) / (1024.0 * 1024.0)


fn mb_to_bytes(mb: Float32) -> Int:
    """Convert megabytes to bytes."""
    return Int(mb * 1024.0 * 1024.0)


# ========================================
# Algorithm Utilities
# ========================================


fn clamp[T: AnyTrivialRegType](value: T, min_val: T, max_val: T) -> T:
    """Clamp value between min and max."""
    if value < min_val:
        return min_val
    elif value > max_val:
        return max_val
    else:
        return value


fn lerp(a: Float32, b: Float32, t: Float32) -> Float32:
    """Linear interpolation between a and b."""
    return a + t * (b - a)


fn normalize_score(
    score: Float32, min_score: Float32, max_score: Float32
) -> Float32:
    """Normalize score to [0, 1] range."""
    if max_score == min_score:
        return 1.0
    return (score - min_score) / (max_score - min_score)


# ========================================
# String Utilities
# ========================================


fn format_performance_stats(operations: Int, time_ms: Float32) -> String:
    """Format performance statistics as string."""
    var ops_per_sec = Float32(operations) / (time_ms / 1000.0)
    return (
        String(operations)
        + " ops in "
        + String(time_ms)
        + "ms ("
        + String(ops_per_sec)
        + " ops/sec)"
    )


fn format_memory_usage(bytes: Int) -> String:
    """Format memory usage as human-readable string."""
    var mb = bytes_to_mb(bytes)
    if mb >= 1024.0:
        var gb = mb / 1024.0
        return String(gb) + " GB"
    else:
        return String(mb) + " MB"
