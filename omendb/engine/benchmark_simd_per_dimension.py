#!/usr/bin/env python3
"""
Per-dimension SIMD optimization benchmark.

Tests each dimension separately to properly measure SIMD kernel performance.
Each test runs in isolation to avoid dimension conflicts.
"""

import sys
import os
sys.path.insert(0, 'python')

import time
import numpy as np
import subprocess
from typing import Dict, List, Tuple

def benchmark_dimension(dimension: int, num_vectors: int = 200) -> Dict:
    """
    Benchmark a specific dimension in isolation.
    Each run starts fresh to avoid dimension conflicts.
    """
    print(f"\n🔬 Testing {dimension}D (specialized kernel: {dimension in [128, 256, 512, 768]})")
    
    # Import fresh each time to avoid dimension conflicts
    if 'omendb.native' in sys.modules:
        del sys.modules['omendb.native']
    
    import omendb.native as native
    
    # Clear database
    native.clear_database()
    
    # Test connection
    try:
        result = native.test_connection()
        print(f"   Module ready: {result}")
    except Exception as e:
        return {'dimension': dimension, 'error': str(e)}
    
    # Generate test vectors
    vectors = []
    for i in range(num_vectors):
        vector = np.random.rand(dimension).astype(np.float32)
        vectors.append(vector.tolist())
    
    # Benchmark insertion
    print(f"   Inserting {num_vectors} vectors...")
    start_time = time.time()
    
    success_count = 0
    for i, vector in enumerate(vectors):
        try:
            result = native.add_vector(f'vec_{i}', vector, {})
            if result:
                success_count += 1
            else:
                print(f"   ❌ Insert failed at vector {i}")
                break
                
        except Exception as e:
            print(f"   ❌ Exception at vector {i}: {e}")
            break
    
    insert_time = time.time() - start_time
    insert_rate = success_count / insert_time if insert_time > 0 else 0
    
    # Verify database state
    try:
        stats = native.get_stats()
        db_count = stats.get('vector_count', 0)
        db_dimension = stats.get('dimension', 0)
    except Exception as e:
        return {'dimension': dimension, 'error': f'Stats error: {e}'}
    
    print(f"   ✅ Inserted: {success_count}/{num_vectors} → {insert_rate:.1f} vec/s")
    print(f"   📊 DB: {db_count} vectors @ {db_dimension}D")
    
    # Quick search test
    search_rate = 0
    if success_count > 0:
        try:
            query = np.random.rand(dimension).astype(np.float32)
            search_start = time.time()
            
            # Run 20 searches
            search_count = 0
            for _ in range(20):
                results = native.search_vectors(query.tolist(), 5, {})
                if results and len(results) > 0:
                    search_count += 1
            
            search_time = time.time() - search_start
            search_rate = 20 / search_time if search_time > 0 else 0
            
            print(f"   🔍 Search: {search_count}/20 → {search_rate:.1f} q/s")
            
        except Exception as e:
            print(f"   ⚠️ Search error: {e}")
    
    return {
        'dimension': dimension,
        'vectors_tested': num_vectors,
        'vectors_inserted': success_count,
        'insertion_rate': insert_rate,
        'search_rate': search_rate,
        'db_count': db_count,
        'db_dimension': db_dimension,
        'success': success_count == num_vectors,
        'specialized_kernel': dimension in [128, 256, 512, 768]
    }

