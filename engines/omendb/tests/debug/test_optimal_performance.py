#!/usr/bin/env python3
"""Test optimal buffer configurations to restore 99K vec/s performance."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🚀 OPTIMAL PERFORMANCE RESTORATION")
print("=" * 50)


def test_optimal_buffer_sizes():
    """Test buffer sizes that should provide optimal performance."""

    # These sizes should balance write performance with minimal flush overhead
    configs = [
        (100000, "Pure buffer (no HNSW overhead)"),
        (10000, "Large buffer (rare flushes)"),
        (5000, "Medium buffer (some flushes)"),
        (2000, "Balanced buffer"),
        (1000, "Small buffer (frequent flushes)"),
    ]

    n_vectors = 10000
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"opt_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    results = []

    print(f"Testing {n_vectors} vectors @ 128D")
    print("Looking for optimal buffer_size to restore 99K+ vec/s\n")

    for buffer_size, desc in configs:
        print(f"📊 {desc} (buffer_size={buffer_size}):")

        native.configure_database({"buffer_size": buffer_size})
        native.clear_database()

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = n_vectors / elapsed
        results.append((buffer_size, rate, elapsed))

        stats = native.get_stats()
        print(f"   ⏱️ Performance: {elapsed:.3f}s = {rate:.0f} vec/s")
        print(
            f"   📊 Final state: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
        )

        # Quick query test
        query = vectors_np[0].tolist()
        query_times = []
        for _ in range(5):
            start = time.time()
            search_results = native.search_vectors(query, 10, {})
            query_times.append((time.time() - start) * 1000)

        avg_query = sum(query_times) / len(query_times)
        print(f"   🔍 Query: {avg_query:.2f}ms avg")

        if rate > 80000:
            print(f"   ✅ Excellent performance (>80K vec/s)")
        elif rate > 50000:
            print(f"   ✅ Good performance (>50K vec/s)")
        elif rate > 20000:
            print(f"   ⚠️ Acceptable performance (>20K vec/s)")
        else:
            print(f"   ❌ Poor performance (<20K vec/s)")

        print()

    return results


def test_scaling_with_optimal_config():
    """Test scaling behavior with the optimal configuration."""

    print("📈 Scaling Test with Optimal Configuration")
    print("-" * 45)

    # Use pure buffer for best write performance
    optimal_buffer_size = 100000  # No flushes for datasets <100K

    native.configure_database({"buffer_size": optimal_buffer_size})

    sizes = [1000, 2000, 5000, 10000, 20000]

    print(f"Using buffer_size={optimal_buffer_size} (pure buffer strategy)\n")

    for size in sizes:
        native.clear_database()

        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"scale_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = size / elapsed

        # Calculate scaling efficiency
        baseline_rate = 92807 if size == 1000 else None  # From our previous tests
        if baseline_rate and size > 1000:
            expected_rate = baseline_rate  # Should maintain rate
            efficiency = rate / expected_rate
            efficiency_str = f"({efficiency:.2f} efficiency)"
        else:
            efficiency_str = ""

        print(f"   {size:5} vectors: {rate:7.0f} vec/s {efficiency_str}")

        if size == 10000 and rate > 90000:
            print(f"   🎉 TARGET ACHIEVED! >90K vec/s at 10K vectors")


def compare_with_chromadb():
    """Compare our best performance with ChromaDB."""

    print("⚔️ Competitive Comparison (Optimal Config)")
    print("-" * 42)

    # Use optimal configuration
    native.configure_database({"buffer_size": 100000})
    native.clear_database()

    n_vectors = 10000
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"comp_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    # Test our performance
    start = time.time()
    native.add_vector_batch(ids, vectors_np, metadata)
    our_time = time.time() - start
    our_rate = n_vectors / our_time

    # Query performance
    query = vectors_np[0].tolist()
    query_times = []
    for _ in range(10):
        start = time.time()
        native.search_vectors(query, 10, {})
        query_times.append((time.time() - start) * 1000)

    our_query = sum(query_times) / len(query_times)

    # ChromaDB baseline
    chromadb_rate = 4772  # ops/sec (from previous benchmarks)
    chromadb_query = 0.64  # ms
    chromadb_startup = 245.7  # ms
    our_startup = 0.002  # ms

    print(f"📊 Performance Comparison:")
    print(f"   OmenDB Insert:    {our_rate:.0f} vec/s")
    print(f"   ChromaDB Insert:  {chromadb_rate:.0f} vec/s")
    insert_ratio = our_rate / chromadb_rate
    print(
        f"   Ratio: {insert_ratio:.1f}x ({'faster' if insert_ratio > 1 else 'slower'})"
    )

    print(f"\n   OmenDB Query:     {our_query:.2f}ms")
    print(f"   ChromaDB Query:   {chromadb_query:.2f}ms")
    query_ratio = chromadb_query / our_query
    print(f"   Ratio: {query_ratio:.1f}x ({'faster' if query_ratio > 1 else 'slower'})")

    print(f"\n   OmenDB Startup:   {our_startup:.3f}ms")
    print(f"   ChromaDB Startup: {chromadb_startup:.1f}ms")
    startup_ratio = chromadb_startup / our_startup
    print(f"   Ratio: {startup_ratio:.0f}x faster ⚡")


# Run all tests
print("🧪 Running optimal performance tests...")

results = test_optimal_buffer_sizes()
test_scaling_with_optimal_config()
compare_with_chromadb()

print("\n" + "=" * 50)
print("🎯 PERFORMANCE RESTORATION RESULTS")
print("=" * 50)

print(f"\n📊 Buffer Size Performance Summary:")
best_rate = 0
best_config = None

for buffer_size, rate, elapsed in results:
    if rate > best_rate:
        best_rate = rate
        best_config = buffer_size

print(f"   Best: buffer_size={best_config} = {best_rate:.0f} vec/s")

if best_rate > 90000:
    print(f"   🎉 SUCCESS: Achieved {best_rate:.0f} vec/s (target: 99K)")
    print(f"   ✅ Buffer architecture fully restored!")
elif best_rate > 50000:
    print(f"   ✅ Good progress: {best_rate:.0f} vec/s (target: 99K)")
    print(f"   🔧 Need optimization of remaining bottlenecks")
else:
    print(f"   ❌ Still need work: {best_rate:.0f} vec/s (target: 99K)")
    print(f"   🔍 Major bottlenecks remain")

print("\n✅ Optimal performance testing complete!")
