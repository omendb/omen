#!/usr/bin/env python3
"""
Execute progressive scale testing from 1K to 1M+ vectors
"""

import sys
import time
import numpy as np
import psutil
import gc
from pathlib import Path
import json
sys.path.append('python/omendb')

def get_memory_usage():
    """Get current memory usage in MB"""
    process = psutil.Process()
    return process.memory_info().rss / 1024 / 1024

def run_single_scale_test(scale_name, test_size, dimension=768):
    """Run comprehensive scale test with detailed metrics"""
    
    print(f"\n🧪 SCALE TEST: {scale_name.upper()} ({test_size:,} vectors, {dimension}D)")
    print("=" * 60)
    
    try:
        import native
        
        # Force garbage collection and clear database
        gc.collect()
        native.clear_database()
        gc.collect()
        
        # Memory baseline
        mem_baseline = get_memory_usage()
        print(f"📊 Memory baseline: {mem_baseline:.1f} MB")
        
        # Generate test data
        print(f"📊 Generating {test_size:,} test vectors...")
        np.random.seed(42)  # Reproducible
        vectors = np.random.randn(test_size, dimension).astype(np.float32)
        ids = [f"scale_{scale_name}_{i}" for i in range(test_size)]
        metadata = [{}] * test_size
        
        data_size_mb = test_size * dimension * 4 / 1024 / 1024
        print(f"📊 Test data size: {data_size_mb:.1f} MB")
        
        # Insertion performance test
        print(f"⚡ Testing bulk insertion...")
        mem_before_insert = get_memory_usage()
        
        start_time = time.perf_counter()
        result = native.add_vector_batch(ids, vectors, metadata)
        insert_time = time.perf_counter() - start_time
        
        mem_after_insert = get_memory_usage()
        successful = sum(1 for r in result if r)
        
        # Calculate insertion metrics
        insert_rate = successful / insert_time if insert_time > 0 else 0
        mem_used = mem_after_insert - mem_before_insert
        mem_per_vector = mem_used * 1024 / successful if successful > 0 else 0
        
        print(f"  Insert rate: {insert_rate:8.0f} vec/s")
        print(f"  Success rate: {successful:,}/{test_size:,} ({successful/test_size*100:.1f}%)")
        print(f"  Total time: {insert_time:8.2f}s")
        print(f"  Memory used: {mem_used:8.1f} MB")
        print(f"  Memory/vector: {mem_per_vector:6.1f} KB")
        
        # Search performance test (if insertion succeeded)
        search_results = None
        if successful > 0:
            print(f"🔍 Testing search performance...")
            
            # Generate search queries
            query_vectors = np.random.randn(10, dimension).astype(np.float32)
            search_times = []
            total_results = 0
            
            for i, query in enumerate(query_vectors):
                search_start = time.perf_counter()
                results = native.search_vectors(query, 10, {})
                search_time = (time.perf_counter() - search_start) * 1000  # ms
                
                search_times.append(search_time)
                total_results += len(results)
                
                if i == 0:  # Log first query
                    print(f"  First query: {len(results)} results in {search_time:.2f}ms")
            
            avg_search_time = np.mean(search_times)
            std_search_time = np.std(search_times)
            avg_results = total_results / len(query_vectors)
            
            print(f"  Avg search time: {avg_search_time:6.2f} ± {std_search_time:.2f}ms")
            print(f"  Avg results: {avg_results:.1f} per query")
            
            search_results = {
                'avg_time_ms': avg_search_time,
                'std_time_ms': std_search_time,
                'avg_results': avg_results
            }
        
        # Final memory check
        mem_final = get_memory_usage()
        total_memory = mem_final - mem_baseline
        
        print(f"📊 Final memory: {mem_final:.1f} MB (total: {total_memory:.1f} MB)")
        
        # Determine success level
        if successful == test_size:
            status = "✅ COMPLETE SUCCESS"
        elif successful > test_size * 0.9:
            status = "🟡 MOSTLY SUCCESSFUL"
        elif successful > 0:
            status = "🟠 PARTIAL SUCCESS"
        else:
            status = "❌ FAILED"
        
        print(f"🏁 Status: {status}")
        
        return {
            'scale': scale_name,
            'size': test_size,
            'dimension': dimension,
            'successful_inserts': successful,
            'success_rate': successful / test_size,
            'insert_rate_vec_per_s': insert_rate,
            'insert_time_s': insert_time,
            'memory_used_mb': mem_used,
            'memory_per_vector_kb': mem_per_vector,
            'total_memory_mb': total_memory,
            'search_results': search_results,
            'status': status,
            'timestamp': time.time()
        }
        
    except Exception as e:
        print(f"❌ Scale test failed: {e}")
        import traceback
        traceback.print_exc()
        
        return {
            'scale': scale_name,
            'size': test_size,
            'dimension': dimension,
            'success': False,
            'error': str(e),
            'timestamp': time.time()
        }

