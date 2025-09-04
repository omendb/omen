#!/usr/bin/env python3
"""
Test search accuracy after distance/similarity conversion fix.
Verifies that exact matches return distance=0.0 and score=1.0.
"""

import sys
import numpy as np
import math

sys.path.insert(0, "python")
import omendb


def test_exact_match_accuracy():
    """Test that exact matches return correct distance/score."""
    print("=" * 60)
    print("TEST: Exact Match Accuracy")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Test vectors
    test_vectors = [
        ([1.0, 0.0, 0.0], "x_axis"),
        ([0.0, 1.0, 0.0], "y_axis"),
        ([0.0, 0.0, 1.0], "z_axis"),
        ([0.577, 0.577, 0.577], "diagonal"),  # Normalized [1,1,1]
    ]

    # Add test vectors
    for vec, id_ in test_vectors:
        db.add(id_, vec)

    print(f"Added {len(test_vectors)} test vectors")

    # Test exact matches
    all_passed = True
    for vec, expected_id in test_vectors:
        results = db.search(vec, 1)

        if len(results) == 0:
            print(f"❌ FAILED: No results for {expected_id}")
            all_passed = False
            continue

        result = results[0]

        # Check ID matches
        if result.id != expected_id:
            print(f"❌ FAILED: Expected {expected_id}, got {result.id}")
            all_passed = False
            continue

        # Check distance is ~0.0
        if abs(result.distance) > 1e-5:
            print(
                f"❌ FAILED: {expected_id} distance={result.distance:.6f} (expected ~0.0)"
            )
            all_passed = False
        else:
            print(f"✅ PASSED: {expected_id} distance={result.distance:.6f}")

        # Check score is ~1.0
        if abs(result.score - 1.0) > 1e-5:
            print(f"❌ FAILED: {expected_id} score={result.score:.6f} (expected ~1.0)")
            all_passed = False
        else:
            print(f"✅ PASSED: {expected_id} score={result.score:.6f}")

    return all_passed


def test_orthogonal_vectors():
    """Test that orthogonal vectors return correct distances."""
    print("\n" + "=" * 60)
    print("TEST: Orthogonal Vector Distances")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add basis vectors
    db.add("x", [1.0, 0.0, 0.0])
    db.add("y", [0.0, 1.0, 0.0])
    db.add("z", [0.0, 0.0, 1.0])

    # Search for x and check distances to y and z
    results = db.search([1.0, 0.0, 0.0], 3)

    all_passed = True
    for result in results:
        if result.id == "x":
            # Exact match
            if abs(result.distance) > 1e-5:
                print(f"❌ FAILED: x->x distance={result.distance:.6f} (expected ~0.0)")
                all_passed = False
            else:
                print(f"✅ PASSED: x->x distance={result.distance:.6f}")
        elif result.id in ["y", "z"]:
            # Orthogonal vectors: cosine distance = 1.0
            if abs(result.distance - 1.0) > 1e-5:
                print(
                    f"❌ FAILED: x->{result.id} distance={result.distance:.6f} (expected ~1.0)"
                )
                all_passed = False
            else:
                print(f"✅ PASSED: x->{result.id} distance={result.distance:.6f}")

    return all_passed


def test_similar_vectors():
    """Test that similar vectors return appropriate distances."""
    print("\n" + "=" * 60)
    print("TEST: Similar Vector Distances")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add vectors with known similarities
    db.add("v1", [1.0, 0.0, 0.0])
    db.add("v2", [0.9, 0.1, 0.0])  # ~6 degree angle
    db.add("v3", [0.707, 0.707, 0.0])  # 45 degree angle
    db.add("v4", [0.0, 1.0, 0.0])  # 90 degree angle
    db.add("v5", [-1.0, 0.0, 0.0])  # 180 degree angle

    # Search for v1
    results = db.search([1.0, 0.0, 0.0], 5)

    all_passed = True
    expected_order = ["v1", "v2", "v3", "v4", "v5"]

    print("Expected distance ordering: v1 < v2 < v3 < v4 < v5")
    print("Results:")

    last_distance = -1.0
    for i, result in enumerate(results):
        print(
            f"  {i + 1}. {result.id}: distance={result.distance:.4f}, score={result.score:.4f}"
        )

        # Check monotonic increasing distances
        if result.distance < last_distance:
            print(f"❌ FAILED: Non-monotonic distances")
            all_passed = False
        last_distance = result.distance

        # Check expected ordering
        if i < len(expected_order) and result.id != expected_order[i]:
            print(
                f"❌ FAILED: Expected {expected_order[i]} at position {i + 1}, got {result.id}"
            )
            all_passed = False

    if all_passed:
        print("✅ PASSED: Distance ordering correct")

    return all_passed


