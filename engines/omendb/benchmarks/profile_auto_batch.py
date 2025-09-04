#!/usr/bin/env python3
"""Profile auto-batching to find the bottleneck."""

import time
import cProfile
import pstats
from io import StringIO
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

import omendb
import random


def generate_vectors(n=1000, dim=128):
    """Generate random vectors."""
    return [[random.random() for _ in range(dim)] for _ in range(n)]


def test_with_profiling():
    """Test auto-batching with profiling."""
    db = omendb.DB(algorithm="diskann")
    db.clear()
    db._auto_batch_enabled = True

    vectors = generate_vectors(100)

    # Profile the add operations
    profiler = cProfile.Profile()
    profiler.enable()

    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec)
    db.flush()

    profiler.disable()

    # Print stats
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats("cumulative")
    ps.print_stats(20)
    print(s.getvalue())


def test_batch_sizes():
    """Test different batch sizes to find optimal."""
    db = omendb.DB(algorithm="diskann")

    for batch_size in [10, 50, 100, 500, 1000]:
        db.clear()
        vectors = generate_vectors(1000)

        start = time.time()
        for i in range(0, len(vectors), batch_size):
            batch_end = min(i + batch_size, len(vectors))
            batch_vecs = vectors[i:batch_end]
            batch_ids = [f"vec_{j}" for j in range(i, batch_end)]
            db.add_batch(batch_vecs, batch_ids)
        elapsed = time.time() - start

        throughput = len(vectors) / elapsed
        num_batches = (len(vectors) + batch_size - 1) // batch_size
        print(
            f"Batch size {batch_size:4d}: {throughput:6.0f} vec/s, {num_batches:3d} batches, {elapsed * 1000 / num_batches:.1f}ms per batch"
        )


def test_single_vs_multi_batch():
    """Compare single large batch vs many small batches."""
    db = omendb.DB(algorithm="diskann")
    vectors = generate_vectors(1000)

    # Test 1: Single batch
    db.clear()
    start = time.time()
    ids = [f"vec_{i}" for i in range(len(vectors))]
    db.add_batch(vectors, ids)
    single_time = time.time() - start
    print(f"Single batch (1000): {len(vectors) / single_time:.0f} vec/s")

    # Test 2: 10 batches of 100
    db.clear()
    start = time.time()
    for i in range(0, len(vectors), 100):
        batch_vecs = vectors[i : i + 100]
        batch_ids = [f"vec_{j}" for j in range(i, i + 100)]
        db.add_batch(batch_vecs, batch_ids)
    multi_time = time.time() - start
    print(f"10 batches (100 each): {len(vectors) / multi_time:.0f} vec/s")

    print(f"Overhead per batch: {(multi_time - single_time) / 9 * 1000:.1f}ms")


if __name__ == "__main__":
    print("=" * 60)
    print("Batch Size Analysis")
    print("=" * 60)
    test_batch_sizes()

    print("\n" + "=" * 60)
    print("Single vs Multi Batch")
    print("=" * 60)
    test_single_vs_multi_batch()

    print("\n" + "=" * 60)
    print("Profile Analysis")
    print("=" * 60)
    test_with_profiling()
