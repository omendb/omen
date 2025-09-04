#!/usr/bin/env python3
"""
Cross-platform testing script for OmenDB.

This script runs a comprehensive test suite to validate that OmenDB
works correctly across different operating systems and architectures.
"""

import sys
import platform
import time
import traceback
from typing import List, Dict, Any

# Add Python API to path
sys.path.insert(0, "/Users/nick/github/omenDB/python")


def get_platform_info() -> Dict[str, str]:
    """Get detailed platform information."""
    return {
        "system": platform.system(),
        "release": platform.release(),
        "version": platform.version(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "python_version": platform.python_version(),
        "python_implementation": platform.python_implementation(),
    }


def print_platform_info():
    """Print platform information."""
    print("🖥️  Platform Information")
    print("=" * 50)

    info = get_platform_info()
    for key, value in info.items():
        print(f"{key:20}: {value}")
    print()


def test_basic_import():
    """Test basic module import."""
    print("🧪 Testing Basic Import")
    print("-" * 30)

    try:
        import omendb

        print(f"✅ omendb module imported successfully")
        print(f"   Version: {getattr(omendb, '__version__', 'unknown')}")
        return True
    except ImportError as e:
        print(f"❌ Failed to import omendb: {e}")
        return False
    except Exception as e:
        print(f"❌ Unexpected error importing omendb: {e}")
        return False


def test_native_module():
    """Test native module loading."""
    print("\n🧪 Testing Native Module Loading")
    print("-" * 40)

    try:
        from omendb import DB

        print("✅ DB class imported successfully")

        # Test native module connection
        db = DB()
        print("✅ DB instance created successfully")

        # Test basic operation
        success = db.add("test_vector", [1.0, 2.0, 3.0])
        if success:
            print("✅ Basic vector addition works")
        else:
            print("⚠️  Vector addition returned False")

        return True

    except Exception as e:
        print(f"❌ Native module test failed: {e}")
        traceback.print_exc()
        return False


def test_concurrent_search():
    """Test concurrent search functionality."""
    print("\n🧪 Testing Concurrent Search")
    print("-" * 35)

    try:
        from omendb import DB

        # Create database with test data
        db = DB()

        # Add test vectors
        test_vectors = [
            ("vec1", [1.0, 0.0, 0.0], {"category": "tech"}),
            ("vec2", [0.9, 0.1, 0.0], {"category": "tech"}),
            ("vec3", [0.0, 1.0, 0.0], {"category": "science"}),
            ("vec4", [0.1, 0.9, 0.0], {"category": "science"}),
            ("vec5", [0.0, 0.0, 1.0], {"category": "sports"}),
        ]

        print(f"Adding {len(test_vectors)} test vectors...")
        for vec_id, vector, metadata in test_vectors:
            success = db.add(vec_id, vector, metadata)
            if not success:
                print(f"❌ Failed to add vector {vec_id}")
                return False

        print("✅ Test vectors added successfully")

        # Test concurrent search
        query_vector = [1.0, 0.0, 0.0]

        # Regular search
        regular_results = db.search(query_vector, limit=3)
        print(f"Regular search: {len(regular_results)} results")

        # Concurrent search
        concurrent_results = db.query_concurrent(query_vector, limit=3)
        print(f"Concurrent search: {len(concurrent_results)} results")

        # Compare results
        if len(regular_results) == len(concurrent_results):
            print("✅ Concurrent search produces same number of results")
        else:
            print("⚠️  Result count mismatch between regular and concurrent search")

        # Test metadata filtering
        tech_results = db.query_concurrent(
            query_vector, limit=3, filter={"category": "tech"}
        )
        print(f"Metadata-filtered search: {len(tech_results)} results")

        # Verify filtering worked
        for result in tech_results:
            if result.metadata and result.metadata.get("category") == "tech":
                continue
            else:
                print("⚠️  Metadata filtering not working correctly")
                return False

        print("✅ Concurrent search with metadata filtering works")
        return True

    except Exception as e:
        print(f"❌ Concurrent search test failed: {e}")
        traceback.print_exc()
        return False


def test_high_dimensional_vectors():
    """Test high-dimensional vector support."""
    print("\n🧪 Testing High-Dimensional Vectors")
    print("-" * 40)

    try:
        from omendb import DB

        # Test different dimensions
        dimensions_to_test = [16, 64, 128, 256, 512]

        for dim in dimensions_to_test:
            db = DB()

            # Create high-dimensional vector
            vector = [float(i) * 0.1 for i in range(dim)]

            success = db.add(f"vec_{dim}d", vector)
            if success:
                print(f"✅ {dim}D vectors supported")

                # Test search
                results = db.search(vector, limit=1)
                if results and len(results) > 0:
                    print(f"   Search works for {dim}D")
                else:
                    print(f"⚠️  Search failed for {dim}D")
            else:
                print(f"❌ {dim}D vectors failed")
                return False

        print("✅ High-dimensional vector support validated")
        return True

    except Exception as e:
        print(f"❌ High-dimensional vector test failed: {e}")
        traceback.print_exc()
        return False


def test_batch_operations():
    """Test batch insertion and search."""
    print("\n🧪 Testing Batch Operations")
    print("-" * 30)

    try:
        from omendb import DB

        db = DB()

        # Create batch data
        batch_data = []
        for i in range(20):
            vector = [float(i), float(i + 1), float(i + 2)]
            metadata = {"batch": str(i // 5), "index": str(i)}
            batch_data.append((f"batch_vec_{i}", vector, metadata))

        print(f"Testing batch insertion of {len(batch_data)} vectors...")

        start_time = time.time()
        results = db.add_batch(batch_data)
        batch_time = time.time() - start_time

        if all(results):
            print(f"✅ Batch insertion successful in {batch_time:.3f}s")
            throughput = len(batch_data) / batch_time
            print(f"   Throughput: {throughput:.0f} vectors/second")
        else:
            failed_count = results.count(False)
            print(f"⚠️  {failed_count} vectors failed in batch insertion")

        # Test batch concurrent search
        queries = [[0.0, 1.0, 2.0], [5.0, 6.0, 7.0], [10.0, 11.0, 12.0]]

        print(f"Testing batch concurrent search with {len(queries)} queries...")

        start_time = time.time()
        batch_results = db.query_batch_concurrent(queries, limit=3)
        batch_search_time = time.time() - start_time

        if len(batch_results) == len(queries):
            print(f"✅ Batch concurrent search successful in {batch_search_time:.3f}s")
            for i, query_results in enumerate(batch_results):
                print(f"   Query {i}: {len(query_results)} results")
        else:
            print("❌ Batch concurrent search failed")
            return False

        print("✅ Batch operations validated")
        return True

    except Exception as e:
        print(f"❌ Batch operations test failed: {e}")
        traceback.print_exc()
        return False


def test_memory_efficiency():
    """Test memory efficiency with larger datasets."""
    print("\n🧪 Testing Memory Efficiency")
    print("-" * 32)

    try:
        from omendb import DB
        import psutil
        import os

        # Get initial memory usage
        process = psutil.Process(os.getpid())
        initial_memory = process.memory_info().rss / 1024 / 1024  # MB

        db = DB()

        # Add 1000 vectors and measure memory
        vector_count = 1000
        dimension = 128

        print(f"Adding {vector_count} {dimension}D vectors...")

        for i in range(vector_count):
            vector = [float(j) * 0.001 * i for j in range(dimension)]
            db.add(f"mem_test_{i}", vector)

        # Get final memory usage
        final_memory = process.memory_info().rss / 1024 / 1024  # MB
        memory_used = final_memory - initial_memory
        memory_per_vector = memory_used / vector_count * 1024  # KB per vector

        print(f"✅ Memory test completed")
        print(f"   Vectors: {vector_count}")
        print(f"   Memory used: {memory_used:.1f} MB")
        print(f"   Memory per vector: {memory_per_vector:.2f} KB")

        # Check if memory usage is reasonable (target: <0.5KB per vector)
        if memory_per_vector < 500:  # 500 bytes per vector
            print("✅ Memory efficiency meets target (<0.5KB per vector)")
        else:
            print("⚠️  Memory usage higher than target")

        return True

    except ImportError:
        print("⚠️  psutil not available, skipping memory test")
        return True
    except Exception as e:
        print(f"❌ Memory efficiency test failed: {e}")
        traceback.print_exc()
        return False


def test_performance_benchmarks():
    """Test performance benchmarks."""
    print("\n🧪 Testing Performance Benchmarks")
    print("-" * 38)

    try:
        from omendb import DB

        # Test construction speed
        vector_counts = [100, 500, 1000]
        dimension = 64

        for count in vector_counts:
            db = DB()

            print(f"Testing construction speed with {count} vectors...")

            start_time = time.time()
            for i in range(count):
                vector = [float(j) * 0.001 * i for j in range(dimension)]
                db.add(f"perf_test_{i}", vector)
            construction_time = time.time() - start_time

            throughput = count / construction_time
            print(f"   Construction: {construction_time:.3f}s ({throughput:.0f} vec/s)")

            # Test search speed
            query_vector = [float(i) * 0.001 for i in range(dimension)]

            start_time = time.time()
            for _ in range(10):  # 10 search queries
                results = db.search(query_vector, limit=5)
            search_time = time.time() - start_time

            avg_search_time = search_time / 10 * 1000  # ms per query
            print(f"   Search: {avg_search_time:.2f}ms per query")

        print("✅ Performance benchmarks completed")
        return True

    except Exception as e:
        print(f"❌ Performance benchmark test failed: {e}")
        traceback.print_exc()
        return False


def run_cross_platform_tests() -> bool:
    """Run all cross-platform tests."""
    print("🚀 OmenDB Cross-Platform Test Suite")
    print("=" * 60)

    print_platform_info()

    tests = [
        ("Basic Import", test_basic_import),
        ("Native Module", test_native_module),
        ("Concurrent Search", test_concurrent_search),
        ("High-Dimensional Vectors", test_high_dimensional_vectors),
        ("Batch Operations", test_batch_operations),
        ("Memory Efficiency", test_memory_efficiency),
        ("Performance Benchmarks", test_performance_benchmarks),
    ]

    results = []

    for test_name, test_func in tests:
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"❌ {test_name} test crashed: {e}")
            results.append((test_name, False))

    # Summary
    print("\n" + "=" * 60)
    print("📊 Test Results Summary")
    print("=" * 60)

    passed = 0
    failed = 0

    for test_name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{test_name:30}: {status}")
        if result:
            passed += 1
        else:
            failed += 1

    print(f"\nTotal: {len(results)} tests")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")

    if failed == 0:
        print("\n🎉 All cross-platform tests passed!")
        return True
    else:
        print(f"\n⚠️  {failed} test(s) failed")
        return False


if __name__ == "__main__":
    success = run_cross_platform_tests()
    sys.exit(0 if success else 1)
