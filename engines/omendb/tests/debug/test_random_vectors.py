#!/usr/bin/env python3
"""Test with random vectors to identify when accuracy drops"""

import omendb
import numpy as np


def test_random_accuracy():
    """Test accuracy with random vectors at different scales"""
    print("TESTING WITH RANDOM VECTORS")
    print("=" * 60)

    db = omendb.DB()
    dim = 128

    # Test at different scales
    test_sizes = [10, 20, 50, 100, 200, 500, 1000]

    for n in test_sizes:
        db.clear()
        db.configure(buffer_size=100)  # Standard buffer

        # Generate random vectors
        np.random.seed(42)  # Reproducible
        vectors = np.random.randn(n, dim).astype(np.float32)

        # Add all vectors
        for i in range(n):
            db.add(f"vec_{i}", vectors[i])

        # Test accuracy on a sample
        test_sample = min(20, n)
        correct = 0
        mismatches = []

        for i in range(test_sample):
            results = db.search(vectors[i], limit=1)
            if results and results[0].id == f"vec_{i}":
                correct += 1
            else:
                found = results[0].id if results else "None"
                mismatches.append((i, found))

        accuracy = (correct / test_sample) * 100

        if accuracy >= 95:
            print(
                f"n={n:4d}: {correct:2d}/{test_sample:2d} correct ({accuracy:3.0f}%) ✅"
            )
        else:
            print(
                f"n={n:4d}: {correct:2d}/{test_sample:2d} correct ({accuracy:3.0f}%) ❌"
            )
            if mismatches:
                print(f"  First 3 mismatches: {mismatches[:3]}")

    print("\n" + "=" * 60)
    print("TESTING NORMALIZED vs UNNORMALIZED")
    print("=" * 60)

    # Test if normalization affects accuracy
    n = 500

    # Test 1: Unnormalized random vectors
    db.clear()
    vectors = np.random.randn(n, dim).astype(np.float32)

    for i in range(n):
        db.add(f"vec_{i}", vectors[i])

    correct = 0
    for i in range(20):
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1

    print(f"Unnormalized: {correct}/20 ({correct * 5}%)")

    # Test 2: Normalized random vectors
    db.clear()
    vectors = np.random.randn(n, dim).astype(np.float32)
    # Normalize vectors
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    vectors = vectors / (norms + 1e-12)

    for i in range(n):
        db.add(f"vec_{i}", vectors[i])

    correct = 0
    for i in range(20):
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1

    print(f"Normalized:   {correct}/20 ({correct * 5}%)")

    print("\n" + "=" * 60)
    print("TESTING VECTOR SIMILARITY DISTRIBUTION")
    print("=" * 60)

    # Check if very similar vectors cause issues
    n = 100

    # Test 1: Well-separated vectors
    db.clear()
    vectors = []
    for i in range(n):
        vec = np.zeros(dim, dtype=np.float32)
        # Each vector has a unique "hot" dimension
        vec[i % dim] = 1.0
        vec[(i * 7) % dim] = 0.5  # Add some variety
        vectors.append(vec)

    vectors = np.array(vectors)

    for i in range(n):
        db.add(f"vec_{i}", vectors[i])

    correct = 0
    for i in range(20):
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1

    print(f"Well-separated: {correct}/20 ({correct * 5}%)")

    # Test 2: Very similar vectors (small perturbations)
    db.clear()
    base_vector = np.random.randn(dim).astype(np.float32)
    vectors = []
    for i in range(n):
        vec = (
            base_vector + np.random.randn(dim).astype(np.float32) * 0.01
        )  # Small noise
        vectors.append(vec)

    vectors = np.array(vectors)

    for i in range(n):
        db.add(f"vec_{i}", vectors[i])

    correct = 0
    for i in range(20):
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1

    print(f"Very similar:   {correct}/20 ({correct * 5}%)")


if __name__ == "__main__":
    test_random_accuracy()
