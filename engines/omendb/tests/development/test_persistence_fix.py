#!/usr/bin/env python3
"""Test persistence bug fix."""

import os
import sys
import numpy as np

# Use development version
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_persistence():
    """Test basic persistence cycle."""
    
    # Clean slate
    test_file = "/tmp/test_persistence_fix.omen"
    if os.path.exists(test_file):
        os.remove(test_file)
    
    print("=" * 60)
    print("TEST 1: Basic Persistence Cycle")
    print("=" * 60)
    
    # Create DB and configure persistence
    db = omendb.DB()
    result = db.set_persistence(test_file, use_wal=True)
    print(f"1. Persistence setup: {result}")
    
    # Add vectors
    vectors = np.random.rand(100, 128).astype(np.float32)
    print(f"Adding {len(vectors)} vectors...")
    ids = db.add_batch(vectors)
    print(f"2. Added {len(ids)} vectors, DB count: {db.count()}")
    
    # Small delay to ensure storage is ready
    import time
    time.sleep(0.1)
    
    # Manual checkpoint
    checkpoint_result = db.checkpoint()
    print(f"3. Checkpoint: {checkpoint_result}")
    
    # Verify file exists
    if os.path.exists(test_file):
        size = os.path.getsize(test_file)
        print(f"4. ✅ File created: {size:,} bytes")
    else:
        print("4. ❌ No persistence file")
        return False
    
    print("\n" + "=" * 60)
    print("TEST 2: Recovery Test")
    print("=" * 60)
    
    # Create new DB instance to test recovery
    db2 = omendb.DB()
    db2.set_persistence(test_file)
    print(f"1. Recovered DB count: {db2.count()}")
    
    # Verify data integrity
    if db2.count() == 100:
        print("2. ✅ Full recovery successful")
    else:
        print(f"2. ❌ Recovery incomplete: {db2.count()}/100")
        return False
    
    # Test search on recovered data
    query = np.random.rand(128).astype(np.float32)
    results = db2.search(query, limit=5)
    print(f"3. Search on recovered data: {len(results)} results")
    
    print("\n" + "=" * 60)
    print("TEST 3: Auto-Checkpoint Test")
    print("=" * 60)
    
    # Test auto-checkpoint during add_batch
    auto_file = "/tmp/auto_checkpoint_test.omen"
    if os.path.exists(auto_file):
        os.remove(auto_file)
    
    db3 = omendb.DB()
    db3.set_persistence(auto_file)
    
    # Add enough vectors to trigger auto-checkpoint (every 1000)
    large_vectors = np.random.rand(1500, 128).astype(np.float32)
    ids = db3.add_batch(large_vectors)
    print(f"1. Added {len(ids)} vectors (should auto-checkpoint)")
    
    # Verify persistence worked
    if os.path.exists(auto_file):
        size = os.path.getsize(auto_file)
        print(f"2. ✅ Auto-checkpoint worked: {size:,} bytes")
    else:
        print("2. ❌ Auto-checkpoint failed")
        return False
    
    print("\n" + "=" * 60)
    print("✅ ALL PERSISTENCE TESTS PASSED")
    print("=" * 60)
    return True

if __name__ == "__main__":
    success = test_persistence()
    sys.exit(0 if success else 1)