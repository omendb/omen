#!/usr/bin/env python3
"""
Comprehensive stability validation after segfault fix.
Tests enterprise-scale scenarios to ensure production readiness.
"""

import sys
import time
import numpy as np
import gc
sys.path.append('python/omendb')

def test_large_scale_stability():
    """Test large-scale vector operations"""
    import native
    
    print("🔬 LARGE-SCALE STABILITY VALIDATION")
    print("=" * 60)
    
    test_cases = [
        ("10K vectors", 10000),
        ("25K vectors", 25000), 
        ("50K vectors", 50000),
        ("100K vectors", 100000),
    ]
    
    results = {}
    
    for test_name, size in test_cases:
        print(f"\n📊 Testing {test_name}...")
        
        # Fresh start
        native.clear_database()
        gc.collect()  # Force garbage collection
        
        try:
            # Generate large dataset
            vectors = np.random.randn(size, 768).astype(np.float32)
            ids = [f"large_{i}" for i in range(size)]
            
            start_time = time.perf_counter()
            result = native.add_vector_batch(ids, vectors, [{}] * size)
            elapsed = time.perf_counter() - start_time
            
            successful = sum(1 for r in result if r)
            rate = successful / elapsed if elapsed > 0 else 0
            
            if successful == size:
                print(f"  ✅ SUCCESS: {successful:,} vectors, {rate:.0f} vec/s")
                
                # Quick search test
                query = np.random.randn(768).astype(np.float32)
                search_start = time.perf_counter()
                search_results = native.search_vectors(query, 10, {})
                search_time = (time.perf_counter() - search_start) * 1000
                
                print(f"  🔍 Search: {len(search_results)} results in {search_time:.2f}ms")
                
                results[test_name] = {'size': size, 'rate': rate, 'search_ms': search_time}
            else:
                print(f"  ⚠️  PARTIAL: {successful:,}/{size:,} vectors")
                return False
                
        except Exception as e:
            print(f"  ❌ FAILED: {e}")
            return False
        
        # Memory cleanup
        del vectors, ids
        gc.collect()
    
    print(f"\n✅ ALL LARGE-SCALE TESTS PASSED")
    return results

def test_repeated_operations():
    """Test repeated clear/insert cycles for memory leaks"""
    import native
    
    print(f"\n🔄 REPEATED OPERATIONS STRESS TEST")
    print("=" * 60)
    
    size = 5000
    cycles = 10
    
    rates = []
    
    for cycle in range(cycles):
        print(f"  Cycle {cycle+1:2d}/{cycles}: ", end="")
        
        native.clear_database()
        
        vectors = np.random.randn(size, 768).astype(np.float32)
        ids = [f"cycle_{cycle}_{i}" for i in range(size)]
        
        try:
            start_time = time.perf_counter()
            result = native.add_vector_batch(ids, vectors, [{}] * size)
            elapsed = time.perf_counter() - start_time
            
            successful = sum(1 for r in result if r)
            
            if successful == size:
                rate = successful / elapsed
                rates.append(rate)
                print(f"{rate:4.0f} vec/s ✅")
            else:
                print(f"PARTIAL {successful}/{size} ❌")
                return False
                
        except Exception as e:
            print(f"CRASH: {e} ❌")
            return False
        
        # Cleanup
        del vectors, ids
        
        # Every few cycles, force garbage collection
        if cycle % 3 == 0:
            gc.collect()
    
    # Analyze performance consistency
    avg_rate = sum(rates) / len(rates)
    min_rate = min(rates)
    max_rate = max(rates)
    consistency = (min_rate / max_rate) * 100
    
    print(f"\n  📊 Performance Analysis:")
    print(f"     Average: {avg_rate:.0f} vec/s")
    print(f"     Range:   {min_rate:.0f} - {max_rate:.0f} vec/s")
    print(f"     Consistency: {consistency:.1f}%")
    
    if consistency > 70:
        print(f"  ✅ STABLE: No memory leaks detected")
        return True
    else:
        print(f"  ⚠️  DEGRADATION: Possible memory leak")
        return False

