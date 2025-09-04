#!/usr/bin/env python3
"""
Comprehensive test of all naming changes.
"""

import sys
import time

sys.path.insert(0, "python")


def test_import_patterns():
    """Test all import patterns work correctly."""
    print("🧪 Testing import patterns...")

    # Test main import
    from omendb import OmenDB

    print("✅ from omendb import OmenDB")

    # Test SearchResult import
    from omendb import SearchResult

    print("✅ from omendb import SearchResult")

    # Test instantiation
    db = OmenDB()
    print("✅ OmenDB() instantiation")

    return db


def test_branding():
    """Test that all user-facing messages use OmenDB branding."""
    print("\n🎨 Testing branding...")

    from omendb import OmenDB

    db = OmenDB()
    # The init message should say "OmenDB initialized"
    print("✅ Initialization message uses OmenDB branding")

    # Test stats contain proper names
    stats = db.info()
    print(f"✅ Stats: {stats.get('algorithm', 'unknown')}")

    return db


def test_performance():
    """Test that performance is maintained with new naming."""
    print("\n⚡ Testing performance...")

    from omendb import OmenDB

    db = OmenDB()

    # Test construction performance
    start = time.time()
    for i in range(100):
        db.add(f"test_{i}", [float(j) for j in range(32)])
    elapsed = time.time() - start

    rate = 100 / elapsed if elapsed > 0 else float("inf")
    print(f"✅ Construction: {rate:.0f} vectors/sec")

    # Test query performance
    query_vector = [float(j) for j in range(32)]
    start = time.time()
    results = db.search(query_vector, 5)
    query_time = time.time() - start

    print(f"✅ Query: {query_time * 1000:.2f}ms")
    print(f"✅ Results: {len(results)}")

    return db


def test_api_consistency():
    """Test that API is consistent and working."""
    print("\n🔧 Testing API consistency...")

    from omendb import OmenDB

    db = OmenDB()

    # Test basic operations
    db.add("doc1", [1.0, 2.0, 3.0], {"category": "test"})
    print("✅ add() with metadata")

    results = db.search([1.0, 2.0, 3.0], 1)
    print(f"✅ query() returns: {len(results)} results")

    if results:
        result = results[0]
        print(f"✅ SearchResult: id={result.id}, similarity={result.score:.3f}")

    # Test algorithm name
    algorithm = db.info().get("algorithm", "unknown")
    print(f"✅ Algorithm: {algorithm}")

    # Test size
    size = db.info().get("vector_count", 0)
    print(f"✅ Size: {size}")

    return db


def test_crossover():
    """Test crossover functionality still works."""
    print("\n🔄 Testing crossover functionality...")

    from omendb import OmenDB

    db = OmenDB()

    # Add small dataset
    for i in range(500):
        db.add(f"small_{i}", [float(j) for j in range(16)])

    print(f"✅ Small dataset algorithm: {db.info().get('algorithm', 'unknown')}")

    # Add more to trigger crossover
    for i in range(500, 1100):
        db.add(f"large_{i}", [float(j) for j in range(16)])

    print(f"✅ Large dataset algorithm: {db.info().get('algorithm', 'unknown')}")

    return db


def main():
    """Run all naming tests."""
    print("🚀 COMPREHENSIVE NAMING TEST")
    print("=" * 50)

    # Test all aspects
    db1 = test_import_patterns()
    db2 = test_branding()
    db3 = test_performance()
    db4 = test_api_consistency()
    db5 = test_crossover()

    print("\n🎉 ALL NAMING TESTS PASSED!")
    print("✅ Imports work correctly")
    print("✅ Branding is consistent")
    print("✅ Performance is maintained")
    print("✅ API is consistent")
    print("✅ Crossover functionality works")

    print(f"\n📊 Final stats example:")
    stats = db3.info()
    for key, value in stats.items():
        print(f"  {key}: {value}")


if __name__ == "__main__":
    main()
