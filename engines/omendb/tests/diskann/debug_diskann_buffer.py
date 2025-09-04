#!/usr/bin/env python3
"""
Debug DiskANN buffer behavior with different sizes.
"""

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "python"))

import omendb
import numpy as np


def test_buffer_behavior():
    """Test DiskANN with different vector counts and buffer sizes."""

    test_cases = [
        (5, 100),  # 5 vectors, 100 buffer - should stay in buffer
        (10, 5),  # 10 vectors, 5 buffer - should flush to DiskANN
        (50, 10),  # 50 vectors, 10 buffer - should flush multiple times
        (100, 50),  # 100 vectors, 50 buffer - should flush twice
    ]

    for num_vectors, buffer_size in test_cases:
        print(f"\n=== Testing {num_vectors} vectors with buffer_size={buffer_size} ===")

        # Create database
        db = omendb.DB(algorithm="diskann", buffer_size=buffer_size)

        # Generate vectors
        vectors = np.random.randn(num_vectors, 128).astype(np.float32)

        # Add vectors
        print(f"Adding {num_vectors} vectors...")
        for i, vec in enumerate(vectors):
            db.add(f"vec_{i}", vec.tolist())
            if i % 10 == 0:
                print(f"  Added {i + 1}/{num_vectors}, count: {db.count()}")

        final_count = db.count()
        print(f"Final count: {final_count}")

        # Test search with first vector
        print("Testing search...")
        query = vectors[0]
        results = db.search(query.tolist(), limit=5)

        print(f"Search results: {len(results)} found")
        if results:
            print(f"  Best match: {results[0].id} (score: {results[0].score:.6f})")
            success = results[0].id == "vec_0" and results[0].score < 0.1
            print(f"  Exact match found: {'✅' if success else '❌'}")
        else:
            print("  ❌ No results found!")

        print("-" * 50)


if __name__ == "__main__":
    test_buffer_behavior()
