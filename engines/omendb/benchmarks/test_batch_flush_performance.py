#!/usr/bin/env python3
"""
Test the performance improvement from batch flush implementation.
Compare performance at the 10K boundary where flush happens.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_flush_performance():
    """Test the improved batch flush performance."""
    
    print("=" * 60)
    print("BATCH FLUSH PERFORMANCE TEST")
    print("=" * 60)
    print()
    print("Testing flush performance at buffer boundaries...")
    print("Previous: 39ms per 100 vectors (one-by-one)")
    print("Target: < 1ms per 100 vectors (batch)")
    print()
    
    # Test configurations
    test_cases = [
        (10000, 10000),  # Exactly fill buffer, trigger flush
        (20000, 10000),  # Two flushes
        (15000, 5000),   # Three flushes
    ]
    
    dimension = 128
    
    for n_vectors, buffer_size in test_cases:
        print(f"Test: {n_vectors} vectors with buffer_size={buffer_size}")
        print("-" * 40)
        
        db = omendb.DB(buffer_size=buffer_size)
        
        # Generate all vectors upfront
        vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(n_vectors)]
        
        # Time the entire operation
        start = time.perf_counter()
        
        # Add in batches to measure flush performance
        batch_size = 1000
        flush_times = []
        
        for i in range(0, n_vectors, batch_size):
            batch_start = time.perf_counter()
            
            end_idx = min(i + batch_size, n_vectors)
            batch_vecs = vectors[i:end_idx]
            batch_ids = ids[i:end_idx]
            
            db.add_batch(batch_vecs, batch_ids)
            
            batch_time = time.perf_counter() - batch_start
            
            # Check if we crossed a flush boundary
            if i > 0 and i % buffer_size == 0:
                flush_times.append(batch_time * 1000)  # Convert to ms
                print(f"  Flush at {i}: {batch_time * 1000:.1f}ms")
        
        # Final stats
        total_time = time.perf_counter() - start
        rate = n_vectors / total_time
        
        print(f"  Total: {n_vectors} vectors in {total_time:.2f}s")
        print(f"  Rate: {rate:.0f} vec/s")
        
        if flush_times:
            avg_flush = sum(flush_times) / len(flush_times)
            print(f"  Average flush time: {avg_flush:.1f}ms")
            
            # Check if we met our target
            if avg_flush < 10:  # Target is < 1ms per 100 vectors, so < 10ms per 1000
                print(f"  ✅ PASS: Flush performance meets target!")
            else:
                print(f"  ⚠️  Flush still slow (target < 10ms)")
        
        print()
        
        # Clean up
        del db
    
    print("=" * 60)
    print("PERFORMANCE ANALYSIS")
    print("=" * 60)
    print()
    print("Expected improvements from batch flush:")
    print("1. Nodes added to graph in batch (faster allocation)")
    print("2. Single memory stats update (less overhead)")
    print("3. Better cache locality (batch operations)")
    print()
    print("If flush is still slow, remaining bottlenecks:")
    print("- Graph connection phase (_connect_node)")
    print("- Beam search for each node")
    print("- Edge pruning operations")

if __name__ == "__main__":
    test_flush_performance()