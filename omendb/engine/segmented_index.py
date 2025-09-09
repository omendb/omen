#!/usr/bin/env python3
"""Segmented Index - Process-level parallelism without Mojo threading.

This gives us parallelism TODAY by using multiple Python processes,
each with its own OmenDB instance. Works around Mojo threading limitations.
"""

import numpy as np
from multiprocessing import Pool, cpu_count
import pickle
import time
from pathlib import Path
import sys

sys.path.insert(0, 'python')
from omendb import DB


class SegmentedDB:
    """Multi-process parallel database using index sharding.
    
    Each segment runs in its own process, giving us true parallelism
    without needing Mojo threading support.
    """
    
    def __init__(self, n_segments=None):
        """Initialize segmented database.
        
        Args:
            n_segments: Number of segments (default: CPU count)
        """
        self.n_segments = n_segments or cpu_count()
        self.segments = []
        
        # Create independent DB instances
        for i in range(self.n_segments):
            db = DB()
            db.clear()
            self.segments.append(db)
        
        print(f"Initialized {self.n_segments}-way segmented index")
        
    def add_batch(self, vectors, ids=None):
        """Add vectors distributed across segments.
        
        Round-robin distribution ensures even load balancing.
        """
        n_vectors = len(vectors)
        if ids is None:
            ids = [f"vec_{i}" for i in range(n_vectors)]
        
        # Distribute vectors across segments
        segment_data = [[] for _ in range(self.n_segments)]
        segment_ids = [[] for _ in range(self.n_segments)]
        
        for i, (vec, vid) in enumerate(zip(vectors, ids)):
            segment_idx = i % self.n_segments
            segment_data[segment_idx].append(vec)
            segment_ids[segment_idx].append(vid)
        
        # Add to each segment (could parallelize this too)
        total_added = 0
        for seg_idx, (seg_vecs, seg_ids) in enumerate(zip(segment_data, segment_ids)):
            if seg_vecs:
                self.segments[seg_idx].add_batch(
                    np.array(seg_vecs, dtype=np.float32),
                    ids=seg_ids
                )
                total_added += len(seg_vecs)
        
        return total_added
    
    def search_parallel(self, query, k=10):
        """Search across all segments in parallel.
        
        Uses multiprocessing to search all segments simultaneously.
        """
        # Each segment searches independently
        with Pool(self.n_segments) as pool:
            # Create search tasks
            tasks = [(seg, query, k) for seg in self.segments]
            
            # Execute parallel search
            segment_results = pool.starmap(_search_segment, tasks)
        
        # Merge results from all segments
        all_results = []
        for seg_results in segment_results:
            all_results.extend(seg_results)
        
        # Sort by distance and return top k
        all_results.sort(key=lambda x: x[1])
        return all_results[:k]
    
    def search_simple(self, query, k=10):
        """Simple sequential search (for comparison)."""
        all_results = []
        
        for segment in self.segments:
            results = segment.search(query, limit=k)
            for r in results:
                all_results.append((r.id, r.distance))
        
        # Sort and return top k
        all_results.sort(key=lambda x: x[1])
        return all_results[:k]
    
    def get_stats(self):
        """Get statistics about the segmented index."""
        stats = {
            'n_segments': self.n_segments,
            'vectors_per_segment': [],
            'total_vectors': 0
        }
        
        for i, seg in enumerate(self.segments):
            count = seg.count()
            stats['vectors_per_segment'].append(count)
            stats['total_vectors'] += count
        
        return stats


def _search_segment(segment, query, k):
    """Helper function for parallel search."""
    results = segment.search(query, limit=k)
    return [(r.id, r.distance) for r in results]


def benchmark_segmented():
    """Benchmark segmented vs single index."""
    
    print("Benchmarking Segmented Index")
    print("="*60)
    
    # Test data
    n_vectors = 4000  # Stay under 5K limit per segment
    vectors = np.random.random((n_vectors, 128)).astype(np.float32)
    query = vectors[0]
    
    # Test 1: Single index
    print("\n1. SINGLE INDEX (baseline):")
    single_db = DB()
    single_db.clear()
    
    start = time.time()
    single_db.add_batch(vectors)
    insert_time = time.time() - start
    print(f"   Insertion: {n_vectors/insert_time:.0f} vec/s")
    
    # Search performance
    search_times = []
    for _ in range(100):
        start = time.time()
        _ = single_db.search(query, limit=10)
        search_times.append(time.time() - start)
    
    avg_search = np.mean(search_times) * 1000
    print(f"   Search: {avg_search:.2f}ms")
    
    # Test 2: Segmented index
    print(f"\n2. SEGMENTED INDEX ({cpu_count()} segments):")
    seg_db = SegmentedDB()
    
    start = time.time()
    seg_db.add_batch(vectors)
    insert_time = time.time() - start
    print(f"   Insertion: {n_vectors/insert_time:.0f} vec/s")
    
    # Search performance (simple)
    search_times = []
    for _ in range(100):
        start = time.time()
        _ = seg_db.search_simple(query, k=10)
        search_times.append(time.time() - start)
    
    avg_search_simple = np.mean(search_times) * 1000
    print(f"   Search (sequential): {avg_search_simple:.2f}ms")
    
    # Stats
    stats = seg_db.get_stats()
    print(f"\n3. DISTRIBUTION:")
    print(f"   Vectors per segment: {stats['vectors_per_segment']}")
    print(f"   Total vectors: {stats['total_vectors']}")
    
    print(f"\n4. ANALYSIS:")
    print(f"   Search overhead: {(avg_search_simple/avg_search - 1)*100:.1f}%")
    print(f"   Effective parallelism: {cpu_count()}x potential with process pool")


if __name__ == "__main__":
    benchmark_segmented()