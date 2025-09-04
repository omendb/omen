#!/usr/bin/env python3
"""
Test Quantization Implementation in Detail
==========================================

Debug exactly where quantization is failing by testing each step.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_quantization_step_by_step():
    """Test quantization implementation step by step."""
    
    print("🔬 Detailed Quantization Test")
    print("=" * 35)
    
    # Create database and enable quantization
    db = omendb.DB()
    result = db.enable_quantization()
    print(f"Quantization enabled: {result}")
    
    if not result:
        print("❌ Quantization could not be enabled - stopping test")
        return False
    
    # Test with a distinctive vector that should quantize differently
    original = np.array([10.0, 20.0, 30.0] + [1.0] * 125, dtype=np.float32)
    print(f"Original vector sample: [{original[0]:.1f}, {original[1]:.1f}, {original[2]:.1f}, ...]")
    
    # Add vector
    vector_id = "test_quantized"
    db.add(vector_id, original)
    print(f"Added vector with ID: {vector_id}")
    
    # Try to retrieve immediately (should be from quantization dict)
    try:
        retrieved = db.get_vector(vector_id)
        
        if retrieved is None:
            print("❌ get_vector returned None - cannot test quantization")
            return False
            
        retrieved_array = np.array(retrieved)
        
        print(f"Retrieved vector sample: [{retrieved_array[0]:.6f}, {retrieved_array[1]:.6f}, {retrieved_array[2]:.6f}, ...]")
        
        # Check differences
        diffs = np.abs(original - retrieved_array)
        max_diff = np.max(diffs)
        mean_diff = np.mean(diffs)
        
        print(f"Max difference: {max_diff:.6f}")
        print(f"Mean difference: {mean_diff:.6f}")
        
        # If quantization is working, we should see some difference due to 8-bit precision
        if max_diff > 0.01:  # More than 1% difference
            print("✅ Quantization appears to be working - significant differences detected")
            return True
        elif max_diff > 0.0001:  # Very small difference
            print("⚠️ Small differences detected - quantization might be working but with high precision")
            return True
        else:
            print("🔴 No quantization detected - vectors are identical")
            
            # Additional debug: check expected quantization behavior manually
            print("\n📊 Manual Quantization Analysis:")
            min_val = float(np.min(original))
            max_val = float(np.max(original))
            scale = (max_val - min_val) / 255.0
            print(f"Expected scale: {scale:.6f}")
            print(f"Expected offset: {min_val:.6f}")
            
            # Simulate quantization
            quantized = np.round((original - min_val) / scale)
            dequantized = quantized * scale + min_val
            expected_diff = np.max(np.abs(original - dequantized))
            print(f"Expected max quantization error: {expected_diff:.6f}")
            
            if expected_diff < 0.0001:
                print("⚠️ Vector has very uniform values - quantization error may be negligible")
            else:
                print("🔴 Quantization should cause visible differences but doesn't - implementation bug")
            
            return False
            
    except Exception as e:
        print(f"❌ Error retrieving vector: {e}")
        return False

def test_with_high_dynamic_range():
    """Test quantization with a vector that has high dynamic range."""
    
    print("\n🎯 High Dynamic Range Test")
    print("=" * 28)
    
    db = omendb.DB()
    db.enable_quantization()
    
    # Create a vector with very high dynamic range to force quantization errors
    high_range = np.array([1000.0, 0.001, -500.0, 250.0] + [1.0] * 124, dtype=np.float32)
    print(f"High range vector: [{high_range[0]:.3f}, {high_range[1]:.3f}, {high_range[2]:.3f}, {high_range[3]:.3f}, ...]")
    
    db.add("high_range", high_range)
    
    try:
        retrieved = db.get_vector("high_range")
        if retrieved is not None:
            retrieved_array = np.array(retrieved)
            max_diff = np.max(np.abs(high_range - retrieved_array))
            print(f"Max difference with high range: {max_diff:.6f}")
            
            if max_diff > 1.0:  # Should be significant with 8-bit quantization
                print("✅ High dynamic range shows quantization effects")
                return True
            else:
                print("🔴 Even high dynamic range shows no quantization")
                return False
        else:
            print("❌ Could not retrieve high range vector")
            return False
    except Exception as e:
        print(f"❌ Error in high range test: {e}")
        return False

if __name__ == "__main__":
    # Test basic quantization
    basic_working = test_quantization_step_by_step()
    
    # Test with high dynamic range
    high_range_working = test_with_high_dynamic_range()
    
    print("\n" + "=" * 50)
    print("QUANTIZATION DETAILED TEST RESULTS")
    print("=" * 50)
    
    if basic_working or high_range_working:
        print("✅ Quantization is working in at least one test")
    else:
        print("🔴 Quantization is not working in any test")
        print("\n📋 Debugging Steps:")
        print("1. Check if quantization is actually being called during add()")
        print("2. Verify ScalarQuantizedVector.quantize() implementation")
        print("3. Check if quantized vectors are being used in get_vector()")
        print("4. Verify dequantization is working correctly")
        
    print(f"\n🔍 Next Steps:")
    print("- If quantization is working: Debug memory tracking")
    print("- If quantization is broken: Debug quantize() method")
    print("- Check if the issue is in add() path or get_vector() path")