#!/usr/bin/env python3
"""
Basic CRUD Test
===============

Simple test to verify basic CRUD operations work.
"""

import numpy as np
import sys
import os
from omendb import DB


def test_basic_operations():
    """Test basic CRUD operations."""
    print("🧪 Testing Basic CRUD Operations")
    print("=" * 50)

    # Create database
    db = DB()

    # Test 1: Add a vector (use 64D for compatibility with init)
    print("\n1. Testing ADD...")
    vec1 = np.random.rand(64).astype(np.float32).tolist()
    success = db.add("vec1", vec1, {"type": "test"})
    print(f"   Add result: {success}")
    assert success == True, "Failed to add vector"

    # Test 2: Check exists
    print("\n2. Testing EXISTS...")
    exists = db.exists("vec1")
    print(f"   Exists result: {exists}")
    assert exists == True, "Vector should exist"

    # Test 3: Get vector
    print("\n3. Testing GET VECTOR...")
    try:
        result = db.get("vec1")
        retrieved = result[0] if result else None
        print(f"   Retrieved: {retrieved}")
        if retrieved:
            np.testing.assert_array_almost_equal(retrieved, vec1, decimal=5)
            print("   ✅ Vector matches!")
    except Exception as e:
        print(f"   ❌ Get vector failed: {e}")

    # Test 4: Get metadata
    print("\n4. Testing GET METADATA...")
    try:
        result = db.get("vec1")
        metadata = result[1] if result else {}
        print(f"   Metadata: {metadata}")
    except Exception as e:
        print(f"   ❌ Get metadata failed: {e}")

    # Test 5: Query
    print("\n5. Testing QUERY...")
    results = db.search(vec1, limit=5)
    print(f"   Query results: {results}")
    assert len(results) > 0, "Query should return results"
    # Check if we found our vector
    found = False
    for result in results:
        if result.id == "vec1":
            found = True
            print(f"   ✅ Found vec1 with similarity: {result.score}")
            break
    assert found, "Should find vec1 in results"

    # Test 6: Update
    print("\n6. Testing UPDATE...")
    vec2 = np.random.rand(64).astype(np.float32).tolist()
    try:
        success = db.add("vec1", vec2, {"type": "updated"})
        print(f"   Update result: {success}")
        assert success == True, "Update should succeed"
    except Exception as e:
        print(f"   ❌ Update failed: {e}")

    # Test 7: Delete
    print("\n7. Testing DELETE...")
    try:
        success = db.delete("vec1")
        print(f"   Delete result: {success}")
        assert success == True, "Delete should succeed"

        exists = db.exists("vec1")
        print(f"   Exists after delete: {exists}")
        assert exists == False, "Vector should not exist after delete"
    except Exception as e:
        print(f"   ❌ Delete failed: {e}")

    print("\n✅ Basic CRUD tests complete!")


if __name__ == "__main__":
    test_basic_operations()
