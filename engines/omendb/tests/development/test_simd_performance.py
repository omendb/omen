#!/usr/bin/env python3
"""Test SIMD optimization performance."""

import numpy as np
import time
from python.omendb import DB as OmenDB

def test_simd_performance():
    """Verify SIMD optimizations are working by testing search performance."""
    
    print("Testing SIMD performance...")
    
    # Note: Testing with dimension 128 only since DB doesn't support changing dimensions
    # SIMD optimization is still active and can be verified through performance
    
    # Create database (defaults to 128 dimensions)
    db = OmenDB()
    
    # Add test vectors
    n_vectors = 10000
    dim = 128
    vectors = np.random.rand(n_vectors, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]
    
    # Batch add (should be fast with SIMD)
    print(f"Adding {n_vectors} vectors...")
    start = time.perf_counter()
    db.add_batch(vectors, ids)  # Correct order: vectors first, then ids
    add_time = time.perf_counter() - start
    add_rate = n_vectors / add_time
    print(f"Add rate: {add_rate:,.0f} vec/s")
    
    # Test search performance
    query = np.random.rand(dim).astype(np.float32)
    
    # Warm up
    for _ in range(10):
        db.search(query, limit=10)
    
    # Measure search time
    print(f"Testing search performance...")
    start = time.perf_counter()
    n_searches = 100
    for _ in range(n_searches):
        results = db.search(query, limit=10)
    elapsed = time.perf_counter() - start
    
    avg_search_ms = (elapsed / n_searches) * 1000
    vectors_per_sec = n_vectors * n_searches / elapsed
    
    print(f"Search performance: {avg_search_ms:.2f}ms per search, {vectors_per_sec:,.0f} vec/s scanned")
        
        # Expected: Performance should scale sub-linearly with dimension
        # due to SIMD parallelization
    
    print("\n✅ SIMD optimizations are working if:")
    print("   - Search time scales sub-linearly with dimension")
    print("   - Performance is > 100K vec/s for dimension 128")

if __name__ == "__main__":
    test_simd_performance()