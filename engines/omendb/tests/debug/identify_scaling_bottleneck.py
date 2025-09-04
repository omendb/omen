#!/usr/bin/env python3
"""Identify the specific component causing scaling degradation."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 SCALING BOTTLENECK ANALYSIS")
print("=" * 50)


def test_linear_scaling():
    """Test if our scaling issue is linear or quadratic."""
    print("\n📈 Testing Scaling Pattern")
    print("-" * 30)

    sizes = [500, 1000, 2000, 4000, 8000]
    rates = []

    native.configure_database({"buffer_size": 100000})  # Large buffer

    for size in sizes:
        native.clear_database()

        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"scale_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = size / elapsed
        rates.append(rate)

        print(f"{size:5} vectors: {rate:7.0f} vec/s ({elapsed:.3f}s)")

    # Analyze scaling pattern
    print(f"\n📊 Scaling Analysis:")
    for i in range(1, len(sizes)):
        size_ratio = sizes[i] / sizes[i - 1]
        rate_ratio = rates[i] / rates[i - 1]
        efficiency = rate_ratio / size_ratio

        print(f"   {sizes[i - 1]:4} → {sizes[i]:4}: {efficiency:.2f} efficiency")
        if efficiency < 0.8:
            print(f"      ⚠️ Significant scaling degradation!")


def test_memory_patterns():
    """Test different memory allocation patterns."""
    print("\n🧠 Testing Memory Allocation Patterns")
    print("-" * 40)

    size = 5000
    native.configure_database({"buffer_size": 100000})

    # Test 1: Fresh arrays each time
    print("Test 1: Fresh NumPy arrays")
    times = []
    for run in range(3):
        native.clear_database()
        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"fresh_{run}_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start
        times.append(elapsed)

    avg_fresh = sum(times) / len(times)
    rate_fresh = size / avg_fresh
    print(f"   Fresh arrays: {rate_fresh:.0f} vec/s (avg)")

    # Test 2: Reused arrays
    print("\nTest 2: Reused NumPy arrays")
    vectors_np = np.random.rand(size, 128).astype(np.float32, order="C")
    base_metadata = [{}] * size

    times = []
    for run in range(3):
        native.clear_database()
        ids = [f"reused_{run}_{i}" for i in range(size)]
        metadata = base_metadata.copy()  # Shallow copy

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start
        times.append(elapsed)

    avg_reused = sum(times) / len(times)
    rate_reused = size / avg_reused
    print(f"   Reused arrays: {rate_reused:.0f} vec/s (avg)")

    improvement = rate_reused / rate_fresh
    print(f"   📊 Reuse improvement: {improvement:.2f}x")


def test_buffer_vs_hnsw_scaling():
    """Test scaling in buffer vs HNSW to isolate the bottleneck."""
    print("\n⚖️ Buffer vs HNSW Scaling Comparison")
    print("-" * 40)

    sizes = [1000, 2500, 5000, 10000]

    print("Pure Buffer (brute force) scaling:")
    buffer_rates = []
    for size in sizes:
        native.configure_database({"buffer_size": 100000})  # Always buffer
        native.clear_database()

        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"buf_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = size / elapsed
        buffer_rates.append(rate)
        stats = native.get_stats()

        print(
            f"   {size:5}: {rate:6.0f} vec/s (buf={stats['buffer_size']}, main={stats['main_index_size']})"
        )

    print(f"\nDirect HNSW (minimal buffer) scaling:")
    hnsw_rates = []
    for size in sizes:
        native.configure_database({"buffer_size": 1})  # Force HNSW
        native.clear_database()

        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"hnsw_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = size / elapsed
        hnsw_rates.append(rate)
        stats = native.get_stats()

        print(
            f"   {size:5}: {rate:6.0f} vec/s (buf={stats['buffer_size']}, main={stats['main_index_size']})"
        )

    # Compare scaling patterns
    print(f"\n📊 Scaling Pattern Analysis:")
    print(f"   Size    Buffer    HNSW   Buffer/HNSW")
    for i, size in enumerate(sizes):
        ratio = buffer_rates[i] / hnsw_rates[i] if hnsw_rates[i] > 0 else 0
        print(
            f"   {size:5}:  {buffer_rates[i]:6.0f}  {hnsw_rates[i]:6.0f}     {ratio:.2f}x"
        )

    return buffer_rates, hnsw_rates


def test_batch_size_impact():
    """Test if batch size itself affects performance."""
    print("\n📦 Batch Size Impact Analysis")
    print("-" * 30)

    total_vectors = 10000
    batch_sizes = [1000, 2500, 5000, 10000]

    native.configure_database({"buffer_size": 100000})

    for batch_size in batch_sizes:
        native.clear_database()

        num_batches = total_vectors // batch_size
        total_time = 0

        print(f"Batch size {batch_size} ({num_batches} batches):")

        for batch_idx in range(num_batches):
            start_idx = batch_idx * batch_size
            end_idx = start_idx + batch_size

            vectors_np = np.random.rand(batch_size, 128).astype(np.float32)
            ids = [f"batch_{batch_idx}_{i}" for i in range(batch_size)]
            metadata = [{}] * batch_size

            start = time.time()
            native.add_vector_batch(ids, vectors_np, metadata)
            elapsed = time.time() - start
            total_time += elapsed

        total_rate = total_vectors / total_time
        per_batch_rate = batch_size / (total_time / num_batches)

        print(
            f"   Overall: {total_rate:.0f} vec/s, Per batch: {per_batch_rate:.0f} vec/s"
        )


# Run all scaling tests
print("🧪 Running scaling bottleneck analysis...")

test_linear_scaling()
test_memory_patterns()
buffer_rates, hnsw_rates = test_buffer_vs_hnsw_scaling()
test_batch_size_impact()

print("\n" + "=" * 50)
print("🎯 SCALING BOTTLENECK CONCLUSIONS")
print("=" * 50)

print("\n💡 Key Findings:")
print("1. Check scaling efficiency ratios above")
print("2. Compare buffer vs HNSW scaling patterns")
print("3. Memory allocation impact on performance")
print("4. Batch size vs performance relationship")

print(f"\n🔧 Recommendations:")
print("- If buffer scales better: Issue is in HNSW implementation")
print("- If memory reuse helps: Issue is allocation overhead")
print("- If smaller batches help: Issue is batch processing")
print("- If scaling efficiency < 0.8: Algorithm complexity issue")

print("\n✅ Scaling analysis complete!")
