#!/usr/bin/env python3
"""Debug the raw distance values being calculated."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def calculate_cosine_distance(vec1, vec2):
    """Calculate cosine distance manually."""
    dot_product = np.dot(vec1, vec2)
    norm1 = np.linalg.norm(vec1)
    norm2 = np.linalg.norm(vec2)

    if norm1 == 0 or norm2 == 0:
        return 2.0

    cosine_sim = dot_product / (norm1 * norm2)
    # Clamp to [-1, 1]
    cosine_sim = np.clip(cosine_sim, -1.0, 1.0)

    # Cosine distance = 1 - cosine_similarity
    return 1.0 - cosine_sim


def test_raw_distances():
    """Test with specific vectors to debug distance calculation."""

    print("Raw Distance Debug Test")
    print("=" * 60)

    db = omendb.DB()

    # Create test vectors with known relationships
    vectors = []

    # vec_0: base vector
    vec_0 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    vectors.append(vec_0)

    # vec_1: identical to vec_0
    vec_1 = np.array([1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    vectors.append(vec_1)

    # vec_2: orthogonal to vec_0
    vec_2 = np.array([0.0, 1.0, 0.0, 0.0], dtype=np.float32)
    vectors.append(vec_2)

    # vec_3: opposite to vec_0
    vec_3 = np.array([-1.0, 0.0, 0.0, 0.0], dtype=np.float32)
    vectors.append(vec_3)

    # vec_4: 45 degree angle
    vec_4 = np.array([0.7071, 0.7071, 0.0, 0.0], dtype=np.float32)
    vectors.append(vec_4)

    vectors = np.array(vectors, dtype=np.float32)
    ids = [f"vec_{i}" for i in range(len(vectors))]

    print("Test vectors:")
    for i, vec in enumerate(vectors):
        print(f"  vec_{i}: {vec}")

    print("\nExpected distances from vec_0:")
    for i, vec in enumerate(vectors):
        dist = calculate_cosine_distance(vec_0, vec)
        sim = 1.0 - dist
        print(f"  vec_{i}: distance={dist:.4f}, similarity={sim:.4f}")

    # Add to database
    print(f"\nAdding {len(vectors)} vectors...")
    results = db.add_batch(vectors, ids)
    successful = sum(1 for r in results if r)
    print(f"Added: {successful}/{len(vectors)}")

    # Search for vec_0
    print("\nSearching for vec_0 (should find identical vec_1 first):")
    query = vec_0
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        idx = int(result.id.split("_")[1])
        expected_dist = calculate_cosine_distance(vec_0, vectors[idx])
        expected_sim = 1.0 - expected_dist

        print(f"  {i + 1}. ID: {result.id}")
        print(f"      Returned score: {result.score:.4f}")
        print(f"      Expected similarity: {expected_sim:.4f}")
        print(f"      Implied distance (1-score): {1.0 - result.score:.4f}")
        print(f"      Expected distance: {expected_dist:.4f}")

        # Check if score matches expected
        if abs(result.score - expected_sim) < 0.001:
            print(f"      ✅ Score matches expected similarity")
        else:
            print(f"      ❌ Score DOES NOT match expected similarity")

    # Test orthogonal search
    print("\nSearching for vec_2 (orthogonal):")
    query = vec_2
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        idx = int(result.id.split("_")[1])
        expected_dist = calculate_cosine_distance(vec_2, vectors[idx])
        expected_sim = 1.0 - expected_dist

        print(
            f"  {i + 1}. ID: {result.id}, Score: {result.score:.4f}, Expected: {expected_sim:.4f}"
        )


if __name__ == "__main__":
    test_raw_distances()
