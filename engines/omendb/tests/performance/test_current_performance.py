#!/usr/bin/env python3
"""
Test current performance - should achieve 90K+ vec/s with batch operations.
"""

import sys
import time
import numpy as np

sys.path.insert(0, "python")

from omendb import DB


def test_current_performance():
    """Test actual current performance."""
    print("🧪 Testing current performance...")
    db = DB()

    # Test batch construction performance (proper methodology)
    print("\n📊 Batch Construction Performance:")
    n_vectors = 10000  # Larger test for realistic numbers
    dimension = 64

    # Generate test data with NumPy for optimal performance
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"test_{i}" for i in range(n_vectors)]

    start = time.time()
    db.add_batch(vectors=vectors, ids=ids)
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else float("inf")
    print(f"  📈 Construction rate: {rate:.0f} vectors/sec")
    print(f"  ⏱️ Time for {n_vectors:,} vectors: {elapsed:.3f}s")
    print(f"  🎯 Target: 91,435 vec/s (lists) | 156,937 vec/s (NumPy)")

    if rate >= 150000:
        print(f"  ✅ EXCELLENT: Achieving NumPy-level performance")
    elif rate >= 90000:
        print(f"  ✅ GOOD: Achieving list-level performance")
    else:
        print(f"  ⚠️ SLOW: Below expected performance")

    # Test query performance
    print("\n🔍 Query Performance:")
    query_vector = np.random.rand(dimension).astype(np.float32)
    start = time.time()
    results = db.search(query_vector, limit=10)
    query_time = time.time() - start

    print(f"  🔍 Query time: {query_time * 1000:.2f}ms")
    print(f"  ✅ Query results: {len(results)}")
    print(f"  🎯 Target: <1ms average")

    # Test if we're using native module
    print("\n🔧 Implementation Details:")
    try:
        stats = db.info()
        print(f"  📊 Stats: {stats}")
    except Exception as e:
        print(f"  ❌ No stats available: {e}")


if __name__ == "__main__":
    test_current_performance()
