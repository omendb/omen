#!/usr/bin/env python3
"""Debug the buffer flush mechanism step by step."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🔍 DEBUG: Buffer Flush Mechanism")
print("=" * 50)


def debug_buffer_flush():
    """Step through buffer flush to find the exact issue."""

    print("\n🧪 Setting up debug test")
    print("-" * 30)

    # Configure with tiny buffer to force immediate flush
    native.configure_database({"buffer_size": 2})  # Only 2 vectors max
    native.clear_database()

    # Add exactly 1 vector (should stay in buffer)
    print("\n1️⃣ Adding 1 vector (should stay in buffer)")
    vectors_np = np.random.rand(1, 128).astype(np.float32)
    ids = ["debug_1"]
    metadata = [{}]

    native.add_vector_batch(ids, vectors_np, metadata)
    stats = native.get_stats()
    print(
        f"   After 1 vector: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )

    # Add exactly 1 more vector (should stay in buffer - total 2)
    print("\n2️⃣ Adding 1 more vector (buffer should be at capacity)")
    vectors_np2 = np.random.rand(1, 128).astype(np.float32)
    ids2 = ["debug_2"]

    native.add_vector_batch(ids2, vectors_np2, [{}])
    stats = native.get_stats()
    print(
        f"   After 2 vectors: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )

    # Add 1 more vector (should trigger flush!)
    print("\n3️⃣ Adding 1 more vector (should trigger flush)")
    print("   Expected: Buffer flushes first 2 vectors to HNSW, keeps 1 new vector")
    vectors_np3 = np.random.rand(1, 128).astype(np.float32)
    ids3 = ["debug_3"]

    print("   🎯 WATCH FOR FLUSH MESSAGES:")
    native.add_vector_batch(ids3, vectors_np3, [{}])
    stats = native.get_stats()
    print(
        f"   After flush: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )
    print(f"   Expected: buffer=1, main=2 (or similar distribution)")

    # Test search to see if vectors are accessible
    print("\n4️⃣ Testing search functionality")
    query = vectors_np[0].tolist()
    try:
        results = native.search_vectors(query, 3, {})
        print(f"   Search found {len(results)} results")
    except Exception as e:
        print(f"   Search failed: {e}")


def debug_large_batch():
    """Test what happens with a large batch that should definitely trigger flush."""

    print("\n🎯 Large Batch Debug Test")
    print("-" * 30)

    # Configure with small buffer
    native.configure_database({"buffer_size": 100})
    native.clear_database()

    print("Adding 500 vectors with buffer_size=100 (should trigger 4+ flushes)")

    vectors_np = np.random.rand(500, 128).astype(np.float32)
    ids = [f"large_{i}" for i in range(500)]
    metadata = [{}] * 500

    print("🎯 WATCHING FOR MULTIPLE FLUSH MESSAGES:")
    native.add_vector_batch(ids, vectors_np, metadata)

    stats = native.get_stats()
    print(
        f"Final result: buffer={stats['buffer_size']}, main={stats['main_index_size']}"
    )
    print(f"Expected: buffer=0-100, main=400-500")


def debug_buffer_size_config():
    """Debug if buffer_size configuration is actually being applied."""

    print("\n⚙️ Buffer Size Configuration Debug")
    print("-" * 40)

    configs = [1, 10, 100, 1000]

    for buffer_size in configs:
        print(f"\nTesting buffer_size={buffer_size}")

        native.configure_database({"buffer_size": buffer_size})
        native.clear_database()

        # Add exactly buffer_size + 1 vectors to force flush
        test_size = buffer_size + 1
        vectors_np = np.random.rand(test_size, 128).astype(np.float32)
        ids = [f"config_{buffer_size}_{i}" for i in range(test_size)]
        metadata = [{}] * test_size

        print(
            f"   Adding {test_size} vectors (should trigger flush after {buffer_size})"
        )
        native.add_vector_batch(ids, vectors_np, metadata)

        stats = native.get_stats()
        buffer_count = stats["buffer_size"]
        main_count = stats["main_index_size"]
        total = buffer_count + main_count

        print(f"   Result: buffer={buffer_count}, main={main_count}, total={total}")

        # Check if behavior matches expected buffer size
        if buffer_count <= buffer_size:
            print(f"   ✅ Buffer size respected (≤{buffer_size})")
        else:
            print(f"   ❌ Buffer size ignored ({buffer_count} > {buffer_size})")


# Run all debug tests
print("🧪 Running buffer flush debug tests...")

debug_buffer_flush()
debug_large_batch()
debug_buffer_size_config()

print("\n" + "=" * 50)
print("🎯 DEBUG CONCLUSIONS")
print("=" * 50)

print("\n💡 Look for patterns in the output above:")
print("1. Are flush messages appearing when expected?")
print("2. Are vectors actually moving from buffer to main index?")
print("3. Is buffer_size configuration being respected?")
print("4. Is the HNSW main index reporting correct size?")

print("\n✅ Buffer flush debugging complete!")
