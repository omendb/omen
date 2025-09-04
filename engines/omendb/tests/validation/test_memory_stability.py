#!/usr/bin/env python3
"""Memory leak detection for OmenDB - critical for production readiness."""

import gc
import time
import psutil
import os
import sys

sys.path.insert(0, "python")


def get_memory_usage():
    """Get current process memory usage in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024


def test_repeated_db_creation():
    """Test repeated DB creation/destruction for leaks."""
    import omendb

    print("Testing repeated DB creation...")
    initial_memory = get_memory_usage()

    for i in range(100):
        db = omendb.DB()
        db.add("test", [1.0] * 128)
        del db
        gc.collect()

        if i % 10 == 0:
            current_memory = get_memory_usage()
            print(
                f"  Iteration {i}: {current_memory:.1f} MB (Δ{current_memory - initial_memory:+.1f} MB)"
            )

    final_memory = get_memory_usage()
    leak = final_memory - initial_memory
    print(f"Memory leak: {leak:.1f} MB over 100 iterations")
    return leak < 10  # Allow up to 10MB growth


def test_large_dataset_churn():
    """Test adding/removing large batches repeatedly."""
    import omendb
    import numpy as np

    print("\nTesting large dataset churn...")
    db = omendb.DB()
    initial_memory = get_memory_usage()

    for cycle in range(10):
        # Add 10K vectors
        vectors = np.random.rand(10000, 128).astype(np.float32)
        batch = [(f"vec_{cycle}_{i}", vec.tolist()) for i, vec in enumerate(vectors)]
        db.add_batch(batch)

        # Query repeatedly
        for _ in range(100):
            db.search(vectors[0].tolist(), limit=10)

        current_memory = get_memory_usage()
        print(
            f"  Cycle {cycle}: {current_memory:.1f} MB (Δ{current_memory - initial_memory:+.1f} MB)"
        )

        # Force garbage collection
        del vectors, batch
        gc.collect()

    final_memory = get_memory_usage()
    leak = final_memory - initial_memory
    print(f"Memory leak: {leak:.1f} MB over 10 cycles")
    return leak < 50  # Allow up to 50MB growth for 100K vectors


def test_query_memory_stability():
    """Test memory usage during intensive querying."""
    import omendb
    import numpy as np

    print("\nTesting query memory stability...")
    db = omendb.DB()

    # Add base dataset
    vectors = np.random.rand(1000, 128).astype(np.float32)
    batch = [(f"vec_{i}", vec.tolist()) for i, vec in enumerate(vectors)]
    db.add_batch(batch)

    initial_memory = get_memory_usage()

    # Query intensively
    for i in range(10000):
        query = np.random.rand(128).astype(np.float32).tolist()
        results = db.search(query, limit=10)

        if i % 1000 == 0:
            current_memory = get_memory_usage()
            print(
                f"  Query {i}: {current_memory:.1f} MB (Δ{current_memory - initial_memory:+.1f} MB)"
            )

    final_memory = get_memory_usage()
    leak = final_memory - initial_memory
    print(f"Memory leak: {leak:.1f} MB over 10K queries")
    return leak < 5  # Should have minimal growth during queries


if __name__ == "__main__":
    print("🔍 OmenDB Memory Leak Detection")
    print("=" * 50)

    tests_passed = 0

    # Run tests
    if test_repeated_db_creation():
        print("✅ Repeated DB creation: PASSED\n")
        tests_passed += 1
    else:
        print("❌ Repeated DB creation: FAILED\n")

    if test_large_dataset_churn():
        print("✅ Large dataset churn: PASSED\n")
        tests_passed += 1
    else:
        print("❌ Large dataset churn: FAILED\n")

    if test_query_memory_stability():
        print("✅ Query memory stability: PASSED\n")
        tests_passed += 1
    else:
        print("❌ Query memory stability: FAILED\n")

    print(
        f"\n{'✅' if tests_passed == 3 else '❌'} Overall: {tests_passed}/3 tests passed"
    )

    if tests_passed < 3:
        print("\n⚠️  Memory leaks detected - not production ready!")
        sys.exit(1)
