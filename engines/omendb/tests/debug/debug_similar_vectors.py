#!/usr/bin/env python3
"""Debug with vectors that have actual similarities."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def debug_similar_vectors():
    """Test with vectors that have varying similarities."""

    print("Debug Similar Vectors Test")
    print("=" * 60)

    db = omendb.DB()

    dimension = 128
    num_vectors = 10

    # Create a base vector
    base_vector = np.random.randn(dimension).astype(np.float32)
    base_vector = base_vector / np.linalg.norm(base_vector)

    # Create variations with different similarities
    vectors = []
    expected_similarities = []

    for i in range(num_vectors):
        # Create vectors with decreasing similarity to base
        noise_level = i * 0.2  # Increasing noise
        noise = np.random.randn(dimension).astype(np.float32) * noise_level
        vec = base_vector + noise
        vec = vec / np.linalg.norm(vec)  # Normalize
        vectors.append(vec)

        # Calculate expected similarity to first vector
        expected_sim = np.dot(vectors[0], vec)
        expected_similarities.append(expected_sim)

    vectors = np.array(vectors, dtype=np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    print(f"Adding {num_vectors} vectors with varying similarities...")
    results = db.add_batch(vectors, ids)
    successful = sum(1 for r in results if r)
    print(f"Added: {successful}/{num_vectors}")

    print("\nExpected similarities to vec_0:")
    for i in range(num_vectors):
        print(f"  vec_{i}: {expected_similarities[i]:.4f}")

    # Search for vec_0
    print(f"\n{'=' * 60}")
    print("Searching for vec_0 with k=5:")
    print(f"{'=' * 60}")

    query = vectors[0]
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        expected_sim = expected_similarities[int(result.id.split("_")[1])]
        print(
            f"  {i + 1}. ID: {result.id}, Score: {result.score:.4f}, Expected: {expected_sim:.4f}"
        )

    # Search with k=10
    print(f"\n{'=' * 60}")
    print("Searching for vec_0 with k=10:")
    print(f"{'=' * 60}")

    results = db.search(query, limit=10)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        idx = int(result.id.split("_")[1])
        expected_sim = expected_similarities[idx]
        diff = abs(result.score - expected_sim)
        status = "✅" if diff < 0.01 else "❌"
        print(
            f"  {i + 1}. ID: {result.id}, Score: {result.score:.4f}, Expected: {expected_sim:.4f} {status}"
        )

    # Test with a different query vector
    print(f"\n{'=' * 60}")
    print("Searching for vec_5 (middle similarity):")
    print(f"{'=' * 60}")

    query = vectors[5]
    results = db.search(query, limit=5)

    print(f"Got {len(results)} results:")
    for i, result in enumerate(results):
        # Calculate actual similarity
        idx = int(result.id.split("_")[1])
        actual_sim = np.dot(vectors[5], vectors[idx])
        diff = abs(result.score - actual_sim)
        status = "✅" if diff < 0.01 else "❌"
        print(
            f"  {i + 1}. ID: {result.id}, Score: {result.score:.4f}, Actual: {actual_sim:.4f} {status}"
        )


if __name__ == "__main__":
    debug_similar_vectors()
