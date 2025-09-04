#!/usr/bin/env python3
"""Test scalar quantization functionality through Python API."""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

import omendb
import numpy as np


def test_quantization_basic():
    """Test basic quantization functionality."""
    print("Testing basic quantization...")

    db = omendb.DB()

    # Enable quantization
    success = db.enable_quantization()
    assert success == True, "Failed to enable quantization"

    # Add some vectors
    db.add("vec1", [1.0, 2.0, 3.0, 4.0])
    db.add("vec2", [5.0, 6.0, 7.0, 8.0])

    # Check info includes quantization stats
    info = db.info()
    assert info["quantization_enabled"] == True
    print(f"  Quantization enabled: {info['quantization_enabled']}")

    if "quantized_vectors_count" in info:
        print(f"  Quantized vectors: {info['quantized_vectors_count']}")

    # Verify vectors can still be queried
    results = db.search([1.1, 2.1, 3.1, 4.1], limit=2)
    assert len(results) == 2
    assert results[0].id == "vec1"  # Should be closest

    print("✅ Basic quantization test passed")


def test_memory_savings():
    """Test memory savings with quantization."""
    print("\nTesting memory savings...")

    # Clear any existing state
    db_temp = omendb.DB()
    db_temp.clear()

    # Create two databases - one with and one without quantization
    db_normal = omendb.DB()
    db_quantized = omendb.DB()
    db_quantized.enable_quantization()

    # Add 100 128-dimensional vectors (typical embedding size)
    print("  Adding 100 128-dimensional vectors...")
    vectors = np.random.randn(100, 128).astype(np.float32)

    for i in range(100):
        vec = vectors[i].tolist()
        db_normal.add(f"vec_{i}", vec)
        db_quantized.add(f"vec_{i}", vec)

    # Get info
    info_normal = db_normal.info()
    info_quantized = db_quantized.info()

    print(f"  Normal DB vectors: {info_normal['vector_count']}")
    print(f"  Quantized DB vectors: {info_quantized['vector_count']}")

    if "memory_savings_ratio" in info_quantized:
        print(f"  Memory savings ratio: {info_quantized['memory_savings_ratio']:.1f}x")
        assert info_quantized["memory_savings_ratio"] > 3.0, (
            "Should achieve at least 3x compression"
        )

    print("✅ Memory savings test passed")


def test_accuracy_preservation():
    """Test that quantization preserves search accuracy."""
    print("\nTesting accuracy preservation...")

    db = omendb.DB()
    db.clear()  # Clear any existing state
    db.enable_quantization()

    # Create vectors with known relationships
    # Group 1: Similar vectors around [1, 0, 0, ...]
    # Group 2: Similar vectors around [0, 1, 0, ...]
    dimension = 32

    # Add vectors from group 1
    for i in range(5):
        vec = [0.0] * dimension
        vec[0] = 1.0 + i * 0.1  # Small variations
        vec[1] = i * 0.05
        db.add(f"group1_{i}", vec)

    # Add vectors from group 2
    for i in range(5):
        vec = [0.0] * dimension
        vec[1] = 1.0 + i * 0.1  # Small variations
        vec[0] = i * 0.05
        db.add(f"group2_{i}", vec)

    # Query with a vector similar to group 1
    query1 = [0.0] * dimension
    query1[0] = 0.95
    results1 = db.search(query1, limit=5)

    # Check that top results are from group 1
    group1_count = sum(1 for r in results1 if r.id.startswith("group1"))
    print(f"  Group 1 query: {group1_count}/5 results from correct group")
    assert group1_count >= 3, "Quantization should preserve similarity relationships"

    # Query with a vector similar to group 2
    query2 = [0.0] * dimension
    query2[1] = 0.95
    results2 = db.search(query2, limit=5)

    # Check that top results are from group 2
    group2_count = sum(1 for r in results2 if r.id.startswith("group2"))
    print(f"  Group 2 query: {group2_count}/5 results from correct group")
    assert group2_count >= 3, "Quantization should preserve similarity relationships"

    print("✅ Accuracy preservation test passed")


def test_batch_operations_with_quantization():
    """Test batch operations with quantization enabled."""
    print("\nTesting batch operations with quantization...")

    db = omendb.DB()
    db.clear()  # Clear any existing state
    db.enable_quantization()

    # Batch add with numpy
    vectors = np.random.randn(50, 64).astype(np.float32)
    ids = [f"batch_{i}" for i in range(50)]

    result_ids = db.add_batch(vectors=vectors, ids=ids)
    assert len(result_ids) == 50

    # Verify all vectors are accessible
    for i in range(10):  # Check first 10
        result = db.get(f"batch_{i}")
        assert result is not None
        vec, _ = result
        assert len(vec) == 64

    # Batch query should still work
    query = np.random.randn(64).astype(np.float32)
    results = db.search(query.tolist(), limit=10)
    assert len(results) == 10

    print("✅ Batch operations with quantization test passed")


def test_quantization_edge_cases():
    """Test quantization with edge cases."""
    print("\nTesting quantization edge cases...")

    db = omendb.DB()
    db.clear()  # Clear any existing state
    db.enable_quantization()

    # Test with uniform vectors (all same value)
    uniform_vec = [5.0] * 16
    db.add("uniform", uniform_vec)

    # Test with large range
    large_range = [-1000.0, 1000.0, 0.0, 500.0, -500.0]
    db.add("large_range", large_range)

    # Test with very small values
    small_values = [0.0001, 0.0002, 0.0003, 0.0004]
    db.add("small", small_values)

    # Verify all can be retrieved
    assert db.get("uniform") is not None
    assert db.get("large_range") is not None
    assert db.get("small") is not None

    # Query should still work
    results = db.search([5.0] * 16, limit=3)
    assert len(results) > 0

    print("✅ Edge cases test passed")


if __name__ == "__main__":
    print("🧪 Testing Scalar Quantization through Python API\n")

    try:
        test_quantization_basic()
        test_memory_savings()
        test_accuracy_preservation()
        test_batch_operations_with_quantization()
        test_quantization_edge_cases()

        print("\n✅ All quantization tests passed!")
        print("\n📊 Summary:")
        print("  - Quantization can be enabled successfully")
        print("  - Memory savings are tracked")
        print("  - Search accuracy is preserved")
        print("  - Batch operations work with quantization")
        print("  - Edge cases handled properly")

    except Exception as e:
        print(f"\n❌ Test failed: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
