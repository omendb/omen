#!/usr/bin/env python3
"""
Test 50K vector insertion timing for RoarGraph vs HNSW
"""

import time
import numpy as np
import sys
import os

# Ensure we're using the local omendb
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb

print("Testing 50K vector insertion timing...")

# Test HNSW
print("\n" + "=" * 60)
print("HNSW with 50,000 vectors")
print("=" * 60)

db_hnsw = omendb.DB(algorithm="hnsw", buffer_size=25000)
vectors = np.random.randn(50000, 128).astype(np.float32)
ids = [f"vec_{i}" for i in range(50000)]

start = time.perf_counter()
db_hnsw.add_batch(vectors, ids)
hnsw_time = time.perf_counter() - start

print(f"HNSW total time: {hnsw_time:.2f}s")
print(f"HNSW throughput: {50000 / hnsw_time:,.0f} vec/s")

# Clear for next test
db_hnsw.clear()

# Test RoarGraph
print("\n" + "=" * 60)
print("RoarGraph with 50,000 vectors")
print("=" * 60)

db_roar = omendb.DB(algorithm="roargraph", buffer_size=25000)

start = time.perf_counter()
db_roar.add_batch(vectors, ids)
roar_time = time.perf_counter() - start

print(f"RoarGraph total time: {roar_time:.2f}s")
print(f"RoarGraph throughput: {50000 / roar_time:,.0f} vec/s")

# Comparison
print("\n" + "=" * 60)
print("RESULTS")
print("=" * 60)
speedup = hnsw_time / roar_time
print(f"RoarGraph is {speedup:.2f}x {'faster' if speedup > 1 else 'slower'} than HNSW")

if speedup > 3.0:
    print("✅ RoarGraph delivers on its promise!")
elif speedup > 2.0:
    print("⚠️ RoarGraph is moderately faster")
else:
    print("❌ RoarGraph doesn't provide sufficient speedup")
