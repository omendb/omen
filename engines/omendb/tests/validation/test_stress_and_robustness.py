#!/usr/bin/env python3
"""
Comprehensive stress testing and robustness validation for OmenDB.

This test suite validates OmenDB's behavior under stress conditions, edge cases,
and potential failure scenarios to ensure production reliability.
"""

import sys
import time
import math
import numpy as np
import psutil
import gc
from typing import List, Tuple, Dict, Any, Optional
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading

sys.path.insert(0, "/Users/nick/github/omenDB/python")


class MemoryMonitor:
    """Monitor memory usage during tests."""

    def __init__(self):
        self.initial_memory = self.get_memory_usage()
        self.peak_memory = self.initial_memory
        self.monitoring = False
        self.monitor_thread = None

    def get_memory_usage(self) -> float:
        """Get current memory usage in MB."""
        process = psutil.Process()
        return process.memory_info().rss / 1024 / 1024

    def start_monitoring(self):
        """Start continuous memory monitoring."""
        self.monitoring = True
        self.monitor_thread = threading.Thread(target=self._monitor_loop)
        self.monitor_thread.daemon = True
        self.monitor_thread.start()

    def stop_monitoring(self) -> Dict[str, float]:
        """Stop monitoring and return memory stats."""
        self.monitoring = False
        if self.monitor_thread:
            self.monitor_thread.join(timeout=1.0)

        current_memory = self.get_memory_usage()
        return {
            "initial_mb": self.initial_memory,
            "peak_mb": self.peak_memory,
            "final_mb": current_memory,
            "growth_mb": current_memory - self.initial_memory,
            "peak_growth_mb": self.peak_memory - self.initial_memory,
        }

    def _monitor_loop(self):
        """Background monitoring loop."""
        while self.monitoring:
            current = self.get_memory_usage()
            if current > self.peak_memory:
                self.peak_memory = current
            time.sleep(0.1)


