#!/usr/bin/env python3
"""Debug why DiskANN beam search only returns 1 result."""

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "python"))

import omendb


def test_beam_search():
    print("Testing DiskANN beam search...")

    # Create database with DiskANN
    db = omendb.DB(algorithm="diskann", buffer_size=5)

    # Add vectors
    for i in range(10):
        db.add(f"vec_{i}", [float(i), float(i + 1), float(i + 2)])

    print("\nSearching for [1.0, 2.0, 3.0]...")
    results = db.search([1.0, 2.0, 3.0], limit=5)

    print(f"Found {len(results)} results")
    for r in results[:3]:
        print(f"  {r.id}: {r.score:.4f}")

    # Test exact match
    print("\nSearching for exact vec_1 [1.0, 2.0, 3.0]...")
    results = db.search([1.0, 2.0, 3.0], limit=5)
    if results and results[0].id == "vec_1":
        print("✅ Found exact match!")
    else:
        print(f"❌ Expected vec_1, got {results[0].id if results else 'none'}")


if __name__ == "__main__":
    test_beam_search()
