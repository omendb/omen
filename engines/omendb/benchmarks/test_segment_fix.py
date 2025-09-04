#!/usr/bin/env python3
"""Test if segment merging fix works."""

import numpy as np
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_segment_merge_fix():
    """Test if the segment merge fix prevents duplicates."""
    print("🧪 Testing Segment Merge Fix")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    # Track all IDs we add
    all_ids = []
    
    # Add first batch
    print("Adding first 5000 vectors...")
    for i in range(5000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)  # Mark with unique ID
        db.add(f"vec_{i}", vector)
        all_ids.append(f"vec_{i}")
    
    print(f"  Count before flush: {db.count()}")
    db.flush()
    print(f"  Count after flush: {db.count()}")
    
    # Add second batch
    print("\nAdding second 5000 vectors...")
    for i in range(5000, 10000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)
        db.add(f"vec_{i}", vector)
        all_ids.append(f"vec_{i}")
    
    print(f"  Count before flush: {db.count()}")
    db.flush()
    print(f"  Count after flush: {db.count()}")
    
    # Add third batch
    print("\nAdding third 5000 vectors...")
    for i in range(10000, 15000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)
        db.add(f"vec_{i}", vector)
        all_ids.append(f"vec_{i}")
    
    print(f"  Count before flush: {db.count()}")
    db.flush()
    print(f"  Count after flush: {db.count()}")
    
    # Final check
    print("\n📊 Final Results:")
    print(f"  Expected count: {len(all_ids)}")
    print(f"  Actual count: {db.count()}")
    
    if db.count() == len(all_ids):
        print("  ✅ Count matches! No duplicates")
    else:
        print(f"  ❌ Count mismatch: {db.count() - len(all_ids):+d} difference")
    
    # Test retrieval
    print("\n🔍 Testing retrieval...")
    errors = 0
    for i in range(0, 15000, 1000):  # Sample every 1000th vector
        vec_id = f"vec_{i}"
        retrieved = db.get(vec_id)
        
        if retrieved is None:
            print(f"  ❌ Missing: {vec_id}")
            errors += 1
        elif retrieved:
            # Check value
            first_val = retrieved[0] if not isinstance(retrieved[0], list) else retrieved[0][0]
            if abs(float(first_val) - i) > 0.01:
                print(f"  ❌ Wrong value for {vec_id}: expected {i}, got {first_val}")
                errors += 1
    
    if errors == 0:
        print("  ✅ All sampled vectors retrieved correctly")
    else:
        print(f"  ❌ {errors} retrieval errors")
    
    return db.count() == len(all_ids) and errors == 0

if __name__ == "__main__":
    success = test_segment_merge_fix()
    
    if success:
        print("\n✅ SEGMENT MERGE FIX SUCCESSFUL!")
    else:
        print("\n❌ Segment merge still has issues")