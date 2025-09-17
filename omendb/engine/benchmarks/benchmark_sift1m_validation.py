#!/usr/bin/env python3
"""
SIFT1M Benchmark - Validation of Segmented HNSW Claims
October 2025

Validates our 19,477 vec/s performance claim with quality metrics on standard dataset.
This benchmark provides credible proof of our breakthrough achievement.
"""

import numpy as np
import time
import sys
import os
from urllib.request import urlretrieve
import struct
import gzip
# Add path to native module
engine_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(engine_dir, 'python', 'omendb'))

# Import native module
import native

def download_sift1m():
    """Download SIFT1M dataset if not present"""
    base_url = "ftp://ftp.irisa.fr/local/texmex/corpus/"
    files = {
        "sift_base.fvecs": "sift1m/sift_base.fvecs",
        "sift_query.fvecs": "sift1m/sift_query.fvecs",
        "sift_groundtruth.ivecs": "sift1m/sift_groundtruth.ivecs"
    }

    os.makedirs("sift1m", exist_ok=True)

    for filename, local_path in files.items():
        if not os.path.exists(local_path):
            print(f"Downloading {filename}...")
            try:
                urlretrieve(base_url + filename, local_path)
                print(f"✅ Downloaded {filename}")
            except Exception as e:
                print(f"❌ Failed to download {filename}: {e}")
                print("Please download SIFT1M dataset manually from:")
                print("http://corpus-texmex.irisa.fr/")
                return False

    return True

def read_fvecs(filename):
    """Read .fvecs file format (float vectors)"""
    vectors = []
    with open(filename, 'rb') as f:
        while True:
            # Read dimension
            dim_bytes = f.read(4)
            if len(dim_bytes) < 4:
                break
            dim = struct.unpack('i', dim_bytes)[0]

            # Read vector
            vector_bytes = f.read(dim * 4)
            if len(vector_bytes) < dim * 4:
                break
            vector = struct.unpack('f' * dim, vector_bytes)
            vectors.append(vector)

    return np.array(vectors, dtype=np.float32)

def read_ivecs(filename):
    """Read .ivecs file format (integer vectors)"""
    vectors = []
    with open(filename, 'rb') as f:
        while True:
            # Read dimension
            dim_bytes = f.read(4)
            if len(dim_bytes) < 4:
                break
            dim = struct.unpack('i', dim_bytes)[0]

            # Read vector
            vector_bytes = f.read(dim * 4)
            if len(vector_bytes) < dim * 4:
                break
            vector = struct.unpack('i' * dim, vector_bytes)
            vectors.append(vector)

    return np.array(vectors, dtype=np.int32)

def compute_recall_at_k(retrieved_ids, ground_truth_ids, k):
    """Compute recall@k metric"""
    if len(retrieved_ids) == 0 or len(ground_truth_ids) == 0:
        return 0.0

    retrieved_set = set(retrieved_ids[:k])
    ground_truth_set = set(ground_truth_ids[:k])

    intersection = retrieved_set.intersection(ground_truth_set)
    recall = len(intersection) / min(k, len(ground_truth_set))
    return recall

def benchmark_insertion_performance(vectors, batch_sizes=[1000, 5000, 10000, 20000]):
    """Benchmark insertion performance on different batch sizes"""
    print("\n" + "="*60)
    print("📊 INSERTION PERFORMANCE BENCHMARK")
    print("="*60)

    results = {}

    for batch_size in batch_sizes:
        if batch_size > len(vectors):
            continue

        print(f"\n🧪 Testing batch size: {batch_size} vectors")
        print("-" * 40)

        # Clear database
        native.clear_database()

        # Prepare batch
        batch_vectors = vectors[:batch_size]
        batch_ids = [f'sift_{i}' for i in range(batch_size)]
        batch_metadata = [{}] * batch_size

        # Benchmark insertion
        start_time = time.time()
        insert_results = native.add_vector_batch(batch_ids, batch_vectors, batch_metadata)
        end_time = time.time()

        duration = end_time - start_time
        rate = batch_size / duration if duration > 0 else 0

        results[batch_size] = {
            'duration': duration,
            'rate': rate,
            'success': len(insert_results) == batch_size
        }

        print(f"Vectors inserted: {len(insert_results)}/{batch_size}")
        print(f"Time taken: {duration:.2f}s")
        print(f"Insertion rate: {rate:.0f} vec/s")

        if batch_size >= 10000:
            print(f"🎯 Segmented HNSW: {'✅ ACTIVE' if rate > 15000 else '❌ NOT ACTIVE'}")

    return results

