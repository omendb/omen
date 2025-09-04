#!/usr/bin/env python3
"""
Test Ultra-Optimization Integration
===================================

Tests the integration of optimized Mojo implementations.
"""

import sys

sys.path.insert(0, "python")

import time
import random
import statistics


def test_optimized_performance():
    """Test optimized implementation directly."""
    print("🔥 TESTING OPTIMIZED IMPLEMENTATION")
    print("=" * 50)

    try:
        # First test: Direct optimized Python implementation
        from omendb.index import BruteForceIndex

        print("✅ Python optimized implementation available")

        # Create index
        index = BruteForceIndex(initial_capacity=10000)

        # Test insertion performance
        test_sizes = [100, 500, 1000, 2000, 5000]

        for size in test_sizes:
            print(f"\n📊 Testing {size} vectors:")

            # Generate test data
            vectors = [[random.random() for _ in range(128)] for _ in range(size)]

            # Time insertion
            start = time.perf_counter()
            for i, vec in enumerate(vectors):
                success = index.add(f"ultra_{i}", vec)
                if not success:
                    print(f"   ❌ Failed to add vector {i}")
                    break
            insertion_time = time.perf_counter() - start
            insertion_rate = size / insertion_time

            print(f"   Insertion: {insertion_rate:,.0f} vec/s ({insertion_time:.3f}s)")

            # Test search performance
            query_vec = vectors[0]
            query_times = []

            for _ in range(20):
                start = time.perf_counter()
                results = index.search(query_vec, k=10)
                query_times.append((time.perf_counter() - start) * 1000)

            avg_query_time = statistics.mean(query_times)
            print(f"   Query: {avg_query_time:.3f}ms avg ({len(results)} results)")

            # Clear for next test
            index.clear()

        # Get stats
        stats = index.get_stats()
        print(f"\n📊 Optimized stats: {stats}")

    except ImportError as e:
        print(f"❌ Could not import optimized implementation: {e}")
    except Exception as e:
        print(f"❌ Error testing optimized: {e}")
        import traceback

        traceback.print_exc()


def compare_implementations():
    """Compare standard API vs optimized."""
    print("\n🚀 IMPLEMENTATION COMPARISON")
    print("=" * 50)

    test_size = 1000
    vectors = [[random.random() for _ in range(128)] for _ in range(test_size)]

    print("1. Standard API (current):")
    try:
        from omendb import DB

        db = DB()
        start = time.perf_counter()
        for i, vec in enumerate(vectors):
            db.add(f"std_{i}", vec)
        std_time = time.perf_counter() - start
        std_rate = test_size / std_time

        # Query test
        query_times = []
        for _ in range(10):
            start = time.perf_counter()
            results = db.search(vectors[0], limit=10)
            query_times.append((time.perf_counter() - start) * 1000)

        std_query_time = statistics.mean(query_times)

        print(f"   Insertion: {std_rate:,.0f} vec/s")
        print(f"   Query: {std_query_time:.3f}ms avg")

        stats = db.stats()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")

    except Exception as e:
        print(f"   ❌ Standard API error: {e}")
        std_rate = 0
        std_query_time = float("inf")

    print("\n2. Optimized (Python):")
    try:
        from omendb.index import BruteForceIndex

        ultra_index = BruteForceIndex()
        start = time.perf_counter()
        for i, vec in enumerate(vectors):
            ultra_index.add(f"ultra_{i}", vec)
        ultra_time = time.perf_counter() - start
        ultra_rate = test_size / ultra_time

        # Query test
        query_times = []
        for _ in range(10):
            start = time.perf_counter()
            results = ultra_index.search(vectors[0], k=10)
            query_times.append((time.perf_counter() - start) * 1000)

        ultra_query_time = statistics.mean(query_times)

        print(f"   Insertion: {ultra_rate:,.0f} vec/s")
        print(f"   Query: {ultra_query_time:.3f}ms avg")

        if std_rate > 0:
            print(f"\n🎯 PERFORMANCE IMPROVEMENT:")
            print(f"   Insertion: {ultra_rate / std_rate:.1f}x faster")
            print(f"   Query: {std_query_time / ultra_query_time:.1f}x faster")

    except Exception as e:
        print(f"   ❌ Optimized error: {e}")


def main():
    """Run optimization tests."""
    print("🎯 ULTRA-OPTIMIZATION INTEGRATION TEST")
    print("🔥 Comparing implementations for performance gains")
    print("=" * 60)

    test_optimized_performance()
    compare_implementations()

    print(f"\n💡 NEXT STEPS:")
    print(f"- Integrate optimized Mojo implementation")
    print(f"- Fix compilation issues in native module")
    print(f"- Benchmark integrated solution")
    print(f"- Document performance improvements")


if __name__ == "__main__":
    main()
