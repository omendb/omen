"""
Test whether buffer actually improves DiskANN performance.

Key questions:
1. Is batch flush faster than incremental inserts?
2. Does buffer help with search latency?
3. What's the optimal buffer size?
"""

import time
import numpy as np
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "python"))

import omendb


def generate_vectors(n, dim=128):
    """Generate random vectors for testing."""
    return np.random.random((n, dim)).astype(np.float32)


def test_direct_diskann(vectors):
    """Test direct insertion into DiskANN without buffer."""
    db = omendb.DB(algorithm="diskann", buffer_size=0)  # No buffer

    start = time.perf_counter()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec)
    insert_time = time.perf_counter() - start

    # Test search
    start = time.perf_counter()
    for _ in range(100):
        results = db.search(vectors[np.random.randint(len(vectors))], limit=10)
    search_time = time.perf_counter() - start

    return insert_time, search_time / 100


def test_buffer_diskann(vectors, buffer_size=1000):
    """Test DiskANN with buffer layer."""
    db = omendb.DB(algorithm="diskann", buffer_size=buffer_size)

    start = time.perf_counter()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec)
    insert_time = time.perf_counter() - start

    # Force final flush if needed
    db.flush()  # Ensure everything is indexed

    # Test search
    start = time.perf_counter()
    for _ in range(100):
        results = db.search(vectors[np.random.randint(len(vectors))], limit=10)
    search_time = time.perf_counter() - start

    return insert_time, search_time / 100


def test_batch_build(vectors):
    """Test batch building."""
    db = omendb.DB(algorithm="diskann")

    ids = [f"vec_{i}" for i in range(len(vectors))]

    start = time.perf_counter()
    db.add_batch(vectors, ids)  # Batch build
    insert_time = time.perf_counter() - start

    # Test search
    start = time.perf_counter()
    for _ in range(100):
        results = db.search(vectors[np.random.randint(len(vectors))], limit=10)
    search_time = time.perf_counter() - start

    return insert_time, search_time / 100


def main():
    print("Testing DiskANN: Direct vs Buffer vs Batch\n")
    print("=" * 60)

    test_sizes = [1000, 5000, 10000, 50000]

    for n in test_sizes:
        print(f"\n📊 Testing with {n} vectors (128D)")
        print("-" * 40)

        vectors = generate_vectors(n, 128)

        # Test direct insertion
        print("Direct DiskANN (no buffer)...")
        direct_insert, direct_search = test_direct_diskann(vectors)
        print(f"  Insert: {direct_insert:.3f}s ({n / direct_insert:.0f} vec/s)")
        print(f"  Search: {direct_search * 1000:.2f}ms")

        # Test with small buffer
        print("\nDiskANN with 1K buffer...")
        buf1k_insert, buf1k_search = test_buffer_diskann(vectors, 1000)
        print(f"  Insert: {buf1k_insert:.3f}s ({n / buf1k_insert:.0f} vec/s)")
        print(f"  Search: {buf1k_search * 1000:.2f}ms")

        # Test with large buffer
        print("\nDiskANN with 10K buffer...")
        buf10k_insert, buf10k_search = test_buffer_diskann(vectors, 10000)
        print(f"  Insert: {buf10k_insert:.3f}s ({n / buf10k_insert:.0f} vec/s)")
        print(f"  Search: {buf10k_search * 1000:.2f}ms")

        # Test batch build
        print("\nBatch build...")
        batch_insert, batch_search = test_batch_build(vectors)
        print(f"  Insert: {batch_insert:.3f}s ({n / batch_insert:.0f} vec/s)")
        print(f"  Search: {batch_search * 1000:.2f}ms")

        # Analysis
        print("\n📈 Analysis:")
        if direct_insert < buf1k_insert:
            print("  ✅ Direct insertion is FASTER - no buffer needed!")
        else:
            speedup = direct_insert / buf1k_insert
            print(f"  ✅ Buffer provides {speedup:.1f}x speedup")

        if batch_insert < direct_insert:
            batch_speedup = direct_insert / batch_insert
            print(f"  ✅ Batch build is {batch_speedup:.1f}x faster than incremental")

        # Search latency comparison
        if abs(direct_search - buf1k_search) < 0.0001:
            print("  ⚡ Search latency identical with/without buffer")
        elif direct_search < buf1k_search:
            print(
                f"  ⚠️  Buffer adds {(buf1k_search - direct_search) * 1000:.2f}ms search overhead"
            )


if __name__ == "__main__":
    main()
