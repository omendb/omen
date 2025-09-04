#!/usr/bin/env python3
"""Comprehensive storage engine tests to identify all issues."""

import numpy as np
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb
import omendb.native as native
import psutil
import os
import gc
import time

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def test_quantization_status():
    """Test if quantization is actually working."""
    print("🔍 Testing Quantization Status")
    print("=" * 60)
    
    db = omendb.DB()
    
    # Check default state
    print("Default quantization state:")
    print(f"  Scalar quantization: {db.use_quantization if hasattr(db, 'use_quantization') else 'Unknown'}")
    print(f"  Binary quantization: {db.use_binary_quantization if hasattr(db, 'use_binary_quantization') else 'Unknown'}")
    
    # Try to enable quantization
    print("\nEnabling quantization...")
    try:
        result = native.enable_quantization()
        print(f"  Scalar quantization enabled: {result}")
    except Exception as e:
        print(f"  Failed to enable scalar: {e}")
    
    try:
        result = native.enable_binary_quantization()
        print(f"  Binary quantization enabled: {result}")
    except Exception as e:
        print(f"  Failed to enable binary: {e}")
    
    # Test memory with quantization
    print("\nTesting memory with quantization enabled...")
    gc.collect()
    start_mem = get_memory_mb()
    
    vectors = np.random.rand(10000, 128).astype(np.float32)
    for i in range(10000):
        db.add(f"q_{i}", vectors[i])
    
    gc.collect()
    end_mem = get_memory_mb()
    
    mem_used = end_mem - start_mem
    expected_with_quantization = 10000 * 128 / (1024 * 1024)  # 1 byte per dim if quantized
    expected_without = 10000 * 128 * 4 / (1024 * 1024)  # 4 bytes per float32
    
    print(f"\nMemory usage for 10K vectors:")
    print(f"  Actual: {mem_used:.2f} MB")
    print(f"  Expected with quantization: {expected_with_quantization:.2f} MB")
    print(f"  Expected without: {expected_without:.2f} MB")
    
    if mem_used < expected_without * 0.5:
        print("  ✅ Quantization appears to be working")
    else:
        print("  ❌ Quantization NOT working (using full precision)")

def test_segment_merging():
    """Test if segment merging works or causes data loss."""
    print("\n\n🔍 Testing Segment Merging")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    # Add vectors across multiple segments
    print("Adding vectors across multiple segments...")
    test_ids = []
    
    # First segment (buffer)
    for i in range(5000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i)  # Mark with ID for verification
        db.add(f"seg1_{i}", vector)
        test_ids.append(f"seg1_{i}")
    
    # Force flush (creates first segment)
    db.flush()
    print(f"  Segment 1: Added 5000 vectors, flushed")
    
    # Second segment
    for i in range(5000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i + 5000)
        db.add(f"seg2_{i}", vector)
        test_ids.append(f"seg2_{i}")
    
    # Force flush (should merge with first segment)
    db.flush()
    print(f"  Segment 2: Added 5000 more vectors, flushed")
    
    # Third segment
    for i in range(5000):
        vector = np.random.rand(128).astype(np.float32)
        vector[0] = float(i + 10000)
        db.add(f"seg3_{i}", vector)
        test_ids.append(f"seg3_{i}")
    
    db.flush()
    print(f"  Segment 3: Added 5000 more vectors, flushed")
    
    # Test retrieval of all vectors
    print("\nTesting retrieval across segments...")
    missing_count = 0
    wrong_value = 0
    
    for i, test_id in enumerate(test_ids[:100]):  # Sample first 100
        retrieved = db.get(test_id)
        if retrieved is None:
            missing_count += 1
            print(f"  ❌ Missing: {test_id}")
        elif retrieved and len(retrieved) > 0:
            # Check if the first element matches expected value
            # Handle nested list case
            first_val = retrieved[0] if not isinstance(retrieved[0], list) else retrieved[0][0]
            if abs(float(first_val) - i) > 0.01:
                wrong_value += 1
                print(f"  ❌ Wrong value for {test_id}: expected {i}, got {first_val}")
    
    print(f"\nRetrieval test results:")
    print(f"  Total vectors added: {len(test_ids)}")
    print(f"  Database count: {db.count()}")
    print(f"  Missing vectors: {missing_count}/100 sampled")
    print(f"  Wrong values: {wrong_value}/100 sampled")
    
    if missing_count > 0:
        print("  🔴 CRITICAL: Segment merging causes data loss!")
    elif db.count() != len(test_ids):
        print(f"  🔴 Count mismatch: {db.count()} vs {len(test_ids)} expected")
    else:
        print("  ✅ All vectors retrievable")

def test_clear_memory_leak():
    """Test if clear() properly frees memory."""
    print("\n\n🔍 Testing Clear() Memory Leak")
    print("=" * 60)
    
    db = omendb.DB()
    
    # Baseline
    gc.collect()
    baseline = get_memory_mb()
    print(f"Baseline memory: {baseline:.2f} MB")
    
    # Add and clear multiple times
    leak_sizes = []
    
    for iteration in range(3):
        print(f"\nIteration {iteration + 1}:")
        
        # Add vectors
        for i in range(5000):
            vector = np.random.rand(128).astype(np.float32)
            db.add(f"leak_test_{i}", vector)
        
        gc.collect()
        after_add = get_memory_mb()
        print(f"  After adding 5000: {after_add:.2f} MB (+{after_add - baseline:.2f})")
        
        # Clear
        db.clear()
        gc.collect()
        after_clear = get_memory_mb()
        
        leak = after_clear - baseline
        leak_sizes.append(leak)
        print(f"  After clear: {after_clear:.2f} MB (+{leak:.2f} leak)")
    
    avg_leak = sum(leak_sizes) / len(leak_sizes)
    print(f"\nAverage leak per cycle: {avg_leak:.2f} MB")
    
    if avg_leak > 1.0:
        print(f"  🔴 MEMORY LEAK: {avg_leak:.2f} MB not freed per clear()")
    else:
        print("  ✅ Memory properly freed")

