#!/usr/bin/env python3
"""Isolate the exact performance bottleneck after buffer fix."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 ISOLATING PERFORMANCE BOTTLENECK")
print("=" * 50)


def test_native_call_overhead():
    """Test overhead of calling native functions."""

    print("\n🧪 Testing Native Call Overhead")
    print("-" * 35)

    # Test just clearing and stats calls
    iterations = 1000

    start = time.time()
    for _ in range(iterations):
        native.clear_database()
        native.get_stats()
    elapsed = time.time() - start

    print(
        f"   {iterations} clear+stats calls: {elapsed:.3f}s = {iterations / elapsed:.0f} ops/s"
    )
    print(f"   Per-call overhead: {elapsed / iterations * 1000:.3f}ms")


def test_minimal_vector_add():
    """Test minimal vector addition to isolate bottleneck."""

    print("\n🧪 Testing Minimal Vector Addition")
    print("-" * 40)

    native.configure_database({"buffer_size": 100000})  # Pure buffer
    native.clear_database()

    # Test adding single vectors one by one
    print("Single vector adds (bypassing batch processing):")

    test_sizes = [10, 100, 1000]

    for size in test_sizes:
        native.clear_database()

        start = time.time()
        for i in range(size):
            # Generate simple vector
            vector = [1.0] * 128  # Simple vector, no random generation
            native.add_vector(f"test_{i}", vector, {})
        elapsed = time.time() - start

        rate = size / elapsed
        print(f"   {size:4} vectors: {elapsed:.3f}s = {rate:.0f} vec/s")


def test_numpy_vs_lists_overhead():
    """Compare NumPy array vs Python list processing."""

    print("\n🧪 NumPy vs Lists Processing")
    print("-" * 35)

    native.configure_database({"buffer_size": 100000})

    size = 5000

    # Test 1: Pure Python lists
    print(f"Test 1: Python lists ({size} vectors)")
    native.clear_database()

    vectors_list = [[1.0] * 128 for _ in range(size)]
    ids = [f"list_{i}" for i in range(size)]
    metadata = [{}] * size

    start = time.time()
    native.add_vector_batch(ids, vectors_list, metadata)
    elapsed_list = time.time() - start
    rate_list = size / elapsed_list

    print(f"   Python lists: {elapsed_list:.3f}s = {rate_list:.0f} vec/s")

    # Test 2: NumPy arrays
    print(f"\nTest 2: NumPy arrays ({size} vectors)")
    native.clear_database()

    vectors_np = np.ones((size, 128), dtype=np.float32)
    ids = [f"numpy_{i}" for i in range(size)]

    start = time.time()
    native.add_vector_batch(ids, vectors_np, metadata)
    elapsed_np = time.time() - start
    rate_np = size / elapsed_np

    print(f"   NumPy arrays: {elapsed_np:.3f}s = {rate_np:.0f} vec/s")
    print(f"   NumPy advantage: {rate_np / rate_list:.1f}x faster")


def test_vector_generation_overhead():
    """Test if random vector generation is the bottleneck."""

    print("\n🧪 Vector Generation Overhead")
    print("-" * 35)

    size = 10000

    # Test 1: Pre-generate vectors
    print("Pre-generating vectors...")
    start = time.time()
    vectors_np = np.random.rand(size, 128).astype(np.float32)
    gen_time = time.time() - start
    print(f"   Vector generation: {gen_time:.3f}s = {size / gen_time:.0f} vec/s")

    # Test 2: Database processing
    print("\nTesting database processing with pre-generated vectors...")
    native.configure_database({"buffer_size": 100000})
    native.clear_database()

    ids = [f"pre_{i}" for i in range(size)]
    metadata = [{}] * size

    start = time.time()
    native.add_vector_batch(ids, vectors_np, metadata)
    db_time = time.time() - start
    db_rate = size / db_time

    print(f"   Database processing: {db_time:.3f}s = {db_rate:.0f} vec/s")
    print(
        f"   Total time: {gen_time + db_time:.3f}s = {size / (gen_time + db_time):.0f} vec/s"
    )


def test_buffer_vs_previous_performance():
    """Compare current performance with what we expect."""

    print("\n📊 Current vs Expected Performance")
    print("-" * 40)

    native.configure_database({"buffer_size": 100000})
    native.clear_database()

    size = 10000
    vectors_np = np.ones((size, 128), dtype=np.float32)  # Simple vectors
    ids = [f"perf_{i}" for i in range(size)]
    metadata = [{}] * size

    start = time.time()
    native.add_vector_batch(ids, vectors_np, metadata)
    elapsed = time.time() - start
    current_rate = size / elapsed

    expected_rate = 99000  # Our previous performance
    ratio = current_rate / expected_rate

    print(f"   Current performance:  {current_rate:.0f} vec/s")
    print(f"   Expected performance: {expected_rate:.0f} vec/s")
    print(f"   Performance ratio:    {ratio:.3f} ({ratio * 100:.1f}%)")

    if ratio < 0.1:
        print(f"   🚨 SEVERE REGRESSION: >90% performance loss")
    elif ratio < 0.5:
        print(f"   ⚠️ MAJOR REGRESSION: >50% performance loss")
    elif ratio < 0.9:
        print(f"   ⚠️ MINOR REGRESSION: <90% of expected")
    else:
        print(f"   ✅ PERFORMANCE RESTORED")


# Run all isolation tests
print("🧪 Running bottleneck isolation tests...\n")

test_native_call_overhead()
test_minimal_vector_add()
test_numpy_vs_lists_overhead()
test_vector_generation_overhead()
test_buffer_vs_previous_performance()

print("\n" + "=" * 50)
print("🎯 BOTTLENECK ANALYSIS RESULTS")
print("=" * 50)

print("\n💡 This should help identify:")
print("1. Is it native function call overhead?")
print("2. Is it the vector processing itself?")
print("3. Is NumPy vs lists making a difference?")
print("4. Is random generation the bottleneck?")
print("5. How severe is the performance regression?")

print("\n✅ Bottleneck isolation complete!")
