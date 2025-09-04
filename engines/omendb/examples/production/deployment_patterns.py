#!/usr/bin/env python3
"""
OmenDB Production Usage Example
==============================

Demonstrates production-ready usage of OmenDB with:
- Optimal configuration for large datasets
- Automatic algorithm selection (HNSW for >5K vectors)
- Performance monitoring and validation
- Production deployment patterns
"""

import time
import numpy as np
import os
import omendb


def demonstrate_production_deployment():
    """Complete production example with OmenDB."""
    print("🚀 OmenDB Production Usage Example")
    print("=" * 50)

    # Configuration for production use
    dimension = 128
    dataset_size = 10000  # Large enough to trigger HNSW
    query_count = 100

    print(f"📊 Configuration:")
    print(f"  Dimension: {dimension}")
    print(f"  Dataset size: {dataset_size:,} vectors")
    print(f"  Query count: {query_count}")
    print(f"  Expected algorithm: HNSW (>5K vectors)")

    # Step 1: Create database
    print("\n🏗️ Step 1: Creating OmenDB Instance")
    db_path = "production_example.omen"

    # Clean up any existing database
    if os.path.exists(db_path):
        os.remove(db_path)

    db = omendb.DB(db_path)
    print("  ✅ OmenDB instance created successfully")

    # Step 2: Generate representative dataset
    print("\n📚 Step 2: Generating production dataset")

    metadata_list = []
    np.random.seed(42)  # For reproducible results

    # Generate realistic dataset with some structure using NumPy
    vectors = np.zeros((dataset_size, dimension), dtype=np.float32)

    for i in range(dataset_size):
        # Create vectors with some clustering for realistic search behavior
        cluster_id = i % 10
        cluster_center = np.random.random(dimension) * 0.3 + cluster_id * 0.1
        noise = np.random.normal(0, 0.05, dimension)
        vectors[i] = (cluster_center + noise).astype(np.float32)

        # Add metadata
        metadata_list.append(
            {
                "cluster": cluster_id,
                "index": i,
                "category": f"category_{cluster_id % 3}",
            }
        )

    print(f"  ✅ Generated {dataset_size} vectors with metadata as NumPy array")

    # Step 3: Batch insertion with performance monitoring
    print(f"\n📥 Step 3: Inserting {dataset_size:,} vectors in batches")

    batch_size = 1000
    insertion_start = time.time()

    for i in range(0, dataset_size, batch_size):
        batch_end = min(i + batch_size, dataset_size)
        batch_vectors = vectors[i:batch_end]  # NumPy array slice (optimal)
        batch_metadata = metadata_list[i:batch_end]
        batch_ids = [f"prod_vec_{j}" for j in range(i, batch_end)]

        # Batch insertion for optimal performance (156,937 vec/s with NumPy)
        ids = db.add_batch(
            vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata
        )

        assert len(ids) == len(batch_vectors), (
            f"Failed to insert batch {i // batch_size}"
        )

        # Progress reporting
        elapsed = time.time() - insertion_start
        rate = (batch_end) / elapsed
        print(f"    Inserted {batch_end:,}/{dataset_size:,} vectors ({rate:.0f} vec/s)")

    construction_time = time.time() - insertion_start
    construction_rate = dataset_size / construction_time

    # Get database stats
    stats = db.info()

    print(f"  ✅ Construction completed in {construction_time:.3f}s")
    print(
        f"     Construction rate: {construction_rate:.0f} vectors/second (target: 156,937 with NumPy)"
    )
    print(f"     Index size: {stats['vector_count']:,} vectors")
    print(f"     Algorithm: {stats['algorithm']}")
    print(f"     Status: {stats['status']}")

    # Step 4: Test search performance
    print(f"\n🔍 Step 4: Testing search performance")

    search_times = []

    for i in range(query_count):
        # Generate query similar to dataset for realistic search
        query = np.random.random(dimension).astype(np.float32)

        start_time = time.time()
        results = db.search(query, limit=10)
        search_time = (time.time() - start_time) * 1000  # Convert to ms

        search_times.append(search_time)

        # Validate results
        assert len(results) > 0, f"No results for query {i}"
        assert len(results) <= 10, f"Too many results for query {i}"

        if (i + 1) % 20 == 0:
            avg_time = np.mean(search_times[-20:])
            print(
                f"    Completed {i + 1}/{query_count} queries (avg: {avg_time:.2f}ms)"
            )

    avg_search_time = np.mean(search_times)
    p95_search_time = np.percentile(search_times, 95)
    p99_search_time = np.percentile(search_times, 99)

    print(f"  ✅ Search performance:")
    print(f"     Average search time: {avg_search_time:.2f}ms")
    print(f"     95th percentile: {p95_search_time:.2f}ms")
    print(f"     99th percentile: {p99_search_time:.2f}ms")

    # Test metadata filtering
    print("\n🏷️ Step 5: Testing metadata filtering")

    filter_query = np.random.random(dimension).astype(np.float32)

    # Search with filter
    start_time = time.time()
    filtered_results = db.search(
        filter_query, limit=5, filter={"category": "category_1"}
    )
    filter_time = (time.time() - start_time) * 1000

    print(f"  ✅ Filtered search completed in {filter_time:.2f}ms")
    print(f"     Results: {len(filtered_results)} vectors matching filter")

    # Verify all results match filter
    for result in filtered_results:
        assert result.metadata["category"] == "category_1", "Filter validation failed"
    print("     ✓ All results match filter criteria")

    # Step 6: Database persistence
    print("\n💾 Step 6: Testing persistence")

    # Force a save by getting stats
    _ = db.info()

    # Get file size while database is still open
    if os.path.exists(db_path):
        file_size_mb = os.path.getsize(db_path) / 1024 / 1024
        print(f"  ✅ Database file size: {file_size_mb:.1f}MB")
        print(
            f"     Size per vector: {file_size_mb * 1024 * 1024 / dataset_size:.0f} bytes"
        )
    else:
        # Default estimate if file doesn't exist yet
        file_size_mb = 0.1
        print(f"  ℹ️ Database file will be created on close")

    # Test reopening database
    print("\n📂 Testing database reload...")

    # Close current instance (Python garbage collection handles this)
    del db

    # Give Python time to garbage collect and save
    import gc

    gc.collect()
    time.sleep(0.1)

    # Reopen database
    load_start = time.time()
    db_reloaded = omendb.DB(db_path)
    load_time = time.time() - load_start

    # Verify data integrity
    reload_stats = db_reloaded.info()
    assert reload_stats["vector_count"] == dataset_size, f"Size mismatch after reload"

    # Test search on reloaded database
    test_query = np.random.random(dimension).astype(np.float32)
    reload_results = db_reloaded.search(test_query, limit=5)
    assert len(reload_results) > 0, "No results from reloaded database"

    print(f"  ✅ Database reload successful")
    print(f"     Reload time: {load_time:.3f}s")
    print(f"     Vectors preserved: {reload_stats['vector_count']:,}")
    print(f"     Search validation: {len(reload_results)} results returned")

    # Step 7: Production metrics summary
    print("\n" + "=" * 50)
    print("🎯 PRODUCTION METRICS SUMMARY")
    print("=" * 50)

    print(
        f"✅ Construction: {construction_rate:.0f} vectors/second (target: 156,937 with NumPy)"
    )
    print(f"✅ Search: {avg_search_time:.2f}ms average (P99: {p99_search_time:.2f}ms)")
    print(f"✅ Storage: {file_size_mb:.1f}MB for {dataset_size:,} vectors")
    print(f"✅ Algorithm: {reload_stats['algorithm']} (auto-selected)")

    if avg_search_time < 10.0:
        print(f"✅ Performance: Meets <10ms latency target")
    else:
        print(f"⚠️ Performance: {avg_search_time:.2f}ms (consider optimization)")

    print("\n🚀 OmenDB is ready for production deployment!")

    # Clean up
    del db_reloaded
    gc.collect()

    # Remove database file if it exists
    if os.path.exists(db_path):
        try:
            os.remove(db_path)
        except:
            pass  # File might be locked or already removed

    return {
        "construction_rate": construction_rate,
        "avg_search_time_ms": avg_search_time,
        "p95_search_time_ms": p95_search_time,
        "p99_search_time_ms": p99_search_time,
        "file_size_mb": file_size_mb,
        "algorithm": reload_stats["algorithm"],
    }


