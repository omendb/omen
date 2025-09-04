#!/usr/bin/env python3
"""
Test Quantization Application
=============================

Test if quantization is actually applied when enabled.
According to audit, flags are set but never checked in add path.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_scalar_quantization():
    """Test if scalar quantization reduces memory usage."""
    
    print("🧪 Testing Scalar Quantization")
    print("=" * 40)
    
    # Test 1: Database without quantization
    print("\n📝 Test 1: Database WITHOUT quantization")
    db1 = omendb.DB()
    
    # Add test vectors
    num_vectors = 1000
    for i in range(num_vectors):
        vector = np.random.rand(128).astype(np.float32) * 10  # Scale up for quantization impact
        db1.add(f"vec_{i}", vector)
    
    # Force flush and get memory usage
    db1.flush()
    stats1 = db1.get_memory_stats()
    
    print(f"Database count: {db1.count()}")
    print(f"Memory without quantization: {stats1.get('total_mb', 0):.2f} MB")
    
    # Test 2: Database with scalar quantization
    print("\n📝 Test 2: Database WITH scalar quantization")
    db2 = omendb.DB()
    
    # Enable scalar quantization
    result = db2.enable_quantization()
    print(f"Enable scalar quantization result: {result}")
    
    # Add same vectors
    for i in range(num_vectors):
        vector = np.random.rand(128).astype(np.float32) * 10  # Same scale
        db2.add(f"vec_{i}", vector)
    
    # Force flush and get memory usage
    db2.flush()
    stats2 = db2.get_memory_stats()
    
    print(f"Database count: {db2.count()}")
    print(f"Memory with quantization: {stats2.get('total_mb', 0):.2f} MB")
    
    # Calculate reduction
    memory1 = stats1.get('total_mb', 0)
    memory2 = stats2.get('total_mb', 0)
    
    if memory1 > 0 and memory2 > 0:
        reduction = memory1 / memory2 
        print(f"\n📊 Memory Reduction Analysis:")
        print(f"Without quantization: {memory1:.2f} MB")
        print(f"With quantization: {memory2:.2f} MB")
        print(f"Reduction ratio: {reduction:.1f}x")
        
        if reduction > 2.0:
            print("✅ Scalar quantization is working!")
            return True
        elif reduction > 1.1:
            print("⚠️ Some reduction but less than expected (should be 4x+)")
            return False
        else:
            print("🔴 NO REDUCTION: Quantization is not being applied!")
            return False
    else:
        print("❌ Could not measure memory usage")
        return False

def test_binary_quantization():
    """Test if binary quantization reduces memory usage."""
    
    print("\n🧪 Testing Binary Quantization")
    print("=" * 40)
    
    # Test with binary quantization
    print("\n📝 Database WITH binary quantization")
    db = omendb.DB()
    
    # Enable binary quantization
    result = db.enable_binary_quantization()
    print(f"Enable binary quantization result: {result}")
    
    # Add test vectors
    num_vectors = 1000
    for i in range(num_vectors):
        vector = np.random.rand(128).astype(np.float32) * 10
        db.add(f"vec_{i}", vector)
    
    # Force flush and get memory usage
    db.flush()
    stats = db.get_memory_stats()
    
    print(f"Database count: {db.count()}")
    print(f"Memory with binary quantization: {stats.get('total_mb', 0):.2f} MB")
    
    # Binary quantization should give massive reduction (32x theoretical)
    memory_mb = stats.get('total_mb', 0)
    
    # Expected: ~1MB for 1000 vectors * 128 dim * 1 bit = 16KB vectors + overhead
    if memory_mb > 0:
        if memory_mb < 5.0:  # Very small memory usage
            print("✅ Binary quantization appears to be working!")
            return True
        else:
            print("🔴 Binary quantization not providing expected reduction")
            print(f"   Expected: <5MB, Got: {memory_mb:.2f}MB")
            return False
    else:
        print("❌ Could not measure memory usage")
        return False

def test_quantization_flags():
    """Test the quantization flag setting directly."""
    
    print("\n🔧 Testing Quantization Flags")
    print("=" * 30)
    
    db = omendb.DB()
    
    # Test scalar quantization flag
    result1 = db.enable_quantization()
    print(f"Scalar quantization enabled: {result1}")
    
    # Test binary quantization flag
    result2 = db.enable_binary_quantization() 
    print(f"Binary quantization enabled: {result2}")
    
    return result1 and result2

if __name__ == "__main__":
    print("Testing quantization application...")
    
    # Test flag setting
    flags_work = test_quantization_flags()
    
    # Test actual quantization
    scalar_works = test_scalar_quantization()
    binary_works = test_binary_quantization()
    
    print("\n" + "=" * 50)
    print("QUANTIZATION TEST RESULTS")
    print("=" * 50)
    
    if flags_work:
        print("✅ Quantization flags can be enabled")
    else:
        print("❌ Quantization flags cannot be enabled")
    
    if scalar_works:
        print("✅ Scalar quantization is working")
    else:
        print("🔴 CRITICAL: Scalar quantization NOT working")
    
    if binary_works:
        print("✅ Binary quantization is working")
    else:
        print("🔴 CRITICAL: Binary quantization NOT working")
    
    if not scalar_works and not binary_works:
        print("\n🔴 CRITICAL ISSUE #3 CONFIRMED:")
        print("   Quantization flags set but never applied in add path")
        print("   Missing 4-32x memory reduction")
        print("   Need to fix quantization logic in native.mojo")