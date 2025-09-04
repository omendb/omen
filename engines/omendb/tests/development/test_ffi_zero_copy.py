#!/usr/bin/env python3
"""Test FFI zero-copy performance."""

import numpy as np
import time
from python.omendb import OmenDB

def test_zero_copy_performance():
    """Test if zero-copy is actually working."""
    
    db = OmenDB(dimension=128)
    
    # Create test data
    n_vectors = 10000
    vectors = np.random.rand(n_vectors, 128).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]
    
    # Test with numpy arrays (should use zero-copy)
    start = time.perf_counter()
    for i in range(n_vectors):
        db.add(ids[i], vectors[i])
    numpy_time = time.perf_counter() - start
    
    # Clear database
    db = OmenDB(dimension=128)
    
    # Test with Python lists (forces copy)
    start = time.perf_counter()
    for i in range(n_vectors):
        db.add(ids[i], vectors[i].tolist())
    list_time = time.perf_counter() - start
    
    print(f"Numpy arrays: {n_vectors/numpy_time:.0f} vec/s")
    print(f"Python lists: {n_vectors/list_time:.0f} vec/s")
    print(f"Speedup: {list_time/numpy_time:.1f}x")
    
    # If zero-copy is working, numpy should be significantly faster
    if numpy_time < list_time * 0.8:
        print("✅ Zero-copy appears to be working!")
    else:
        print("❌ Zero-copy may not be working properly")
    
    return numpy_time, list_time

if __name__ == "__main__":
    test_zero_copy_performance()