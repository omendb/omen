#!/usr/bin/env python3
"""
Test batch query implementation to verify functionality and measure performance improvement.
"""

import time
import sys
import os

sys.path.append("python")

from omendb import DB


def test_batch_query():
    """Test that batch query works correctly and provides performance improvement."""
    print("🧪 Testing batch query implementation...")

    # Create database and add test vectors
    db = DB()

    # Add test vectors
    print("📊 Adding test vectors...")
    vectors = []
    for i in range(1000):
        vector = [float(i + j) for j in range(64)]  # 64D vectors
        vectors.append(vector)
        db.add(f"vec_{i}", vector)

    print(f"✅ Added {len(vectors)} vectors")

    # Create batch of queries
    num_queries = 20
    query_vectors = []
    for i in range(num_queries):
        query = [float(i * 10 + j) for j in range(64)]
        query_vectors.append(query)

    print(f"🔍 Prepared {num_queries} query vectors")

    # Test individual queries (baseline)
    print("\n📈 Testing individual queries...")
    start_time = time.time()
    individual_results = []
    for query in query_vectors:
        results = db.search(query, limit=5)
        individual_results.append(results)
    individual_time = time.time() - start_time

    print(
        f"⏱️  Individual queries: {individual_time:.4f}s ({num_queries / individual_time:.1f} QPS)"
    )

    # Test batch queries (optimized)
    print("\n🚀 Testing batch queries...")
    start_time = time.time()

    # Use the native batch_query_vectors function directly
    import omendb.native as native

    batch_results = native.batch_query_vectors(query_vectors, 5)

    batch_time = time.time() - start_time

    print(f"⏱️  Batch queries: {batch_time:.4f}s ({num_queries / batch_time:.1f} QPS)")

    # Calculate improvement
    if batch_time > 0 and individual_time > 0:
        improvement = individual_time / batch_time
        print(f"🎯 Performance improvement: {improvement:.2f}x")

        if improvement >= 1.3:  # Expected ~1.5x improvement
            print("✅ Batch query optimization successful!")
        else:
            print("⚠️  Improvement less than expected (target: 1.5x)")

    # Verify results are consistent
    print("\n🔍 Validating result consistency...")
    if len(batch_results) == len(individual_results):
        print(f"✅ Result count matches: {len(batch_results)} queries")

        # Check first query results
        if len(batch_results) > 0 and len(individual_results) > 0:
            batch_first = batch_results[0]
            individual_first = individual_results[0]

            if len(batch_first) == len(individual_first):
                print(
                    f"✅ First query result count matches: {len(batch_first)} results"
                )

                # Check first result ID matches
                if len(batch_first) > 0 and len(individual_first) > 0:
                    batch_id = batch_first[0]["id"]
                    individual_id = individual_first[0].id

                    if batch_id == individual_id:
                        print(f"✅ First result ID matches: {batch_id}")
                    else:
                        print(
                            f"⚠️  Result IDs differ: batch={batch_id}, individual={individual_id}"
                        )
            else:
                print(
                    f"⚠️  Result counts differ: batch={len(batch_first)}, individual={len(individual_first)}"
                )
    else:
        print(
            f"⚠️  Query counts differ: batch={len(batch_results)}, individual={len(individual_results)}"
        )

    return improvement if "improvement" in locals() else 0.0


if __name__ == "__main__":
    try:
        improvement = test_batch_query()
        print(f"\n🏆 Final result: {improvement:.2f}x improvement")

        if improvement >= 1.3:
            print("🎉 Batch query optimization SUCCESSFUL!")
            exit(0)
        else:
            print("❌ Batch query optimization needs improvement")
            exit(1)

    except Exception as e:
        print(f"❌ Test failed: {e}")
        import traceback

        traceback.print_exc()
        exit(1)
