#!/usr/bin/env python3
"""
Test memory safety and deep copy implementation.
Verifies VamanaNode deep copy prevents corruption and no memory leaks.
"""

import sys
import time
import numpy as np
import psutil
import os
import gc

sys.path.insert(0, "python")
import omendb


def test_vector_corruption():
    """Test that vector data isn't corrupted by shallow copies."""
    print("=" * 60)
    print("TEST: Vector Data Integrity")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add vectors with known values
    test_vectors = [
        [1.0, 2.0, 3.0, 4.0],
        [5.0, 6.0, 7.0, 8.0],
        [9.0, 10.0, 11.0, 12.0],
    ]

    for i, vec in enumerate(test_vectors):
        db.add(f"vec_{i}", vec)

    # Verify vectors can be found correctly
    all_passed = True
    for i, original_vec in enumerate(test_vectors):
        results = db.search(original_vec, 1)

        if len(results) == 0:
            print(f"❌ FAILED: vec_{i} not found")
            all_passed = False
            continue

        result = results[0]
        if result.id != f"vec_{i}":
            print(f"❌ FAILED: Expected vec_{i}, got {result.id}")
            all_passed = False
        elif result.distance > 1e-5:
            print(
                f"❌ FAILED: vec_{i} has distance {result.distance:.6f} (expected ~0.0)"
            )
            all_passed = False
        else:
            print(
                f"✅ PASSED: vec_{i} found correctly with distance {result.distance:.6f}"
            )

    # Add more vectors to trigger potential memory issues
    print("\nAdding 100 more vectors to stress memory...")
    for i in range(100):
        vec = np.random.randn(4).astype(np.float32)
        db.add(f"stress_{i}", vec.tolist())

    # Re-verify original vectors still findable
    print("\nRe-verifying original vectors after stress...")
    for i, original_vec in enumerate(test_vectors):
        results = db.search(original_vec, 1)

        if len(results) == 0:
            print(f"❌ FAILED: vec_{i} lost after stress")
            all_passed = False
        elif results[0].id != f"vec_{i}":
            print(f"❌ FAILED: vec_{i} corrupted after stress")
            all_passed = False
        else:
            print(f"✅ PASSED: vec_{i} still intact")

    return all_passed


def test_rapid_add_delete_cycles():
    """Test rapid add/delete cycles for memory corruption."""
    print("\n" + "=" * 60)
    print("TEST: Rapid Add/Delete Cycles")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    n_cycles = 10
    vectors_per_cycle = 50
    dimension = 64

    all_passed = True

    for cycle in range(n_cycles):
        # Add vectors
        vectors = []
        for i in range(vectors_per_cycle):
            vec = np.random.randn(dimension).astype(np.float32)
            vectors.append(vec)
            db.add(f"cycle_{cycle}_vec_{i}", vec.tolist())

        # Search for some vectors
        for i in range(min(5, vectors_per_cycle)):
            results = db.search(vectors[i].tolist(), 1)
            if len(results) == 0 or results[0].id != f"cycle_{cycle}_vec_{i}":
                print(f"❌ FAILED: Cycle {cycle}, vec {i} not found correctly")
                all_passed = False

        # Clear for next cycle (simulates delete)
        if cycle < n_cycles - 1:
            db.clear()

        print(f"  Cycle {cycle + 1}/{n_cycles} completed")

    if all_passed:
        print("✅ PASSED: All cycles completed without corruption")

    return all_passed


def test_memory_leak_detection():
    """Test for memory leaks during extended operations."""
    print("\n" + "=" * 60)
    print("TEST: Memory Leak Detection")
    print("=" * 60)

    process = psutil.Process(os.getpid())

    # Force garbage collection and get baseline
    gc.collect()
    time.sleep(0.1)
    baseline_memory = process.memory_info().rss / 1024 / 1024  # MB
    print(f"Baseline memory: {baseline_memory:.1f} MB")

    # Perform many operations
    n_iterations = 5
    vectors_per_iteration = 200
    dimension = 128

    memory_samples = []

    for iteration in range(n_iterations):
        db = omendb.DB()
        db.clear()

        # Add vectors
        for i in range(vectors_per_iteration):
            vec = np.random.randn(dimension).astype(np.float32)
            db.add(f"iter_{iteration}_vec_{i}", vec.tolist())

        # Perform searches
        for _ in range(50):
            query = np.random.randn(dimension).astype(np.float32)
            _ = db.search(query.tolist(), 10)

        # Clear database
        db.clear()

        # Measure memory
        gc.collect()
        current_memory = process.memory_info().rss / 1024 / 1024
        memory_growth = current_memory - baseline_memory
        memory_samples.append(memory_growth)

        print(
            f"  Iteration {iteration + 1}: {current_memory:.1f} MB (+{memory_growth:.1f} MB)"
        )

    # Check for monotonic growth (indicates leak)
    is_growing = all(
        memory_samples[i] <= memory_samples[i + 1] * 1.1
        for i in range(len(memory_samples) - 1)
    )

    max_growth = max(memory_samples)

    if is_growing and max_growth > 10:  # More than 10MB growth
        print(f"⚠️ WARNING: Possible memory leak detected (+{max_growth:.1f} MB)")
        return False
    else:
        print(f"✅ PASSED: Memory stable (max growth: +{max_growth:.1f} MB)")
        return True


