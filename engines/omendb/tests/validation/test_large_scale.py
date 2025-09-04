#!/usr/bin/env python3
"""Large-scale dataset testing for OmenDB - test with 100K+ vectors."""

import time
import numpy as np
import sys
import gc

sys.path.insert(0, "python")


def test_scale_performance(sizes=[1_000, 10_000, 50_000, 100_000, 500_000]):
    """Test performance at various scales."""
    import omendb

    print("🔢 Testing performance at scale")
    print("=" * 60)

    dimension = 128
    results = []

    for num_vectors in sizes:
        print(f"\nTesting {num_vectors:,} vectors @ {dimension}D...")

        # Generate test data
        print("  Generating data...", end="", flush=True)
        vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
        print(" Done")

        # Test construction
        print("  Building index...", end="", flush=True)
        db = omendb.DB()
        start = time.perf_counter()

        # Add in batches of 5000 for optimal performance
        batch_size = 5000
        for i in range(0, num_vectors, batch_size):
            end_idx = min(i + batch_size, num_vectors)
            batch = [(f"vec_{j}", vectors[j].tolist()) for j in range(i, end_idx)]
            db.add_batch(batch)

        construction_time = time.perf_counter() - start
        construction_rate = num_vectors / construction_time
        print(f" Done ({construction_rate:.0f} vec/s)")

        # Test query performance
        print("  Testing queries...", end="", flush=True)
        query_times = []
        for _ in range(100):
            query_idx = np.random.randint(0, num_vectors)
            start = time.perf_counter()
            results = db.search(vectors[query_idx].tolist(), limit=10)
            query_times.append((time.perf_counter() - start) * 1000)

        avg_query_time = np.mean(query_times)
        p95_query_time = np.percentile(query_times, 95)
        print(f" Done (avg: {avg_query_time:.2f}ms, p95: {p95_query_time:.2f}ms)")

        # Test memory usage
        import psutil
        import os

        process = psutil.Process(os.getpid())
        memory_mb = process.memory_info().rss / 1024 / 1024
        bytes_per_vector = (memory_mb * 1024 * 1024) / num_vectors

        results.append(
            {
                "vectors": num_vectors,
                "construction_rate": construction_rate,
                "avg_query_ms": avg_query_time,
                "p95_query_ms": p95_query_time,
                "memory_mb": memory_mb,
                "bytes_per_vector": bytes_per_vector,
            }
        )

        # Cleanup
        del db, vectors
        gc.collect()

    return results


def test_recall_at_scale():
    """Test search accuracy at scale."""
    import omendb

    print("\n\n🎯 Testing recall accuracy at scale")
    print("=" * 60)

    # Use smaller dataset for recall testing
    num_vectors = 10_000
    dimension = 128
    limit = 10

    print(f"Testing with {num_vectors:,} vectors @ {dimension}D...")

    # Generate clustered data for better recall testing
    print("  Generating clustered data...", end="", flush=True)
    num_clusters = 10
    vectors = []
    labels = []

    for cluster_id in range(num_clusters):
        # Generate cluster center
        center = np.random.rand(dimension).astype(np.float32)

        # Generate points around center
        cluster_size = num_vectors // num_clusters
        for _ in range(cluster_size):
            noise = np.random.normal(0, 0.1, dimension).astype(np.float32)
            vectors.append(center + noise)
            labels.append(cluster_id)

    vectors = np.array(vectors)
    labels = np.array(labels)
    print(" Done")

    # Build index
    print("  Building index...", end="", flush=True)
    db = omendb.DB()
    batch = [(f"vec_{i}", vec.tolist()) for i, vec in enumerate(vectors)]
    db.add_batch(batch)
    print(" Done")

    # Test recall
    print("  Testing recall...", end="", flush=True)
    num_queries = 100
    correct_at_k = 0

    for _ in range(num_queries):
        # Pick random vector
        query_idx = np.random.randint(0, num_vectors)
        query_vec = vectors[query_idx]
        query_label = labels[query_idx]

        # Search
        results = db.search(query_vec.tolist(), limit=top_k)

        # Check if any result is from same cluster
        for result in results:
            result_idx = int(result.id.split("_")[1])
            if labels[result_idx] == query_label:
                correct_at_k += 1
                break

    recall = correct_at_k / num_queries
    print(f" Done (Recall@{top_k}: {recall:.2%})")

    return recall


