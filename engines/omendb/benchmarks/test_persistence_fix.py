#!/usr/bin/env python3
"""Test if memory-mapped storage persistence fix works."""

import numpy as np
import sys
import os
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb
import omendb.native as native

def test_persistence_fix():
    """Test if persistence now works after fix."""
    print("🧪 Testing Memory-Mapped Storage Fix")
    print("=" * 60)
    
    path = "/tmp/test_persistence_fix"
    
    # Clean up old files
    for ext in ['.vectors', '.graph', '.meta']:
        if os.path.exists(path + ext):
            os.remove(path + ext)
    
    # Part 1: Save vectors
    print("Part 1: Saving vectors...")
    native._reset()
    native.enable_memory_mapped_storage()
    
    db = omendb.DB()
    db.set_persistence(path)
    
    # Add test vectors with unique markers
    test_count = 250
    for i in range(test_count):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)  # Mark with unique ID
        vector[1] = float(i * 2)  # Second marker for validation
        db.add(f"test_{i}", vector)
    
    print(f"  Added {test_count} vectors")
    print(f"  Count: {db.count()}")
    
    # Checkpoint
    print("\nCheckpointing...")
    success = db.checkpoint()
    print(f"  Checkpoint result: {success}")
    
    # Check file sizes
    vector_file = path + ".vectors"
    if os.path.exists(vector_file):
        size = os.path.getsize(vector_file)
        print(f"  Vector file size: {size:,} bytes")
        expected_min = test_count * 128 * 4  # vectors * dims * float32
        if size >= expected_min:
            print(f"  ✅ File size looks correct (>= {expected_min:,} bytes)")
        else:
            print(f"  ❌ File too small (expected >= {expected_min:,} bytes)")
    else:
        print("  ❌ Vector file not created")
        return False
    
    # Part 2: Recovery
    print("\nPart 2: Testing recovery...")
    native._reset()
    native.enable_memory_mapped_storage()
    
    db2 = omendb.DB()
    db2.set_persistence(path)
    
    recovered_count = db2.count()
    print(f"  Recovered: {recovered_count}/{test_count} vectors")
    
    # Test retrieval of specific vectors
    print("\n  Testing retrieval of recovered vectors...")
    errors = 0
    for i in [0, 50, 100, 150, 200, 249]:  # Sample vectors
        vec_id = f"test_{i}"
        retrieved = db2.get(vec_id)
        
        if retrieved is None:
            print(f"    ❌ Missing: {vec_id}")
            errors += 1
        else:
            # Check markers
            first_val = retrieved[0] if not isinstance(retrieved[0], list) else retrieved[0][0]
            second_val = retrieved[1] if not isinstance(retrieved[1], list) else retrieved[1][0]
            
            # Note: Values might be normalized, so check ratio
            if second_val != 0 and abs(second_val / first_val - 2.0) < 0.1:
                print(f"    ✅ {vec_id}: markers preserved (ratio check)")
            else:
                print(f"    ❌ {vec_id}: wrong values [{first_val:.2f}, {second_val:.2f}]")
                errors += 1
    
    # Final verdict
    print("\n" + "=" * 60)
    if recovered_count == test_count and errors == 0:
        print("🎉 PERSISTENCE FIXED! All vectors recovered correctly")
        return True
    elif recovered_count > 0:
        print(f"⚠️ PARTIAL SUCCESS: Recovered {recovered_count}/{test_count} vectors")
        return False
    else:
        print("❌ PERSISTENCE STILL BROKEN: No vectors recovered")
        return False

if __name__ == "__main__":
    success = test_persistence_fix()
    
    if not success:
        print("\nDebugging: Let's check what's in memory-mapped storage...")
        # Additional debugging could go here