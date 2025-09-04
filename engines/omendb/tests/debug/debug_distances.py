#!/usr/bin/env python3
"""Debug the distance values being calculated."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def debug_distances():
    """Debug distance calculations."""

    print("Debug Distance Calculations")
    print("=" * 60)

    db = omendb.DB()

    dimension = 4  # Small dimension for easy debugging

    # Create simple test vectors
    vec1 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    vec2 = np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32)  # Orthogonal to vec1
    vec3 = np.array([1.0, 1.0, 0.0, 0.0], dtype=np.float32)  # 45 degree angle
    vec4 = np.array([-1.0, 0.0, 0.0, 0.0], dtype=np.float32)  # Opposite to vec1
    vec5 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)  # Same as vec1

    # Normalize them
    vec3 = vec3 / np.linalg.norm(vec3)

    vectors = np.array([vec1, vec2, vec3, vec4, vec5], dtype=np.float32)
    ids = ["vec1", "vec2", "vec3", "vec4", "vec5"]

    print("Test vectors:")
    for i, (id, vec) in enumerate(zip(ids, vectors)):
        print(f"  {id}: {vec}")

    print("\nExpected cosine similarities with vec1:")
    for i, (id, vec) in enumerate(zip(ids, vectors)):
        sim = np.dot(vec1, vec)
        dist = 1.0 - sim
        print(f"  {id}: similarity={sim:.4f}, distance={dist:.4f}")

    # Add to database
    print(f"\nAdding {len(vectors)} vectors...")
    results = db.add_batch(vectors, ids)
    successful = sum(1 for r in results if r)
    print(f"Added: {successful}/{len(vectors)}")

    # Search for vec1
    print("\nSearching for vec1 (k=5):")
    query = vec1
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        # Calculate expected similarity
        idx = ids.index(result.id)
        expected_sim = np.dot(vec1, vectors[idx])
        expected_dist = 1.0 - expected_sim

        # The score should be similarity = 1 - distance
        actual_dist = 1.0 - result.score

        print(f"  {i + 1}. ID: {result.id}")
        print(f"      Score: {result.score:.4f} (should be similarity)")
        print(f"      Expected similarity: {expected_sim:.4f}")
        print(f"      Implied distance: {actual_dist:.4f}")
        print(f"      Expected distance: {expected_dist:.4f}")

    # Try another search
    print("\nSearching for vec3 (45 degree angle, k=5):")
    query = vec3
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        idx = ids.index(result.id)
        expected_sim = np.dot(vec3, vectors[idx])
        print(
            f"  {i + 1}. ID: {result.id}, Score: {result.score:.4f}, Expected: {expected_sim:.4f}"
        )


if __name__ == "__main__":
    debug_distances()
