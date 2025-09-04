#!/usr/bin/env python3
"""Test adding vectors individually instead of batch."""

import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_individual():
    """Test with individual adds."""

    print("Individual Add Test")
    print("=" * 60)

    db = omendb.DB()

    # Create simple vectors
    vec1 = [1.0, 0.0, 0.0, 0.0]
    vec2 = [0.0, 1.0, 0.0, 0.0]
    vec3 = [0.7071, 0.7071, 0.0, 0.0]

    vectors = [vec1, vec2, vec3]
    ids = ["vec1", "vec2", "vec3"]

    print("Vectors:")
    for id, vec in zip(ids, vectors):
        print(f"  {id}: {vec}")

    # Add vectors individually
    print(f"\nAdding {len(vectors)} vectors individually...")
    for id, vec in zip(ids, vectors):
        success = db.add(id, vec)
        print(f"  Added {id}: {success}")

    # Search for vec1
    print("\nSearching for vec1 (k=3):")
    query = vec1
    results = db.search(query, limit=3)

    print(f"Results:")
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Check scores
    print("\nScore check:")
    for result in results:
        if result.id == "vec1":
            if abs(result.score - 1.0) < 0.01:
                print(f"  ✅ vec1 score correct: {result.score:.4f}")
            else:
                print(f"  ❌ vec1 score wrong: {result.score:.4f} (expected 1.0)")

        if result.id == "vec2":
            if abs(result.score - 0.0) < 0.01:
                print(f"  ✅ vec2 score correct: {result.score:.4f}")
            else:
                print(f"  ❌ vec2 score wrong: {result.score:.4f} (expected 0.0)")

        if result.id == "vec3":
            if abs(result.score - 0.707) < 0.1:
                print(f"  ✅ vec3 score correct: {result.score:.4f}")
            else:
                print(f"  ❌ vec3 score wrong: {result.score:.4f} (expected ~0.707)")


if __name__ == "__main__":
    test_individual()
