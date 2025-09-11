#!/usr/bin/env python3
"""
Implement SIFT1M benchmark - the gold standard for vector search evaluation
"""

import sys
import time
import numpy as np
import urllib.request
import struct
from pathlib import Path
import hashlib
sys.path.append('python/omendb')

def download_file_with_progress(url, filepath):
    """Download file with progress indication"""
    
    def progress_hook(block_num, block_size, total_size):
        downloaded = block_num * block_size
        if total_size > 0:
            percent = min(100, (downloaded / total_size) * 100)
            print(f"\r  Downloading: {percent:.1f}% ({downloaded//1024//1024}MB/{total_size//1024//1024}MB)", end='')
    
    urllib.request.urlretrieve(url, filepath, progress_hook)
    print()  # New line after progress

def parse_fvecs(filename):
    """Parse .fvecs file format used by SIFT1M dataset"""
    
    print(f"📄 Parsing {filename}...")
    
    vectors = []
    with open(filename, 'rb') as f:
        while True:
            # Read dimension (4 bytes, little endian int)
            dim_bytes = f.read(4)
            if len(dim_bytes) < 4:
                break
            
            dim = struct.unpack('<i', dim_bytes)[0]
            
            # Read vector data (dim * 4 bytes, little endian floats)
            vector_bytes = f.read(dim * 4)
            if len(vector_bytes) < dim * 4:
                break
            
            vector = struct.unpack(f'<{dim}f', vector_bytes)
            vectors.append(vector)
    
    print(f"  Loaded {len(vectors)} vectors, {len(vector[0])}D")
    return np.array(vectors, dtype=np.float32)

def parse_ivecs(filename):
    """Parse .ivecs file format (ground truth nearest neighbors)"""
    
    print(f"📄 Parsing {filename}...")
    
    ground_truth = []
    with open(filename, 'rb') as f:
        while True:
            # Read number of neighbors (4 bytes)
            k_bytes = f.read(4)
            if len(k_bytes) < 4:
                break
            
            k = struct.unpack('<i', k_bytes)[0]
            
            # Read neighbor IDs (k * 4 bytes)
            neighbors_bytes = f.read(k * 4)
            if len(neighbors_bytes) < k * 4:
                break
            
            neighbors = struct.unpack(f'<{k}i', neighbors_bytes)
            ground_truth.append(list(neighbors))
    
    print(f"  Loaded ground truth for {len(ground_truth)} queries, {len(ground_truth[0])} neighbors each")
    return ground_truth

def download_sift1m_dataset():
    """Download and prepare SIFT1M dataset"""
    
    print("📊 SIFT1M DATASET DOWNLOAD & PREPARATION")
    print("=" * 60)
    print("SIFT1M: The gold standard benchmark for vector search")
    print("Used by Faiss, Hnswlib, Qdrant, and all major vector DBs")
    print("=" * 60)
    
    # Create data directory
    data_dir = Path('benchmark_data')
    data_dir.mkdir(exist_ok=True)
    
    # Dataset URLs (from INRIA TEXMEX corpus)
    dataset_info = {
        'base': {
            'url': 'ftp://ftp.irisa.fr/local/texmex/corpus/sift/sift_base.fvecs',
            'filename': 'sift_base.fvecs',
            'description': '1,000,000 base vectors (128D)',
            'size_mb': 512
        },
        'query': {
            'url': 'ftp://ftp.irisa.fr/local/texmex/corpus/sift/sift_query.fvecs', 
            'filename': 'sift_query.fvecs',
            'description': '10,000 query vectors (128D)',
            'size_mb': 5
        },
        'groundtruth': {
            'url': 'ftp://ftp.irisa.fr/local/texmex/corpus/sift/sift_groundtruth.ivecs',
            'filename': 'sift_groundtruth.ivecs', 
            'description': 'Ground truth k-NN for recall measurement',
            'size_mb': 4
        }
    }
    
    downloaded_files = {}
    
    for dataset_type, info in dataset_info.items():
        filepath = data_dir / info['filename']
        
        print(f"\n📁 {dataset_type.upper()}: {info['description']}")
        
        if filepath.exists():
            print(f"  ✅ Already downloaded: {filepath}")
            downloaded_files[dataset_type] = filepath
            continue
        
        print(f"  📥 Downloading {info['filename']} ({info['size_mb']}MB)...")
        print(f"  📎 URL: {info['url']}")
        
        try:
            download_file_with_progress(info['url'], filepath)
            print(f"  ✅ Downloaded successfully: {filepath}")
            downloaded_files[dataset_type] = filepath
            
        except Exception as e:
            print(f"  ❌ Download failed: {e}")
            print(f"  🌐 Note: FTP downloads may fail - try manual download")
            downloaded_files[dataset_type] = None
    
    return downloaded_files

