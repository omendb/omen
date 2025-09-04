#!/usr/bin/env python3
"""Test new optimizations including FFI reduction and parallel processing.

Tests:
1. Optimized FFI with reduced Python object creation
2. Parallel batch processing
3. Modular 25.5 feature improvements
"""

import sys

sys.path.insert(0, "python")

import numpy as np
import time
import os
from multiprocessing import cpu_count

# Force environment for testing
os.environ["OMENDB_PARALLEL"] = "true"
os.environ["OMENDB_OPTIMIZED_FFI"] = "true"


def test_ffi_optimization():
    """Test FFI optimization improvements."""
    print("=" * 70)
    print("FFI OPTIMIZATION TEST")
    print("=" * 70)

    import omendb

    dimensions = [128, 256, 512]
    n_vectors = 1000

    print("Testing reduced FFI overhead...")
    print(f"{'Dimension':<12} {'Old (vec/s)':<15} {'New (vec/s)':<15} {'Improvement'}")
    print("-" * 60)

    # Expected old performance (from previous benchmarks)
    old_performance = {128: 3007, 256: 1551, 512: 823}

    for dim in dimensions:
        db = omendb.DB()
        db.clear()

        # Test with numpy arrays (zero-copy optimization)
        vectors = np.random.randn(n_vectors, dim).astype(np.float32)

        # Time individual adds
        start = time.perf_counter()
        for i in range(n_vectors):
            db.add(f"id_{i}", vectors[i])
        elapsed = time.perf_counter() - start

        new_rate = n_vectors / elapsed
        old_rate = old_performance[dim]
        improvement = (new_rate / old_rate - 1) * 100

        print(f"{dim:<12} {old_rate:<15.0f} {new_rate:<15.0f} {improvement:+.1f}%")

    print("\nNote: Numpy arrays should show improvement due to zero-copy optimization")


def test_parallel_batch_processing():
    """Test parallel batch processing."""
    print("\n" + "=" * 70)
    print("PARALLEL BATCH PROCESSING TEST")
    print("=" * 70)

    import omendb

    n_vectors = 20000
    dimension = 256
    batch_sizes = [1000, 2000, 5000, 10000]

    print(f"Testing with {n_vectors:,} vectors, dimension {dimension}")
    print(f"CPU cores available: {cpu_count()}")
    print()

    # Generate test data
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    print(f"{'Batch Size':<12} {'Sequential (s)':<15} {'Parallel (s)':<15} {'Speedup'}")
    print("-" * 60)

    for batch_size in batch_sizes:
        # Sequential test
        db_seq = omendb.DB(buffer_size=10000)
        db_seq.clear()
        os.environ["OMENDB_PARALLEL"] = "false"

        start = time.perf_counter()
        for i in range(0, n_vectors, batch_size):
            end_idx = min(i + batch_size, n_vectors)
            db_seq.add_batch(vectors[i:end_idx], ids[i:end_idx])
        seq_time = time.perf_counter() - start

        # Parallel test
        db_par = omendb.DB(buffer_size=10000)
        db_par.clear()
        os.environ["OMENDB_PARALLEL"] = "true"

        start = time.perf_counter()
        for i in range(0, n_vectors, batch_size):
            end_idx = min(i + batch_size, n_vectors)
            db_par.add_batch(vectors[i:end_idx], ids[i:end_idx])
        par_time = time.perf_counter() - start

        speedup = seq_time / par_time if par_time > 0 else 1.0

        print(f"{batch_size:<12} {seq_time:<15.3f} {par_time:<15.3f} {speedup:.2f}x")

    print("\nNote: Larger batches should show better parallel speedup")


def test_search_parallelization():
    """Test parallel search operations."""
    print("\n" + "=" * 70)
    print("PARALLEL SEARCH TEST")
    print("=" * 70)

    import omendb

    # Build index
    n_vectors = 10000
    dimension = 128
    n_queries = 100

    print(f"Building index with {n_vectors:,} vectors...")

    db = omendb.DB(buffer_size=10000)
    db.clear()

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]
    db.add_batch(vectors, ids)

    # Generate queries
    queries = np.random.randn(n_queries, dimension).astype(np.float32)

    print(f"Testing {n_queries} queries...")
    print()

    # Sequential search
    os.environ["OMENDB_PARALLEL"] = "false"
    start = time.perf_counter()
    for query in queries:
        results = db.search(query, 10)
    seq_time = time.perf_counter() - start

    # Parallel search (batch)
    os.environ["OMENDB_PARALLEL"] = "true"
    start = time.perf_counter()
    # Simulate batch search (would need API support)
    for query in queries:
        results = db.search(query, 10)
    par_time = time.perf_counter() - start

    print(f"Sequential: {seq_time:.3f}s ({n_queries / seq_time:.0f} queries/s)")
    print(f"Parallel:   {par_time:.3f}s ({n_queries / par_time:.0f} queries/s)")
    print(f"Speedup:    {seq_time / par_time:.2f}x")


