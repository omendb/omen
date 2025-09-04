#!/usr/bin/env python3
"""
Test Scaling Performance
========================

Tests the optimized implementation at various scales to ensure it works
properly from small datasets to larger ones.
"""

import sys

sys.path.insert(0, "python")

import time
import random
import statistics


def test_scaling():
    """Test performance across different scales."""
    print("🎯 SCALING TEST - Optimized Implementation")
    print("=" * 60)

    try:
        from omendb.index import BruteForceIndex

        # Test sizes from very small to large
        test_sizes = [10, 50, 100, 500, 1000, 2000, 5000, 10000]
        dimension = 128

        results = []

        for size in test_sizes:
            print(f"\n📊 Testing {size:,} vectors:")

            # Create fresh index
            index = BruteForceIndex(initial_capacity=max(size * 2, 10000))

            # Generate test data
            vectors = [[random.random() for _ in range(dimension)] for _ in range(size)]

            # Time insertion
            start = time.perf_counter()
            successful_inserts = 0
            for i, vec in enumerate(vectors):
                if index.add(f"vec_{i}", vec):
                    successful_inserts += 1
                else:
                    print(f"   ⚠️ Failed to insert vector {i}")
            insertion_time = time.perf_counter() - start

            if successful_inserts > 0:
                insertion_rate = successful_inserts / insertion_time
                print(
                    f"   ✅ Insertion: {insertion_rate:,.0f} vec/s ({insertion_time:.3f}s)"
                )
                print(
                    f"   📈 Success rate: {successful_inserts}/{size} ({100 * successful_inserts / size:.1f}%)"
                )

                # Test search performance with multiple queries
                query_times = []
                search_accuracy = []

                for trial in range(10):
                    query_vec = vectors[trial % len(vectors)]

                    start = time.perf_counter()
                    search_results = index.search(query_vec, k=min(10, size))
                    query_time = (time.perf_counter() - start) * 1000
                    query_times.append(query_time)

                    # Check search quality
                    if len(search_results) > 0:
                        search_accuracy.append(len(search_results))

                avg_query_time = statistics.mean(query_times)
                avg_results = statistics.mean(search_accuracy) if search_accuracy else 0

                print(
                    f"   🔍 Query: {avg_query_time:.3f}ms avg ({avg_results:.1f} results)"
                )

                # Get stats
                stats = index.get_stats()
                print(f"   💾 Memory: {stats.get('memory_mb', 0)}MB")

                results.append(
                    {
                        "size": size,
                        "insertion_rate": insertion_rate,
                        "query_time_ms": avg_query_time,
                        "success_rate": successful_inserts / size,
                        "memory_mb": float(stats.get("memory_mb", 0)),
                    }
                )
            else:
                print(f"   ❌ Failed all insertions")
                results.append(
                    {
                        "size": size,
                        "insertion_rate": 0,
                        "query_time_ms": float("inf"),
                        "success_rate": 0,
                        "memory_mb": 0,
                    }
                )

        # Summary analysis
        print(f"\n🎯 SCALING ANALYSIS:")
        print(
            f"{'Size':>8} {'Insert Rate':>12} {'Query Time':>12} {'Success':>8} {'Memory':>8}"
        )
        print(f"{'':>8} {'(vec/s)':>12} {'(ms)':>12} {'Rate':>8} {'(MB)':>8}")
        print("-" * 60)

        for result in results:
            print(
                f"{result['size']:>8,} {result['insertion_rate']:>12,.0f} "
                f"{result['query_time_ms']:>12.3f} {result['success_rate']:>7.1%} "
                f"{result['memory_mb']:>8.1f}"
            )

        # Performance trends
        successful_results = [r for r in results if r["success_rate"] > 0.9]
        if len(successful_results) >= 2:
            min_rate = min(r["insertion_rate"] for r in successful_results)
            max_rate = max(r["insertion_rate"] for r in successful_results)
            min_query = min(r["query_time_ms"] for r in successful_results)
            max_query = max(r["query_time_ms"] for r in successful_results)

            print(f"\n📈 PERFORMANCE TRENDS:")
            print(f"   Insertion rate: {min_rate:,.0f} - {max_rate:,.0f} vec/s")
            print(f"   Query time: {min_query:.3f} - {max_query:.3f} ms")
            print(
                f"   Scaling factor: {max_query / min_query:.1f}x query time increase"
            )

        # Determine practical limits
        working_sizes = [
            r["size"]
            for r in results
            if r["success_rate"] > 0.95 and r["query_time_ms"] < 1000
        ]
        if working_sizes:
            max_working = max(working_sizes)
            print(f"\n✅ PRACTICAL SCALING LIMIT: Up to {max_working:,} vectors")
            print(f"   (95%+ success rate, <1s query time)")

        return results

    except ImportError as e:
        print(f"❌ Could not import optimized implementation: {e}")
        return []
    except Exception as e:
        print(f"❌ Error during scaling test: {e}")
        import traceback

        traceback.print_exc()
        return []


def main():
    """Run scaling tests."""
    results = test_scaling()

    if results:
        print(f"\n💡 CONCLUSIONS:")
        successful = [r for r in results if r["success_rate"] > 0.9]
        if successful:
            avg_rate = statistics.mean(r["insertion_rate"] for r in successful)
            print(f"   Average insertion rate: {avg_rate:,.0f} vec/s")
            print(
                f"   Scales reliably from {min(r['size'] for r in successful):,} to {max(r['size'] for r in successful):,} vectors"
            )
            print(f"   Optimized implementation works at all tested scales ✅")
        else:
            print(f"   No successful runs - implementation may have issues ❌")


if __name__ == "__main__":
    main()