def implement_recall_calculation():
    """Implement Recall@K calculation for quality measurement"""
    
    def calculate_recall_at_k(retrieved_ids, ground_truth_ids, k):
        """Calculate Recall@K for a single query"""
        
        if k > len(retrieved_ids):
            k = len(retrieved_ids)
        if k > len(ground_truth_ids):
            k = len(ground_truth_ids)
            
        retrieved_k = set(retrieved_ids[:k])
        ground_truth_k = set(ground_truth_ids[:k])
        
        intersection = retrieved_k.intersection(ground_truth_k)
        recall = len(intersection) / len(ground_truth_k) if ground_truth_k else 0.0
        
        return recall
    
    def calculate_average_recall(all_retrieved, all_ground_truth, k_values=[1, 10, 100]):
        """Calculate average Recall@K across all queries"""
        
        if len(all_retrieved) != len(all_ground_truth):
            raise ValueError("Retrieved and ground truth must have same length")
        
        recall_results = {}
        
        for k in k_values:
            recalls = []
            for retrieved, ground_truth in zip(all_retrieved, all_ground_truth):
                recall = calculate_recall_at_k(retrieved, ground_truth, k)
                recalls.append(recall)
            
            avg_recall = np.mean(recalls)
            recall_results[f'recall@{k}'] = avg_recall
        
        return recall_results
    
    print("🎯 RECALL@K CALCULATION IMPLEMENTATION")
    print("=" * 60)
    print("Industry-standard quality metrics:")
    print("  • Recall@1:   Fraction of queries where rank-1 result is correct")
    print("  • Recall@10:  Fraction of top-10 results that are actually nearest")  
    print("  • Recall@100: Fraction of top-100 results that are actually nearest")
    print()
    print("✅ IMPLEMENTED: calculate_recall_at_k, calculate_average_recall")
    
    return calculate_recall_at_k, calculate_average_recall

def implement_scale_testing():
    """Implement progressive scale testing framework"""
    
    print("\n⚡ PROGRESSIVE SCALE TESTING FRAMEWORK")
    print("=" * 60)
    
    scale_test_plan = {
        'micro': {'size': 1000, 'purpose': 'Development testing'},
        'small': {'size': 10000, 'purpose': 'Current tested scale'}, 
        'medium': {'size': 50000, 'purpose': 'Production small'},
        'large': {'size': 100000, 'purpose': 'Production typical'},
        'xlarge': {'size': 500000, 'purpose': 'Production large'},
        'enterprise': {'size': 1000000, 'purpose': 'Enterprise scale'},
        'web-scale': {'size': 10000000, 'purpose': 'Web-scale (if feasible)'}
    }
    
    def run_scale_test(scale_name, test_size, dimension=128):
        """Run single scale test with comprehensive metrics"""
        
        print(f"\n🧪 SCALE TEST: {scale_name.upper()} ({test_size:,} vectors)")
        print("-" * 40)
        
        # Generate test data
        print("  📊 Generating test data...")
        vectors = np.random.randn(test_size, dimension).astype(np.float32)
        ids = [f"scale_{scale_name}_{i}" for i in range(test_size)]
        
        # Import OmenDB
        try:
            import native
            native.clear_database()
        except Exception as e:
            print(f"  ❌ Failed to initialize: {e}")
            return None
        
        # Memory baseline
        import psutil
        process = psutil.Process()
        mem_before = process.memory_info().rss / 1024 / 1024
        
        # Insertion test
        print("  ⚡ Testing bulk insertion...")
        start_time = time.perf_counter()
        
        try:
            result = native.add_vector_batch(ids, vectors, [{}] * test_size)
            insert_time = time.perf_counter() - start_time
            successful = sum(1 for r in result if r)
            
            mem_after = process.memory_info().rss / 1024 / 1024
            
            # Calculate metrics
            insert_rate = successful / insert_time if insert_time > 0 else 0
            mem_per_vector = (mem_after - mem_before) * 1024 / successful if successful > 0 else 0
            
            print(f"    Insert rate: {insert_rate:8.0f} vec/s")
            print(f"    Success rate: {successful:,}/{test_size:,}")
            print(f"    Total time: {insert_time:8.2f}s")
            print(f"    Memory used: {mem_after - mem_before:8.1f} MB")
            print(f"    Memory/vector: {mem_per_vector:6.1f} KB")
            
            # Search test
            print("  🔍 Testing search performance...")
            query = np.random.randn(dimension).astype(np.float32)
            
            search_times = []
            for _ in range(10):  # Multiple search tests
                search_start = time.perf_counter()
                search_results = native.search_vectors(query, 10, {})
                search_time = (time.perf_counter() - search_start) * 1000
                search_times.append(search_time)
            
            avg_search_time = np.mean(search_times)
            print(f"    Search time: {avg_search_time:8.2f}ms avg")
            print(f"    Results found: {len(search_results)}")
            
            return {
                'scale': scale_name,
                'size': test_size,
                'insert_rate': insert_rate,
                'insert_time': insert_time,
                'memory_mb': mem_after - mem_before,
                'memory_per_vector_kb': mem_per_vector,
                'search_time_ms': avg_search_time,
                'success': True
            }
            
        except Exception as e:
            print(f"  ❌ Scale test failed: {e}")
            return {
                'scale': scale_name,
                'size': test_size,
                'success': False,
                'error': str(e)
            }
    
    print("Scale testing targets:")
    for scale, info in scale_test_plan.items():
        print(f"  {scale:12} - {info['size']:8,} vectors - {info['purpose']}")
    
    print(f"\n✅ READY: run_scale_test function implemented")
    print("📊 METRICS: Insert rate, memory usage, search latency")
    
    return run_scale_test, scale_test_plan

