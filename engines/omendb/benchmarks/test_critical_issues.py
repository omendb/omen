#!/usr/bin/env python3
"""Test critical storage engine issues identified in audit."""

import numpy as np
import sys
import os
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb
import omendb.native as native

def test_vector_normalization():
    """Test if vectors are being normalized (changing user data)."""
    print("🔍 Testing Vector Normalization Issue")
    print("=" * 60)
    
    db = omendb.DB()
    
    # Create a vector with known values
    original = np.array([3.0, 4.0] + [0.0] * 126, dtype=np.float32)
    # Magnitude = sqrt(9 + 16) = 5
    # Normalized = [0.6, 0.8, 0, 0, ...]
    
    db.add("test_norm", original)
    retrieved = db.get("test_norm")
    
    print(f"Original vector: [{original[0]:.2f}, {original[1]:.2f}, ...]")
    if retrieved:
        first_val = retrieved[0] if not isinstance(retrieved[0], list) else retrieved[0][0]
        second_val = retrieved[1] if not isinstance(retrieved[1], list) else retrieved[1][0]
        print(f"Retrieved vector: [{first_val:.2f}, {second_val:.2f}, ...]")
        
        # Check if normalized
        expected_norm = [0.6, 0.8]
        if abs(first_val - expected_norm[0]) < 0.01:
            print("❌ VECTORS ARE NORMALIZED - User data changed!")
            return False
        elif abs(first_val - original[0]) < 0.01:
            print("✅ Original values preserved")
            return True
        else:
            print(f"⚠️ Unexpected values returned")
            return False
    else:
        print("❌ Failed to retrieve vector")
        return False

def test_memory_mapped_persistence():
    """Test if memory-mapped storage actually persists data."""
    print("\n🔍 Testing Memory-Mapped Persistence")
    print("=" * 60)
    
    # Reset and enable memory-mapped storage
    native._reset()
    native.enable_memory_mapped_storage()
    
    db = omendb.DB()
    path = "/tmp/test_critical_mmap"
    
    # Clean up old files
    for ext in ['.vectors', '.graph', '.meta']:
        if os.path.exists(path + ext):
            os.remove(path + ext)
    
    db.set_persistence(path)
    
    # Add test vectors
    print("Adding 100 test vectors...")
    for i in range(100):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)  # Mark with ID
        db.add(f"persist_{i}", vector)
    
    print(f"Count before checkpoint: {db.count()}")
    
    # Checkpoint
    success = db.checkpoint()
    print(f"Checkpoint result: {success}")
    
    # Check if files exist and have data
    vector_file = path + ".vectors"
    if os.path.exists(vector_file):
        size = os.path.getsize(vector_file)
        print(f"Vector file size: {size} bytes")
        if size < 100 * 128 * 4:  # Should be at least 100 vectors * 128 dims * 4 bytes
            print(f"❌ File too small for 100 vectors (expected >{100*128*4} bytes)")
    else:
        print("❌ Vector file not created")
    
    # Test recovery
    print("\nTesting recovery...")
    native._reset()
    native.enable_memory_mapped_storage()
    db2 = omendb.DB()
    db2.set_persistence(path)
    
    recovered = db2.count()
    print(f"Recovered {recovered}/100 vectors")
    
    if recovered == 100:
        print("✅ Persistence working!")
        return True
    else:
        print(f"❌ PERSISTENCE BROKEN - Lost {100 - recovered} vectors")
        return False