def test_concurrent_operations():
    """Test concurrent read/write operations."""
    import omendb
    import threading

    print("\n\n🔄 Testing concurrent operations")
    print("=" * 60)

    db = omendb.DB()
    dimension = 128
    errors = []

    def writer_thread(thread_id, num_ops):
        """Thread that adds vectors."""
        try:
            for i in range(num_ops):
                vec = np.random.rand(dimension).astype(np.float32).tolist()
                db.add(f"thread_{thread_id}_vec_{i}", vec)
        except Exception as e:
            errors.append(f"Writer {thread_id}: {e}")

    def reader_thread(thread_id, num_ops):
        """Thread that queries vectors."""
        try:
            for _ in range(num_ops):
                query = np.random.rand(dimension).astype(np.float32).tolist()
                results = db.search(query, limit=5)
        except Exception as e:
            errors.append(f"Reader {thread_id}: {e}")

    # Start threads
    print("  Starting concurrent threads...")
    threads = []

    # 5 writer threads
    for i in range(5):
        t = threading.Thread(target=writer_thread, args=(i, 100))
        threads.append(t)
        t.start()

    # 5 reader threads
    for i in range(5):
        t = threading.Thread(target=reader_thread, args=(i, 100))
        threads.append(t)
        t.start()

    # Wait for completion
    for t in threads:
        t.join()

    if errors:
        print(f"  ❌ {len(errors)} errors occurred:")
        for err in errors[:5]:  # Show first 5 errors
            print(f"    - {err}")
    else:
        print("  ✅ All concurrent operations completed successfully")

    return len(errors) == 0


if __name__ == "__main__":
    print("🚀 OmenDB Large-Scale Testing Suite")
    print("=" * 80)

    # Test 1: Scale performance
    scale_results = test_scale_performance()

    print("\n\n📊 Performance Summary")
    print("=" * 60)
    print(
        f"{'Vectors':<12} {'Construction':<15} {'Query (avg)':<12} {'Query (p95)':<12} {'Memory':<10} {'Bytes/Vec':<10}"
    )
    print(
        f"{'-------':<12} {'------------':<15} {'-----------':<12} {'-----------':<12} {'------':<10} {'----------':<10}"
    )

    for result in scale_results:
        print(
            f"{result['vectors']:<12,} {result['construction_rate']:<15,.0f} "
            f"{result['avg_query_ms']:<12.2f} {result['p95_query_ms']:<12.2f} "
            f"{result['memory_mb']:<10.0f} {result['bytes_per_vector']:<10.0f}"
        )

    # Test 2: Recall accuracy
    recall = test_recall_at_scale()

    # Test 3: Concurrent operations
    concurrent_ok = test_concurrent_operations()

    # Final summary
    print("\n\n✅ Large-Scale Test Summary")
    print("=" * 60)

    # Check if performance degrades at scale
    if len(scale_results) >= 2:
        small_rate = scale_results[0]["construction_rate"]
        large_rate = scale_results[-1]["construction_rate"]
        degradation = (small_rate - large_rate) / small_rate * 100

        print(f"Construction rate degradation: {degradation:.1f}%")
        print(f"Recall accuracy: {recall:.1%}")
        print(f"Concurrent operations: {'✅ PASS' if concurrent_ok else '❌ FAIL'}")

        if degradation > 50:
            print("\n⚠️  Significant performance degradation at scale!")
        elif recall < 0.8:
            print("\n⚠️  Poor recall accuracy!")
        elif not concurrent_ok:
            print("\n⚠️  Thread safety issues detected!")
        else:
            print("\n✅ OmenDB scales well to large datasets!")
