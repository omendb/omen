#!/usr/bin/env python3
"""Test to debug distance calculation issue."""

import numpy as np
import sys
import os

# Add the local development path for omendb
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def manual_cosine_distance(vec1, vec2):
    """Calculate cosine distance manually."""
    vec1 = np.array(vec1, dtype=np.float32)
    vec2 = np.array(vec2, dtype=np.float32)

    dot_product = np.dot(vec1, vec2)
    norm1 = np.linalg.norm(vec1)
    norm2 = np.linalg.norm(vec2)

    if norm1 == 0 or norm2 == 0:
        return 2.0

    cosine_sim = dot_product / (norm1 * norm2)
    cosine_sim = np.clip(cosine_sim, -1.0, 1.0)

    # Cosine distance = 1 - cosine_similarity
    return 1.0 - cosine_sim


def test_distance():
    """Test distance calculation with known vectors."""

    print("Distance Calculation Debug Test")
    print("=" * 60)

    # Test vectors
    vec1 = [1.0, 0.0, 0.0, 0.0]  # Unit vector along x-axis
    vec2 = [1.0, 0.0, 0.0, 0.0]  # Identical to vec1
    vec3 = [0.0, 1.0, 0.0, 0.0]  # Orthogonal to vec1
    vec4 = [-1.0, 0.0, 0.0, 0.0]  # Opposite to vec1

    print("Test vectors:")
    print(f"  vec1: {vec1}")
    print(f"  vec2: {vec2} (identical to vec1)")
    print(f"  vec3: {vec3} (orthogonal to vec1)")
    print(f"  vec4: {vec4} (opposite to vec1)")

    print("\nManual calculations:")
    print(
        f"  Distance vec1->vec1: {manual_cosine_distance(vec1, vec1):.4f} (should be 0)"
    )
    print(
        f"  Distance vec1->vec2: {manual_cosine_distance(vec1, vec2):.4f} (should be 0)"
    )
    print(
        f"  Distance vec1->vec3: {manual_cosine_distance(vec1, vec3):.4f} (should be 1)"
    )
    print(
        f"  Distance vec1->vec4: {manual_cosine_distance(vec1, vec4):.4f} (should be 2)"
    )

    print("\nSimilarities (1 - distance):")
    print(
        f"  Similarity vec1->vec1: {1.0 - manual_cosine_distance(vec1, vec1):.4f} (should be 1)"
    )
    print(
        f"  Similarity vec1->vec2: {1.0 - manual_cosine_distance(vec1, vec2):.4f} (should be 1)"
    )
    print(
        f"  Similarity vec1->vec3: {1.0 - manual_cosine_distance(vec1, vec3):.4f} (should be 0)"
    )
    print(
        f"  Similarity vec1->vec4: {1.0 - manual_cosine_distance(vec1, vec4):.4f} (should be -1)"
    )

    # Now test with OmenDB
    print("\n" + "=" * 60)
    print("Testing with OmenDB:")
    print("=" * 60)

    db = omendb.DB()

    # Add vectors individually to avoid batch issues
    print("\nAdding vectors individually...")
    db.add("vec1", vec1)
    db.add("vec2", vec2)
    db.add("vec3", vec3)
    db.add("vec4", vec4)
    print("Added 4 vectors")

    # Search for vec1
    print("\nSearching for vec1 (should find vec1 and vec2 with score 1.0):")
    results = db.search(vec1, limit=4)

    for i, result in enumerate(results):
        expected_dist = manual_cosine_distance(vec1, eval(f"vec{result.id[-1]}"))
        expected_sim = 1.0 - expected_dist

        print(f"  {i + 1}. {result.id}:")
        print(f"      OmenDB score: {result.score:.4f}")
        print(f"      Expected similarity: {expected_sim:.4f}")
        print(
            f"      Match: {'✅' if abs(result.score - expected_sim) < 0.01 else '❌'}"
        )


if __name__ == "__main__":
    test_distance()
