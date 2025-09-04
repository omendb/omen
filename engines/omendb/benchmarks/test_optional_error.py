#!/usr/bin/env python3
"""
Minimal test to find the .value() on empty Optional error.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_optional_error():
    """Test adding exactly 10K vectors with buffer_size=10000."""
    
    print("Testing .value() on empty Optional error")
    print("=" * 60)
    
    # This configuration triggers the error
    db = omendb.DB(quantization="scalar", buffer_size=10000)
    
    dimension = 128
    n_vectors = 20000  # Test at the crash point
    
    try:
        # Add vectors in batches of 1000 like the scale test
        batch_size = 1000
        print(f"Adding {n_vectors} vectors in batches of {batch_size}...")
        
        for i in range(0, n_vectors, batch_size):
            end_idx = min(i + batch_size, n_vectors)
            batch_vectors = np.random.rand(end_idx - i, dimension).astype(np.float32)
            batch_ids = [f"vec_{j}" for j in range(i, end_idx)]
            
            db.add_batch(batch_vectors, batch_ids)
            if (i + batch_size) % 5000 == 0:
                print(f"  Added {i + batch_size} vectors")
        
        print("✅ Success adding vectors!")
        
        # Now try search
        print("Testing search...")
        query = np.random.rand(dimension).astype(np.float32)
        results = db.search(query)  # Note: no k parameter
        print(f"✅ Search returned {len(results)} results")
        
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_optional_error()