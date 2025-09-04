#!/usr/bin/env python3
"""
Debug if vectors are actually making it into the DiskANN index.
"""

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "python"))

import omendb


def test_flush_behavior():
    """Test if vectors flush properly to DiskANN."""
    print("=== Testing DiskANN Flush Behavior ===")

    # Create database with small buffer to force flush
    db = omendb.DB(algorithm="diskann", buffer_size=3)

    print("Initial count:", db.count())

    # Add vectors one by one and check count
    test_vectors = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],  # Should trigger flush after this
        [1.0, 1.0, 0.0],  # This should go to DiskANN
        [1.0, 0.0, 1.0],  # This should go to DiskANN
    ]

    for i, vec in enumerate(test_vectors):
        print(f"\nAdding vector {i}: {vec}")
        success = db.add(f"vec_{i}", vec)
        count = db.count()
        print(f"  Add success: {success}")
        print(f"  Count: {count}")

        # Test search after each add
        results = db.search([1.0, 0.0, 0.0], limit=10)
        print(f"  Search results: {len(results)}")
        if results:
            print(f"    Best: {results[0].id} (score: {results[0].score:.4f})")
        else:
            print(f"    No results found!")

    print(f"\nFinal count: {db.count()}")

    # Test comprehensive search
    print("\n=== Final Search Test ===")
    for i, test_vec in enumerate(test_vectors):
        results = db.search(test_vec, limit=3)
        expected_id = f"vec_{i}"
        print(f"Search for vec_{i}: {len(results)} results")
        if results and results[0].id == expected_id:
            print(f"  ✅ Found {expected_id} (score: {results[0].score:.4f})")
        else:
            print(
                f"  ❌ Expected {expected_id}, got: {results[0].id if results else 'none'}"
            )


if __name__ == "__main__":
    test_flush_behavior()
