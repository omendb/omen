#!/usr/bin/env python3
"""Test batch processing with numpy arrays vs lists."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_numpy_vs_list():
    """Compare numpy batch vs list batch processing."""

    print("NumPy vs List Batch Test")
    print("=" * 60)

    # Create test vectors
    vec1 = [1.0, 0.0, 0.0, 0.0]
    vec2 = [0.0, 1.0, 0.0, 0.0]
    vec3 = [0.7071, 0.7071, 0.0, 0.0]

    ids = ["vec1", "vec2", "vec3"]

    # Test with lists
    print("\n1. Testing with Python lists:")
    print("-" * 40)

    db_list = omendb.DB()
    list_vectors = [vec1, vec2, vec3]

    print(f"Adding {len(list_vectors)} vectors as lists...")
    results = db_list.add_batch(list_vectors, ids)
    print(f"Added: {sum(1 for r in results if r)}/{len(list_vectors)}")

    print("\nSearching for vec1:")
    results = db_list.search(vec1, limit=3)
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Test with numpy
    print("\n2. Testing with NumPy arrays:")
    print("-" * 40)

    # Clear the global database since OmenDB uses single instance per process
    db_numpy = omendb.DB()
    db_numpy.clear()  # Reset the database state
    numpy_vectors = np.array([vec1, vec2, vec3], dtype=np.float32)

    print(f"Adding {len(numpy_vectors)} vectors as numpy array...")
    print(f"  Array shape: {numpy_vectors.shape}")
    print(f"  Array dtype: {numpy_vectors.dtype}")
    results = db_numpy.add_batch(numpy_vectors, ids)
    print(f"Added: {sum(1 for r in results if r)}/{len(numpy_vectors)}")

    print("\nSearching for vec1:")
    results = db_numpy.search(vec1, limit=3)
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Test individual adds for comparison
    print("\n3. Testing with individual adds (reference):")
    print("-" * 40)

    db_individual = omendb.DB()
    db_individual.clear()  # Reset the database state

    print(f"Adding {len(ids)} vectors individually...")
    for id, vec in zip(ids, [vec1, vec2, vec3]):
        success = db_individual.add(id, vec)
        print(f"  Added {id}: {success}")

    print("\nSearching for vec1:")
    results = db_individual.search(vec1, limit=3)
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Expected results
    print("\n" + "=" * 60)
    print("Expected results:")
    print("  1. vec1: score=1.0000")
    print("  2. vec3: score~0.7071")
    print("  3. vec2: score=0.0000")


if __name__ == "__main__":
    test_numpy_vs_list()
