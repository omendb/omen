#!/usr/bin/env python3
"""
Honest test of startup time including all real-world overhead.
"""

import sys
import os
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))


def test_cold_startup():
    """Test completely cold startup including import overhead."""
    print("🧪 Testing COLD startup (including imports)...")

    # Measure import time
    import_start = time.perf_counter()
    import omendb

    import_time = time.perf_counter() - import_start

    # Measure DB creation time
    creation_start = time.perf_counter()
    db = omendb.DB()
    creation_time = time.perf_counter() - creation_start

    # Measure first operation time
    first_op_start = time.perf_counter()
    vector = [1.0] * 128
    db.add("test", vector)
    first_op_time = time.perf_counter() - first_op_start

    # Measure first query time
    first_query_start = time.perf_counter()
    results = db.search(vector, limit=1)
    first_query_time = time.perf_counter() - first_query_start

    print(f"   Import time: {import_time * 1000:.3f}ms")
    print(
        f"   DB creation: {creation_time * 1000:.6f}ms ({creation_time * 1000000:.1f} μs)"
    )
    print(f"   First add: {first_op_time * 1000:.3f}ms")
    print(f"   First query: {first_query_time * 1000:.3f}ms")
    print(f"   Total cold startup: {(import_time + creation_time) * 1000:.3f}ms")

    return creation_time


def test_warm_startup():
    """Test warm startup (DB creation only, imports already cached)."""
    print("\n🔥 Testing WARM startup (imports cached)...")

    times = []
    for i in range(100):
        start = time.perf_counter()
        import omendb

        db = omendb.DB()
        end = time.perf_counter()
        times.append(end - start)

    avg_time = sum(times) / len(times)
    min_time = min(times)
    max_time = max(times)

    print(f"   Average: {avg_time * 1000:.6f}ms ({avg_time * 1000000:.1f} μs)")
    print(f"   Minimum: {min_time * 1000:.6f}ms ({min_time * 1000000:.1f} μs)")
    print(f"   Maximum: {max_time * 1000:.6f}ms ({max_time * 1000000:.1f} μs)")

    return avg_time


def test_immediate_query_readiness():
    """Test if we can immediately query after creation."""
    print("\n⚡ Testing immediate query readiness...")

    import omendb

    # Create DB and immediately try operations
    db = omendb.DB()

    # Test 1: Add vector immediately
    add_start = time.perf_counter()
    success = db.add("test1", [1.0] * 128)
    add_time = time.perf_counter() - add_start

    # Test 2: Query immediately after add
    query_start = time.perf_counter()
    results = db.search([1.0] * 128, limit=1)
    query_time = time.perf_counter() - query_start

    print(f"   Immediate add: {add_time * 1000:.3f}ms (success: {success})")
    print(f"   Immediate query: {query_time * 1000:.3f}ms (results: {len(results)})")
    print(f"   Ready for use: {'✅ YES' if success and len(results) > 0 else '❌ NO'}")


if __name__ == "__main__":
    print("🎯 Realistic Startup Time Analysis")
    print("=" * 45)

    cold_time = test_cold_startup()
    warm_time = test_warm_startup()
    test_immediate_query_readiness()

    print(f"\n📊 Startup Analysis:")
    print(f"   Cold startup claim: 0.001ms")
    print(
        f"   Actual warm creation: {warm_time * 1000:.3f}ms ({warm_time * 1000000:.0f}x slower)"
    )
    print(f"   Cold vs warm ratio: {cold_time / warm_time:.0f}x")

    if warm_time < 0.001:
        print("✅ Claim is realistic for warm startup")
    else:
        print(
            "❌ Claim needs adjustment - more realistic: {:.3f}ms".format(
                warm_time * 1000
            )
        )
