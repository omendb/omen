#!/usr/bin/env python3
"""
Final validation at 5K scale after all optimizations.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def validate_5k_scale():
    """Validate HNSW at 5K scale with all optimizations."""

    print("🚀 FINAL VALIDATION AT 5K SCALE")
    print("=" * 50)

    n_vectors = 5000
    n_queries = 100
    dimension = 128
    np.random.seed(42)

    # Generate data
    print(f"📊 Generating {n_vectors} database vectors...")
    database_vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    print(f"📊 Generating {n_queries} query vectors...")
    query_vectors = np.random.randn(n_queries, dimension).astype(np.float32)

    # Clear and build
    native.clear_database()

    print(f"\n🔄 Building HNSW index with {n_vectors} vectors...")
    start_time = time.time()

    ids = [f"test_{i:06d}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors
    result = native.add_vector_batch(ids, database_vectors, metadata)

    build_time = time.time() - start_time
    build_rate = n_vectors / build_time

    print(f"✅ Built in {build_time:.2f}s ({build_rate:.0f} vec/s)")

    # Test search performance and quality
    print(f"\n🔍 Testing search quality with {n_queries} queries...")

    total_recall_10 = 0.0
    total_search_time = 0.0

    for i, query in enumerate(query_vectors):
        # Ground truth
        distances = np.linalg.norm(database_vectors - query, axis=1)
        gt_indices = set(np.argsort(distances)[:10])

        # HNSW search
        start_time = time.time()
        results = native.search_vectors(query.tolist(), 10, {})
        search_time = time.time() - start_time
        total_search_time += search_time

        # Extract indices
        found_indices = set()
        for result in results[:10]:
            try:
                idx = int(result['id'].split('_')[1])
                found_indices.add(idx)
            except:
                pass

        # Compute recall
        recall = len(gt_indices.intersection(found_indices)) / 10
        total_recall_10 += recall

        # Show progress
        if (i + 1) % 20 == 0:
            print(f"   Processed {i+1}/{n_queries} queries...")

    avg_recall = total_recall_10 / n_queries
    avg_search_time = total_search_time / n_queries * 1000  # ms

    print(f"\n📊 RESULTS AT 5K SCALE:")
    print(f"Construction: {build_rate:.0f} vec/s")
    print(f"Search: {avg_search_time:.1f}ms average")
    print(f"Recall@10: {avg_recall:.1%}")

    # Final assessment
    print(f"\n🎯 FINAL ASSESSMENT:")

    if build_rate >= 100 and avg_recall >= 0.85:
        print("✅ EXCELLENT: Production-ready performance and quality!")
        print("   - Construction speed acceptable")
        print("   - Search quality near 90% target")
        print("   - Ready for enterprise scale")
    elif build_rate >= 50 and avg_recall >= 0.70:
        print("🟡 GOOD: Acceptable for most use cases")
        print("   - Some room for optimization")
        print("   - Quality sufficient for production")
    else:
        print("⚠️ NEEDS WORK: Performance or quality issues remain")

    return build_rate, avg_recall, avg_search_time

if __name__ == "__main__":
    try:
        build_rate, recall, search_time = validate_5k_scale()

        print(f"\n📈 SUMMARY METRICS:")
        print(f"Build: {build_rate:.0f} vec/s")
        print(f"Search: {search_time:.1f}ms")
        print(f"Recall: {recall:.1%}")

    except Exception as e:
        print(f"💥 Validation failed: {e}")
        import traceback
        traceback.print_exc()