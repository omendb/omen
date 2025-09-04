#!/usr/bin/env python3
"""
Memory profiling test to identify exact leak sources in OmenDB.
"""

import gc
import psutil
import os
import sys
import time
import tracemalloc

# Add parent directory to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../../")))

import omendb
import numpy as np


def get_memory_usage():
    """Get current process memory usage in MB."""
    process = psutil.Process()
    return process.memory_info().rss / 1024 / 1024


def profile_db_creation():
    """Profile memory usage during DB creation/destruction."""
    print("\n=== Testing DB Creation/Destruction ===")

    # Start memory tracking
    tracemalloc.start()
    initial_memory = get_memory_usage()

    # Test 1: Simple creation/deletion
    print("\nTest 1: Simple DB lifecycle")
    for i in range(10):
        db = omendb.DB()
        del db
        gc.collect()

        if i % 5 == 0:
            current_memory = get_memory_usage()
            print(
                f"  Iteration {i}: {current_memory:.1f} MB (delta: {current_memory - initial_memory:.1f} MB)"
            )

    # Test 2: DB with vectors
    print("\nTest 2: DB with vectors")
    initial_memory = get_memory_usage()

    for i in range(10):
        db = omendb.DB()
        # Add some vectors
        for j in range(100):
            vec = np.random.rand(128).astype(np.float32).tolist()
            db.add(f"vec_{j}", vec)
        del db
        gc.collect()

        current_memory = get_memory_usage()
        print(
            f"  Iteration {i}: {current_memory:.1f} MB (delta: {current_memory - initial_memory:.1f} MB)"
        )

    # Get top memory allocations
    snapshot = tracemalloc.take_snapshot()
    top_stats = snapshot.statistics("lineno")

    print("\nTop 5 memory allocations:")
    for stat in top_stats[:5]:
        print(f"  {stat}")


def profile_vector_operations():
    """Profile memory during vector operations."""
    print("\n=== Testing Vector Operations ===")

    db = omendb.DB()
    initial_memory = get_memory_usage()

    # Test adding vectors in batches
    print("\nAdding vectors in batches:")
    for batch in range(5):
        vectors = []
        ids = []
        for i in range(1000):
            vec = np.random.rand(128).astype(np.float32).tolist()
            vectors.append(vec)
            ids.append(f"batch_{batch}_vec_{i}")

        # Use add_batch for batch operations
        batch_data = [(id, vec, None) for id, vec in zip(ids, vectors)]
        db.add_batch(batch_data)
        current_memory = get_memory_usage()
        print(
            f"  After batch {batch} (total {(batch + 1) * 1000} vectors): {current_memory:.1f} MB (delta: {current_memory - initial_memory:.1f} MB)"
        )

    # Test searching
    print("\nSearching vectors:")
    search_initial = get_memory_usage()

    for i in range(100):
        query = np.random.rand(128).astype(np.float32).tolist()
        results = db.search(query, limit=10)

        if i % 20 == 0:
            current_memory = get_memory_usage()
            print(
                f"  After {i} searches: {current_memory:.1f} MB (delta: {current_memory - search_initial:.1f} MB)"
            )

    del db
    gc.collect()


def profile_hnsw_migration():
    """Profile memory during HNSW migration."""
    print("\n=== Testing HNSW Migration ===")

    db = omendb.DB()
    initial_memory = get_memory_usage()

    # Add vectors up to migration threshold
    print("\nAdding vectors to trigger HNSW migration:")

    # Add 4,500 vectors (below threshold)
    vectors = []
    ids = []
    for i in range(4500):
        vec = np.random.rand(128).astype(np.float32).tolist()
        vectors.append(vec)
        ids.append(f"vec_{i}")

    db.add_many(ids, vectors)
    before_migration = get_memory_usage()
    print(f"  Before migration (4,500 vectors): {before_migration:.1f} MB")

    # Add 1,000 more to trigger migration
    vectors = []
    ids = []
    for i in range(4500, 5500):
        vec = np.random.rand(128).astype(np.float32).tolist()
        vectors.append(vec)
        ids.append(f"vec_{i}")

    db.add_many(ids, vectors)
    after_migration = get_memory_usage()
    print(f"  After migration (5,500 vectors): {after_migration:.1f} MB")
    print(f"  Migration memory cost: {after_migration - before_migration:.1f} MB")

    # Check if we're duplicating data
    stats = db.info()
    print(f"\nDatabase stats after migration:")
    print(f"  Total vectors: {stats['total_vectors']}")
    print(f"  Using brute force: {stats.get('using_brute_force', 'N/A')}")

    del db
    gc.collect()


def main():
    """Run all memory profiling tests."""
    print("OmenDB Memory Profiling")
    print("=" * 50)

    profile_db_creation()
    profile_vector_operations()
    profile_hnsw_migration()

    # Final memory check
    gc.collect()
    time.sleep(1)  # Let cleanup finish
    final_memory = get_memory_usage()
    print(f"\n\nFinal memory usage: {final_memory:.1f} MB")


if __name__ == "__main__":
    main()
