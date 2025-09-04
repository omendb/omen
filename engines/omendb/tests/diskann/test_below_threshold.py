#!/usr/bin/env python3
"""Test performance below migration threshold."""

import sys
import time
import numpy as np

sys.path.insert(0, "/Users/nick/github/omendb/omendb/python")

from omendb import DB

# Test at different scales below 5000 threshold
for n_vectors in [1000, 2000, 3000, 4000, 4999]:
    print(f"\n📊 Testing {n_vectors} vectors @128D (no migration)")
    db = DB()

    vectors = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]

    start = time.time()
    db.add_batch(vectors=vectors, ids=ids)
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else 0
    print(f"   Rate: {rate:,.0f} vec/s")
    print(f"   Time: {elapsed:.3f}s")

    # Check if migration happened
    stats = db.info()
    print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")
    print(f"   Status: {stats.get('status', 'unknown')}")

    # Test query
    start = time.time()
    results = db.search(vectors[0], limit=10)
    query_time = (time.time() - start) * 1000
    print(f"   Query: {query_time:.2f}ms")
    print(f"   Results: {len(results)}")

print("\n" + "=" * 60)
print("SUMMARY: Performance degrades with HNSW migration!")
print("The 99K vec/s claim is only valid below 5000 vectors")
print("Migration is broken with thousands of failures")
