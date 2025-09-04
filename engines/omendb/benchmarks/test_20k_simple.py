#!/usr/bin/env python3
"""
Simple test at 20K to isolate the crash.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_20k():
    """Test adding exactly 20K vectors."""
    
    print("Testing 20K vectors without quantization...")
    
    # Test without quantization first
    db = omendb.DB(buffer_size=10000)  # No quantization
    
    dimension = 128
    n_vectors = 20000
    batch_size = 1000
    
    try:
        for i in range(0, n_vectors, batch_size):
            end_idx = min(i + batch_size, n_vectors)
            batch_vectors = np.random.rand(end_idx - i, dimension).astype(np.float32)
            batch_ids = [f"vec_{j}" for j in range(i, end_idx)]
            
            print(f"Adding batch {i//batch_size + 1}: vectors {i}-{end_idx-1}")
            db.add_batch(batch_vectors, batch_ids)
            
            if i == 9000:
                print("  -> About to trigger first flush at 10K...")
            elif i == 10000:
                print("  -> First flush completed, continuing...")
            elif i == 19000:
                print("  -> About to trigger second flush at 20K...")
        
        print("✅ SUCCESS: Added all 20K vectors without crash!")
        
    except Exception as e:
        print(f"❌ CRASH: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    test_20k()