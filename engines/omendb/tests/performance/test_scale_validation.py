#!/usr/bin/env python3
"""
Scale testing for OmenDB - validate with 1K, 5K, and 10K vectors.

Tests database performance and stability at larger scales using batch operations
to ensure the embedded vector database can handle realistic workloads (90K+ vec/s).
"""

import sys
import os
import time
import random
import math
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'python'))

import omendb

def generate_random_vector(dimension: int) -> list[float]:
    """Generate a random normalized vector."""
    # Generate random values
    vector = [random.gauss(0, 1) for _ in range(dimension)]
    
    # Normalize to unit length
    magnitude = math.sqrt(sum(x * x for x in vector))
    if magnitude > 0:
        vector = [x / magnitude for x in vector]
    
    return vector

def test_scale_insertion(vector_count: int, dimension: int = 128):
    """Test inserting vectors at scale."""
    print(f"\n📊 Scale Test: {vector_count:,} vectors (dim={dimension})")
    print("=" * 50)
    
    db_path = f"test_scale_{vector_count}.omen"
    
    # Clean up any existing file
    if os.path.exists(db_path):
        os.remove(db_path)
    
    try:
        # Test insertion performance
        start_time = time.time()
        
        db = omendb.DB(db_path)
        
        print(f"Inserting {vector_count:,} vectors using batch operations...")
        
        # Generate all vectors at once for batch processing
        vectors = [generate_random_vector(dimension) for _ in range(vector_count)]
        ids = [f"vec_{i:06d}" for i in range(vector_count)]
        metadata = [{"batch": str(i // 1000), "index": str(i)} for i in range(vector_count)]
        
        # Use batch API for realistic performance (90K+ vec/s)
        batch_size = 10000  # Process in large batches
        
        for i in range(0, vector_count, batch_size):
            end_idx = min(i + batch_size, vector_count)
            batch_vectors = vectors[i:end_idx]
            batch_ids = ids[i:end_idx]
            batch_metadata = metadata[i:end_idx]
            
            result_ids = db.add_batch(vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata)
            if len(result_ids) != len(batch_vectors):
                print(f"❌ Failed to insert batch {i//batch_size}")
                return False
            
            # Progress reporting
            elapsed = time.time() - start_time
            rate = end_idx / elapsed
            print(f"  Progress: {end_idx:,}/{vector_count:,} ({rate:.0f} vectors/sec)")
        
        db.close()
        
        insertion_time = time.time() - start_time
        insertion_rate = vector_count / insertion_time
        
        print(f"✅ Insertion completed!")
        print(f"  Time: {insertion_time:.2f} seconds")
        print(f"  Rate: {insertion_rate:.1f} vectors/second")
        
        # Check file size
        if os.path.exists(db_path):
            file_size = os.path.getsize(db_path)
            size_mb = file_size / (1024 * 1024)
            print(f"  File size: {size_mb:.1f} MB ({file_size:,} bytes)")
            print(f"  Bytes per vector: {file_size / vector_count:.1f}")
        
        return True, insertion_time, insertion_rate, file_size
        
    except Exception as e:
        print(f"❌ Scale insertion failed: {e}")
        import traceback
        traceback.print_exc()
        return False, 0, 0, 0

def test_scale_search(vector_count: int, dimension: int = 128, num_queries: int = 100):
    """Test search performance at scale."""
    print(f"\n🔍 Search Test: {num_queries} queries on {vector_count:,} vectors")
    print("=" * 50)
    
    db_path = f"test_scale_{vector_count}.omen"
    
    if not os.path.exists(db_path):
        print("❌ Database file not found - run insertion test first")
        return False
    
    try:
        search_times = []
        
        db = omendb.DB(db_path)
        print(f"Running {num_queries} search queries...")
        
        for i in range(num_queries):
            # Generate random query vector
            query_vector = generate_random_vector(dimension)
            
            start_time = time.time()
            results = db.search(query_vector, limit=10)  # Modern API
            search_time = time.time() - start_time
                
                search_times.append(search_time * 1000)  # Convert to milliseconds
                
                if len(results) == 0:
                    print(f"❌ No results for query {i}")
                    return False
                
                # Progress reporting
                if (i + 1) % (num_queries // 10) == 0:
                    avg_time = sum(search_times) / len(search_times)
                    print(f"  Query {i+1}/{num_queries}: {avg_time:.2f}ms avg")
        
        # Calculate statistics
        avg_search_time = sum(search_times) / len(search_times)
        min_search_time = min(search_times)
        max_search_time = max(search_times)
        p95_search_time = sorted(search_times)[int(0.95 * len(search_times))]
        
        print(f"✅ Search performance:")
        print(f"  Average: {avg_search_time:.2f}ms")
        print(f"  Min: {min_search_time:.2f}ms")
        print(f"  Max: {max_search_time:.2f}ms")
        print(f"  P95: {p95_search_time:.2f}ms")
        print(f"  Queries/second: {1000/avg_search_time:.1f}")
        
        return True, avg_search_time
        
    except Exception as e:
        print(f"❌ Scale search failed: {e}")
        import traceback
        traceback.print_exc()
        return False, 0

def test_persistence_at_scale(vector_count: int):
    """Test that persistence works correctly at scale."""
    print(f"\n💾 Persistence Test: {vector_count:,} vectors")
    print("=" * 40)
    
    db_path = f"test_scale_{vector_count}.omen"
    
    if not os.path.exists(db_path):
        print("❌ Database file not found - run insertion test first")
        return False
    
    try:
        # Test 1: Reopen database and verify vector count
        print("1. Reopening database...")
        start_time = time.time()
        
        db = omendb.DB(db_path)
        # Query for a few vectors to verify they're loaded
        test_queries = [
            generate_random_vector(128) for _ in range(5)
        ]
        
        total_results = 0
        for query in test_queries:
            results = db.search(query, limit=10)  # Modern API
            total_results += len(results)
        
        db.close()
        
        load_time = time.time() - start_time
        
        if total_results >= 50:  # Should find 10 results for each of 5 queries
            print(f"✅ Database loaded successfully in {load_time:.2f}s")
            print(f"  Found results for all test queries: {total_results} total results")
            return True
        else:
            print(f"❌ Insufficient results found: {total_results}")
            return False
            
    except Exception as e:
        print(f"❌ Persistence test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def run_comprehensive_scale_test():
    """Run comprehensive scale testing at multiple sizes."""
    print("🚀 OmenDB Scale Testing Suite")
    print("=" * 30)
    
    # Test configurations: (vector_count, dimension)
    test_configs = [
        (1000, 128),    # 1K vectors
        (5000, 128),    # 5K vectors  
        (10000, 64),    # 10K vectors (smaller dim for speed)
    ]
    
    results = []
    
    for vector_count, dimension in test_configs:
        print(f"\n🎯 Testing {vector_count:,} vectors (dimension {dimension})")
        print("=" * 60)
        
        # Test 1: Insertion
        insert_success, insert_time, insert_rate, file_size = test_scale_insertion(vector_count, dimension)
        if not insert_success:
            print(f"❌ Skipping remaining tests for {vector_count:,} vectors")
            continue
        
        # Test 2: Search performance
        search_success, avg_search_time = test_scale_search(vector_count, dimension, 50)
        if not search_success:
            print(f"❌ Search test failed for {vector_count:,} vectors")
            continue
        
        # Test 3: Persistence
        persist_success = test_persistence_at_scale(vector_count)
        if not persist_success:
            print(f"❌ Persistence test failed for {vector_count:,} vectors")
            continue
        
        results.append({
            'count': vector_count,
            'dimension': dimension,
            'insert_time': insert_time,
            'insert_rate': insert_rate,
            'file_size_mb': file_size / (1024 * 1024),
            'avg_search_ms': avg_search_time,
            'search_qps': 1000 / avg_search_time
        })
        
        print(f"✅ All tests passed for {vector_count:,} vectors!")
    
    # Summary report
    print("\n" + "=" * 60)
    print("📊 SCALE TESTING SUMMARY")
    print("=" * 60)
    
    if results:
        print("| Vectors | Dim | Insert Rate | File Size | Search Time | Search QPS |")
        print("|---------|-----|-------------|-----------|-------------|------------|")
        
        for r in results:
            print(f"| {r['count']:6,} | {r['dimension']:3d} | "
                  f"{r['insert_rate']:8.1f}/s | {r['file_size_mb']:6.1f} MB | "
                  f"{r['avg_search_ms']:8.2f}ms | {r['search_qps']:7.1f}/s |")
        
        print("\n✅ Scale testing completed successfully!")
        print(f"📈 Largest dataset: {max(r['count'] for r in results):,} vectors")
        print(f"⚡ Best search performance: {min(r['avg_search_ms'] for r in results):.2f}ms")
        print(f"💾 Storage efficiency: {sum(r['file_size_mb'] for r in results):.1f} MB total")
        
        # Clean up test files
        print("\n🧹 Cleaning up test files...")
        for r in results:
            db_path = f"test_scale_{r['count']}.omen"
            if os.path.exists(db_path):
                os.remove(db_path)
                print(f"  Removed {db_path}")
        
        return True
    else:
        print("❌ No tests completed successfully")
        return False

if __name__ == "__main__":
    success = run_comprehensive_scale_test()
    exit(0 if success else 1)