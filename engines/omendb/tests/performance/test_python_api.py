#!/usr/bin/env python3
"""Test Python API for core functionality"""

from omendb import DB
import numpy as np


def test_python_api():
    print("=== Testing Python API ===")

    # Create database
    print("1. Creating database...")
    db = DB("test_api.omen")

    # Add vectors
    print("2. Adding vectors...")
    for i in range(10):
        vec = np.random.rand(128).astype(np.float32)
        db.add(f"vec_{i}", vec)

    # Search
    print("3. Searching...")
    query = np.random.rand(128).astype(np.float32)
    results = db.search(query, limit=5)

    print(f"   Found {len(results)} results")
    for idx, result in enumerate(results[:3]):
        print(f"   [{idx}] {result.id}: distance={result.distance:.4f}")

    # Get stats
    print("4. Getting stats...")
    stats = db.info()
    print(f"   Vector count: {stats.get('vector_count', 0)}")
    print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")

    print("✅ Python API test passed!")
    return True


if __name__ == "__main__":
    test_python_api()
