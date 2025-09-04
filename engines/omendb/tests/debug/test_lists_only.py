#!/usr/bin/env python3
"""Test with Python lists only (no numpy) to isolate the issue."""

import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_lists():
    """Test with Python lists only."""

    print("List-only Test (no numpy)")
    print("=" * 60)

    db = omendb.DB()

    # Create simple vectors as lists
    vec1 = [1.0, 0.0, 0.0, 0.0]
    vec2 = [0.0, 1.0, 0.0, 0.0]
    vec3 = [0.7071, 0.7071, 0.0, 0.0]  # ~45 degree angle

    vectors = [vec1, vec2, vec3]
    ids = ["vec1", "vec2", "vec3"]

    print("Vectors:")
    for id, vec in zip(ids, vectors):
        print(f"  {id}: {vec}")

    # Add vectors
    print(f"\nAdding {len(vectors)} vectors...")
    results = db.add_batch(vectors, ids)
    print(f"Added: {sum(1 for r in results if r)}/{len(vectors)}")

    # Search for vec1
    print("\nSearching for vec1 (k=3):")
    query = vec1
    results = db.search(query, limit=3)

    print(f"Results:")
    for i, result in enumerate(results):
        print(f"  {i + 1}. {result.id}: score={result.score:.4f}")

    # Expected: vec1 (1.0), vec3 (~0.707), vec2 (0.0)
    print("\nExpected order: vec1 (1.0), vec3 (~0.707), vec2 (0.0)")
    actual_order = [r.id for r in results]
    print(f"Actual order: {', '.join(actual_order)}")

    # Check scores
    print("\nScore check:")
    for result in results:
        if result.id == "vec1" and abs(result.score - 1.0) < 0.01:
            print(f"  ✅ vec1 score correct: {result.score:.4f}")
        elif result.id == "vec1":
            print(f"  ❌ vec1 score wrong: {result.score:.4f} (expected 1.0)")

        if result.id == "vec2" and abs(result.score - 0.0) < 0.01:
            print(f"  ✅ vec2 score correct: {result.score:.4f}")
        elif result.id == "vec2":
            print(f"  ❌ vec2 score wrong: {result.score:.4f} (expected 0.0)")

        if result.id == "vec3" and abs(result.score - 0.707) < 0.1:
            print(f"  ✅ vec3 score correct: {result.score:.4f}")
        elif result.id == "vec3":
            print(f"  ❌ vec3 score wrong: {result.score:.4f} (expected ~0.707)")


if __name__ == "__main__":
    test_lists()