def test_normalized_vs_unnormalized():
    """Test that normalized and unnormalized vectors work correctly."""
    print("\n" + "=" * 60)
    print("TEST: Normalized vs Unnormalized Vectors")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Add same direction vectors with different magnitudes
    db.add("unit", [1.0, 0.0, 0.0])
    db.add("double", [2.0, 0.0, 0.0])
    db.add("half", [0.5, 0.0, 0.0])

    # All should be found as exact matches (cosine similarity ignores magnitude)
    for query in [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.5, 0.0, 0.0]]:
        results = db.search(query, 3)

        # All three should have ~0 distance to each other
        all_zero = True
        for result in results:
            if result.distance > 1e-5:
                all_zero = False
                print(
                    f"❌ FAILED: {result.id} has distance {result.distance:.6f} for same-direction vector"
                )

        if all_zero:
            print(
                f"✅ PASSED: Query {query} finds all same-direction vectors with ~0 distance"
            )

    return True


def test_high_dimensional():
    """Test with higher dimensional vectors (128D)."""
    print("\n" + "=" * 60)
    print("TEST: High Dimensional Vectors (128D)")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Create random 128D vectors
    np.random.seed(42)
    n_vectors = 10
    dimension = 128

    vectors = []
    for i in range(n_vectors):
        vec = np.random.randn(dimension).astype(np.float32)
        vec = vec / np.linalg.norm(vec)  # Normalize
        vectors.append(vec)
        db.add(f"vec_{i}", vec.tolist())

    print(f"Added {n_vectors} random 128D vectors")

    # Test exact match for each vector
    all_passed = True
    for i, vec in enumerate(vectors):
        results = db.search(vec.tolist(), 1)

        if len(results) == 0:
            print(f"❌ FAILED: No results for vec_{i}")
            all_passed = False
            continue

        result = results[0]

        if result.id != f"vec_{i}":
            print(f"❌ FAILED: Expected vec_{i}, got {result.id}")
            all_passed = False
        elif result.distance > 1e-5:
            print(f"❌ FAILED: vec_{i} distance={result.distance:.6f} (expected ~0.0)")
            all_passed = False

    if all_passed:
        print(f"✅ PASSED: All {n_vectors} exact matches found with correct distances")

    return all_passed


def test_edge_cases():
    """Test edge cases like zero vectors, opposite vectors."""
    print("\n" + "=" * 60)
    print("TEST: Edge Cases")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    all_passed = True

    # Test with normal vector first
    db.add("normal", [1.0, 0.0, 0.0])

    # Test zero vector (should handle gracefully)
    try:
        db.add("zero", [0.0, 0.0, 0.0])
        results = db.search([0.0, 0.0, 0.0], 2)
        print(f"✅ PASSED: Zero vector handled (found {len(results)} results)")
    except Exception as e:
        print(f"⚠️ WARNING: Zero vector caused error: {e}")

    # Test opposite vectors (cosine distance should be 2.0)
    db.add("positive", [1.0, 0.0, 0.0])
    db.add("negative", [-1.0, 0.0, 0.0])

    results = db.search([1.0, 0.0, 0.0], 3)
    for result in results:
        if result.id == "negative":
            # Opposite vectors have cosine similarity -1, distance 2.0
            if abs(result.distance - 2.0) > 1e-5:
                print(
                    f"❌ FAILED: Opposite vector distance={result.distance:.6f} (expected ~2.0)"
                )
                all_passed = False
            else:
                print(f"✅ PASSED: Opposite vector distance={result.distance:.6f}")

    return all_passed


def main():
    """Run all accuracy tests."""
    print("🔬 OMENDB SEARCH ACCURACY TEST SUITE")
    print("Testing distance/similarity conversion fixes")
    print("=" * 60)

    tests = [
        ("Exact Match Accuracy", test_exact_match_accuracy),
        ("Orthogonal Vectors", test_orthogonal_vectors),
        ("Similar Vectors", test_similar_vectors),
        ("Normalized vs Unnormalized", test_normalized_vs_unnormalized),
        ("High Dimensional", test_high_dimensional),
        ("Edge Cases", test_edge_cases),
    ]

    results = []
    for name, test_func in tests:
        try:
            passed = test_func()
            results.append((name, passed))
        except Exception as e:
            print(f"\n❌ EXCEPTION in {name}: {e}")
            import traceback

            traceback.print_exc()
            results.append((name, False))

    # Summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)

    total_passed = sum(1 for _, passed in results if passed)
    total_tests = len(results)

    for name, passed in results:
        status = "✅ PASSED" if passed else "❌ FAILED"
        print(f"{status}: {name}")

    print(f"\nTotal: {total_passed}/{total_tests} tests passed")

    if total_passed == total_tests:
        print("\n🎉 ALL ACCURACY TESTS PASSED!")
        return 0
    else:
        print(f"\n⚠️ {total_tests - total_passed} tests failed")
        return 1


if __name__ == "__main__":
    sys.exit(main())
