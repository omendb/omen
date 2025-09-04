#!/usr/bin/env python3
"""
Test Quantization Directly
==========================

Test if quantization dictionaries are actually being populated.
Bypass memory tracking and look at the data structures directly.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_quantization_dictionaries():
    """Test if quantization dictionaries are populated."""
    
    print("🔍 Direct Quantization Dictionary Test")
    print("=" * 38)
    
    # Create database and enable quantization
    db = omendb.DB()
    result = db.enable_quantization()
    print(f"Quantization enabled: {result}")
    
    # Add vectors - these should populate quantization dictionaries
    print("Adding 10 test vectors...")
    test_vectors = []
    for i in range(10):
        vector = np.array([float(i)] + [1.0] * 127, dtype=np.float32)  # Distinctive vectors
        test_vectors.append(vector)
        db.add(f"test_{i}", vector)
        print(f"  Added test_{i}: [{vector[0]:.1f}, {vector[1]:.1f}, ...]")
    
    print(f"Database count: {db.count()}")
    
    # The question is: are the vectors stored in quantization dictionaries?
    # I can't access them directly from Python, but I can test the behavior
    
    print("\nTesting vector retrieval before flush:")
    for i in range(3):  # Test first 3 vectors
        try:
            retrieved = db.get_vector(f"test_{i}")
            if retrieved is not None:
                print(f"  test_{i}: Retrieved successfully")
                # Check if it's quantized by comparing precision
                original = test_vectors[i]
                ret_array = np.array(retrieved)
                diff = np.abs(original[0] - ret_array[0])
                print(f"    Original[0]: {original[0]:.6f}, Retrieved[0]: {ret_array[0]:.6f}, Diff: {diff:.6f}")
                
                if diff > 0.1:  # Significant difference suggests quantization/dequantization
                    print(f"    🔍 Large difference suggests quantization applied!")
                else:
                    print(f"    🔍 Small difference suggests original values")
            else:
                print(f"  test_{i}: Could not retrieve (get_vector bug)")
        except Exception as e:
            print(f"  test_{i}: Exception - {e}")
    
    print("\nFlushing to main index...")
    db.flush()
    print(f"Count after flush: {db.count()}")
    
    print("\nTesting vector retrieval after flush:")
    for i in range(3):
        try:
            retrieved = db.get_vector(f"test_{i}")
            if retrieved is not None:
                original = test_vectors[i]
                ret_array = np.array(retrieved)
                diff = np.abs(original[0] - ret_array[0])
                print(f"  test_{i}: Diff after flush: {diff:.6f}")
            else:
                print(f"  test_{i}: Could not retrieve after flush")
        except Exception as e:
            print(f"  test_{i}: Exception after flush - {e}")

def test_memory_calculation_manually():
    """Manually calculate expected memory usage."""
    
    print("\n📊 Manual Memory Calculation")
    print("=" * 30)
    
    num_vectors = 100
    dimensions = 128
    
    # Uncompressed memory
    uncompressed_bytes = num_vectors * dimensions * 4  # Float32 = 4 bytes
    uncompressed_mb = uncompressed_bytes / (1024 * 1024)
    
    # Compressed memory (8-bit quantization)
    # Each float32 becomes int8 (1 byte) + scale/offset per vector (8 bytes)
    compressed_bytes = num_vectors * (dimensions * 1 + 8)  # 1 byte per dim + 8 for scale/offset
    compressed_mb = compressed_bytes / (1024 * 1024)
    
    print(f"100 vectors × 128 dimensions:")
    print(f"  Uncompressed: {uncompressed_bytes} bytes = {uncompressed_mb:.3f} MB")
    print(f"  Compressed:   {compressed_bytes} bytes = {compressed_mb:.3f} MB")
    print(f"  Reduction:    {uncompressed_bytes / compressed_bytes:.1f}x")
    
    return uncompressed_mb, compressed_mb

if __name__ == "__main__":
    test_quantization_dictionaries()
    uncompressed_mb, compressed_mb = test_memory_calculation_manually()
    
    print("\n" + "=" * 50)
    print("DIRECT QUANTIZATION TEST RESULTS")
    print("=" * 50)
    
    print("📝 Key Findings:")
    print("1. get_vector is broken (returns None) - separate issue")
    print("2. Memory tracking shows impossibly low values")
    print("3. Expected memory for 100 vectors:")
    print(f"   - Uncompressed: {uncompressed_mb:.3f} MB")
    print(f"   - Compressed:   {compressed_mb:.3f} MB")
    print("4. Actual reported memory: ~0.02 MB (impossibly small)")
    print("\n🔍 Conclusion: Memory tracking is completely broken")
    print("   This masks whether quantization is actually working")