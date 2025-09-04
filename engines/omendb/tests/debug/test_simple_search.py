#!/usr/bin/env python3
"""Simplest possible test to isolate the search bug."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_simple():
    """Simplest test case."""

    print("Simple Search Test")
    print("=" * 60)

    db = omendb.DB()

    # Just 3 vectors
    vec1 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    vec2 = np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32)
    vec3 = np.array([0.5, 0.5, 0.0, 0.0], dtype=np.float32)
    vec3 = vec3 / np.linalg.norm(vec3)  # Normalize

    vectors = np.array([vec1, vec2, vec3], dtype=np.float32)
    ids = ["vec1", "vec2", "vec3"]

    print("Vectors:")
    for id, vec in zip(ids, vectors):
        print(f"  {id}: {vec}")

    # Calculate expected similarities to vec1
    print("\nExpected cosine similarities to vec1:")
    for id, vec in zip(ids, vectors):
        sim = np.dot(vec1, vec)
        print(f"  {id}: {sim:.4f}")

    # Add vectors
    print(f"\nAdding {len(vectors)} vectors...")
    results = db.add_batch(vectors, ids)
    print(f"Added: {sum(1 for r in results if r)}/{len(vectors)}")

    # Search for vec1
    print("\nSearching for vec1 (k=3):")
    results = db.search(vec1, limit=3)

    print(f"Results:")
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Expected: vec1 (1.0), vec3 (0.707), vec2 (0.0)
    print("\nExpected order: vec1 (1.0), vec3 (0.707), vec2 (0.0)")
    actual_order = [r.id for r in results]
    print(f"Actual order: {', '.join(actual_order)}")

    if actual_order == ["vec1", "vec3", "vec2"]:
        print("Order is correct!")
    else:
        print("Order is WRONG!")


if __name__ == "__main__":
    test_simple()
