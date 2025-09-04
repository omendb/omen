#!/usr/bin/env python3
"""Debug the specific flush condition logic."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 DEBUG: Flush Condition Logic")
print("=" * 40)


def test_individual_vs_batch():
    """Compare individual adds vs batch adds."""

    print("\n🧪 Individual vs Batch Behavior")
    print("-" * 35)

    # Test 1: Individual adds (working case)
    print("Test 1: Individual adds with buffer_size=2")
    native.configure_database({"buffer_size": 2})
    native.clear_database()

    # Add vectors one by one
    for i in range(4):
        vector = np.random.rand(128).astype(np.float32).tolist()
        native.add_vector(f"individual_{i}", vector, {})
        stats = native.get_stats()
        print(
            f"   After add #{i + 1}: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
        )

    print("\nTest 2: Batch add with same data")
    native.clear_database()

    vectors = [np.random.rand(128).astype(np.float32).tolist() for _ in range(4)]
    ids = [f"batch_{i}" for i in range(4)]
    metadata = [{}] * 4

    native.add_vector_batch(ids, vectors, metadata)
    stats = native.get_stats()
    print(
        f"   Batch result: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )


def test_buffer_size_values():
    """Test if buffer_size values are being read correctly."""

    print("\n⚙️ Buffer Size Value Debug")
    print("-" * 30)

    configs = [1, 2, 5, 10]

    for buffer_size in configs:
        print(f"\nConfiguring buffer_size={buffer_size}")
        native.configure_database({"buffer_size": buffer_size})
        native.clear_database()

        # Try to trigger flush with exactly buffer_size+1 individual adds
        print(f"Adding {buffer_size + 1} vectors individually:")
        for i in range(buffer_size + 1):
            vector = np.random.rand(128).astype(np.float32).tolist()
            native.add_vector(f"test_{buffer_size}_{i}", vector, {})

            stats = native.get_stats()
            buffer_count = stats["buffer_size"]
            main_count = stats["main_index_size"]

            if i == 0:
                print(f"   After 1st: buffer={buffer_count}, main={main_count}")
            elif i == buffer_size - 1:
                print(
                    f"   After {buffer_size}th: buffer={buffer_count}, main={main_count}"
                )
            elif i == buffer_size:
                print(
                    f"   After {buffer_size + 1}th: buffer={buffer_count}, main={main_count}"
                )
                if main_count > 0:
                    print(f"   ✅ Flush triggered at {buffer_size + 1} vectors")
                else:
                    print(f"   ❌ No flush triggered")


def test_small_batch_sizes():
    """Test batch processing with very small batches."""

    print("\n📦 Small Batch Size Test")
    print("-" * 25)

    # Test adding small batches that should trigger flushes
    native.configure_database({"buffer_size": 3})
    native.clear_database()

    # Add batch of 2 (should fit in buffer)
    print("Adding batch of 2 vectors (buffer_size=3)")
    vectors1 = np.random.rand(2, 128).astype(np.float32).tolist()
    ids1 = ["batch1_0", "batch1_1"]
    native.add_vector_batch(ids1, vectors1, [{}, {}])

    stats = native.get_stats()
    print(
        f"   After batch 1: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )

    # Add batch of 2 more (should trigger flush)
    print("Adding batch of 2 more vectors (total would be 4 > 3)")
    vectors2 = np.random.rand(2, 128).astype(np.float32).tolist()
    ids2 = ["batch2_0", "batch2_1"]
    native.add_vector_batch(ids2, vectors2, [{}, {}])

    stats = native.get_stats()
    print(
        f"   After batch 2: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )


# Run debug tests
print("🧪 Running flush condition debug tests...")

test_individual_vs_batch()
test_buffer_size_values()
test_small_batch_sizes()

print("\n" + "=" * 40)
print("🎯 FLUSH CONDITION ANALYSIS")
print("=" * 40)

print("\n💡 Key Questions:")
print("1. Do individual adds trigger flushes correctly?")
print("2. Do batch adds behave differently?")
print("3. Are buffer_size values being applied?")
print("4. Is the flush condition logic working?")

print("\n✅ Flush condition debugging complete!")
