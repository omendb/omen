#!/usr/bin/env python3
"""Test beamwidth control functionality."""

import numpy as np
import time
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_beamwidth():
    """Test search with different beamwidth values."""
    
    # Create DB and add test vectors
    db = omendb.DB()
    
    # Add 1000 test vectors
    np.random.seed(42)
    vectors = np.random.rand(1000, 128).astype(np.float32)
    
    print("Adding 1000 test vectors...")
    for i in range(len(vectors)):
        db.add(f"vec_{i}", vectors[i])
    
    # Test query
    query = np.random.rand(128).astype(np.float32)
    
    print("\nTesting different beamwidth values:")
    print("-" * 50)
    
    # Test different beamwidth values
    beamwidths = [None, 10, 20, 50, 100, 200]
    
    for bw in beamwidths:
        start = time.perf_counter()
        
        if bw is None:
            results = db.search(query, limit=10)
            bw_str = "auto"
        else:
            results = db.search(query, limit=10, beamwidth=bw)
            bw_str = str(bw)
        
        elapsed = (time.perf_counter() - start) * 1000
        
        print(f"Beamwidth {bw_str:>5s}: {elapsed:6.2f} ms")
        
        # Show top result
        if results:
            print(f"  Top result: {results[0].id} (score: {results[0].score:.4f})")
    
    print("\nAnalysis:")
    print("- Auto beamwidth selects based on dataset size")
    print("- Higher beamwidth = better accuracy but slower")
    print("- Lower beamwidth = faster but may miss best matches")

if __name__ == "__main__":
    test_beamwidth()