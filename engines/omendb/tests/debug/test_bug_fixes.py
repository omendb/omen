#!/usr/bin/env python3
"""Comprehensive test to verify all bug fixes are working."""

import numpy as np
import sys
import os

# Add the local development path
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def test_warmup_dimension_fix():
    """Test that warmup doesn't interfere with actual dimension."""
    print("\n1. Testing warmup dimension fix:")
    print("-" * 40)

    # Create DB and add a vector with dimension 8
    db = omendb.DB()
    db.clear()

    # Add vector with dimension 8 (not 128 from warmup)
    vec = [1.0] * 8
    success = db.add("test_vec", vec)
    print(f"  Added dimension-8 vector: {success}")

    # Search should work
    results = db.search(vec)
    print(f"  Search found: {len(results)} results")
    if results:
        print(f"  Best match: {results[0].id} (score={results[0].score:.4f})")

    assert success, "Failed to add vector"
    assert len(results) > 0, "Search failed"
    assert results[0].score > 0.99, "Incorrect score"
    print("  ✅ Warmup dimension fix working")


def test_double_conversion_fix():
    """Test that similarity scores are not double-converted."""
    print("\n2. Testing double-conversion fix:")
    print("-" * 40)

    db = omendb.DB()
    db.clear()

    # Add orthogonal vectors
    v1 = [1.0, 0.0, 0.0, 0.0]
    v2 = [0.0, 1.0, 0.0, 0.0]
    v3 = [0.707, 0.707, 0.0, 0.0]  # 45 degrees from v1

    db.add("v1", v1)
    db.add("v2", v2)
    db.add("v3", v3)

    # Search for v1
    results = db.search(v1, limit=3)

    scores = {r.id: r.score for r in results}
    print(f"  v1 similarity to v1: {scores.get('v1', 0):.4f} (expected 1.0)")
    print(f"  v1 similarity to v3: {scores.get('v3', 0):.4f} (expected ~0.707)")
    print(f"  v1 similarity to v2: {scores.get('v2', 0):.4f} (expected 0.0)")

    assert "v1" in scores, "v1 not found in results"
    assert abs(scores["v1"] - 1.0) < 0.01, f"Wrong score for v1: {scores['v1']}"

    assert "v3" in scores, "v3 not found in results"
    assert abs(scores["v3"] - 0.707) < 0.01, f"Wrong score for v3: {scores['v3']}"

    # v2 should have lowest score (orthogonal)
    if "v2" in scores:
        assert abs(scores["v2"] - 0.0) < 0.01, f"Wrong score for v2: {scores['v2']}"
    print("  ✅ Double-conversion fix working")


def test_batch_corruption_fix():
    """Test that batch processing uses correct padded dimensions."""
    print("\n3. Testing batch corruption fix:")
    print("-" * 40)

    db = omendb.DB()
    db.clear()

    # Create vectors with dimension that requires padding
    dim = 10  # Will be padded to 16 for SIMD
    vectors = np.random.randn(100, dim).astype(np.float32)
    # Normalize
    vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

    ids = [f"vec_{i}" for i in range(100)]

    # Batch add
    results = db.add_batch(vectors, ids)
    print(f"  Added {len(results)}/{len(ids)} vectors")

    # Verify all vectors are searchable
    found_count = 0
    for i in range(10):  # Test first 10
        query = vectors[i]
        results = db.search(query, limit=1)
        if results and results[0].id == ids[i]:
            found_count += 1

    print(f"  Found {found_count}/10 vectors correctly")
    assert found_count == 10, f"Only found {found_count}/10 vectors"
    print("  ✅ Batch corruption fix working")


def test_numpy_batch_fix():
    """Test that numpy batch processing works with global DB."""
    print("\n4. Testing numpy batch fix:")
    print("-" * 40)

    # Test 1: Lists
    db = omendb.DB()
    db.clear()

    vecs_list = [[1.0, 0.0], [0.0, 1.0], [0.707, 0.707]]
    ids = ["v1", "v2", "v3"]
    results = db.add_batch(vecs_list, ids)
    print(f"  List batch: {len(results)}/{len(ids)} added")

    # Test 2: NumPy (requires clear due to single global DB)
    db.clear()
    vecs_np = np.array(vecs_list, dtype=np.float32)
    results = db.add_batch(vecs_np, ids)
    print(f"  NumPy batch: {len(results)}/{len(ids)} added")

    # Verify search works
    results = db.search([1.0, 0.0], limit=3)
    scores = [r.score for r in results]
    print(f"  Search scores: {scores}")

    # We may get 2 or 3 results depending on score filtering
    assert len(results) >= 2, f"Expected at least 2 results, got {len(results)}"
    assert abs(scores[0] - 1.0) < 0.01, "Wrong best score"
    print("  ✅ NumPy batch fix working")


def test_search_accuracy():
    """Test that search returns correct number of results."""
    print("\n5. Testing search accuracy:")
    print("-" * 40)

    db = omendb.DB()
    db.clear()

    # Add 10 vectors
    for i in range(10):
        vec = [0.0] * 128
        vec[i] = 1.0  # Different dimensions active
        db.add(f"vec_{i}", vec)

    # Search should find all 10
    query = [0.1] * 128  # Slightly similar to all
    results = db.search(query, limit=10)

    print(f"  Added 10 vectors")
    print(f"  Search returned {len(results)} results")

    assert len(results) == 10, f"Expected 10 results, got {len(results)}"

    # Verify IDs are unique
    ids = [r.id for r in results]
    unique_ids = set(ids)
    print(f"  Unique IDs: {len(unique_ids)}")

    assert len(unique_ids) == 10, f"Duplicate IDs in results"
    print("  ✅ Search accuracy working")


def main():
    """Run all bug fix tests."""
    print("🧪 OmenDB Bug Fix Verification Suite")
    print("=" * 60)

    try:
        test_warmup_dimension_fix()
        test_double_conversion_fix()
        test_batch_corruption_fix()
        test_numpy_batch_fix()
        test_search_accuracy()

        print("\n" + "=" * 60)
        print("✅ ALL BUG FIXES VERIFIED!")
        print("=" * 60)

    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
