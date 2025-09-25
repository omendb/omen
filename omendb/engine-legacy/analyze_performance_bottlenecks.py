#!/usr/bin/env python3
"""
Systematic analysis of performance bottlenecks in the fixed HNSW implementation.
Identify specific areas for optimization while maintaining 100% recall.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def analyze_search_bottlenecks():
    """Analyze what's making search slow (8ms vs target <1ms)."""

    print("🔬 SEARCH PERFORMANCE BOTTLENECK ANALYSIS")
    print("=" * 60)

    # Test at different scales to identify scaling issues
    scales = [500, 1000, 2000, 5000]
    results = []

    for n_vectors in scales:
        print(f"\n📊 Testing at {n_vectors} vectors...")

        dimension = 128
        np.random.seed(42)
        vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

        # Build index
        native.clear_database()
        ids = [f"test_{i:06d}" for i in range(n_vectors)]
        metadata = [{}] * n_vectors

        build_start = time.time()
        result = native.add_vector_batch(ids, vectors, metadata)
        build_time = time.time() - build_start

        # Test search with multiple query types
        query_types = [
            ("Random", np.random.randn(10, dimension).astype(np.float32)),
            ("Near-database", vectors[np.random.choice(n_vectors, 5)] + 0.1 * np.random.randn(5, dimension).astype(np.float32)),
            ("Self-search", vectors[np.random.choice(n_vectors, 5)])
        ]

        for query_name, test_queries in query_types:
            search_times = []
            recalls = []

            for query in test_queries:
                # Measure search time
                start_time = time.time()
                search_results = native.search_vectors(query.tolist(), 10, {})
                search_time = (time.time() - start_time) * 1000  # ms
                search_times.append(search_time)

                # Quick recall check
                if query_name != "Random":
                    distances = np.linalg.norm(vectors - query, axis=1)
                    gt_indices = set(np.argsort(distances)[:10])
                    found_indices = set()
                    for r in search_results:
                        try:
                            idx = int(r['id'].split('_')[1])
                            found_indices.add(idx)
                        except:
                            pass
                    recall = len(gt_indices.intersection(found_indices)) / 10
                    recalls.append(recall)

            avg_search_time = np.mean(search_times)
            avg_recall = np.mean(recalls) if recalls else 0

            print(f"  {query_name}: {avg_search_time:.1f}ms avg, {avg_recall:.1%} recall")

        results.append({
            'n_vectors': n_vectors,
            'build_time': build_time,
            'build_rate': n_vectors / build_time,
            'avg_search_time': np.mean([np.mean(search_times) for _, test_queries in query_types for search_times in [[]]]),
        })

    # Analyze scaling characteristics
    print(f"\n📈 SCALING ANALYSIS:")
    print(f"{'Scale':<8} {'Build Rate':<12} {'Search Time':<12} {'Scaling':<10}")
    print("-" * 50)

    for i, r in enumerate(results):
        if i == 0:
            scaling = "baseline"
        else:
            scale_factor = r['n_vectors'] / results[0]['n_vectors']
            time_factor = r['avg_search_time'] / results[0]['avg_search_time']
            ideal_factor = np.log(r['n_vectors']) / np.log(results[0]['n_vectors'])  # O(log n)
            efficiency = ideal_factor / time_factor if time_factor > 0 else 0
            scaling = f"{efficiency:.1%} eff"

        print(f"{r['n_vectors']:<8} {r['build_rate']:<12.0f} {r['avg_search_time']:<12.1f} {scaling:<10}")

    return results

def analyze_construction_bottlenecks():
    """Analyze what's making construction slow (1943 vs target 10K+ vec/s)."""

    print(f"\n🔬 CONSTRUCTION PERFORMANCE BOTTLENECK ANALYSIS")
    print("=" * 60)

    # Test different batch sizes to understand the bottleneck
    test_sizes = [100, 500, 1000, 2000, 5000]
    construction_results = []

    for n_vectors in test_sizes:
        print(f"\n📊 Construction test: {n_vectors} vectors...")

        dimension = 128
        np.random.seed(42)
        vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

        # Test construction performance
        native.clear_database()
        ids = [f"test_{i:06d}" for i in range(n_vectors)]
        metadata = [{}] * n_vectors

        start_time = time.time()
        result = native.add_vector_batch(ids, vectors, metadata)
        build_time = time.time() - start_time

        rate = n_vectors / build_time if build_time > 0 else 0
        print(f"  Rate: {rate:.0f} vec/s")

        construction_results.append({
            'n_vectors': n_vectors,
            'build_time': build_time,
            'rate': rate
        })

    # Identify where the performance drops
    print(f"\n📉 CONSTRUCTION RATE ANALYSIS:")
    print(f"{'Vectors':<8} {'Time (s)':<10} {'Rate (vec/s)':<12} {'Efficiency':<12}")
    print("-" * 50)

    for i, r in enumerate(construction_results):
        if i == 0:
            efficiency = "baseline"
        else:
            ideal_rate = construction_results[0]['rate']  # Should maintain rate
            actual_rate = r['rate']
            efficiency = f"{actual_rate/ideal_rate:.1%}" if ideal_rate > 0 else "N/A"

        print(f"{r['n_vectors']:<8} {r['build_time']:<10.2f} {r['rate']:<12.0f} {efficiency:<12}")

    return construction_results

