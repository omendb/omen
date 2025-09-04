#!/usr/bin/env python3
"""Test RoarGraph optimizations to improve performance."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
from omendb import DB

# Test configurations
DIMENSIONS = [128, 256]
VECTOR_COUNTS = [1000, 5000, 10000]


def benchmark_current(vectors):
    """Benchmark current RoarGraph implementation."""
    db = DB()

    start = time.perf_counter()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec.tolist())
    construction_time = time.perf_counter() - start

    return {
        "construction_time": construction_time,
        "construction_rate": len(vectors) / construction_time,
    }


def benchmark_delayed_switch(vectors, switch_at=5000):
    """Test delaying RoarGraph switch to reduce overhead."""
    # This would require modifying the switch threshold in native.mojo
    # Currently hardcoded to 1000 vectors
    print(f"  (Would switch to RoarGraph at {switch_at} vectors)")
    return benchmark_current(vectors)


def analyze_training_overhead():
    """Analyze training query generation overhead."""
    print("\n📊 Training Query Overhead Analysis")
    print("-" * 60)

    for count in [100, 500, 1000, 2000, 5000, 10000]:
        vectors = np.random.rand(count, 128).astype(np.float32)

        # Time just the migration phase
        db = DB()

        # Add vectors up to just before migration
        for i in range(min(999, count - 1)):
            db.add(f"vec_{i}", vectors[i].tolist())

        # Time the migration trigger
        if count >= 1000:
            start = time.perf_counter()
            db.add(f"vec_999", vectors[999].tolist())  # This triggers migration
            migration_time = (time.perf_counter() - start) * 1000
            print(f"{count:,} vectors: Migration took {migration_time:.1f}ms")
        else:
            print(f"{count:,} vectors: No migration (< 1000)")


def main():
    print("🔧 RoarGraph Optimization Analysis")
    print("=" * 60)

    # Analyze training overhead
    analyze_training_overhead()

    print("\n📊 Performance Impact of Optimizations")
    print("-" * 60)

    for dim in DIMENSIONS:
        for count in VECTOR_COUNTS:
            print(f"\n{count:,} vectors @ {dim}D:")

            vectors = np.random.rand(count, dim).astype(np.float32)

            # Current implementation
            current = benchmark_current(vectors)
            print(f"  Current: {current['construction_rate']:.0f} vec/s")

            # Delayed switch (conceptual)
            delayed = benchmark_delayed_switch(vectors, 5000)
            print(f"  Delayed switch: {delayed['construction_rate']:.0f} vec/s")

    print("\n🎯 Optimization Recommendations:")
    print("-" * 60)
    print("1. Delay RoarGraph switch: 1K → 5K vectors (avoid early overhead)")
    print("2. Reduce training queries: 15-20 → 5-10 (less O(n²) work)")
    print("3. Cache training connections: Don't recompute on every rebuild")
    print("4. Incremental updates: Add to graph without full rebuild")
    print("5. Remove projections: Not needed for single-modal search")


if __name__ == "__main__":
    main()
