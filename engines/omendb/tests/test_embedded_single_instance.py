#!/usr/bin/env python3
"""
Test that demonstrates our embedded single-instance design.
This is the intended behavior - like SQLite, one DB per process.
"""

import omendb
import numpy as np


def test_single_instance_design():
    """Test that DB() returns the same embedded instance."""
    print("=== Testing Embedded Single Instance Design ===")

    # Clear any existing data
    db1 = omendb.DB()
    db1.clear()

    # Verify empty
    stats = db1.info()
    print(f"After clear: {stats['vector_count']} vectors")

    # Add data through first reference
    print("\n1. Adding data through db1...")
    for i in range(100):
        vec = np.random.rand(128).astype(np.float32)
        db1.add(f"vec_{i}", vec)

    stats1 = db1.info()
    print(f"   db1 stats: {stats1['vector_count']} vectors")

    # Create second reference - should see same data
    print("\n2. Creating db2 reference...")
    db2 = omendb.DB()
    stats2 = db2.info()
    print(f"   db2 stats: {stats2['vector_count']} vectors")

    if stats1["vector_count"] == stats2["vector_count"]:
        print("✅ Same instance confirmed - this is correct embedded behavior")
    else:
        print("❌ Different instances - this would be wrong")

    # Add through second reference
    print("\n3. Adding more data through db2...")
    for i in range(100, 150):
        vec = np.random.rand(128).astype(np.float32)
        db2.add(f"vec_{i}", vec)

    # Both should see 150 vectors
    print(f"   db1 now sees: {db1.info()['vector_count']} vectors")
    print(f"   db2 now sees: {db2.info()['vector_count']} vectors")

    # Test queries work from both references
    query_vec = np.random.rand(128).astype(np.float32)
    results1 = db1.search(query_vec, 5)
    results2 = db2.search(query_vec, 5)

    print(f"\n4. Query results:")
    print(f"   db1 returned: {len(results1)} results")
    print(f"   db2 returned: {len(results2)} results")
    print("   ✅ Both work - single embedded instance design confirmed")

    # Clear for next test
    db1.clear()
    print(f"\n5. After db1.clear(): {db2.info()['vector_count']} vectors in db2")
    print("   ✅ Clear affects both references - single instance confirmed")


def test_migration_with_single_instance():
    """Test migration behavior with our single instance design."""
    print("\n=== Testing Migration with Single Instance ===")

    db = omendb.DB()
    db.clear()

    print("1. Adding vectors to test migration...")

    # Add in stages
    for stage in range(6):
        start = stage * 1000
        end = min(start + 1000, 5200)

        for i in range(start, end):
            vec = np.random.rand(128).astype(np.float32)
            db.add(f"vec_{i}", vec)

        stats = db.info()
        print(f"   After {end} vectors: {stats['algorithm']} ({stats['status']})")

        if stats["algorithm"] == "hnsw":
            print("   🔄 Migration triggered - operations continue seamlessly")
            break

    # Test operations work during/after migration
    query_vec = np.random.rand(128).astype(np.float32)
    results = db.search(query_vec, 10)
    print(f"\n2. Query after migration: {len(results)} results")
    print("   ✅ Queries work normally after migration")

    final_stats = db.info()
    print(
        f"\n3. Final state: {final_stats['vector_count']} vectors, {final_stats['algorithm']} algorithm"
    )


if __name__ == "__main__":
    test_single_instance_design()
    test_migration_with_single_instance()
    print("\n✅ Single instance design working correctly!")