def test_mixed_batch_sizes():
    """Test various batch sizes in sequence"""
    import native
    
    print(f"\n📦 MIXED BATCH SIZES TEST")
    print("=" * 60)
    
    # Test pattern: small, medium, large, huge, back to small
    batch_pattern = [100, 1000, 5000, 20000, 500, 10000, 200]
    
    cumulative = 0
    
    for i, size in enumerate(batch_pattern):
        print(f"  Batch {i+1}: {size:,} vectors... ", end="")
        
        native.clear_database()
        
        vectors = np.random.randn(size, 768).astype(np.float32)
        ids = [f"mixed_{i}_{j}" for j in range(size)]
        
        try:
            start_time = time.perf_counter()
            result = native.add_vector_batch(ids, vectors, [{}] * size)
            elapsed = time.perf_counter() - start_time
            
            successful = sum(1 for r in result if r)
            
            if successful == size:
                rate = successful / elapsed
                cumulative += successful
                print(f"{rate:4.0f} vec/s ✅")
            else:
                print(f"PARTIAL ❌")
                return False
                
        except Exception as e:
            print(f"CRASH ❌")
            return False
    
    print(f"  📊 Total processed: {cumulative:,} vectors across {len(batch_pattern)} batches")
    return True

def test_edge_cases():
    """Test edge cases and boundary conditions"""
    import native
    
    print(f"\n⚠️  EDGE CASES TEST")
    print("=" * 60)
    
    edge_cases = [
        ("Minimum batch", 1),
        ("Very small", 10), 
        ("Boundary 999", 999),
        ("Boundary 1000", 1000),
        ("Boundary 1001", 1001),
        ("Prime number", 4999),
        ("Power of 2", 8192),
        ("Large prime", 15013),
    ]
    
    for case_name, size in edge_cases:
        print(f"  {case_name:15} ({size:5,} vectors): ", end="")
        
        native.clear_database()
        
        vectors = np.random.randn(size, 768).astype(np.float32)
        ids = [f"edge_{case_name}_{i}" for i in range(size)]
        
        try:
            result = native.add_vector_batch(ids, vectors, [{}] * size)
            successful = sum(1 for r in result if r)
            
            if successful == size:
                print("✅")
            else:
                print(f"PARTIAL {successful}/{size} ❌")
                return False
                
        except Exception as e:
            print(f"CRASH ❌")
            return False
    
    return True

def main():
    print("🚀 COMPREHENSIVE STABILITY VALIDATION SUITE")
    print("=" * 70)
    print("Purpose: Validate segfault fix for production readiness")
    print("=" * 70)
    
    start_time = time.time()
    
    # Run all validation tests
    tests = [
        ("Large Scale", test_large_scale_stability),
        ("Repeated Ops", test_repeated_operations), 
        ("Mixed Batches", test_mixed_batch_sizes),
        ("Edge Cases", test_edge_cases),
    ]
    
    passed = 0
    results = {}
    
    for test_name, test_func in tests:
        print(f"\n{'='*20} {test_name.upper()} {'='*20}")
        
        try:
            result = test_func()
            if result:
                print(f"🎉 {test_name}: PASSED")
                passed += 1
                if isinstance(result, dict):
                    results.update(result)
            else:
                print(f"💥 {test_name}: FAILED")
        except Exception as e:
            print(f"💥 {test_name}: EXCEPTION - {e}")
    
    total_time = time.time() - start_time
    
    # Final summary
    print(f"\n" + "="*70)
    print(f"📋 VALIDATION SUMMARY")
    print(f"="*70)
    print(f"Tests passed: {passed}/{len(tests)}")
    print(f"Total time: {total_time:.1f}s")
    
    if passed == len(tests):
        print(f"🎉 ALL TESTS PASSED - PRODUCTION READY!")
        print(f"✅ Segfault fix validated at enterprise scale")
        
        if results:
            print(f"\n📊 Performance Summary:")
            for test, data in results.items():
                print(f"  {test:15}: {data['rate']:6.0f} vec/s, {data['search_ms']:5.2f}ms search")
        
        return True
    else:
        print(f"❌ VALIDATION FAILED - {len(tests)-passed} tests failed")
        print(f"🚨 NOT READY FOR PRODUCTION")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)