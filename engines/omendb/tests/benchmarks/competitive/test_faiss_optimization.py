#!/usr/bin/env python3
"""
Simple test to verify Faiss-style HNSW optimization doesn't crash.
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb
import numpy as np


def test_basic_faiss_optimization():
    """Test basic functionality without triggering complex operations."""
    print("🧪 Testing Faiss-style HNSW optimization...")

    db = omendb.DB()

    # Add a small number of vectors to test basic functionality
    dimensions = 128
    num_vectors = 10

    vectors = []
    for i in range(num_vectors):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"test_{i}" for i in range(num_vectors)]

    try:
        # Test individual adds
        for i in range(num_vectors):
            success = db.add(ids[i], vectors[i])
            if not success:
                print(f"❌ Failed to add vector {i}")
                return False
            print(f"✅ Added vector {i}")

        # Test search
        query = vectors[0]  # Use first vector as query
        results = db.search(query, limit=3)
        print(f"✅ Search returned {len(results)} results")

        # Test stats
        stats = db.stats()
        print(f"✅ Stats: algorithm={stats.get('algorithm')}, size={stats.get('size')}")

        return True

    except Exception as e:
        print(f"❌ Error during test: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    print("🎯 Simple Faiss Optimization Test")
    print("=" * 40)

    success = test_basic_faiss_optimization()

    if success:
        print("\n✅ Basic test passed - Faiss optimization works!")
    else:
        print("\n❌ Basic test failed - need to debug implementation")
