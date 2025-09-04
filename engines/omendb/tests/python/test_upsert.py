#!/usr/bin/env python3
"""Test upsert functionality."""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

import omendb
# import pytest


def test_upsert_new_vector():
    """Test upserting a new vector (insert behavior)."""
    db = omendb.DB()

    # Upsert a new vector
    success = db.upsert("vec1", [1.0, 2.0, 3.0], {"type": "test"})
    assert success == True

    # Verify it was added
    assert db.count() == 1
    assert db.exists("vec1") == True

    # Get the vector and check metadata
    result = db.get("vec1")
    assert result is not None, "Vector should exist after upsert"
    vector, metadata = result
    assert vector == [1.0, 2.0, 3.0]
    assert metadata == {"type": "test"}


def test_upsert_existing_vector():
    """Test upserting an existing vector (update behavior)."""
    db = omendb.DB()

    # Add initial vector
    db.add("vec1", [1.0, 2.0, 3.0], {"type": "original"})
    assert db.count() == 1
    print(f"  Initial count: {db.count()}")

    # Upsert with new values
    success = db.upsert("vec1", [4.0, 5.0, 6.0], {"type": "updated"})
    print(f"  Upsert success: {success}")
    assert success == True

    # Verify count didn't change
    count_after = db.count()
    print(f"  Count after upsert: {count_after}")
    assert count_after == 1

    # Verify vector and metadata were updated
    result = db.get("vec1")
    if result is None:
        raise AssertionError("Vector not found after upsert")
    vector, metadata = result
    assert vector == [4.0, 5.0, 6.0]
    assert metadata == {"type": "updated"}


def test_upsert_batch():
    """Test batch upsert functionality."""
    db = omendb.DB()
    db.clear()  # Clear any existing state

    # Add some initial vectors
    db.add("vec1", [1.0, 2.0, 3.0], {"type": "original"})
    db.add("vec2", [4.0, 5.0, 6.0], {"type": "original"})

    # Batch upsert - mix of new and existing
    vectors = [
        [7.0, 8.0, 9.0],  # Update vec1
        [10.0, 11.0, 12.0],  # Update vec2
        [13.0, 14.0, 15.0],  # New vec3
    ]
    ids = ["vec1", "vec2", "vec3"]
    metadata = [{"type": "updated"}, {"type": "updated"}, {"type": "new"}]

    result_ids = db.upsert_batch(vectors=vectors, ids=ids, metadata=metadata)
    assert len(result_ids) == 3
    assert set(result_ids) == {"vec1", "vec2", "vec3"}

    # Verify count increased by 1 (only vec3 was new)
    assert db.count() == 3

    # Verify updates
    result1 = db.get("vec1")
    assert result1 is not None
    vec1, meta1 = result1
    print(f"  vec1 after upsert_batch: {vec1}")
    print(f"  Expected: [7.0, 8.0, 9.0]")
    assert vec1 == [7.0, 8.0, 9.0]
    assert meta1 == {"type": "updated"}

    result3 = db.get("vec3")
    assert result3 is not None
    vec3, meta3 = result3
    assert vec3 == [13.0, 14.0, 15.0]
    assert meta3 == {"type": "new"}


def test_upsert_dimension_mismatch():
    """Test upsert with dimension mismatch."""
    db = omendb.DB()

    # Add initial vector
    db.add("vec1", [1.0, 2.0, 3.0])

    # Try to upsert with different dimension
    try:
        db.upsert("vec1", [1.0, 2.0, 3.0, 4.0])
        assert False, "Should have raised ValidationError"
    except omendb.ValidationError as e:
        assert "Dimension mismatch" in str(e)


def test_upsert_empty_database():
    """Test upsert on empty database (pure insert)."""
    db = omendb.DB()
    db.clear()  # Ensure we start with empty database

    # Upsert into empty database
    success = db.upsert("vec1", [1.0, 2.0, 3.0])
    print(f"  Upsert to empty DB success: {success}")
    assert success == True
    count = db.count()
    print(f"  Count after upsert to empty DB: {count}")
    assert count == 1


if __name__ == "__main__":
    # Run all tests
    print("Testing upsert functionality...\n")

    passed = 0
    failed = 0

    tests = [
        ("test_upsert_new_vector", test_upsert_new_vector),
        ("test_upsert_existing_vector", test_upsert_existing_vector),
        ("test_upsert_batch", test_upsert_batch),
        ("test_upsert_dimension_mismatch", test_upsert_dimension_mismatch),
        ("test_upsert_empty_database", test_upsert_empty_database),
    ]

    for test_name, test_func in tests:
        try:
            test_func()
            print(f"✅ {test_name} passed")
            passed += 1
        except Exception as e:
            import traceback

            print(f"❌ {test_name} failed: {e}")
            traceback.print_exc()
            failed += 1

    print(f"\nResults: {passed} passed, {failed} failed")
    if failed > 0:
        sys.exit(1)