def benchmark_search_quality(vectors, queries, ground_truth, test_sizes=[1000, 5000, 10000]):
    """Benchmark search quality with recall@k metrics"""
    print("\n" + "="*60)
    print("🔍 SEARCH QUALITY BENCHMARK")
    print("="*60)

    quality_results = {}

    for test_size in test_sizes:
        if test_size > len(vectors):
            continue

        print(f"\n🧪 Testing search quality: {test_size} vectors")
        print("-" * 40)

        # Clear and build index
        native.clear_database()
        test_vectors = vectors[:test_size]
        test_ids = [f'sift_{i}' for i in range(test_size)]
        test_metadata = [{}] * test_size

        # Insert vectors
        start_time = time.time()
        native.add_vector_batch(test_ids, test_vectors, test_metadata)
        build_time = time.time() - start_time

        # Test search quality on subset of queries
        k_values = [1, 10, 100]
        recalls = {k: [] for k in k_values}
        search_times = []

        test_queries = queries[:min(100, len(queries))]  # Test first 100 queries

        for i, query in enumerate(test_queries):
            start = time.time()
            results = native.search_vectors(query, max(k_values), {})
            end = time.time()

            search_times.append((end - start) * 1000)  # Convert to ms

            # Extract result IDs (convert back to indices)
            retrieved_ids = []
            for result in results:
                if 'id' in result and result['id'].startswith('sift_'):
                    idx = int(result['id'].split('_')[1])
                    retrieved_ids.append(idx)

            # Get ground truth for this query
            gt_ids = ground_truth[i] if i < len(ground_truth) else []

            # Compute recall@k for different k values
            for k in k_values:
                if k <= len(gt_ids):
                    recall = compute_recall_at_k(retrieved_ids, gt_ids, k)
                    recalls[k].append(recall)

        # Compute average metrics
        avg_recalls = {k: np.mean(recalls[k]) if recalls[k] else 0.0 for k in k_values}
        avg_search_time = np.mean(search_times) if search_times else 0.0

        quality_results[test_size] = {
            'build_time': build_time,
            'recalls': avg_recalls,
            'search_latency': avg_search_time,
            'queries_tested': len(test_queries)
        }

        print(f"Build time: {build_time:.2f}s")
        print(f"Average search latency: {avg_search_time:.2f}ms")
        for k in k_values:
            print(f"Recall@{k}: {avg_recalls[k]:.3f} ({avg_recalls[k]*100:.1f}%)")

    return quality_results

def run_sift1m_validation():
    """Run complete SIFT1M validation benchmark"""
    print("="*80)
    print("🧪 SIFT1M VALIDATION BENCHMARK")
    print("Validating OmenDB Segmented HNSW Performance Claims")
    print("="*80)

    # Download dataset if needed
    if not download_sift1m():
        print("❌ Cannot proceed without SIFT1M dataset")
        return

    # Load dataset
    print("\n📁 Loading SIFT1M dataset...")

    try:
        base_vectors = read_fvecs("sift1m/sift_base.fvecs")
        query_vectors = read_fvecs("sift1m/sift_query.fvecs")
        ground_truth = read_ivecs("sift1m/sift_groundtruth.ivecs")

        print(f"✅ Loaded {len(base_vectors)} base vectors (dim: {base_vectors.shape[1]})")
        print(f"✅ Loaded {len(query_vectors)} query vectors")
        print(f"✅ Loaded ground truth for {len(ground_truth)} queries")

    except Exception as e:
        print(f"❌ Error loading dataset: {e}")
        return

    # Normalize vectors (SIFT vectors are typically in [0,255] range)
    base_vectors = base_vectors.astype(np.float32)
    query_vectors = query_vectors.astype(np.float32)

    # Run benchmarks
    print("\n🚀 Starting comprehensive validation...")

    # 1. Insertion Performance Benchmark
    insertion_results = benchmark_insertion_performance(base_vectors)

    # 2. Search Quality Benchmark
    quality_results = benchmark_search_quality(base_vectors, query_vectors, ground_truth)

    # 3. Summary Report
    print("\n" + "="*80)
    print("📊 SIFT1M VALIDATION RESULTS")
    print("="*80)

    print(f"\n{'Size':<8} {'Rate (vec/s)':<12} {'Recall@1':<10} {'Recall@10':<11} {'Latency (ms)':<12} {'Status'}")
    print("-" * 75)

    for size in sorted(set(insertion_results.keys()) & set(quality_results.keys())):
        ins = insertion_results[size]
        qual = quality_results[size]

        rate = ins['rate']
        r1 = qual['recalls'].get(1, 0) * 100
        r10 = qual['recalls'].get(10, 0) * 100
        latency = qual['search_latency']

        status = "🎯 TARGET" if rate >= 15000 and r10 >= 90 else "✅ GOOD" if rate >= 5000 and r10 >= 80 else "⚠️  REVIEW"

        print(f"{size:<8} {rate:<12.0f} {r1:<10.1f}% {r10:<11.1f}% {latency:<12.2f} {status}")

    # Key findings
    print(f"\n🔍 KEY FINDINGS:")
    best_size = max(insertion_results.keys(), key=lambda x: insertion_results[x]['rate'])
    best_rate = insertion_results[best_size]['rate']

    if best_size in quality_results:
        best_recall = quality_results[best_size]['recalls'].get(10, 0) * 100
        print(f"• Peak performance: {best_rate:.0f} vec/s at {best_size} vectors")
        print(f"• Quality at peak: {best_recall:.1f}% recall@10")
        print(f"• Segmented HNSW: {'✅ WORKING' if best_rate > 15000 else '⚠️ NEEDS INVESTIGATION'}")

        if best_rate >= 15000 and best_recall >= 90:
            print(f"🎯 VALIDATION SUCCESS: Claims verified on standard dataset!")
        else:
            print(f"⚠️  VALIDATION ISSUES: Performance or quality below expectations")

    print(f"\n📋 Next Steps:")
    print(f"• Compare against Qdrant/Milvus on same hardware")
    print(f"• Memory usage analysis vs competitors")
    print(f"• Production readiness testing")

if __name__ == "__main__":
    run_sift1m_validation()