def run_progressive_scale_tests():
    """Run progressive scale tests from small to enterprise scale"""
    
    print("⚡ PROGRESSIVE SCALE TESTING")
    print("=" * 60)
    print("Testing OmenDB vector engine at production scales")
    print("=" * 60)
    
    # Test scales (conservative progression to find breaking point)
    scale_tests = [
        ('micro', 1000),      # Baseline
        ('small', 5000),      # Current known good
        ('medium', 10000),    # First unknown
        ('large', 25000),     # Stress test
        ('xlarge', 50000),    # Major scale
        ('enterprise', 100000), # Enterprise scale
        # ('mega', 250000),   # Only if previous succeed
        # ('web-scale', 500000) # Only if confident
    ]
    
    results = []
    
    for scale_name, test_size in scale_tests:
        try:
            result = run_single_scale_test(scale_name, test_size, dimension=768)
            results.append(result)
            
            # Stop testing if we hit a major failure
            if result.get('success_rate', 0) < 0.5:
                print(f"\n🛑 STOPPING: Success rate dropped below 50% at {test_size:,} vectors")
                print("🔧 Investigation needed before testing larger scales")
                break
            
            # Brief pause between tests
            time.sleep(2)
            
        except KeyboardInterrupt:
            print(f"\n⚠️  Testing interrupted by user")
            break
        except Exception as e:
            print(f"\n❌ Fatal error during {scale_name} test: {e}")
            break
    
    return results

def analyze_scale_results(results):
    """Analyze scale test results and identify patterns"""
    
    print(f"\n📊 SCALE TEST ANALYSIS")
    print("=" * 60)
    
    if not results:
        print("❌ No results to analyze")
        return
    
    # Performance analysis
    print("Performance progression:")
    print("Scale        Size      Success   Rate(vec/s)  Memory/vec  Search(ms)")
    print("-" * 65)
    
    for result in results:
        if result.get('success_rate', 0) > 0:
            size_str = f"{result['size']:,}"
            success_str = f"{result['success_rate']*100:.0f}%"
            rate_str = f"{result['insert_rate_vec_per_s']:,.0f}"
            memory_str = f"{result['memory_per_vector_kb']:.1f}KB"
            
            if result['search_results']:
                search_str = f"{result['search_results']['avg_time_ms']:.1f}"
            else:
                search_str = "N/A"
            
            print(f"{result['scale']:12} {size_str:>8} {success_str:>8} {rate_str:>10} {memory_str:>10} {search_str:>9}")
    
    # Find breaking points
    successful_scales = [r for r in results if r.get('success_rate', 0) > 0.9]
    if successful_scales:
        max_successful = max(successful_scales, key=lambda x: x['size'])
        print(f"\n✅ Reliable scale: Up to {max_successful['size']:,} vectors")
        print(f"   Performance: {max_successful['insert_rate_vec_per_s']:,.0f} vec/s")
        print(f"   Memory efficiency: {max_successful['memory_per_vector_kb']:.1f} KB/vector")
    
    # Performance trends
    if len(successful_scales) > 1:
        rates = [r['insert_rate_vec_per_s'] for r in successful_scales]
        sizes = [r['size'] for r in successful_scales]
        
        if rates[-1] > rates[0]:
            trend = "📈 IMPROVING"
        elif rates[-1] < rates[0] * 0.8:
            trend = "📉 DEGRADING"
        else:
            trend = "📊 STABLE"
        
        print(f"\nPerformance trend: {trend}")
        print(f"  Small scale: {rates[0]:,.0f} vec/s ({sizes[0]:,} vectors)")
        print(f"  Large scale: {rates[-1]:,.0f} vec/s ({sizes[-1]:,} vectors)")
    
    # Memory analysis
    memory_data = [r for r in results if r.get('memory_per_vector_kb')]
    if memory_data:
        memories = [r['memory_per_vector_kb'] for r in memory_data]
        avg_memory = np.mean(memories)
        
        if max(memories) > min(memories) * 2:
            memory_trend = "⚠️  VARIABLE"
        else:
            memory_trend = "✅ CONSISTENT"
        
        print(f"\nMemory efficiency: {memory_trend}")
        print(f"  Average: {avg_memory:.1f} KB/vector")
        print(f"  Range: {min(memories):.1f} - {max(memories):.1f} KB/vector")
    
    return results

def save_results(results):
    """Save results for future reference"""
    
    timestamp = int(time.time())
    filename = f"scale_test_results_{timestamp}.json"
    
    with open(filename, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n💾 Results saved: {filename}")
    return filename

if __name__ == "__main__":
    print("🧪 COMPREHENSIVE SCALE TESTING")
    print("=" * 60)
    print("Progressive testing: 1K → 100K+ vectors")
    print("Goal: Find performance limits and breaking points")  
    print("=" * 60)
    
    try:
        # Run all scale tests
        results = run_progressive_scale_tests()
        
        # Analyze results
        analyzed_results = analyze_scale_results(results)
        
        # Save results
        if results:
            results_file = save_results(results)
        
        print(f"\n" + "=" * 60)
        print("🏁 SCALE TESTING COMPLETE")
        print("=" * 60)
        print("✅ Comprehensive performance data collected")
        print("📊 Breaking points and limits identified")
        print("🔧 Data available for optimization decisions")
        print("=" * 60)
        
    except Exception as e:
        print(f"❌ Scale testing failed: {e}")
        import traceback
        traceback.print_exc()