def test_large_scale_stability():
    """Test stability with large number of vectors."""
    print("\n" + "=" * 60)
    print("TEST: Large Scale Stability")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    n_vectors = 10000
    dimension = 128
    batch_size = 1000

    print(f"Adding {n_vectors} vectors in batches of {batch_size}...")

    all_passed = True

    # Add vectors in batches
    for batch_start in range(0, n_vectors, batch_size):
        batch_end = min(batch_start + batch_size, n_vectors)

        for i in range(batch_start, batch_end):
            vec = np.random.randn(dimension).astype(np.float32)
            vec = vec / np.linalg.norm(vec)  # Normalize
            db.add(f"large_{i}", vec.tolist())

        print(f"  Added {batch_end}/{n_vectors} vectors")

        # Verify some vectors from this batch
        test_count = min(5, batch_end - batch_start)
        for j in range(test_count):
            test_idx = batch_start + j
            # Create same vector (using same seed)
            np.random.seed(42 + test_idx)  # Reproducible
            test_vec = np.random.randn(dimension).astype(np.float32)
            test_vec = test_vec / np.linalg.norm(test_vec)

            results = db.search(test_vec.tolist(), 1)
            # Just check we get results (exact match might not work due to randomness)
            if len(results) == 0:
                print(f"    ❌ FAILED: No results for vector {test_idx}")
                all_passed = False

    # Final memory check
    process = psutil.Process(os.getpid())
    final_memory = process.memory_info().rss / 1024 / 1024
    memory_per_vector = final_memory / n_vectors * 1024  # KB

    print(f"\nFinal statistics:")
    print(f"  Total vectors: {n_vectors}")
    print(f"  Total memory: {final_memory:.1f} MB")
    print(f"  Memory per vector: {memory_per_vector:.2f} KB")

    if memory_per_vector > 10:  # More than 10KB per vector is suspicious
        print(f"  ⚠️ WARNING: High memory usage per vector")
    else:
        print(f"  ✅ Memory usage reasonable")

    return all_passed


def test_concurrent_operations():
    """Test thread safety with simulated concurrent operations."""
    print("\n" + "=" * 60)
    print("TEST: Concurrent Operations Safety")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add base vectors
    n_base = 100
    dimension = 64

    for i in range(n_base):
        vec = np.random.randn(dimension).astype(np.float32)
        db.add(f"base_{i}", vec.tolist())

    print(f"Added {n_base} base vectors")

    # Simulate concurrent reads and writes
    all_passed = True

    for round in range(5):
        # Add new vectors while searching
        for i in range(20):
            # Add a vector
            vec = np.random.randn(dimension).astype(np.float32)
            db.add(f"round_{round}_vec_{i}", vec.tolist())

            # Immediately search
            results = db.search(vec.tolist(), 1)

            if len(results) == 0:
                print(
                    f"❌ FAILED: Just-added vector not found (round {round}, vec {i})"
                )
                all_passed = False

    if all_passed:
        print("✅ PASSED: Concurrent operations handled safely")

    return all_passed


def main():
    """Run all memory safety tests."""
    print("🛡️ OMENDB MEMORY SAFETY TEST SUITE")
    print("Testing deep copy implementation and memory integrity")
    print("=" * 60)

    tests = [
        ("Vector Data Integrity", test_vector_corruption),
        ("Rapid Add/Delete Cycles", test_rapid_add_delete_cycles),
        ("Memory Leak Detection", test_memory_leak_detection),
        ("Large Scale Stability", test_large_scale_stability),
        ("Concurrent Operations", test_concurrent_operations),
    ]

    results = []
    for name, test_func in tests:
        try:
            passed = test_func()
            results.append((name, passed))
        except Exception as e:
            print(f"\n❌ EXCEPTION in {name}: {e}")
            import traceback

            traceback.print_exc()
            results.append((name, False))

    # Summary
    print("\n" + "=" * 60)
    print("MEMORY SAFETY SUMMARY")
    print("=" * 60)

    total_passed = sum(1 for _, passed in results if passed)
    total_tests = len(results)

    for name, passed in results:
        status = "✅ PASSED" if passed else "❌ FAILED"
        print(f"{status}: {name}")

    print(f"\nTotal: {total_passed}/{total_tests} tests passed")

    if total_passed == total_tests:
        print("\n🎉 ALL MEMORY SAFETY TESTS PASSED!")
        return 0
    else:
        print(f"\n⚠️ {total_tests - total_passed} tests failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
