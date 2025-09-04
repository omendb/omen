#!/usr/bin/env python3
"""
Minimal DiskANN test to isolate the search issue.
"""

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "python"))

import omendb


def test_minimal():
    """Test with just 3 vectors to debug the search issue."""
    print("=== Minimal DiskANN Test ===")

    # Create database
    db = omendb.DB(algorithm="diskann", buffer_size=10)
    print(f"Initial count: {db.count()}")

    # Add 3 vectors
    vectors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]

    for i, vec in enumerate(vectors):
        print(f"Adding vector {i}: {vec}")
        success = db.add(f"vec_{i}", vec)
        print(f"  Add result: {success}")
        print(f"  Count after add: {db.count()}")

    print(f"\nFinal count: {db.count()}")
    print("All vectors added successfully!")

    # Test search
    print("\n=== Testing Search ===")
    query = [1.0, 0.0, 0.0]  # Should match vec_0
    print(f"Searching for: {query}")

    results = db.search(query, limit=3)
    print(f"Search results: {len(results)} found")

    for i, result in enumerate(results):
        print(f"  {i + 1}. ID: {result.id}, Score: {result.score:.6f}")

    # Test if we can find the exact match
    if results and results[0].id == "vec_0":
        print("✅ Found exact match!")
        return True
    else:
        print("❌ No exact match found")
        return False


if __name__ == "__main__":
    success = test_minimal()
    sys.exit(0 if success else 1)