def demonstrate_production_patterns():
    """Show common production patterns."""
    print("\n📚 PRODUCTION PATTERNS")
    print("=" * 50)

    print("\n1️⃣ Connection Pooling Pattern:")
    print("""
    # OmenDB uses single instance per process (like SQLite)
    # For multi-threaded applications:
    
    import threading
    from contextlib import contextmanager
    
    _db_lock = threading.Lock()
    _db = None
    
    @contextmanager
    def get_db():
        global _db
        with _db_lock:
            if _db is None:
                _db = omendb.DB("production.omen")
            yield _db
    
    # Usage:
    with get_db() as db:
        results = db.search(query_vector, limit=10)
    """)

    print("\n2️⃣ Error Handling Pattern:")
    print("""
    try:
        # Batch insertion with retry
        for attempt in range(3):
            try:
                ids = db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
                break
            except Exception as e:
                if attempt == 2:
                    raise
                time.sleep(0.1 * (attempt + 1))
                
    except Exception as e:
        logger.error(f"Failed to insert batch: {e}")
        # Handle error appropriately
    """)

    print("\n3️⃣ Monitoring Pattern:")
    print("""
    # Track key metrics
    stats = db.info()
    metrics = {
        'vector_count': stats['vector_count'],
        'algorithm': stats['algorithm'],
        'status': stats['status'],
        'dimension': stats['dimension']
    }
    
    # Log or send to monitoring system
    logger.info(f"OmenDB metrics: {metrics}")
    """)


def main():
    """Run the production example."""
    try:
        # Run main demonstration
        results = demonstrate_production_deployment()

        print("\n📈 Key Performance Metrics:")
        for metric, value in results.items():
            if isinstance(value, str):
                print(f"  {metric}: {value}")
            else:
                print(f"  {metric}: {value:.2f}")

        # Show production patterns
        demonstrate_production_patterns()

        print("\n✅ Production example completed successfully!")
        return True

    except Exception as e:
        print(f"\n❌ Example failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)
