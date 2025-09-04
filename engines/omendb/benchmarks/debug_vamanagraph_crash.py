#!/usr/bin/env python3
"""
Minimal test to reproduce and debug VamanaGraph crash at ~20K vectors.
This test incrementally adds vectors and reports exactly where the crash occurs.
"""

import sys
import numpy as np
import time
import gc
import traceback
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_incremental_crash():
    """Add vectors incrementally to find exact crash point."""
    
    print("=" * 60)
    print("VAMANAGRAPH CRASH DEBUGGING")
    print("=" * 60)
    print()
    
    # Use quantization to trigger VamanaGraph
    db = omendb.DB(quantization="scalar", buffer_size=1000)  # Small buffer to trigger flushes
    
    dimension = 128
    batch_size = 100
    max_vectors = 30000  # Should crash before this
    
    print(f"Config: dim={dimension}, batch={batch_size}, quantization=True")
    print(f"Buffer size: 1000 (to trigger frequent flushes)")
    print()
    
    last_successful = 0
    
    try:
        for i in range(0, max_vectors, batch_size):
            # Generate batch
            vectors = np.random.rand(batch_size, dimension).astype(np.float32)
            ids = [f"vec_{j}" for j in range(i, i + batch_size)]
            
            # Try to add batch
            try:
                db.add_batch(vectors, ids)
                last_successful = i + batch_size
                
                # Report progress at key points
                if last_successful % 1000 == 0:
                    print(f"✓ {last_successful:6d} vectors added successfully")
                
                # Extra reporting near expected crash
                if 18000 <= last_successful <= 22000:
                    if last_successful % 100 == 0:
                        print(f"  → {last_successful} vectors OK (watching for crash)")
                
                # Force garbage collection periodically
                if last_successful % 5000 == 0:
                    gc.collect()
                    
            except Exception as e:
                print(f"\n❌ CRASH at {i} vectors (after {last_successful} successful)")
                print(f"Error: {e}")
                print("\nTraceback:")
                traceback.print_exc()
                
                # Try to get more info
                print(f"\nLast successful batch: {last_successful - batch_size} to {last_successful}")
                print(f"Failed batch: {i} to {i + batch_size}")
                
                # Test if we can still query
                print("\nTesting if database is still queryable...")
                try:
                    query = np.random.rand(dimension).astype(np.float32)
                    results = db.search(query, k=5)
                    print(f"  → Search works, found {len(results)} results")
                except:
                    print(f"  → Search also crashes")
                
                break
                
    except KeyboardInterrupt:
        print(f"\nInterrupted at {last_successful} vectors")
    
    print()
    print("=" * 60)
    print("CRASH ANALYSIS")
    print("=" * 60)
    print(f"Last successful count: {last_successful}")
    print(f"Crash occurred between {last_successful} and {last_successful + batch_size}")
    print()
    print("Likely causes:")
    print("1. Memory allocation failure in CSRGraph")
    print("2. Array bounds violation in graph operations")
    print("3. Integer overflow in index calculations")
    print("4. Unsafe memory access in quantization")

def test_direct_flush_crash():
    """Test if crash happens specifically during flush."""
    
    print("\n" + "=" * 60)
    print("TESTING FLUSH OPERATION DIRECTLY")
    print("=" * 60)
    print()
    
    db = omendb.DB(quantization="scalar", buffer_size=20000)  # Buffer right at crash point
    
    dimension = 128
    n = 20000  # Exactly at crash point
    
    print(f"Adding {n} vectors to buffer (no flush yet)...")
    
    # Fill buffer exactly to limit
    vectors = np.random.rand(n, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n)]
    
    try:
        # This should fill buffer but not flush
        for i in range(0, n-1, 1000):
            batch = vectors[i:i+1000]
            batch_ids = ids[i:i+1000]
            db.add_batch(batch, batch_ids)
            print(f"  Added {i+1000} vectors to buffer")
        
        print(f"\nBuffer full with {n-1} vectors. Next add will trigger flush...")
        print("Adding one more vector to trigger flush...")
        
        # This single add should trigger flush
        trigger_vector = np.random.rand(dimension).astype(np.float32)
        db.add(f"trigger_{n}", trigger_vector)
        
        print("✓ Flush succeeded!")
        
    except Exception as e:
        print(f"\n❌ CRASH during flush!")
        print(f"Error: {e}")
        traceback.print_exc()

if __name__ == "__main__":
    print("Testing VamanaGraph crash at ~20K vectors\n")
    
    # Test 1: Find exact crash point
    test_incremental_crash()
    
    # Test 2: Test flush directly
    test_direct_flush_crash()