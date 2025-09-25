#!/usr/bin/env python3
"""Profile to find the real performance bottleneck."""

import sys
import numpy as np
import time
import cProfile
import pstats
from io import StringIO

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 PROFILING: Finding Our Real Performance Bottleneck")
print("=" * 70)


def profile_function(func, *args, **kwargs):
    """Profile a function call and return stats."""
    pr = cProfile.Profile()
    pr.enable()

    start_time = time.time()
    result = func(*args, **kwargs)
    end_time = time.time()

    pr.disable()

    s = StringIO()
    ps = pstats.Stats(pr, stream=s).sort_stats("tottime")
    ps.print_stats(10)  # Top 10 functions by time

    return result, end_time - start_time, s.getvalue()


def test_native_batch_performance():
    """Baseline: Test native batch performance (our 94K vec/s)."""
    print("\n🚀 Baseline: Native Batch Performance")
    print("-" * 50)

    native.configure_database({"buffer_size": 100000})
    native.clear_database()

    n_vectors = 10000
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"native_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    result, elapsed, profile_output = profile_function(
        native.add_vector_batch, ids, vectors_np, metadata
    )

    rate = n_vectors / elapsed
    print(f"✅ Native batch: {rate:.0f} vec/s ({elapsed:.3f}s)")

    if rate < 80000:
        print("⚠️ BOTTLENECK FOUND in native batch!")
        print("\nProfile (top functions by time):")
        print(profile_output)

    return rate, profile_output


def test_api_wrapper_overhead():
    """Test if API wrapper adds overhead."""
    print("\n🔍 Testing API Wrapper Overhead")
    print("-" * 50)

    # Test high-level API
    import omendb

    db = omendb.DB(buffer_size=100000)

    n_vectors = 5000  # Smaller for profiling
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"api_{i}" for i in range(n_vectors)]

    result, elapsed, profile_output = profile_function(db.add_batch, vectors_np, ids)

    rate = len(result) / elapsed
    print(f"📊 High-level API: {rate:.0f} vec/s ({elapsed:.3f}s)")

    if rate < 40000:
        print("⚠️ BOTTLENECK FOUND in API wrapper!")
        print("\nProfile (top functions by time):")
        print(profile_output)

    return rate, profile_output


def test_memory_allocation_overhead():
    """Test if memory allocation is the bottleneck."""
    print("\n🧠 Testing Memory Allocation Overhead")
    print("-" * 50)

    native.configure_database({"buffer_size": 100000})
    native.clear_database()

    # Pre-allocate everything to eliminate allocation overhead
    n_vectors = 5000
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32, order="C")
    ids = [f"mem_{i}" for i in range(n_vectors)]  # Pre-created
    metadata = [{}] * n_vectors  # Pre-created

    # Make sure arrays are contiguous and optimal
    assert vectors_np.flags["C_CONTIGUOUS"]
    assert vectors_np.dtype == np.float32

    print(
        f"Pre-allocated: {n_vectors} vectors, {len(ids)} IDs, {len(metadata)} metadata"
    )

    # Test with pre-allocated data
    result, elapsed, profile_output = profile_function(
        native.add_vector_batch, ids, vectors_np, metadata
    )

    rate = n_vectors / elapsed
    print(f"💾 Pre-allocated: {rate:.0f} vec/s ({elapsed:.3f}s)")

    return rate, profile_output


def test_different_sizes():
    """Test if dataset size affects performance scaling."""
    print("\n📈 Testing Performance Scaling by Size")
    print("-" * 50)

    sizes = [1000, 2500, 5000, 10000]

    native.configure_database({"buffer_size": 100000})

    for size in sizes:
        native.clear_database()

        vectors_np = np.random.rand(size, 128).astype(np.float32)
        ids = [f"scale_{i}" for i in range(size)]
        metadata = [{}] * size

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = size / elapsed
        rate_per_1k = rate / (size / 1000)  # Normalize to rate per 1K vectors

        print(f"   {size:5} vectors: {rate:6.0f} vec/s ({rate_per_1k:6.0f} per 1K)")

        if size == 10000 and rate < 80000:
            print(f"   ⚠️ Performance degradation at {size} vectors!")


def compare_with_original_performance():
    """Try to identify what changed since our 99K vec/s."""
    print("\n🎯 Attempting to Reproduce Original 99K vec/s")
    print("-" * 50)

    # Try various configurations that might restore original performance
    configs = [
        (1, "Direct HNSW (minimal buffer)"),
        (10, "Tiny buffer"),
        (100, "Small buffer"),
        (1000, "Medium buffer"),
        (100000, "Large buffer (pure brute force)"),
    ]

    n_vectors = 8000  # Similar to previous tests
    vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"orig_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    best_rate = 0
    best_config = None

    for buffer_size, desc in configs:
        native.configure_database({"buffer_size": buffer_size})
        native.clear_database()

        start = time.time()
        native.add_vector_batch(ids, vectors_np, metadata)
        elapsed = time.time() - start

        rate = n_vectors / elapsed

        if rate > best_rate:
            best_rate = rate
            best_config = (buffer_size, desc)

        print(f"   {desc:30}: {rate:6.0f} vec/s")

    print(f"\n🏆 Best configuration: {best_config[1]} = {best_rate:.0f} vec/s")

    if best_rate > 90000:
        print("✅ Found our 99K vec/s! Configuration matters!")
    else:
        print("❌ Still missing performance. Need deeper investigation.")

    return best_rate, best_config


# Run comprehensive profiling
print("🧪 Running comprehensive bottleneck analysis...")

native_rate, native_profile = test_native_batch_performance()
api_rate, api_profile = test_api_wrapper_overhead()
mem_rate, mem_profile = test_memory_allocation_overhead()
test_different_sizes()
best_rate, best_config = compare_with_original_performance()

print("\n" + "=" * 70)
print("🎯 BOTTLENECK ANALYSIS RESULTS")
print("=" * 70)

print(f"\n📊 Performance Comparison:")
print(f"   Native batch:      {native_rate:6.0f} vec/s")
print(f"   API wrapper:       {api_rate:6.0f} vec/s")
print(f"   Pre-allocated:     {mem_rate:6.0f} vec/s")
print(f"   Best config:       {best_rate:6.0f} vec/s")
print(f"   Target (original): 99,000 vec/s")

print(f"\n🔍 Bottleneck Analysis:")

# Identify the biggest performance drop
bottlenecks = [
    ("Native implementation", native_rate, 99000),
    ("API wrapper overhead", api_rate, native_rate),
    ("Memory allocation", mem_rate, native_rate),
]

for name, current, baseline in bottlenecks:
    if baseline > 0:
        drop = ((baseline - current) / baseline) * 100
        if drop > 20:  # >20% drop is significant
            print(f"   ⚠️ {name}: {drop:.1f}% performance loss")
            print(f"      {current:.0f} vs {baseline:.0f} vec/s")

print(f"\n💡 Recommendations:")

if best_rate > 90000:
    print(f"   ✅ Use {best_config[1].lower()} configuration")
    print(f"   ✅ We can achieve {best_rate:.0f} vec/s (near original)")
elif native_rate < 80000:
    print(f"   🔧 Fix native implementation bottleneck")
    print(f"   🔍 Profile native code for memory/algorithm issues")
elif api_rate < native_rate * 0.5:
    print(f"   🔧 Fix API wrapper overhead")
    print(f"   🔍 Optimize Python/Mojo FFI calls")
else:
    print(f"   🔍 Need deeper investigation - bottleneck not identified")

print("\n✅ Bottleneck profiling complete!")
