#!/usr/bin/env python3
"""System-level memory profiling to identify actual memory usage."""

import numpy as np
import psutil
import os
import gc
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def get_memory_info():
    """Get detailed memory info."""
    process = psutil.Process(os.getpid())
    info = process.memory_info()
    return {
        'rss': info.rss / (1024 * 1024),  # MB
        'vms': info.vms / (1024 * 1024),  # MB
    }

def profile_component_memory():
    """Profile memory usage of different components."""
    
    print("\nSystem Memory Profiling for OmenDB Components")
    print("="*60)
    
    # Test parameters
    num_vectors = 100000
    dimension = 128
    batch_size = 10000
    
    print(f"\nTest configuration:")
    print(f"  Vectors: {num_vectors:,}")
    print(f"  Dimension: {dimension}")
    print(f"  Batch size: {batch_size:,}")
    
    # Generate test data
    print(f"\nGenerating test data...")
    start_mem = get_memory_info()
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    after_data_mem = get_memory_info()
    
    data_size = after_data_mem['rss'] - start_mem['rss']
    print(f"  Test data memory: {data_size:.1f} MB")
    print(f"  Theoretical size: {(num_vectors * dimension * 4) / (1024*1024):.1f} MB")
    
    # Test different ID types
    print("\n" + "="*60)
    print("1. Testing String IDs vs Integer IDs")
    print("-"*40)
    
    # String IDs
    string_ids = [f"vector_{i:08d}" for i in range(num_vectors)]
    after_string_ids = get_memory_info()
    string_id_mem = after_string_ids['rss'] - after_data_mem['rss']
    print(f"  String IDs ({num_vectors:,}): {string_id_mem:.1f} MB")
    
    # Clean up string IDs
    del string_ids
    gc.collect()
    
    # Integer IDs (just indices)
    int_ids = list(range(num_vectors))
    after_int_ids = get_memory_info()
    int_id_mem = after_int_ids['rss'] - after_data_mem['rss']
    print(f"  Integer IDs ({num_vectors:,}): {int_id_mem:.1f} MB")
    print(f"  Savings with int IDs: {string_id_mem - int_id_mem:.1f} MB")
    
    del int_ids
    gc.collect()
    
    # Test quantization modes
    print("\n" + "="*60)
    print("2. Testing Quantization Modes")
    print("-"*40)
    
    modes = [
        ("Normal (Float32)", None),
        ("Scalar (Int8)", "scalar"),
        ("Binary (1-bit)", "binary")
    ]
    
    results = {}
    
    for mode_name, mode_type in modes:
        print(f"\n{mode_name}:")
        
        # Create DB
        db = omendb.DB()
        if mode_type == "scalar":
            db.enable_quantization()
        elif mode_type == "binary":
            db.enable_binary_quantization()
        
        # Measure before adding vectors
        before_add = get_memory_info()
        
        # Add vectors in batches
        for i in range(0, num_vectors, batch_size):
            batch = vectors[i:i+batch_size]
            db.add_batch(batch)
            if i == 0:
                print(f"  First batch added...")
        
        # Measure after adding all vectors
        after_add = get_memory_info()
        
        memory_used = after_add['rss'] - before_add['rss']
        results[mode_name] = memory_used
        
        print(f"  Memory used: {memory_used:.1f} MB")
        print(f"  Per vector: {(memory_used * 1024 * 1024) / num_vectors:.1f} bytes")
        
        # Clean up
        del db
        gc.collect()
    
    # Component breakdown
    print("\n" + "="*60)
    print("3. Component Memory Breakdown (100K vectors)")
    print("-"*40)
    
    # Theoretical sizes
    vector_float32 = (num_vectors * dimension * 4) / (1024*1024)
    vector_int8 = (num_vectors * dimension * 1) / (1024*1024)
    vector_binary = (num_vectors * dimension / 8) / (1024*1024)
    
    # Graph structure (R=48, 4 bytes per edge)
    graph_edges = (num_vectors * 48 * 4) / (1024*1024)
    
    # Metadata (estimated)
    metadata_per_vector = 100  # bytes (ID, timestamps, etc.)
    metadata_total = (num_vectors * metadata_per_vector) / (1024*1024)
    
    print(f"\nTheoretical sizes:")
    print(f"  Vectors (Float32):     {vector_float32:8.2f} MB")
    print(f"  Vectors (Int8):        {vector_int8:8.2f} MB")
    print(f"  Vectors (Binary):      {vector_binary:8.2f} MB")
    print(f"  Graph (R=48):          {graph_edges:8.2f} MB")
    print(f"  Metadata (est):        {metadata_total:8.2f} MB")
    
    print(f"\nActual measured:")
    for mode_name, memory in results.items():
        print(f"  {mode_name:20s}: {memory:8.2f} MB")
    
    print(f"\nOverhead analysis:")
    if "Normal (Float32)" in results:
        base = results["Normal (Float32)"]
        expected = vector_float32 + graph_edges
        overhead = base - expected
        print(f"  Expected (vec+graph):  {expected:8.2f} MB")
        print(f"  Actual:                {base:8.2f} MB")
        print(f"  Overhead:              {overhead:8.2f} MB ({overhead/base*100:.1f}%)")
    
    # Summary
    print("\n" + "="*60)
    print("OPTIMIZATION OPPORTUNITIES")
    print("="*60)
    
    print("\n1. Replace string IDs with integers")
    print(f"   Potential savings: ~{string_id_mem:.1f} MB per 100K vectors")
    
    print("\n2. Optimize metadata storage")
    print(f"   Current overhead: ~{metadata_total:.1f} MB per 100K vectors")
    
    print("\n3. Graph structure optimization")
    print(f"   Current size: ~{graph_edges:.1f} MB (could use compression)")
    
    print("\n4. Python dict → Mojo structures")
    print("   Eliminate Python object overhead")

if __name__ == "__main__":
    profile_component_memory()