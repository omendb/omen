#!/usr/bin/env python3
"""
Test adding vectors one by one to find exact crash point.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_individual_adds():
    """Add vectors individually to pinpoint crash."""
    
    print("Testing individual vector additions with quantization")
    print("=" * 60)
    
    db = omendb.DB(quantization="scalar", buffer_size=10000)
    
    dimension = 128
    max_vectors = 15000
    
    for i in range(max_vectors):
        try:
            vector = np.random.rand(dimension).astype(np.float32)
            db.add(f"vec_{i}", vector)
            
            if i % 1000 == 0:
                print(f"✓ Added {i} vectors")
            
            # Extra logging near expected crash
            if 9990 <= i <= 10010:
                print(f"  → Vector {i} added successfully")
                
        except Exception as e:
            print(f"\n❌ CRASH at vector {i}")
            print(f"Error: {e}")
            break
    else:
        print(f"\n✅ Successfully added all {max_vectors} vectors!")

if __name__ == "__main__":
    test_individual_adds()