#!/usr/bin/env python3
"""
Test Quantization Memory Usage
==============================

Since quantization IS working, test if memory tracking reflects the reduction.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_quantization_memory_tracking():
    """Test if memory tracking shows quantization benefits."""
    
    print("📊 Quantization Memory Tracking Test")
    print("=" * 40)
    
    # Test without quantization first
    print("1. Testing WITHOUT quantization:")
    db_no_quant = omendb.DB()
    
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32) * 100  # Large range
        db_no_quant.add(f"vec_{i}", vector)
    
    db_no_quant.flush()  # Force to main index
    stats_no_quant = db_no_quant.get_memory_stats()
    
    print(f"   Count: {db_no_quant.count()}")
    print(f"   Memory: {stats_no_quant.get('total_mb', 0):.3f} MB")
    
    # Test with quantization
    print("\n2. Testing WITH quantization:")
    db_quant = omendb.DB()
    result = db_quant.enable_quantization()
    print(f"   Quantization enabled: {result}")
    
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32) * 100  # Same range
        db_quant.add(f"vec_{i}", vector)
    
    db_quant.flush()  # Force to main index
    stats_quant = db_quant.get_memory_stats()
    
    print(f"   Count: {db_quant.count()}")
    print(f"   Memory: {stats_quant.get('total_mb', 0):.3f} MB")
    
    # Compare
    mem_no_quant = stats_no_quant.get('total_mb', 0)
    mem_quant = stats_quant.get('total_mb', 0)
    
    print(f"\n📈 Memory Comparison:")
    print(f"   Without quantization: {mem_no_quant:.3f} MB")
    print(f"   With quantization:    {mem_quant:.3f} MB")
    
    if mem_quant > 0 and mem_no_quant > 0:
        ratio = mem_no_quant / mem_quant
        print(f"   Reduction ratio: {ratio:.1f}x")
        
        if ratio > 2.0:
            print("   ✅ Quantization shows significant memory reduction")
            return True
        elif ratio > 1.1:
            print("   ⚠️ Quantization shows some memory reduction")
            return True
        else:
            print("   🔴 Quantization shows no meaningful memory reduction")
            return False
    else:
        print("   🔴 Memory tracking returned invalid values")
        return False

def test_quantization_count_tracking():
    """Test if quantized vector counts are tracked correctly."""
    
    print("\n🔢 Quantization Count Tracking")
    print("=" * 33)
    
    db = omendb.DB()
    db.enable_quantization()
    
    print("Adding 100 vectors with quantization enabled...")
    for i in range(100):
        vector = np.random.rand(128).astype(np.float32) * 100
        db.add(f"vec_{i}", vector)
    
    stats = db.get_memory_stats()
    
    print(f"Database count: {db.count()}")
    print(f"Quantized vectors count: {stats.get('quantized_vectors_count', 0)}")
    
    # These should match if all vectors are being quantized
    db_count = db.count()
    quant_count = stats.get('quantized_vectors_count', 0)
    
    if quant_count == db_count:
        print("✅ All vectors are being stored in quantization dict")
        return True
    elif quant_count > 0:
        print(f"⚠️ Only {quant_count}/{db_count} vectors are quantized")
        return False
    else:
        print("🔴 No vectors are being stored in quantization dict")
        return False

def calculate_expected_savings():
    """Calculate expected memory savings from quantization."""
    
    print("\n🧮 Expected Memory Savings Calculation")
    print("=" * 42)
    
    num_vectors = 1000
    dimensions = 128
    
    # Uncompressed: Float32 = 4 bytes per dimension
    uncompressed_bytes = num_vectors * dimensions * 4
    uncompressed_mb = uncompressed_bytes / (1024 * 1024)
    
    # Compressed: UInt8 = 1 byte per dimension + 8 bytes for scale/offset per vector
    compressed_bytes = num_vectors * (dimensions * 1 + 8)
    compressed_mb = compressed_bytes / (1024 * 1024)
    
    reduction_ratio = uncompressed_bytes / compressed_bytes
    
    print(f"For {num_vectors} vectors × {dimensions} dimensions:")
    print(f"  Uncompressed: {uncompressed_bytes:,} bytes = {uncompressed_mb:.3f} MB")
    print(f"  Compressed:   {compressed_bytes:,} bytes = {compressed_mb:.3f} MB") 
    print(f"  Expected reduction: {reduction_ratio:.1f}x")
    
    return uncompressed_mb, compressed_mb, reduction_ratio

if __name__ == "__main__":
    # Calculate expected savings
    expected_uncompressed, expected_compressed, expected_ratio = calculate_expected_savings()
    
    # Test memory tracking
    memory_tracking_works = test_quantization_memory_tracking()
    
    # Test count tracking  
    count_tracking_works = test_quantization_count_tracking()
    
    print("\n" + "=" * 50)
    print("QUANTIZATION MEMORY TEST RESULTS")
    print("=" * 50)
    
    print(f"📊 Expected Results:")
    print(f"   - Uncompressed: {expected_uncompressed:.3f} MB")
    print(f"   - Compressed:   {expected_compressed:.3f} MB")
    print(f"   - Expected ratio: {expected_ratio:.1f}x")
    
    print(f"\n🔍 Test Results:")
    if memory_tracking_works:
        print("   ✅ Memory tracking shows quantization benefits")
    else:
        print("   🔴 Memory tracking doesn't show quantization benefits")
        
    if count_tracking_works:
        print("   ✅ Quantized vector counts are tracked correctly")
    else:
        print("   🔴 Quantized vector counts are not tracked correctly")
    
    if memory_tracking_works and count_tracking_works:
        print("\n🎉 Quantization is fully working - memory and tracking!")
    elif count_tracking_works:
        print("\n⚠️ Quantization works but memory tracking is broken")
    else:
        print("\n🔴 Quantization has issues with both memory and counting")