#!/usr/bin/env python3
"""Test that quantization fix actually works."""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

print("=" * 60)
print("Testing Quantization Fix")
print("=" * 60)

# Test 1: Small scale (1000 vectors)
print("\n=== Test 1: Small Scale (1000 vectors) ===")
vectors = np.random.rand(1000, 128).astype(np.float32)

# Without quantization
print("\nWithout quantization:")
db1 = omendb.DB()
db1.add_batch(vectors)
stats1 = db1.get_memory_stats()
normal_memory = stats1.get('total_mb', 0)
print(f"  Total memory: {normal_memory:.2f} MB")
print(f"  Graph memory: {stats1.get('graph_mb', 0):.2f} MB")

# With quantization
print("\nWith quantization:")
db2 = omendb.DB(quantization="scalar")
db2.add_batch(vectors)
stats2 = db2.get_memory_stats()
quant_memory = stats2.get('total_mb', 0)
print(f"  Total memory: {quant_memory:.2f} MB")
print(f"  Graph memory: {stats2.get('graph_mb', 0):.2f} MB")

# Calculate reduction
if normal_memory > 0:
    reduction = (1 - quant_memory / normal_memory) * 100
    print(f"\nMemory reduction: {reduction:.1f}%")
    if reduction > 0:
        print("✅ Quantization is now saving memory!")
    else:
        print("❌ Still broken - using more memory")

# Test 2: Verify get_vector works with quantization
print("\n=== Test 2: Vector Retrieval ===")
retrieved = db2.get_vector("vec_0")
if retrieved is not None:
    print(f"✅ Can retrieve vectors (length: {len(retrieved)})")
else:
    print("❌ Cannot retrieve vectors")

# Test 3: Larger scale (10K vectors)
print("\n=== Test 3: Larger Scale (10K vectors) ===")
large_vectors = np.random.rand(10001, 128).astype(np.float32)  # 10001 to trigger index building

# Without quantization
db3 = omendb.DB()
db3.add_batch(large_vectors)
stats3 = db3.get_memory_stats()
normal_large = stats3.get('total_mb', 0)
print(f"Normal: {normal_large:.2f} MB")

# With quantization
db4 = omendb.DB(quantization="scalar")
db4.add_batch(large_vectors)
stats4 = db4.get_memory_stats()
quant_large = stats4.get('total_mb', 0)
print(f"Quantized: {quant_large:.2f} MB")

if normal_large > 0:
    reduction_large = (1 - quant_large / normal_large) * 100
    print(f"Reduction: {reduction_large:.1f}%")
    
    # Expected: ~75% reduction
    if reduction_large > 60:
        print("✅ Quantization working properly at scale!")
    elif reduction_large > 40:
        print("⚠️ Some savings but less than expected")
    else:
        print("❌ Quantization not achieving expected savings")

print("\n" + "=" * 60)
if reduction > 0 and reduction_large > 40:
    print("🎉 QUANTIZATION FIXED! 🎉")
else:
    print("❌ Quantization still has issues")
print("=" * 60)