def run_comprehensive_benchmark_suite():
    """Execute comprehensive benchmark suite (implementation plan)"""
    
    print(f"\n🏗️  COMPREHENSIVE BENCHMARK SUITE")
    print("=" * 60)
    print("Implementation status and next steps")
    print("=" * 60)
    
    # Phase 1: Dataset infrastructure  
    print("📊 PHASE 1: Dataset Infrastructure")
    downloaded = download_sift1m_dataset()
    
    if any(path is None for path in downloaded.values()):
        print("⚠️  SIFT1M download incomplete - manual download may be required")
    
    # Phase 2: Quality metrics
    print(f"\n🎯 PHASE 2: Quality Metrics")
    calculate_recall_at_k, calculate_average_recall = implement_recall_calculation()
    
    # Phase 3: Scale testing
    print(f"\n⚡ PHASE 3: Scale Testing")
    run_scale_test, scale_test_plan = implement_scale_testing()
    
    # Phase 4: What's next
    print(f"\n🚀 PHASE 4: Next Implementation Steps")
    print("1. Complete SIFT1M download (may require manual FTP)")
    print("2. Implement SIFT1M benchmark runner")  
    print("3. Run progressive scale tests (1K → 1M vectors)")
    print("4. Compare results against Faiss/Hnswlib baselines")
    print("5. Generate comprehensive performance report")
    
    return {
        'sift1m_files': downloaded,
        'recall_functions': (calculate_recall_at_k, calculate_average_recall),
        'scale_test_function': run_scale_test,
        'scale_plan': scale_test_plan
    }

if __name__ == "__main__":
    print("🧪 COMPREHENSIVE BENCHMARK IMPLEMENTATION")
    print("=" * 60)
    print("Implementing industry-standard benchmarking")
    print("=" * 60)
    
    try:
        benchmark_suite = run_comprehensive_benchmark_suite()
        
        print(f"\n" + "=" * 60)
        print("🏁 BENCHMARK SUITE IMPLEMENTATION COMPLETE")
        print("=" * 60)
        print("✅ SIFT1M download framework implemented")
        print("✅ Recall@K calculation functions ready")
        print("✅ Progressive scale testing framework ready")
        print("🔧 NEXT: Execute scale tests and SIFT1M benchmarking")
        print("=" * 60)
        
    except Exception as e:
        print(f"❌ Benchmark implementation failed: {e}")
        import traceback
        traceback.print_exc()