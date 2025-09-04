#!/usr/bin/env python3
"""Debug heap selection by testing with known distances."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_heap_selection():
    """Test if heap selection is working correctly."""

    print("Heap Selection Debug Test")
    print("=" * 60)

    db = omendb.DB()

    # Create vectors with predictable distances
    # We'll use vectors along different axes to get exact distances
    vectors = []

    # Create 10 vectors with known cosine distances
    for i in range(10):
        vec = np.zeros(4, dtype=np.float32)
        if i == 0:
            vec[0] = 1.0  # [1, 0, 0, 0]
        elif i == 1:
            vec[0] = 1.0  # [1, 0, 0, 0] - identical to vec_0
        elif i == 2:
            vec[1] = 1.0  # [0, 1, 0, 0] - orthogonal
        elif i == 3:
            vec[0] = -1.0  # [-1, 0, 0, 0] - opposite
        elif i == 4:
            vec[0] = 0.9  # [0.9, 0, 0, 0] - close to vec_0
            vec = vec / np.linalg.norm(vec)
        elif i == 5:
            vec[0] = 0.8  # [0.8, 0, 0, 0] - somewhat close
            vec = vec / np.linalg.norm(vec)
        elif i == 6:
            vec[0] = 0.5  # [0.5, 0, 0, 0] - medium distance
            vec = vec / np.linalg.norm(vec)
        elif i == 7:
            vec[0] = 0.3  # [0.3, 0, 0, 0] - farther
            vec = vec / np.linalg.norm(vec)
        elif i == 8:
            vec[0] = 0.1  # [0.1, 0, 0, 0] - far
            vec = vec / np.linalg.norm(vec)
        elif i == 9:
            vec[0] = -0.5  # [-0.5, 0, 0, 0] - negative correlation
            vec = vec / np.linalg.norm(vec)

        vectors.append(vec)

    vectors = np.array(vectors, dtype=np.float32)
    ids = [f"vec_{i}" for i in range(len(vectors))]

    # Calculate expected similarities to vec_0
    query = vectors[0]
    expected_sims = []
    for vec in vectors:
        sim = np.dot(query, vec)
        expected_sims.append(sim)

    print("Vectors and expected similarities to vec_0:")
    for i, (vec, sim) in enumerate(zip(vectors, expected_sims)):
        print(f"  vec_{i}: {vec[:2]}... similarity={sim:.4f}")

    # Add to database
    print(f"\nAdding {len(vectors)} vectors...")
    results = db.add_batch(vectors, ids)
    successful = sum(1 for r in results if r)
    print(f"Added: {successful}/{len(vectors)}")

    # Search with different k values
    for k in [3, 5, 10]:
        print(f"\n{'=' * 60}")
        print(f"Searching for vec_0 with k={k}:")
        print(f"{'=' * 60}")

        results = db.search(query, limit=k)

        print(f"Got {len(results)} results:")
        print(f"{'ID':<10} {'Score':<10} {'Expected':<10} {'Match':<10}")
        print("-" * 40)

        for result in results:
            idx = int(result.id.split("_")[1])
            expected = expected_sims[idx]
            match = "✅" if abs(result.score - expected) < 0.01 else "❌"
            print(
                f"{result.id:<10} {result.score:<10.4f} {expected:<10.4f} {match:<10}"
            )

        # Check if results are in correct order
        print("\nOrder check (should be sorted by similarity descending):")
        sorted_indices = sorted(
            range(len(expected_sims)), key=lambda i: expected_sims[i], reverse=True
        )
        expected_order = [f"vec_{i}" for i in sorted_indices[:k]]
        actual_order = [r.id for r in results]

        print(f"  Expected order: {expected_order}")
        print(f"  Actual order:   {actual_order}")

        if expected_order == actual_order:
            print("  ✅ Order is correct!")
        else:
            print("  ❌ Order is WRONG!")


if __name__ == "__main__":
    test_heap_selection()