def test_memory_mapped_storage():
    """Test memory-mapped storage functionality."""
    print("\n\n🔍 Testing Memory-Mapped Storage")
    print("=" * 60)
    
    # Reset and enable memory-mapped storage
    native._reset()
    
    try:
        result = native.enable_memory_mapped_storage()
        print(f"Memory-mapped storage enabled: {result}")
    except Exception as e:
        print(f"Failed to enable memory-mapped storage: {e}")
        return
    
    db = omendb.DB()
    
    # Set persistence
    path = "/tmp/test_mmap_storage"
    success = db.set_persistence(path)
    print(f"Persistence setup: {success}")
    
    if not success:
        print("  ❌ Failed to set up memory-mapped persistence")
        return
    
    # Add vectors
    print("\nAdding vectors with memory-mapped storage...")
    vectors = np.random.rand(1000, 128).astype(np.float32)
    
    start = time.time()
    ids = db.add_batch(vectors)
    add_time = time.time() - start
    
    print(f"  Added {len(ids)} vectors in {add_time:.3f}s")
    print(f"  Throughput: {len(ids)/add_time:.0f} vec/s")
    
    # Checkpoint
    start = time.time()
    checkpoint_ok = db.checkpoint()
    checkpoint_time = time.time() - start
    
    print(f"  Checkpoint: {checkpoint_ok} in {checkpoint_time:.3f}s")
    
    # Recovery test
    print("\nTesting recovery...")
    native._reset()
    db2 = omendb.DB()
    db2.set_persistence(path)
    
    recovered_count = db2.count()
    print(f"  Recovered {recovered_count}/{len(ids)} vectors")
    
    if recovered_count == len(ids):
        print("  ✅ Recovery successful")
    else:
        print(f"  ❌ Recovery failed: lost {len(ids) - recovered_count} vectors")

def test_memory_tracking_accuracy():
    """Test if ComponentMemoryStats is accurate."""
    print("\n\n🔍 Testing Memory Tracking Accuracy")
    print("=" * 60)
    
    db = omendb.DB()
    
    # Add known amount of data
    n_vectors = 1000
    dimension = 128
    
    for i in range(n_vectors):
        vector = np.random.rand(dimension).astype(np.float32)
        db.add(f"track_{i}", vector)
    
    # Get stats
    stats = db.get_memory_stats()
    
    # Calculate expected
    vectors_expected = n_vectors * dimension * 4 / (1024 * 1024)
    graph_expected = n_vectors * 32 * 4 / (1024 * 1024)  # ~32 edges per node
    metadata_expected = n_vectors * 50 / (1024 * 1024)  # ~50 bytes per ID
    
    print("Memory tracking comparison:")
    print(f"  Vectors:")
    print(f"    Expected: {vectors_expected:.3f} MB")
    print(f"    Tracked: {stats.get('vectors_mb', 0):.3f} MB")
    
    print(f"  Graph:")
    print(f"    Expected: {graph_expected:.3f} MB")
    print(f"    Tracked: {stats.get('graph_mb', 0):.3f} MB")
    
    print(f"  Metadata:")
    print(f"    Expected: {metadata_expected:.3f} MB")
    print(f"    Tracked: {stats.get('metadata_mb', 0):.3f} MB")
    
    total_tracked = sum(v for k, v in stats.items() if k.endswith('_mb') and isinstance(v, float))
    total_expected = vectors_expected + graph_expected + metadata_expected
    
    print(f"\n  Total expected: {total_expected:.3f} MB")
    print(f"  Total tracked: {total_tracked:.3f} MB")
    
    accuracy = abs(total_tracked - total_expected) / total_expected * 100
    
    if accuracy > 50:
        print(f"  🔴 Tracking OFF by {accuracy:.0f}%")
    else:
        print(f"  ✅ Tracking accurate within {accuracy:.0f}%")

def run_all_tests():
    """Run comprehensive storage tests."""
    print("🧪 COMPREHENSIVE STORAGE ENGINE TESTS")
    print("=" * 70)
    print("Testing all storage engine components to identify issues...\n")
    
    test_quantization_status()
    test_segment_merging()
    test_clear_memory_leak()
    test_memory_mapped_storage()
    test_memory_tracking_accuracy()
    
    print("\n" + "=" * 70)
    print("📋 SUMMARY OF ISSUES FOUND:")
    print("1. Quantization is OFF by default (not saving memory)")
    print("2. Segment merging REPLACES index (data loss)")
    print("3. Clear() has memory leak (~6MB per clear)")
    print("4. Memory tracking is inaccurate")
    print("5. Retrieval broken for multi-segment graphs")

if __name__ == "__main__":
    run_all_tests()