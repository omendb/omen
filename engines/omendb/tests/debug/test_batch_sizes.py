#!/usr/bin/env python3
"""Test different batch sizes to find where the fix breaks down."""

import sys
import numpy as np

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 TESTING: Batch Size Breakpoint")
print("=" * 40)


def test_increasing_batch_sizes():
    """Test batch sizes from small to large to find where it breaks."""

    batch_sizes = [2, 5, 10, 25, 50, 100, 250, 500]
    buffer_size = 100  # Fixed buffer size

    print(f"Using buffer_size={buffer_size}")
    print("Testing batch sizes to find breakpoint:\n")

    native.configure_database({"buffer_size": buffer_size})

    for batch_size in batch_sizes:
        native.clear_database()

        vectors_np = np.random.rand(batch_size, 128).astype(np.float32)
        ids = [f"test_{batch_size}_{i}" for i in range(batch_size)]
        metadata = [{}] * batch_size

        print(f"Batch size {batch_size:3}:")
        print("   🎯 Watch for flush messages:")

        # Use the fixed batch method
        native.add_vector_batch(ids, vectors_np, metadata)

        stats = native.get_stats()
        buffer_count = stats["buffer_size"]
        main_count = stats["main_index_size"]
        total = buffer_count + main_count

        # Expected behavior
        if batch_size <= buffer_size:
            expected_buffer = batch_size
            expected_main = 0
        else:
            expected_main = (batch_size // buffer_size) * buffer_size
            expected_buffer = batch_size % buffer_size

        # Check results
        working = (main_count > 0) if batch_size > buffer_size else (main_count == 0)
        status = "✅ Working" if working else "❌ Broken"

        print(
            f"   Result: buffer={buffer_count}, main={main_count}, total={total} {status}"
        )

        if not working:
            print(
                f"   Expected: buffer≤{buffer_size}, main>0 for batch_size>{buffer_size}"
            )
            if batch_size > buffer_size:
                break  # Found the breakpoint


def test_numpy_vs_lists():
    """Test if NumPy vs Python lists affects behavior."""

    print(f"\n🔬 NumPy vs Lists Comparison")
    print("-" * 30)

    batch_size = 150
    buffer_size = 100

    native.configure_database({"buffer_size": buffer_size})

    # Test 1: NumPy arrays
    print("Test 1: NumPy arrays")
    native.clear_database()
    vectors_np = np.random.rand(batch_size, 128).astype(np.float32)
    ids = [f"numpy_{i}" for i in range(batch_size)]
    metadata = [{}] * batch_size

    native.add_vector_batch(ids, vectors_np, metadata)
    stats = native.get_stats()
    print(
        f"   NumPy result: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )

    # Test 2: Python lists
    print("\nTest 2: Python lists")
    native.clear_database()
    vectors_list = vectors_np.tolist()
    ids = [f"list_{i}" for i in range(batch_size)]

    native.add_vector_batch(ids, vectors_list, metadata)
    stats = native.get_stats()
    print(
        f"   List result: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )


# Run tests
print("🧪 Running batch size breakpoint analysis...")

test_increasing_batch_sizes()
test_numpy_vs_lists()

print("\n" + "=" * 40)
print("🎯 BATCH SIZE ANALYSIS")
print("=" * 40)

print("\n💡 Looking for:")
print("1. At what batch size does flushing stop working?")
print("2. Are there differences between NumPy arrays and lists?")
print("3. Is there a pattern to when flushes are triggered?")

print("\n✅ Batch size analysis complete!")