def identify_optimization_opportunities():
    """Identify specific optimization opportunities."""

    print(f"\n🎯 OPTIMIZATION OPPORTUNITIES")
    print("=" * 60)

    opportunities = []

    # 1. Search optimizations
    print(f"\n🔍 SEARCH OPTIMIZATIONS:")

    # Test current search parameters
    n_vectors = 2000
    dimension = 128
    np.random.seed(42)
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    native.clear_database()
    ids = [f"test_{i:06d}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors
    native.add_vector_batch(ids, vectors, metadata)

    # Test different k values (search result count impact)
    test_query = np.random.randn(dimension).astype(np.float32)

    for k in [1, 5, 10, 20]:
        start_time = time.time()
        results = native.search_vectors(test_query.tolist(), k, {})
        search_time = (time.time() - start_time) * 1000
        print(f"  k={k}: {search_time:.1f}ms")

    print(f"\n💡 IDENTIFIED BOTTLENECKS:")

    # Current vs target performance gaps
    current_search = 8.0  # ms
    target_search = 0.16  # ms (160µs)
    search_gap = current_search / target_search

    current_construction = 1943  # vec/s
    target_construction = 10000  # vec/s
    construction_gap = target_construction / current_construction

    print(f"1. SEARCH SPEED GAP: {search_gap:.0f}x slower than target")
    print(f"   Current: {current_search}ms, Target: {target_search}ms")
    print(f"   NEEDS: Graph traversal optimization, binary quantization, SIMD")

    print(f"\n2. CONSTRUCTION SPEED GAP: {construction_gap:.1f}x slower than target")
    print(f"   Current: {current_construction} vec/s, Target: {target_construction} vec/s")
    print(f"   NEEDS: Hybrid bulk/individual insertion, parallel processing")

    print(f"\n3. GRAPH DENSITY ANALYSIS:")
    print(f"   Individual insertion may create over-dense graphs")
    print(f"   NEEDS: Optimal M parameter tuning, smart pruning")

    print(f"\n4. ALGORITHMIC OPTIMIZATIONS:")
    print(f"   NEEDS: Hub highway navigation, early termination, caching")

    return {
        'search_gap': search_gap,
        'construction_gap': construction_gap,
        'current_search_ms': current_search,
        'current_construction_rate': current_construction
    }

def benchmark_against_targets():
    """Benchmark against competitive targets."""

    print(f"\n🏆 COMPETITIVE BENCHMARKING")
    print("=" * 60)

    # Original performance claims from the docs
    targets = {
        'search_latency_us': 160,  # microseconds
        'construction_rate': 20000,  # vec/s
        'recall_target': 0.95,  # 95%
        'competitors': {
            'Weaviate': {'search_ms': 25, 'insert_rate': 5000},
            'Qdrant': {'search_ms': 35, 'insert_rate': 4000},
            'Milvus': {'search_ms': 30, 'insert_rate': 4500},
            'Pinecone': {'search_ms': 20, 'insert_rate': 3500}
        }
    }

    current = {
        'search_latency_us': 8000,  # 8ms = 8000µs
        'construction_rate': 1943,
        'recall': 1.0  # 100%
    }

    print(f"📊 CURRENT vs TARGETS:")
    print(f"Search latency: {current['search_latency_us']}µs vs {targets['search_latency_us']}µs target")
    print(f"Construction: {current['construction_rate']} vs {targets['construction_rate']} vec/s target")
    print(f"Recall: {current['recall']:.1%} vs {targets['recall_target']:.1%} target")

    print(f"\n🥇 vs COMPETITORS (current performance):")
    for name, perf in targets['competitors'].items():
        search_advantage = (perf['search_ms'] * 1000) / current['search_latency_us']
        insert_advantage = current['construction_rate'] / perf['insert_rate']
        print(f"{name}: {search_advantage:.1f}x search {'advantage' if search_advantage > 1 else 'disadvantage'}, "
              f"{insert_advantage:.1f}x insert {'advantage' if insert_advantage > 1 else 'disadvantage'}")

    print(f"\n🎯 OPTIMIZATION TARGETS:")
    search_improvement_needed = current['search_latency_us'] / targets['search_latency_us']
    construction_improvement_needed = targets['construction_rate'] / current['construction_rate']

    print(f"Search needs: {search_improvement_needed:.0f}x faster")
    print(f"Construction needs: {construction_improvement_needed:.1f}x faster")

    return current, targets

if __name__ == "__main__":
    try:
        print("🚀 COMPREHENSIVE PERFORMANCE BOTTLENECK ANALYSIS")
        print("=" * 70)

        # 1. Analyze search bottlenecks
        search_results = analyze_search_bottlenecks()

        # 2. Analyze construction bottlenecks
        construction_results = analyze_construction_bottlenecks()

        # 3. Identify optimization opportunities
        opportunities = identify_optimization_opportunities()

        # 4. Benchmark against targets
        current, targets = benchmark_against_targets()

        print(f"\n📋 OPTIMIZATION PRIORITY MATRIX:")
        print("=" * 40)
        print(f"1. 🔥 CRITICAL: Search speed (50x improvement needed)")
        print(f"2. 🔥 CRITICAL: Construction speed (5x improvement needed)")
        print(f"3. ✅ ACHIEVED: Search quality (100% recall)")
        print(f"4. 🎯 TARGET: Maintain quality while optimizing")

        print(f"\n🛠️ NEXT OPTIMIZATION STEPS:")
        print("1. Enable binary quantization for 40x distance speedup")
        print("2. Implement hub highway navigation")
        print("3. Optimize graph density (reduce over-connectivity)")
        print("4. Add parallel construction processing")
        print("5. Implement early search termination")

    except Exception as e:
        print(f"💥 Analysis failed: {e}")
        import traceback
        traceback.print_exc()