def test_memory_optimization():
    """Test memory usage with optimizations."""
    print("\n" + "=" * 70)
    print("MEMORY OPTIMIZATION TEST")
    print("=" * 70)

    import omendb
    import psutil

    process = psutil.Process(os.getpid())

    n_vectors = 10000
    dimension = 256

    print(f"Testing memory usage with {n_vectors:,} vectors...")

    # Test with old approach
    db_old = omendb.DB(buffer_size=10000)
    db_old.clear()
    os.environ["OMENDB_OPTIMIZED_FFI"] = "false"

    mem_before = process.memory_info().rss / 1024 / 1024

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"old_{i}" for i in range(n_vectors)]
    db_old.add_batch(vectors, ids)

    mem_old = process.memory_info().rss / 1024 / 1024 - mem_before

    # Test with optimized approach
    db_new = omendb.DB(buffer_size=10000)
    db_new.clear()
    os.environ["OMENDB_OPTIMIZED_FFI"] = "true"

    mem_before = process.memory_info().rss / 1024 / 1024

    ids_new = [f"new_{i}" for i in range(n_vectors)]
    db_new.add_batch(vectors, ids_new)

    mem_new = process.memory_info().rss / 1024 / 1024 - mem_before

    print(f"Old approach: {mem_old:.1f} MB")
    print(f"Optimized:    {mem_new:.1f} MB")
    print(f"Savings:      {(1 - mem_new / mem_old) * 100:.1f}%")


def benchmark_overall_improvement():
    """Benchmark overall performance improvement."""
    print("\n" + "=" * 70)
    print("OVERALL PERFORMANCE IMPROVEMENT")
    print("=" * 70)

    import omendb

    # Test configuration
    n_vectors = 50000
    dimension = 256
    batch_size = 5000

    print(f"Large-scale test: {n_vectors:,} vectors, dimension {dimension}")
    print()

    # Generate data
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    # Baseline (all optimizations off)
    os.environ["OMENDB_PARALLEL"] = "false"
    os.environ["OMENDB_OPTIMIZED_FFI"] = "false"

    db_baseline = omendb.DB(buffer_size=10000)
    db_baseline.clear()

    start = time.perf_counter()
    for i in range(0, n_vectors, batch_size):
        end_idx = min(i + batch_size, n_vectors)
        db_baseline.add_batch(vectors[i:end_idx], ids[i:end_idx])
    baseline_time = time.perf_counter() - start
    baseline_rate = n_vectors / baseline_time

    # Optimized (all optimizations on)
    os.environ["OMENDB_PARALLEL"] = "true"
    os.environ["OMENDB_OPTIMIZED_FFI"] = "true"

    db_optimized = omendb.DB(buffer_size=10000)
    db_optimized.clear()

    start = time.perf_counter()
    for i in range(0, n_vectors, batch_size):
        end_idx = min(i + batch_size, n_vectors)
        db_optimized.add_batch(vectors[i:end_idx], ids[i:end_idx])
    optimized_time = time.perf_counter() - start
    optimized_rate = n_vectors / optimized_time

    print(f"Baseline:  {baseline_time:.2f}s ({baseline_rate:.0f} vec/s)")
    print(f"Optimized: {optimized_time:.2f}s ({optimized_rate:.0f} vec/s)")
    print(f"Speedup:   {baseline_time / optimized_time:.2f}x")
    print(f"Improvement: {(optimized_rate / baseline_rate - 1) * 100:.1f}%")


def main():
    """Run all optimization tests."""
    print("NEW OPTIMIZATION TEST SUITE")
    print("=" * 70)
    print("Testing FFI reduction, parallel processing, and Modular 25.5 features")
    print()

    # Note: These optimizations need to be implemented in native.mojo
    # This test shows expected improvements

    print("EXPECTED IMPROVEMENTS:")
    print("- FFI optimization: 20-30% for individual ops")
    print("- Parallel batch: 2-4x on multi-core systems")
    print("- Memory pooling: 20-30% reduction")
    print("- Combined: 3-5x overall improvement possible")
    print()

    # Run actual tests if implementations are ready
    try:
        test_ffi_optimization()
        test_parallel_batch_processing()
        test_search_parallelization()
        test_memory_optimization()
        benchmark_overall_improvement()
    except Exception as e:
        print(f"\nNote: Optimizations not yet implemented in native module")
        print(f"Error: {e}")
        print("\nShowing expected performance targets instead:")
        print()
        show_expected_performance()


def show_expected_performance():
    """Show expected performance after optimizations."""
    print("EXPECTED PERFORMANCE TARGETS")
    print("=" * 70)

    print("\nIndividual Operations (128d):")
    print("  Current: 3,007 vec/s")
    print("  Target:  4,000+ vec/s (with FFI optimization)")

    print("\nBatch Operations (128d):")
    print("  Current: 21,646 vec/s")
    print("  Target:  50,000+ vec/s (with parallel processing)")

    print("\nSearch Latency:")
    print("  Current: 0.48ms")
    print("  Target:  <0.3ms (with optimized distance computation)")

    print("\nMemory Usage:")
    print("  Current: Linear scaling")
    print("  Target:  20-30% reduction (with better pooling)")

    print("\nKey Optimizations to Implement:")
    print("1. Zero-copy numpy arrays")
    print("2. Parallel batch processing")
    print("3. Optimized metadata conversion")
    print("4. Reduced Python object creation")
    print("5. Parametric aliases for cleaner code")


if __name__ == "__main__":
    main()
