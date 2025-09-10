"""
Benchmark all storage implementations.
Compare wrapper vs direct vs optimized.
"""

from omendb.storage_v3_wrapper import VectorStorageV3
from omendb.storage_direct import DirectStorage
from omendb.storage_optimized import OptimizedStorage
from memory import UnsafePointer
from collections import List
from time import perf_counter
from random import seed, random_float64
import os

fn generate_vectors(count: Int, dim: Int) -> UnsafePointer[Float32]:
    """Generate random test vectors."""
    seed(42)
    var vectors = UnsafePointer[Float32].alloc(count * dim)
    for i in range(count * dim):
        vectors[i] = Float32(random_float64() * 2.0 - 1.0)
    return vectors

fn benchmark_wrapper() raises:
    """Benchmark storage_v3_wrapper."""
    print("\n=== VectorStorageV3 (Wrapper) ===")
    
    var dimension = 768
    var storage = VectorStorageV3("test_wrapper", dimension)
    
    # Test write performance
    var batch_sizes = [100, 1000, 10000]
    
    for batch_size in batch_sizes:
        var vectors = generate_vectors(batch_size, dimension)
        
        var start = perf_counter()
        for i in range(batch_size):
            var id = "vec_" + String(i)
            var vec_ptr = vectors.offset(i * dimension)
            _ = storage.save_vector(id, vec_ptr)
        var elapsed = perf_counter() - start
        
        var throughput = Float64(batch_size) / elapsed
        print("Batch ", batch_size, ": ", Int(throughput), " vec/s")
        
        vectors.free()
    
    storage.close()
    
    # Cleanup
    try:
        os.remove("test_wrapper.mmap")
    except:
        pass

fn benchmark_direct() raises:
    """Benchmark direct storage."""
    print("\n=== DirectStorage (No Python FFI) ===")
    
    var dimension = 768
    var storage = DirectStorage("test_direct", dimension)
    
    var batch_sizes = [100, 1000, 10000]
    
    for batch_size in batch_sizes:
        var vectors = generate_vectors(batch_size, dimension)
        
        var start = perf_counter()
        for i in range(batch_size):
            var id = "vec_" + String(i)
            var vec_ptr = vectors.offset(i * dimension)
            _ = storage.save_vector(id, vec_ptr)
        var elapsed = perf_counter() - start
        
        var throughput = Float64(batch_size) / elapsed
        print("Batch ", batch_size, ": ", Int(throughput), " vec/s")
        
        vectors.free()
    
    storage.close()
    
    # Cleanup
    try:
        os.remove("test_direct.db")
    except:
        pass

fn benchmark_optimized() raises:
    """Benchmark optimized storage."""
    print("\n=== OptimizedStorage (Parallel + SIMD) ===")
    
    var dimension = 768
    var storage = OptimizedStorage("test_optimized", dimension)
    
    var batch_sizes = [100, 1000, 10000]
    
    for batch_size in batch_sizes:
        var vectors = generate_vectors(batch_size, dimension)
        
        var start = perf_counter()
        var saved = storage.save_batch_parallel(vectors, batch_size)
        var elapsed = perf_counter() - start
        
        var throughput = Float64(batch_size) / elapsed
        print("Batch ", batch_size, ": ", Int(throughput), " vec/s")
        
        vectors.free()
    
    # Show stats
    print(storage.get_stats())
    
    storage.close()
    
    # Cleanup
    try:
        os.remove("test_optimized.opt")
    except:
        pass

fn compare_all() raises:
    """Compare all implementations."""
    print("\n=== Head-to-Head Comparison (10K vectors) ===")
    
    var dimension = 768
    var num_vectors = 10000
    var vectors = generate_vectors(num_vectors, dimension)
    
    # Test wrapper
    var storage1 = VectorStorageV3("test_cmp1", dimension)
    var start1 = perf_counter()
    for i in range(num_vectors):
        _ = storage1.save_vector("vec_" + String(i), vectors.offset(i * dimension))
    var time1 = perf_counter() - start1
    storage1.close()
    
    # Test direct
    var storage2 = DirectStorage("test_cmp2", dimension)
    var start2 = perf_counter()
    for i in range(num_vectors):
        _ = storage2.save_vector("vec_" + String(i), vectors.offset(i * dimension))
    var time2 = perf_counter() - start2
    storage2.close()
    
    # Test optimized
    var storage3 = OptimizedStorage("test_cmp3", dimension)
    var start3 = perf_counter()
    _ = storage3.save_batch_parallel(vectors, num_vectors)
    var time3 = perf_counter() - start3
    storage3.close()
    
    # Calculate throughputs
    var tp1 = Float64(num_vectors) / time1
    var tp2 = Float64(num_vectors) / time2
    var tp3 = Float64(num_vectors) / time3
    
    print("\nResults for ", num_vectors, " vectors:")
    print("  Wrapper:   ", Int(tp1), " vec/s (baseline)")
    print("  Direct:    ", Int(tp2), " vec/s (", Int(tp2/tp1), "x faster)")
    print("  Optimized: ", Int(tp3), " vec/s (", Int(tp3/tp1), "x faster)")
    
    # Compare to industry
    var industry_best = 83000  # Milvus
    print("\nGap to industry best (", industry_best, " vec/s):")
    print("  Wrapper:   ", Int(industry_best/tp1), "x slower")
    print("  Direct:    ", Int(industry_best/tp2), "x slower")
    print("  Optimized: ", Int(industry_best/tp3), "x slower")
    
    if tp3 > 10000:
        print("\n✅ ACHIEVED 10,000+ vec/s target!")
    else:
        print("\n🎯 Need ", Int(10000/tp3), "x more improvement for target")
    
    vectors.free()
    
    # Cleanup
    for path in ["test_cmp1.mmap", "test_cmp2.db", "test_cmp3.opt"]:
        try:
            os.remove(path)
        except:
            pass

fn main() raises:
    """Run all benchmarks."""
    print("=" * 60)
    print("STORAGE PERFORMANCE BENCHMARK")
    print("=" * 60)
    
    benchmark_wrapper()
    benchmark_direct()
    benchmark_optimized()
    compare_all()
    
    print("\n" + "=" * 60)
    print("BENCHMARK COMPLETE")
    print("=" * 60)