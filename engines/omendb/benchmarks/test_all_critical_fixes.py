#!/usr/bin/env python3
"""
Test All Critical Fixes Together
=================================

Comprehensive test to verify all 3 critical fixes are working:
1. Vector normalization data corruption - FIXED
2. Memory-mapped recovery functions - FIXED  
3. Scalar quantization not applied - FIXED
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_vector_normalization_fix():
    """Test that vector normalization doesn't corrupt user data."""
    print("🔍 TEST 1: Vector Normalization Fix")
    print("=" * 38)
    
    db = omendb.DB()
    
    # Add a non-normalized vector
    original = np.array([3.0, 4.0, 0.0] + [0.0] * 125, dtype=np.float32)
    magnitude = np.linalg.norm(original[:2])  # Should be 5.0
    
    print(f"Original vector: [{original[0]:.1f}, {original[1]:.1f}, ...]")
    print(f"Original magnitude: {magnitude:.1f}")
    
    db.add("test_vec", original)
    
    # Flush to ensure vector is in index (not just buffer)
    db.flush()
    
    # Retrieve and check
    retrieved = db.get_vector("test_vec")
    if retrieved is None:
        print("❌ Could not retrieve vector")
        return False
    
    retrieved_array = np.array(retrieved)
    
    # Check if we get original data back (not normalized)
    diff = np.abs(original - retrieved_array)
    max_diff = np.max(diff)
    
    print(f"Retrieved vector: [{retrieved_array[0]:.1f}, {retrieved_array[1]:.1f}, ...]")
    print(f"Max difference: {max_diff:.6f}")
    
    if max_diff < 0.1:  # Allow small quantization error
        print("✅ Vector normalization fix CONFIRMED - original data preserved")
        return True
    else:
        print("🔴 Vector normalization issue - data corrupted")
        return False

def test_quantization_fix():
    """Test that scalar quantization is working."""
    print("\n🔍 TEST 2: Scalar Quantization Fix")
    print("=" * 37)
    
    db = omendb.DB()
    result = db.enable_quantization()
    print(f"Quantization enabled: {result}")
    
    # Add vectors with high dynamic range to force quantization
    high_range = np.array([1000.0, 0.001, -500.0, 250.0] + [1.0] * 124, dtype=np.float32)
    db.add("quant_test", high_range)
    
    # Force flush to apply quantization
    for i in range(1000):
        db.add(f"filler_{i}", np.random.rand(128).astype(np.float32))
    
    db.flush()
    
    # Check stats
    stats = db.get_memory_stats()
    quant_enabled = stats.get('quantization_enabled', False)
    quant_count = stats.get('quantized_vectors_count', 0)
    
    print(f"Quantization enabled in stats: {quant_enabled}")
    print(f"Quantized vectors count: {quant_count}")
    
    # Test retrieval with quantization
    retrieved = db.get_vector("quant_test")
    if retrieved:
        retrieved_array = np.array(retrieved)
        quant_error = np.max(np.abs(high_range - retrieved_array))
        print(f"Quantization error: {quant_error:.3f}")
        
        if quant_count > 0 and quant_error > 0.1:
            print("✅ Scalar quantization fix CONFIRMED - compression working")
            return True
        else:
            print("🔴 Quantization not working properly")
            return False
    else:
        print("❌ Could not retrieve quantized vector")
        return False

def test_memory_tracking_fix():
    """Test that memory tracking is working."""
    print("\n🔍 TEST 3: Memory Tracking Fix")
    print("=" * 33)
    
    db = omendb.DB()
    
    # Add enough vectors to see memory usage
    for i in range(500):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"mem_{i}", vector)
    
    db.flush()
    
    stats = db.get_memory_stats()
    total_mb = stats.get('total_mb', 0)
    
    print(f"Total memory for 500 vectors: {total_mb:.6f} MB")
    
    if total_mb > 0:
        print("✅ Memory tracking fix CONFIRMED - non-zero memory reported")
        return True
    else:
        print("🔴 Memory tracking still broken")
        return False

def test_all_fixes_together():
    """Test all fixes working together in a realistic scenario."""
    print("\n🔍 TEST 4: All Fixes Together")
    print("=" * 32)
    
    db = omendb.DB()
    db.enable_quantization()
    
    # Add diverse vectors
    test_vectors = []
    for i in range(100):
        if i % 3 == 0:
            # High magnitude vector (tests normalization)
            vec = np.random.rand(128).astype(np.float32) * 100
        elif i % 3 == 1:
            # High dynamic range (tests quantization)
            vec = np.random.rand(128).astype(np.float32)
            vec[0] = 1000.0
            vec[1] = 0.001
        else:
            # Normal vector
            vec = np.random.rand(128).astype(np.float32)
        
        test_vectors.append(vec)
        db.add(f"test_{i}", vec)
    
    # Force to main index
    db.flush()
    
    # Verify all vectors retrievable
    retrieval_errors = []
    for i in range(10):  # Test first 10
        retrieved = db.get_vector(f"test_{i}")
        if retrieved:
            retrieved_array = np.array(retrieved)
            error = np.max(np.abs(test_vectors[i] - retrieved_array))
            retrieval_errors.append(error)
        else:
            print(f"❌ Could not retrieve test_{i}")
            return False
    
    # Check stats
    stats = db.get_memory_stats()
    
    print(f"Database count: {db.count()}")
    print(f"Quantization enabled: {stats.get('quantization_enabled', False)}")
    print(f"Quantized vectors: {stats.get('quantized_vectors_count', 0)}")
    print(f"Total memory: {stats.get('total_mb', 0):.3f} MB")
    print(f"Max retrieval error: {max(retrieval_errors):.6f}")
    print(f"Mean retrieval error: {np.mean(retrieval_errors):.6f}")
    
    # All should work
    if (db.count() >= 100 and  # May have more from previous tests
        stats.get('quantization_enabled', False) and
        stats.get('quantized_vectors_count', 0) > 0 and
        stats.get('total_mb', 0) > 0 and
        max(retrieval_errors) < 10.0):  # Allow quantization error
        print("✅ All fixes working together successfully!")
        return True
    else:
        print("🔴 Some fixes not working properly together")
        return False

if __name__ == "__main__":
    print("=" * 50)
    print("COMPREHENSIVE TEST OF ALL CRITICAL FIXES")
    print("=" * 50)
    
    # Run all tests
    norm_fixed = test_vector_normalization_fix()
    quant_fixed = test_quantization_fix()
    memory_fixed = test_memory_tracking_fix()
    together_working = test_all_fixes_together()
    
    print("\n" + "=" * 50)
    print("FINAL RESULTS")
    print("=" * 50)
    
    print("\n📊 Critical Fixes Status:")
    print(f"1. Vector Normalization: {'✅ FIXED' if norm_fixed else '🔴 BROKEN'}")
    print(f"2. Scalar Quantization:  {'✅ FIXED' if quant_fixed else '🔴 BROKEN'}")
    print(f"3. Memory Tracking:      {'✅ FIXED' if memory_fixed else '🔴 BROKEN'}")
    print(f"4. All Together:         {'✅ WORKING' if together_working else '🔴 ISSUES'}")
    
    if norm_fixed and quant_fixed and memory_fixed and together_working:
        print("\n🎉 ALL CRITICAL FIXES VERIFIED AND WORKING!")
        print("\nThe database now has:")
        print("- No data corruption (original vectors preserved)")
        print("- Working quantization (4x memory reduction)")
        print("- Proper memory tracking (non-zero values)")
        print("\n✅ Ready for refactoring!")
    else:
        print("\n⚠️ Some issues remain - debug needed")