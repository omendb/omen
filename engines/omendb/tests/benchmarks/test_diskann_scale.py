#!/usr/bin/env python3
"""
Test DiskANN at various scales to verify production readiness.

Tests:
1. Small scale (100 vectors) - baseline
2. Medium scale (1,000 vectors) - first real test
3. Large scale (10,000 vectors) - production typical
4. Extra large (100,000 vectors) - stress test
"""

import sys
import time
import numpy as np
import psutil
import os

# Add path to import omendb
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))
import omendb


def measure_memory():
    """Get current memory usage in MB."""
    process = psutil.Process()
    return process.memory_info().rss / 1024 / 1024


def test_scale(size, dimension=128, k=10):
    """Test DiskANN at a specific scale."""
    print(f"\n{'=' * 60}")
    print(f"Testing DiskANN with {size:,} vectors @ {dimension}D")
    print(f"{'=' * 60}")

    # Track metrics
    metrics = {
        "size": size,
        "dimension": dimension,
        "insert_time": 0,
        "insert_vec_per_sec": 0,
        "query_time": 0,
        "query_latency_ms": 0,
        "memory_mb": 0,
        "recall_at_10": 0,
        "errors": [],
    }

    try:
        # Initialize DiskANN
        print("Initializing DiskANN index...")
        start_mem = measure_memory()
        db = omendb.DB(algorithm="diskann", buffer_size=1000)

        # Generate random vectors
        print(f"Generating {size:,} random vectors...")
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        # Normalize for cosine similarity
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors = vectors / norms

        # Test batch insertion
        print(f"Inserting {size:,} vectors...")
        start = time.time()

        # Insert in batches to avoid overwhelming
        batch_size = min(1000, size)
        for i in range(0, size, batch_size):
            end_idx = min(i + batch_size, size)
            batch_vecs = vectors[i:end_idx]
            batch_ids = ids[i:end_idx]

            # Add batch
            db.add_batch(batch_vecs, batch_ids)

            # Progress update
            if (i + batch_size) % 10000 == 0:
                elapsed = time.time() - start
                rate = (i + batch_size) / elapsed
                print(
                    f"  Progress: {i + batch_size:,}/{size:,} vectors ({rate:.0f} vec/s)"
                )

        insert_time = time.time() - start
        metrics["insert_time"] = insert_time
        metrics["insert_vec_per_sec"] = size / insert_time

        print(f"✅ Insertion complete: {metrics['insert_vec_per_sec']:.0f} vec/s")

        # Get index stats
        stats = db.get_stats()
        print(f"\nIndex Stats:")
        print(f"  Nodes: {stats.get('nodes', 0):,}")
        print(f"  Buffer size: {stats.get('buffer_size', 0):,}")
        print(f"  Main index size: {stats.get('main_index_size', 0):,}")

        # Test queries
        print(f"\nTesting {min(100, size)} queries...")
        query_count = min(100, size)
        query_indices = np.random.choice(size, query_count, replace=False)

        recalls = []
        start = time.time()

        for idx in query_indices:
            query_vec = vectors[idx]
            true_id = ids[idx]

            # Search
            results = db.search(query_vec, limit=k)

            # Check recall
            result_ids = [r[0] for r in results]
            if true_id in result_ids:
                recalls.append(1.0)
            else:
                recalls.append(0.0)

        query_time = time.time() - start
        metrics["query_time"] = query_time
        metrics["query_latency_ms"] = (query_time / query_count) * 1000
        metrics["recall_at_10"] = np.mean(recalls)

        print(f"✅ Query complete:")
        print(f"  Average latency: {metrics['query_latency_ms']:.2f} ms")
        print(f"  Recall@{k}: {metrics['recall_at_10']:.2%}")

        # Memory usage
        end_mem = measure_memory()
        metrics["memory_mb"] = end_mem - start_mem
        print(f"\nMemory usage: {metrics['memory_mb']:.1f} MB")

        # Test surgical delete
        if size <= 1000:
            print(f"\nTesting surgical delete...")
            delete_count = min(10, size // 10)
            delete_ids = ids[:delete_count]

            for del_id in delete_ids:
                success = db.delete(del_id)
                if not success:
                    metrics["errors"].append(f"Failed to delete {del_id}")

            print(f"✅ Deleted {delete_count} vectors")

            # Verify deletion
            for del_id in delete_ids:
                # Search for deleted vector
                idx = ids.index(del_id)
                results = db.search(vectors[idx], limit=1)
                if results and results[0][0] == del_id:
                    metrics["errors"].append(f"Deleted vector {del_id} still found")

    except Exception as e:
        metrics["errors"].append(str(e))
        print(f"❌ Error: {e}")

    return metrics


def main():
    """Run scale tests."""
    print("DiskANN Scale Testing")
    print("=" * 60)

    # Test at different scales
    test_sizes = [
        100,  # Small - should work
        1_000,  # Medium - first real test
        10_000,  # Large - production typical
        # 100_000, # Extra large - stress test (uncomment if others pass)
    ]

    results = []
    for size in test_sizes:
        try:
            metrics = test_scale(size)
            results.append(metrics)

            # Check for errors
            if metrics["errors"]:
                print(f"\n⚠️ Errors encountered:")
                for error in metrics["errors"]:
                    print(f"  - {error}")

            # Performance summary
            print(f"\n📊 Performance Summary for {size:,} vectors:")
            print(f"  Insert: {metrics['insert_vec_per_sec']:.0f} vec/s")
            print(f"  Query: {metrics['query_latency_ms']:.2f} ms")
            print(f"  Recall: {metrics['recall_at_10']:.2%}")
            print(f"  Memory: {metrics['memory_mb']:.1f} MB")

        except Exception as e:
            print(f"\n❌ Test failed for {size:,} vectors: {e}")
            results.append({"size": size, "errors": [str(e)]})

    # Final summary
    print("\n" + "=" * 60)
    print("FINAL SUMMARY")
    print("=" * 60)

    for result in results:
        size = result["size"]
        if result.get("errors"):
            print(f"\n{size:,} vectors: ❌ FAILED")
            for error in result["errors"]:
                print(f"  - {error}")
        else:
            print(f"\n{size:,} vectors: ✅ PASSED")
            print(f"  Insert: {result['insert_vec_per_sec']:.0f} vec/s")
            print(f"  Query: {result['query_latency_ms']:.2f} ms")
            print(f"  Recall: {result['recall_at_10']:.2%}")

    # Production readiness assessment
    print("\n" + "=" * 60)
    print("PRODUCTION READINESS ASSESSMENT")
    print("=" * 60)

    # Check if 10K test passed
    ten_k_result = next((r for r in results if r["size"] == 10_000), None)

    if ten_k_result and not ten_k_result.get("errors"):
        print("✅ DiskANN is production ready!")
        print(f"  - Handles 10K vectors successfully")
        print(f"  - Insert performance: {ten_k_result['insert_vec_per_sec']:.0f} vec/s")
        print(f"  - Query latency: {ten_k_result['query_latency_ms']:.2f} ms")
        print(f"  - Recall: {ten_k_result['recall_at_10']:.2%}")
    else:
        print("❌ DiskANN is NOT production ready")
        print("  - Failed at production scale (10K vectors)")
        print("  - More optimization needed")

        # Identify bottlenecks
        if results:
            last_working = max(
                (r for r in results if not r.get("errors")),
                key=lambda x: x["size"],
                default=None,
            )
            if last_working:
                print(f"  - Maximum working scale: {last_working['size']:,} vectors")


if __name__ == "__main__":
    main()