def main():
    """Run comprehensive per-dimension benchmark."""
    
    print("=" * 70)
    print("🎯 SIMD OPTIMIZATION: PER-DIMENSION BENCHMARK")
    print("=" * 70)
    
    # Test dimensions including specialized kernels
    test_dimensions = [
        64,    # Small dimension
        128,   # ✨ Specialized kernel
        192,   # Mid-size (adaptive)
        256,   # ✨ Specialized kernel  
        384,   # Large (adaptive)
        512,   # ✨ Specialized kernel
        768,   # ✨ Specialized kernel
        1024,  # Very large (adaptive)
    ]
    
    results = []
    
    for dimension in test_dimensions:
        try:
            result = benchmark_dimension(dimension, num_vectors=150)
            results.append(result)
            
            # Small delay between tests
            time.sleep(0.5)
            
        except Exception as e:
            print(f"❌ Failed to test {dimension}D: {e}")
            results.append({
                'dimension': dimension,
                'error': str(e),
                'success': False
            })
    
    # Analysis Report
    print("\n" + "=" * 70)
    print("📊 SIMD OPTIMIZATION RESULTS")
    print("=" * 70)
    
    successful_results = [r for r in results if r.get('success', False)]
    
    if not successful_results:
        print("❌ No successful benchmarks completed")
        return
    
    # Results table
    print(f"\n{'Dimension':<10} {'Kernel':<12} {'Rate (v/s)':<12} {'Search (q/s)':<12} {'Success'}")
    print("─" * 70)
    
    for result in results:
        if 'error' in result:
            print(f"{result['dimension']:<10}D {'ERROR':<12} {'─':<12} {'─':<12} ❌")
            continue
            
        kernel_type = "Specialized" if result.get('specialized_kernel') else "Adaptive"
        insert_rate = result.get('insertion_rate', 0)
        search_rate = result.get('search_rate', 0)
        success_icon = "✅" if result.get('success') else "❌"
        
        print(f"{result['dimension']:<10}D {kernel_type:<12} {insert_rate:<12.1f} {search_rate:<12.1f} {success_icon}")
    
    # Performance analysis
    specialized_results = [r for r in successful_results if r.get('specialized_kernel')]
    adaptive_results = [r for r in successful_results if not r.get('specialized_kernel')]
    
    print(f"\n🎯 PERFORMANCE ANALYSIS:")
    
    if specialized_results:
        specialized_rates = [r['insertion_rate'] for r in specialized_results]
        max_specialized = max(specialized_rates)
        avg_specialized = sum(specialized_rates) / len(specialized_rates)
        
        print(f"   🚀 Specialized kernels (128D, 256D, 512D, 768D):")
        print(f"      Peak rate: {max_specialized:.1f} vectors/second") 
        print(f"      Average: {avg_specialized:.1f} vectors/second")
    
    if adaptive_results:
        adaptive_rates = [r['insertion_rate'] for r in adaptive_results]
        max_adaptive = max(adaptive_rates)
        avg_adaptive = sum(adaptive_rates) / len(adaptive_rates)
        
        print(f"   ⚡ Adaptive kernels (other dimensions):")
        print(f"      Peak rate: {max_adaptive:.1f} vectors/second")
        print(f"      Average: {avg_adaptive:.1f} vectors/second")
    
    # Overall performance
    all_rates = [r['insertion_rate'] for r in successful_results]
    overall_max = max(all_rates)
    overall_avg = sum(all_rates) / len(all_rates)
    
    print(f"\n   📈 Overall performance:")
    print(f"      Peak rate: {overall_max:.1f} vectors/second")
    print(f"      Average rate: {overall_avg:.1f} vectors/second")
    
    # Improvement assessment (baseline ~2000 vec/s)
    baseline = 2000
    improvement_factor = overall_max / baseline
    
    print(f"\n   🎯 Improvement vs baseline (~{baseline} v/s):")
    if improvement_factor >= 3.0:
        print(f"      🎉 EXCELLENT! {improvement_factor:.1f}x speedup achieved")
    elif improvement_factor >= 2.0:
        print(f"      🚀 GOOD! {improvement_factor:.1f}x speedup achieved")
    elif improvement_factor >= 1.5:
        print(f"      ⚡ MODERATE! {improvement_factor:.1f}x speedup achieved")  
    else:
        print(f"      ⚠️ LIMITED! {improvement_factor:.1f}x speedup (investigate needed)")
    
    # Specialized kernel efficiency
    if specialized_results and adaptive_results:
        spec_avg = sum(r['insertion_rate'] for r in specialized_results) / len(specialized_results)
        adapt_avg = sum(r['insertion_rate'] for r in adaptive_results) / len(adaptive_results)
        kernel_advantage = spec_avg / adapt_avg if adapt_avg > 0 else 1.0
        
        print(f"   🔧 Specialized kernel advantage: {kernel_advantage:.1f}x faster")
    
    print(f"\n🎯 Next Steps:")
    print(f"   - If performance is good: Implement RobustPrune algorithm")
    print(f"   - If performance is limited: Profile distance calculation vs graph operations")
    print(f"   - Memory pool expansion for 100K+ vectors")

if __name__ == "__main__":
    main()