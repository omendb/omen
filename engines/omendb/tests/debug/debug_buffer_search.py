#!/usr/bin/env python3
"""Debug buffer search issue."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def debug_search():
    """Debug the search behavior."""

    print("Debug Buffer Search")
    print("=" * 60)

    # Create DB with specific buffer size
    db = omendb.DB()

    # Create exactly 10 vectors
    dimension = 128
    num_vectors = 10

    # Create vectors with known patterns for debugging
    vectors = []
    for i in range(num_vectors):
        # Create a vector with a unique pattern
        vec = np.zeros(dimension, dtype=np.float32)
        vec[i % dimension] = 1.0  # Put 1.0 at different positions
        vec[0] = float(i) / 10.0  # Add unique identifier
        # Normalize
        norm = np.linalg.norm(vec)
        if norm > 0:
            vec = vec / norm
        vectors.append(vec)

    vectors = np.array(vectors, dtype=np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    print(f"Adding {num_vectors} vectors with unique patterns...")
    results = db.add_batch(vectors, ids)
    successful = sum(1 for r in results if r)
    print(f"Added: {successful}/{num_vectors}")

    # Search for each vector with different k values
    for k in [1, 5, 10, 15]:
        print(f"\n{'=' * 60}")
        print(f"Testing with k={k}")
        print(f"{'=' * 60}")

        found_count = 0
        for i in range(num_vectors):
            query = vectors[i]
            expected_id = ids[i]

            results = db.search(query, limit=k)

            # Check if expected vector was found
            result_ids = [r.id for r in results] if results else []

            if expected_id in result_ids:
                found_count += 1
                # Find the position
                position = result_ids.index(expected_id) + 1
                print(f"  vec_{i}: ✅ Found at position {position}/{len(results)}")
            else:
                print(f"  vec_{i}: ❌ NOT FOUND (got {len(results)} results)")

        print(f"\nSummary for k={k}: Found {found_count}/{num_vectors} vectors")

    # Now let's check what's actually in the buffer
    print(f"\n{'=' * 60}")
    print("Testing actual similarity scores")
    print(f"{'=' * 60}")

    # Search for vec_0 and show all results
    query = vectors[0]
    results = db.search(query, limit=20)

    print(f"\nSearching for vec_0, requesting 20 results:")
    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        print(f"  {i + 1}. ID: {result.id}, Score: {result.score:.6f}")

    # Manually calculate similarities for verification
    print(f"\n{'=' * 60}")
    print("Manual similarity calculation for vec_0:")
    print(f"{'=' * 60}")

    for i in range(num_vectors):
        # Calculate cosine similarity manually
        dot_product = np.dot(vectors[0], vectors[i])
        print(f"  vec_{i}: cosine_similarity = {dot_product:.6f}")


if __name__ == "__main__":
    debug_search()
