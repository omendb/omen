#!/usr/bin/env python3
"""
Systematic diagnosis of binary quantization performance.
Identify why we're only getting 1.1x speedup instead of 40x.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def diagnose_binary_quantization():
    """Systematically diagnose binary quantization performance."""

    print("🔬 BINARY QUANTIZATION DIAGNOSTIC")
    print("=" * 50)

    # Small test to isolate the issue
    n_vectors = 1000
    dimension = 128
    np.random.seed(42)

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    # Build index
    native.clear_database()
    ids = [f"test_{i:06d}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    print(f"\n📊 Building index with {n_vectors} vectors...")
    native.add_vector_batch(ids, vectors, metadata)

    # Test 1: Verify binary quantization is enabled
    print(f"\n🔍 TEST 1: Binary Quantization Status")
    print("-" * 30)

    # Enable binary quantization explicitly
    try:
        result = native.enable_binary_quantization()
        print(f"Binary quantization enable result: {result}")
    except Exception as e:
        print(f"Enable error: {e}")

    # Test 2: Individual distance computation timing
    print(f"\n🔍 TEST 2: Direct Distance Computation Speed")
    print("-" * 40)

    query = np.random.randn(dimension).astype(np.float32)

    # Run a single search and time it component by component
    search_times = []
    for trial in range(50):  # More trials for precise measurement
        start_time = time.time()
        results = native.search_vectors(query.tolist(), 10, {})
        search_time = (time.time() - start_time) * 1000  # ms
        search_times.append(search_time)

    avg_search_time = np.mean(search_times)
    std_search_time = np.std(search_times)

    print(f"Search time: {avg_search_time:.3f} ± {std_search_time:.3f} ms")
    print(f"Distance computations per search: ~{n_vectors} (worst case)")
    print(f"Time per distance: {avg_search_time * 1000 / n_vectors:.2f} µs")

    # Test 3: Scale analysis - check if binary quantization scales differently
    print(f"\n🔍 TEST 3: Scaling Analysis")
    print("-" * 30)

    scales = [100, 500, 1000, 2000]
    scale_results = []

    for scale in scales:
        if scale > n_vectors:
            continue

        # Test search performance at different scales
        scale_times = []
        for _ in range(10):
            start_time = time.time()
            results = native.search_vectors(query.tolist(), min(10, scale), {})
            scale_time = (time.time() - start_time) * 1000
            scale_times.append(scale_time)

        avg_time = np.mean(scale_times)
        per_distance = avg_time * 1000 / scale  # µs per distance
        scale_results.append((scale, avg_time, per_distance))

        print(f"  {scale:4d} vectors: {avg_time:.2f}ms ({per_distance:.2f} µs/dist)")

    # Test 4: Compare with theoretical expectations
    print(f"\n🔍 TEST 4: Performance Analysis")
    print("-" * 30)

    # Expected performance
    expected_without_bq = 1000  # µs per distance (Float32 computation)
    expected_with_bq = 25      # µs per distance (40x speedup)

    actual_per_distance = avg_search_time * 1000 / n_vectors

    print(f"Expected without BQ: {expected_without_bq} µs/distance")
    print(f"Expected with BQ: {expected_with_bq} µs/distance")
    print(f"Actual measured: {actual_per_distance:.1f} µs/distance")

    if actual_per_distance < 50:
        print("✅ FAST: Either BQ is working or SIMD is very effective")
    elif actual_per_distance < 200:
        print("🟡 MODERATE: Some optimization active, but not full BQ speedup")
    elif actual_per_distance < 500:
        print("🟡 SLOW: Limited optimization, probably SIMD only")
    else:
        print("❌ VERY SLOW: No significant optimization active")

    # Test 5: Memory usage analysis
    print(f"\n🔍 TEST 5: Memory Usage Analysis")
    print("-" * 30)

    # Original vectors: n_vectors * dimension * 4 bytes
    original_memory = n_vectors * dimension * 4

    # Binary quantized: n_vectors * (dimension / 8) bytes
    binary_memory = n_vectors * (dimension // 8)

    compression_ratio = original_memory / binary_memory

    print(f"Original memory: {original_memory / 1024:.1f} KB")
    print(f"Binary memory: {binary_memory / 1024:.1f} KB")
    print(f"Compression ratio: {compression_ratio:.1f}x")

    if compression_ratio > 25:
        print("✅ Binary quantization memory savings achieved")
    else:
        print("❌ Binary quantization memory savings not achieved")

    return {
        'avg_search_time_ms': avg_search_time,
        'per_distance_us': actual_per_distance,
        'compression_ratio': compression_ratio,
        'scale_results': scale_results
    }

def test_binary_quantization_components():
    """Test individual components of binary quantization."""

    print(f"\n🔬 BINARY QUANTIZATION COMPONENT TEST")
    print("=" * 50)

    # Create a small controlled test
    dimension = 128
    test_vectors = np.random.randn(2, dimension).astype(np.float32)

    native.clear_database()

    # Add just two vectors
    ids = ["vec_a", "vec_b"]
    metadata = [{}, {}]

    native.add_vector_batch(ids, test_vectors, metadata)

    # Enable binary quantization
    native.enable_binary_quantization()

    # Test search between these vectors
    print(f"\n🔍 Testing binary quantization on controlled vectors...")

    query = test_vectors[0]  # Search for first vector

    # Time multiple searches
    times = []
    for _ in range(100):
        start = time.time()
        results = native.search_vectors(query.tolist(), 2, {})
        times.append((time.time() - start) * 1000000)  # µs

    avg_time_us = np.mean(times)
    print(f"Search time: {avg_time_us:.1f} µs")
    print(f"Per distance: {avg_time_us / 2:.1f} µs")

    # Should find exact match as first result
    if results and len(results) > 0:
        print(f"Top result ID: {results[0]['id']}")
        print(f"✅ Exact match found" if results[0]['id'] == 'vec_a' else "❌ Exact match not found")

    return avg_time_us / 2  # µs per distance

if __name__ == "__main__":
    try:
        print("🚀 BINARY QUANTIZATION PERFORMANCE DIAGNOSIS")
        print("=" * 60)

        # Full diagnosis
        results = diagnose_binary_quantization()

        # Component test
        per_distance_us = test_binary_quantization_components()

        print(f"\n📊 DIAGNOSIS SUMMARY:")
        print("=" * 30)
        print(f"Search time: {results['avg_search_time_ms']:.2f} ms")
        print(f"Per distance: {results['per_distance_us']:.1f} µs")
        print(f"Compression: {results['compression_ratio']:.1f}x")
        print(f"Component test: {per_distance_us:.1f} µs/distance")

        print(f"\n🎯 ANALYSIS:")
        if results['per_distance_us'] < 50:
            print("✅ Good performance - BQ likely working")
            print("   But may need algorithm-level optimizations for 40x")
        elif results['per_distance_us'] < 200:
            print("🟡 Moderate performance - partial optimization")
            print("   BQ may be working but with overhead")
        else:
            print("❌ Poor performance - BQ not providing expected speedup")
            print("   Need to investigate BQ activation or implementation")

        expected_40x_time = 1000 / 40  # µs
        actual_speedup = 1000 / results['per_distance_us']

        print(f"\nExpected 40x speedup: {expected_40x_time:.1f} µs/distance")
        print(f"Actual speedup: {actual_speedup:.1f}x vs baseline")
        print(f"Gap to target: {40 / actual_speedup:.1f}x improvement needed")

    except Exception as e:
        print(f"💥 Diagnosis failed: {e}")
        import traceback
        traceback.print_exc()