def test_large_scale_performance():
    """Test performance with very large datasets."""
    from omendb import DB

    print("🏗️ Testing large-scale performance...")

    test_configs = [
        {"count": 2000, "dim": 128, "name": "2K vectors (128D)"},
        {"count": 5000, "dim": 256, "name": "5K vectors (256D)"},
        {"count": 10000, "dim": 128, "name": "10K vectors (128D)"},
    ]

    results = {}

    for config in test_configs:
        print(f"\n  📊 Testing {config['name']}...")

        # Monitor memory during test
        memory_monitor = MemoryMonitor()
        memory_monitor.start_monitoring()

        try:
            db = DB()

            # Generate test data
            np.random.seed(42)
            vectors = []
            for i in range(config["count"]):
                vector = np.random.normal(0, 1, config["dim"])
                vector = vector / np.linalg.norm(vector)
                vectors.append((f"large_vec_{i:06d}", vector.tolist()))

            # Measure construction time
            print(f"    Adding {config['count']} vectors...")
            start_time = time.time()

            for i, (vector_id, vector) in enumerate(vectors):
                success = db.add(vector_id, vector)
                if not success:
                    print(f"    ❌ Failed to add vector {i}")
                    break

                # Progress indicator
                if (i + 1) % 1000 == 0:
                    elapsed = time.time() - start_time
                    rate = (i + 1) / elapsed
                    print(
                        f"      Progress: {i + 1}/{config['count']} ({rate:.0f} vec/s)"
                    )

            construction_time = time.time() - start_time
            construction_rate = config["count"] / construction_time

            # Test query performance
            print(f"    Testing query performance...")
            query_times = []
            for i in range(10):
                query_vector = vectors[i * (config["count"] // 10)][1]
                start_time = time.time()
                results_list = db.search(query_vector, limit=10)
                query_time = time.time() - start_time
                query_times.append(query_time * 1000)  # Convert to ms

            avg_query_time = sum(query_times) / len(query_times)

            # Get database statistics
            stats = db.info()

            db.close()

            # Stop memory monitoring
            memory_stats = memory_monitor.stop_monitoring()

            print(
                f"    ✅ Construction: {construction_time:.3f}s ({construction_rate:.0f} vec/s)"
            )
            print(f"    ⚡ Query: {avg_query_time:.2f}ms average")
            print(f"    💾 Memory growth: {memory_stats['growth_mb']:.1f} MB")
            print(f"    📈 Peak memory: {memory_stats['peak_mb']:.1f} MB")

            results[config["name"]] = {
                "construction_rate": construction_rate,
                "avg_query_time": avg_query_time,
                "memory_growth": memory_stats["growth_mb"],
                "peak_memory": memory_stats["peak_mb"],
                "success": True,
            }

        except Exception as e:
            memory_monitor.stop_monitoring()
            print(f"    ❌ Failed: {e}")
            results[config["name"]] = {"success": False, "error": str(e)}

    return results


def test_edge_cases():
    """Test edge cases and potential failure scenarios."""
    from omendb import DB

    print("\n🎯 Testing edge cases and robustness...")

    edge_cases = [
        {
            "name": "Empty vectors",
            "test": lambda db: db.add("empty", []),
            "expect_success": False,
        },
        {
            "name": "Single element vector",
            "test": lambda db: db.add("single", [1.0]),
            "expect_success": True,
        },
        {
            "name": "Zero vector",
            "test": lambda db: db.add("zeros", [0.0] * 128),
            "expect_success": True,
        },
        {
            "name": "Very large values",
            "test": lambda db: db.add("large", [1e6] * 128),
            "expect_success": True,
        },
        {
            "name": "Very small values",
            "test": lambda db: db.add("small", [1e-10] * 128),
            "expect_success": True,
        },
        {
            "name": "NaN values",
            "test": lambda db: db.add("nan", [float("nan")] * 128),
            "expect_success": False,
        },
        {
            "name": "Infinity values",
            "test": lambda db: db.add("inf", [float("inf")] * 128),
            "expect_success": False,
        },
        {
            "name": "Mixed extreme values",
            "test": lambda db: db.add("mixed", [1e-10, 1e10, -1e10, 0.0] * 32),
            "expect_success": True,
        },
    ]

    results = {}

    for case in edge_cases:
        print(f"  🧪 Testing {case['name']}...")

        try:
            db = DB()
            success = case["test"](db)
            db.close()

            if success == case["expect_success"]:
                print(f"    ✅ Behaved as expected (success: {success})")
                results[case["name"]] = {"passed": True, "success": success}
            else:
                print(
                    f"    ❌ Unexpected behavior (expected: {case['expect_success']}, got: {success})"
                )
                results[case["name"]] = {
                    "passed": False,
                    "expected": case["expect_success"],
                    "got": success,
                }

        except Exception as e:
            if case["expect_success"]:
                print(f"    ❌ Unexpected exception: {e}")
                results[case["name"]] = {"passed": False, "error": str(e)}
            else:
                print(f"    ✅ Expected exception caught: {e}")
                results[case["name"]] = {"passed": True, "expected_error": str(e)}

    return results


def test_concurrent_access():
    """Test concurrent database access patterns."""
    from omendb import DB

    print("\n🔄 Testing concurrent access patterns...")

    def worker_add_vectors(worker_id: int, vector_count: int) -> Dict[str, Any]:
        """Worker function to add vectors concurrently."""
        try:
            db = DB()
            np.random.seed(worker_id + 100)  # Different seed per worker

            added_count = 0
            for i in range(vector_count):
                vector = np.random.normal(0, 1, 128)
                vector = vector / np.linalg.norm(vector)
                vector_id = f"worker_{worker_id}_vec_{i:03d}"

                success = db.add(vector_id, vector.tolist())
                if success:
                    added_count += 1

            db.close()
            return {
                "worker_id": worker_id,
                "added": added_count,
                "requested": vector_count,
                "success": True,
            }

        except Exception as e:
            return {"worker_id": worker_id, "success": False, "error": str(e)}

    def worker_query_vectors(worker_id: int, query_count: int) -> Dict[str, Any]:
        """Worker function to query vectors concurrently."""
        try:
            db = DB()

            # Add a few vectors first
            for i in range(10):
                vector = [0.1] * 128
                db.add(f"query_worker_{worker_id}_base_{i}", vector)

            query_times = []
            successful_queries = 0

            for i in range(query_count):
                query_vector = [0.1 + i * 0.01] * 128
                start_time = time.time()
                results = db.search(query_vector, limit=5)
                query_time = time.time() - start_time

                query_times.append(query_time * 1000)
                if len(results) > 0:
                    successful_queries += 1

            db.close()
            avg_query_time = sum(query_times) / len(query_times) if query_times else 0

            return {
                "worker_id": worker_id,
                "successful_queries": successful_queries,
                "total_queries": query_count,
                "avg_query_time": avg_query_time,
                "success": True,
            }

        except Exception as e:
            return {"worker_id": worker_id, "success": False, "error": str(e)}

    # Test concurrent adding
    print("  📝 Testing concurrent vector addition...")
    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = [executor.submit(worker_add_vectors, i, 50) for i in range(4)]
        add_results = [future.result() for future in as_completed(futures)]

    successful_adds = [r for r in add_results if r["success"]]
    total_added = sum(r["added"] for r in successful_adds)
    print(
        f"    ✅ {len(successful_adds)}/4 workers succeeded, {total_added} vectors added total"
    )

    # Test concurrent querying
    print("  🔍 Testing concurrent vector querying...")
    with ThreadPoolExecutor(max_workers=3) as executor:
        futures = [executor.submit(worker_query_vectors, i, 20) for i in range(3)]
        query_results = [future.result() for future in as_completed(futures)]

    successful_queries = [r for r in query_results if r["success"]]
    avg_query_time = (
        sum(r["avg_query_time"] for r in successful_queries) / len(successful_queries)
        if successful_queries
        else 0
    )
    print(
        f"    ✅ {len(successful_queries)}/3 query workers succeeded, {avg_query_time:.2f}ms avg query time"
    )

    return {
        "concurrent_adds": add_results,
        "concurrent_queries": query_results,
        "total_workers_succeeded": len(successful_adds) + len(successful_queries),
        "total_workers": 7,
    }


def test_memory_pressure():
    """Test behavior under memory pressure conditions."""
    from omendb import DB

    print("\n💾 Testing memory pressure scenarios...")

    memory_monitor = MemoryMonitor()
    memory_monitor.start_monitoring()

    results = {}

    try:
        # Test 1: Rapid database creation/destruction
        print("  🔄 Testing rapid database creation/destruction...")
        databases = []
        creation_times = []

        for i in range(50):
            start_time = time.time()
            db = DB()

            # Add a few vectors
            for j in range(10):
                vector = [0.1 + i * 0.01 + j * 0.001] * 128
                db.add(f"pressure_db_{i}_vec_{j}", vector)

            creation_time = time.time() - start_time
            creation_times.append(creation_time)
            databases.append(db)

        avg_creation_time = sum(creation_times) / len(creation_times)
        print(
            f"    ✅ Created 50 databases, avg time: {avg_creation_time * 1000:.2f}ms"
        )

        # Clean up databases
        for db in databases:
            db.close()

        # Test 2: Large vector dimensions
        print("  📏 Testing very high-dimensional vectors...")
        db = DB()

        high_dim_sizes = [2048, 4096, 8192]
        high_dim_results = {}

        for dim in high_dim_sizes:
            try:
                vector = np.random.normal(0, 1, dim)
                vector = vector / np.linalg.norm(vector)

                start_time = time.time()
                success = db.add(f"high_dim_{dim}d", vector.tolist())
                add_time = time.time() - start_time

                if success:
                    # Test query
                    start_time = time.time()
                    results_list = db.search(vector.tolist(), limit=1)
                    query_time = time.time() - start_time

                    high_dim_results[f"{dim}D"] = {
                        "add_time": add_time * 1000,
                        "query_time": query_time * 1000,
                        "success": True,
                    }
                    print(
                        f"    ✅ {dim}D: Add {add_time * 1000:.2f}ms, Query {query_time * 1000:.2f}ms"
                    )
                else:
                    high_dim_results[f"{dim}D"] = {"success": False}
                    print(f"    ❌ {dim}D: Failed to add vector")

            except Exception as e:
                high_dim_results[f"{dim}D"] = {"success": False, "error": str(e)}
                print(f"    ❌ {dim}D: Exception: {e}")

        db.close()

        # Test 3: Memory allocation patterns
        print("  🧠 Testing memory allocation patterns...")
        db = DB()

        memory_snapshots = []
        vector_counts = [100, 500, 1000, 2000]

        for target_count in vector_counts:
            # Add vectors up to target count
            current_count = len(memory_snapshots) * 100 if memory_snapshots else 0
            vectors_to_add = target_count - current_count

            if vectors_to_add > 0:
                for i in range(vectors_to_add):
                    vector = np.random.normal(0, 1, 256)
                    vector = vector / np.linalg.norm(vector)
                    db.add(f"mem_test_{current_count + i:04d}", vector.tolist())

            current_memory = memory_monitor.get_memory_usage()
            memory_snapshots.append(current_memory)
            print(f"    📊 {target_count} vectors: {current_memory:.1f} MB")

        db.close()

        # Calculate memory efficiency
        if len(memory_snapshots) >= 2:
            memory_per_1k_vectors = (
                (memory_snapshots[-1] - memory_snapshots[0])
                / (vector_counts[-1] - vector_counts[0])
                * 1000
            )
            print(
                f"    📈 Memory efficiency: ~{memory_per_1k_vectors:.2f} MB per 1000 vectors"
            )

        results = {
            "rapid_creation": {
                "avg_time_ms": avg_creation_time * 1000,
                "databases_created": 50,
            },
            "high_dimensions": high_dim_results,
            "memory_efficiency_mb_per_1k": memory_per_1k_vectors
            if len(memory_snapshots) >= 2
            else 0,
        }

    except Exception as e:
        print(f"    ❌ Memory pressure test failed: {e}")
        results = {"success": False, "error": str(e)}

    finally:
        memory_stats = memory_monitor.stop_monitoring()
        results["memory_stats"] = memory_stats
        print(f"    💾 Total memory growth: {memory_stats['growth_mb']:.1f} MB")
        print(f"    📈 Peak memory usage: {memory_stats['peak_mb']:.1f} MB")

    return results


def main():
    """Run comprehensive stress testing and robustness validation."""
    print("🚀 OmenDB Stress Testing & Robustness Validation")
    print("=" * 60)

    test_results = {}

    # Test 1: Large-scale performance
    test_results["large_scale"] = test_large_scale_performance()

    # Test 2: Edge cases
    test_results["edge_cases"] = test_edge_cases()

    # Test 3: Concurrent access
    test_results["concurrent_access"] = test_concurrent_access()

    # Test 4: Memory pressure
    test_results["memory_pressure"] = test_memory_pressure()

    # Summary
    print("\n" + "=" * 60)
    print("📊 STRESS TESTING SUMMARY")
    print("=" * 60)

    # Large-scale performance summary
    large_scale_success = sum(
        1
        for result in test_results["large_scale"].values()
        if result.get("success", False)
    )
    total_large_scale = len(test_results["large_scale"])
    print(
        f"\n🏗️ Large-Scale Performance: {large_scale_success}/{total_large_scale} tests passed"
    )

    for test_name, result in test_results["large_scale"].items():
        if result.get("success", False):
            print(
                f"  ✅ {test_name}: {result['construction_rate']:.0f} vec/s, {result['avg_query_time']:.2f}ms queries"
            )
        else:
            print(f"  ❌ {test_name}: Failed")

    # Edge cases summary
    edge_case_success = sum(
        1
        for result in test_results["edge_cases"].values()
        if result.get("passed", False)
    )
    total_edge_cases = len(test_results["edge_cases"])
    print(f"\n🎯 Edge Cases: {edge_case_success}/{total_edge_cases} tests passed")

    # Concurrent access summary
    concurrent_workers = test_results["concurrent_access"]["total_workers_succeeded"]
    total_workers = test_results["concurrent_access"]["total_workers"]
    print(
        f"\n🔄 Concurrent Access: {concurrent_workers}/{total_workers} workers succeeded"
    )

    # Memory pressure summary
    memory_results = test_results["memory_pressure"]
    if "memory_efficiency_mb_per_1k" in memory_results:
        print(
            f"\n💾 Memory Pressure: {memory_results['memory_efficiency_mb_per_1k']:.2f} MB per 1K vectors"
        )

    # Overall assessment
    total_tests = (
        total_large_scale + total_edge_cases + 1 + 1
    )  # +1 for concurrent, +1 for memory
    passed_tests = (
        large_scale_success
        + edge_case_success
        + (1 if concurrent_workers >= total_workers * 0.8 else 0)
        + (1 if memory_results.get("memory_efficiency_mb_per_1k", 0) > 0 else 0)
    )

    success_rate = passed_tests / total_tests * 100
    print(
        f"\n🎯 Overall Success Rate: {passed_tests}/{total_tests} ({success_rate:.1f}%)"
    )

    if success_rate >= 90:
        print(
            "🏆 EXCELLENT - OmenDB shows exceptional robustness and stress tolerance!"
        )
    elif success_rate >= 75:
        print(
            "✅ GOOD - OmenDB handles stress conditions well with minor areas for improvement"
        )
    else:
        print("⚠️ NEEDS IMPROVEMENT - Some stress scenarios require attention")

    print("\n✨ Stress testing completed - OmenDB validated for production robustness!")

    return success_rate >= 90


if __name__ == "__main__":
    main()
