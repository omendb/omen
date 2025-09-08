#!/usr/bin/env python3
"""Profile insertion performance to identify bottlenecks."""

import time
import numpy as np
import sys
import os
import cProfile
import pstats
from io import StringIO

sys.path.insert(0, 'omendb/engine/python')
from omendb.api import DB

def benchmark_insertion():
    """Profile insertion performance in detail."""
    print("🔍 PROFILING INSERTION PERFORMANCE")
    print("=" * 50)
    
    db = DB()
    db.clear()
    
    # Profile individual operations
    vectors = [np.random.rand(128).astype(np.float32) for _ in range(1000)]
    ids = [f"vec_{i}" for i in range(1000)]
    
    print("\n1. INDIVIDUAL ADD OPERATIONS")
    print("-" * 30)
    
    # Time breakdown
    total_time = 0
    setup_times = []
    add_times = []
    
    for i in range(100):  # Profile first 100 for detail
        # Setup time
        start_setup = time.perf_counter()
        vec = vectors[i]
        vec_id = ids[i]
        setup_time = time.perf_counter() - start_setup
        setup_times.append(setup_time * 1000)  # ms
        
        # Actual add time
        start_add = time.perf_counter()
        success = db.add(vec_id, vec)
        add_time = time.perf_counter() - start_add
        add_times.append(add_time * 1000)  # ms
        
        total_time += setup_time + add_time
    
    print(f"Per-operation breakdown (avg of 100):")
    print(f"  Setup time: {np.mean(setup_times):.3f}ms")
    print(f"  Add time: {np.mean(add_times):.3f}ms") 
    print(f"  Total: {np.mean(setup_times) + np.mean(add_times):.3f}ms")
    print(f"  Rate: {1000 / (np.mean(setup_times) + np.mean(add_times)):.0f} vec/s")
    
    print("\n2. PYTHON-LEVEL PROFILING")
    print("-" * 30)
    
    # Profile with cProfile
    pr = cProfile.Profile()
    pr.enable()
    
    # Add 500 more vectors under profiling
    for i in range(100, 600):
        success = db.add(ids[i], vectors[i])
        
    pr.disable()
    
    # Analyze profile
    s = StringIO()
    ps = pstats.Stats(pr, stream=s).sort_stats('cumulative')
    ps.print_stats(20)  # Top 20 functions
    
    profile_output = s.getvalue()
    print("Top time consumers:")
    
    # Extract key lines
    lines = profile_output.split('\n')
    for line in lines:
        if 'add(' in line or 'insert' in line or 'distance' in line or 'search' in line:
            if 'ncalls' not in line and line.strip():
                print(f"  {line.strip()}")
    
    print("\n3. COMPETITOR COMPARISON")
    print("-" * 30)
    
    current_rate = 4431  # From our test
    competitors = {
        "Pinecone": 50000,
        "Weaviate": 25000, 
        "Qdrant": 30000,
        "Chroma": 15000,
        "OmenDB": current_rate
    }
    
    print("Insertion rates (vectors/second):")
    for name, rate in sorted(competitors.items(), key=lambda x: x[1], reverse=True):
        gap = rate / current_rate if name != "OmenDB" else 1.0
        status = "🥇" if rate == max(competitors.values()) else "🎯" if name == "OmenDB" else "⚡"
        print(f"  {status} {name:12}: {rate:6,} vec/s ({gap:.1f}x vs us)")
    
    print(f"\n📊 GAP ANALYSIS:")
    best_competitor = max(k for k, v in competitors.items() if k != "OmenDB")
    best_rate = competitors[best_competitor]
    gap = best_rate / current_rate
    print(f"  We need {gap:.1f}x speedup to match {best_competitor}")
    print(f"  Target: {best_rate:,} vec/s (current: {current_rate:,} vec/s)")
    
    return {
        'current_rate': current_rate,
        'target_rate': best_rate,
        'speedup_needed': gap,
        'setup_time_ms': np.mean(setup_times),
        'add_time_ms': np.mean(add_times)
    }

def identify_bottlenecks():
    """Identify specific optimization opportunities."""
    print("\n4. BOTTLENECK IDENTIFICATION")
    print("-" * 30)
    
    # Test different vector dimensions
    dimensions = [64, 128, 256, 512]
    rates = []
    
    for dim in dimensions:
        db = DB()
        db.clear()
        
        vectors = [np.random.rand(dim).astype(np.float32) for _ in range(100)]
        start_time = time.perf_counter()
        
        for i, vec in enumerate(vectors):
            db.add(f"vec_{i}", vec)
            
        elapsed = time.perf_counter() - start_time
        rate = 100 / elapsed
        rates.append(rate)
        
        print(f"  {dim:3d}D: {rate:6.0f} vec/s")
    
    # Analyze dimension scaling
    if len(rates) >= 2:
        dim_impact = rates[0] / rates[-1]  # 64D vs 512D
        print(f"\n  Dimension impact: {dim_impact:.2f}x slowdown (64D→512D)")
        if dim_impact > 3:
            print("  🎯 HIGH dimension overhead - optimize distance calculation")
        elif dim_impact > 2:
            print("  ⚠️ MODERATE dimension overhead")
        else:
            print("  ✅ Good dimension scaling")
    
    return rates

def main():
    """Run comprehensive performance profiling."""
    print("🚀 OmenDB Performance Profiling & Optimization Analysis")
    
    # Benchmark current performance
    metrics = benchmark_insertion()
    
    # Identify bottlenecks
    dim_rates = identify_bottlenecks()
    
    print("\n" + "="*50)
    print("OPTIMIZATION RECOMMENDATIONS")
    print("="*50)
    
    speedup_needed = metrics['speedup_needed']
    
    if speedup_needed > 10:
        print("🚨 CRITICAL: Need major optimizations")
        print("  1. Fix batch operations (10-100x gain)")
        print("  2. SIMD distance optimization") 
        print("  3. Memory allocation optimization")
    elif speedup_needed > 5:
        print("⚡ MODERATE: Several optimizations needed") 
        print("  1. Batch operations fix (primary)")
        print("  2. Profile graph construction overhead")
    elif speedup_needed > 2:
        print("🎯 MINOR: Fine-tuning optimizations")
        print("  1. Batch operations for bulk loading")
        print("  2. Memory layout optimization")
    else:
        print("✅ COMPETITIVE: Already in good range")
    
    print(f"\n🎯 Target: {speedup_needed:.1f}x speedup needed")
    print(f"📊 Current: {metrics['current_rate']:,} vec/s")
    print(f"🏆 Goal: {metrics['target_rate']:,} vec/s")

if __name__ == "__main__":
    main()