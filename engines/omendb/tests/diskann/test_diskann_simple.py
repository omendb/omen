#!/usr/bin/env python3
"""
Simple test to verify DiskANN integration works.
"""

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "python"))

import omendb
import numpy as np


def test_diskann_basic():
    """Test basic DiskANN functionality."""
    print("Testing DiskANN basic functionality...")

    # Create database with DiskANN
    db = omendb.DB(algorithm="diskann", buffer_size=100)

    # Add some vectors
    vectors = np.random.randn(100, 128).astype(np.float32)

    print("Adding 100 vectors...")
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec.tolist())

    print("Testing search...")
    # Search for similar vectors
    query = vectors[0]
    results = db.search(query.tolist(), limit=10)

    print(f"Found {len(results)} results")
    for i, result in enumerate(results[:3]):
        print(f"  {i + 1}. {result.id} (score: {result.score:.4f})")

    # Test that we can find the exact match
    if len(results) > 0 and results[0].id == "vec_0":
        print("✅ Found exact match as top result!")
        return True
    else:
        print("⚠️ Exact match not found in top results")
        return False


if __name__ == "__main__":
    success = test_diskann_basic()
    if success:
        print("\n✅ DiskANN test passed!")
        sys.exit(0)
    else:
        print("\n❌ DiskANN test failed!")
        sys.exit(1)