def test_quantization_application():
    """Test if quantization actually reduces memory when enabled."""
    print("\n🔍 Testing Quantization Application")
    print("=" * 60)
    
    # Test 1: Default state
    native._reset()
    db1 = omendb.DB()
    
    # Add vectors without quantization
    print("Adding 1000 vectors WITHOUT quantization...")
    import gc
    import psutil
    
    gc.collect()
    process = psutil.Process(os.getpid())
    mem_before = process.memory_info().rss / 1024 / 1024
    
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32)
        db1.add(f"no_quant_{i}", vector)
    
    gc.collect()
    mem_after = process.memory_info().rss / 1024 / 1024
    mem_no_quant = mem_after - mem_before
    print(f"Memory used: {mem_no_quant:.2f} MB")
    
    # Test 2: With quantization
    native._reset()
    native.enable_quantization()
    db2 = omendb.DB()
    
    print("\nAdding 1000 vectors WITH quantization enabled...")
    gc.collect()
    mem_before = process.memory_info().rss / 1024 / 1024
    
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32)
        db2.add(f"quant_{i}", vector)
    
    gc.collect()
    mem_after = process.memory_info().rss / 1024 / 1024
    mem_with_quant = mem_after - mem_before
    print(f"Memory used: {mem_with_quant:.2f} MB")
    
    reduction = (mem_no_quant - mem_with_quant) / mem_no_quant * 100
    print(f"\nMemory reduction: {reduction:.1f}%")
    
    if reduction > 50:  # Should be ~75% reduction with 8-bit quantization
        print("✅ Quantization is working!")
        return True
    else:
        print("❌ QUANTIZATION NOT APPLIED - No memory savings")
        return False

def test_memory_tracking():
    """Test if memory tracking is accurate."""
    print("\n🔍 Testing Memory Tracking Accuracy")
    print("=" * 60)
    
    db = omendb.DB()
    
    # Add exactly 100 vectors
    for i in range(100):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"track_{i}", vector)
    
    stats = db.get_memory_stats()
    
    # Calculate expected
    vectors_expected = 100 * 128 * 4 / (1024 * 1024)  # 0.049 MB
    
    # Get tracked
    vectors_tracked = stats.get('vectors_mb', 0)
    graph_tracked = stats.get('graph_mb', 0)
    total_tracked = stats.get('total_mb', 0)
    
    print(f"Expected vectors: {vectors_expected:.3f} MB")
    print(f"Tracked vectors: {vectors_tracked:.3f} MB")
    print(f"Tracked graph: {graph_tracked:.3f} MB")
    print(f"Tracked total: {total_tracked:.3f} MB")
    
    # Check accuracy
    if total_tracked < 0.01:
        print("❌ Memory tracking reporting near zero")
        return False
    elif abs(total_tracked - vectors_expected) / vectors_expected > 10:  # More than 10x off
        print(f"❌ Memory tracking off by {abs(total_tracked - vectors_expected) / vectors_expected:.0f}x")
        return False
    else:
        print("✅ Memory tracking reasonably accurate")
        return True

def run_critical_tests():
    """Run all critical issue tests."""
    print("🚨 CRITICAL STORAGE ENGINE TESTS")
    print("=" * 70)
    print("Testing critical issues identified in audit...\n")
    
    results = {
        "Vector Normalization": test_vector_normalization(),
        "Memory-Mapped Persistence": test_memory_mapped_persistence(),
        "Quantization Application": test_quantization_application(),
        "Memory Tracking Accuracy": test_memory_tracking()
    }
    
    print("\n" + "=" * 70)
    print("📋 CRITICAL ISSUES SUMMARY:")
    print("-" * 70)
    
    for issue, passed in results.items():
        status = "✅ WORKING" if passed else "❌ BROKEN"
        print(f"{issue:30} {status}")
    
    critical_count = sum(1 for p in results.values() if not p)
    print(f"\n🔴 {critical_count} CRITICAL ISSUES NEED FIXING")
    
    if critical_count == 0:
        print("🎉 All critical issues resolved!")
    else:
        print("\nPriority fixes needed:")
        if not results["Vector Normalization"]:
            print("1. Vectors being normalized - user data changed")
        if not results["Memory-Mapped Persistence"]:
            print("2. Persistence completely broken - data loss")
        if not results["Quantization Application"]:
            print("3. Quantization not applied - missing memory savings")
        if not results["Memory Tracking Accuracy"]:
            print("4. Memory tracking inaccurate - can't trust metrics")

if __name__ == "__main__":
    run_critical_tests()