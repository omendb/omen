#!/usr/bin/env python3
"""Deep dive into buffer flush memory behavior."""

import numpy as np
import omendb
import psutil
import os
import gc

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def analyze_buffer_behavior():
    """Analyze memory behavior around buffer flushes."""
    print("🔬 Buffer Flush Memory Analysis")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    dimension = 128
    
    # Track memory at each stage
    gc.collect()
    base_mem = get_memory_mb()
    print(f"Base memory: {base_mem:.2f} MB")
    
    # Add vectors up to buffer threshold
    print("\n📊 Adding vectors around buffer threshold (10000)")
    
    memory_log = []
    
    for i in range(12000):
        vector = np.random.rand(dimension).astype(np.float32)
        db.add(f"vec_{i}", vector)
        
        # Track around the 10K boundary
        if i in [9990, 9995, 9999, 10000, 10001, 10005, 10010, 10050, 10100, 10500, 11000]:
            gc.collect()
            current_mem = get_memory_mb()
            mem_used = current_mem - base_mem
            
            # Get internal stats
            stats = db.get_memory_stats()
            tracked = sum(v for k, v in stats.items() if k.endswith('_mb') and isinstance(v, float))
            
            memory_log.append({
                'count': i + 1,
                'actual_mb': mem_used,
                'tracked_mb': tracked,
                'vectors_mb': stats.get('vectors_mb', 0),
                'graph_mb': stats.get('graph_mb', 0),
                'buffer_mb': stats.get('buffer_mb', 0),
                'metadata_mb': stats.get('metadata_mb', 0)
            })
            
            print(f"  {i+1:5d} vectors: {mem_used:6.2f} MB actual, {tracked:6.2f} MB tracked")
            if i == 9999:
                print("    >>> BEFORE BUFFER FLUSH <<<")
            elif i == 10001:
                print("    >>> AFTER BUFFER FLUSH <<<")
    
    # Analyze the log
    print("\n📈 Memory Jump Analysis:")
    
    before_flush = next(m for m in memory_log if m['count'] == 10000)
    after_flush = next(m for m in memory_log if m['count'] == 10001)
    
    actual_jump = after_flush['actual_mb'] - before_flush['actual_mb']
    tracked_jump = after_flush['tracked_mb'] - before_flush['tracked_mb']
    
    print(f"  Actual memory jump: {actual_jump:.2f} MB")
    print(f"  Tracked memory jump: {tracked_jump:.2f} MB")
    
    print(f"\n  Component changes during flush:")
    print(f"    Vectors: {before_flush['vectors_mb']:.2f} → {after_flush['vectors_mb']:.2f} MB")
    print(f"    Graph: {before_flush['graph_mb']:.2f} → {after_flush['graph_mb']:.2f} MB")
    print(f"    Buffer: {before_flush['buffer_mb']:.2f} → {after_flush['buffer_mb']:.2f} MB")
    print(f"    Metadata: {before_flush['metadata_mb']:.2f} → {after_flush['metadata_mb']:.2f} MB")
    
    # Check for memory duplication
    print("\n🔍 Checking for memory duplication during flush:")
    
    expected_graph_mem = 10000 * 32 * 4 / (1024 * 1024)  # 32 edges * 4 bytes
    actual_graph_mem = after_flush['graph_mb']
    
    print(f"  Expected graph memory for 10K nodes: {expected_graph_mem:.2f} MB")
    print(f"  Actual graph memory: {actual_graph_mem:.2f} MB")
    
    if actual_graph_mem > expected_graph_mem * 1.5:
        print(f"  ⚠️  GRAPH MEMORY ISSUE: {actual_graph_mem/expected_graph_mem:.1f}x expected!")
    
    # Check vector storage
    expected_vector_mem = 10000 * 128 * 4 / (1024 * 1024)
    actual_vector_mem = after_flush['vectors_mb']
    
    print(f"\n  Expected vector memory: {expected_vector_mem:.2f} MB")
    print(f"  Actual vector memory: {actual_vector_mem:.2f} MB")
    
    if actual_vector_mem > expected_vector_mem * 1.5:
        print(f"  ⚠️  VECTOR STORAGE ISSUE: {actual_vector_mem/expected_vector_mem:.1f}x expected!")
        print(f"  🔴 Vectors are likely stored multiple times!")

def test_clear_behavior():
    """Test if clear() actually frees memory."""
    print("\n\n🧹 Testing Clear() Memory Behavior")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    gc.collect()
    base_mem = get_memory_mb()
    print(f"Base memory: {base_mem:.2f} MB")
    
    # Add vectors
    print("\nAdding 10000 vectors...")
    for i in range(10000):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"vec_{i}", vector)
    
    gc.collect()
    after_add = get_memory_mb()
    print(f"After adding: {after_add:.2f} MB (+{after_add - base_mem:.2f} MB)")
    
    # Clear database
    print("\nClearing database...")
    db.clear()
    
    gc.collect()
    after_clear = get_memory_mb()
    print(f"After clear: {after_clear:.2f} MB (+{after_clear - base_mem:.2f} MB)")
    
    mem_freed = after_add - after_clear
    print(f"\nMemory freed: {mem_freed:.2f} MB")
    
    if after_clear - base_mem > 1.0:  # More than 1MB still held
        print(f"⚠️  MEMORY LEAK: {after_clear - base_mem:.2f} MB not freed after clear()!")
    else:
        print("✅ Memory properly freed")

def check_vector_storage_locations():
    """Check where vectors are being stored."""
    print("\n\n🗺️ Vector Storage Location Analysis")
    print("=" * 60)
    
    # This would require access to internal structures
    print("Checking storage in:")
    print("  1. VectorStore.vector_store (original vectors)")
    print("  2. VectorStore.quantized_vectors (if quantization enabled)")
    print("  3. VectorStore.binary_vectors (if binary quantization)")
    print("  4. VectorBuffer (temporary buffer)")
    print("  5. DiskANNIndex internal storage")
    print("  6. VamanaGraph vectors")
    
    # We can test indirectly
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    # Add one vector and check memory
    gc.collect()
    base = get_memory_mb()
    
    vector = np.random.rand(128).astype(np.float32)
    db.add("test", vector)
    
    gc.collect()
    after_one = get_memory_mb()
    
    # Memory for one 128D float32 vector
    expected = 128 * 4 / (1024 * 1024)  # 0.0005 MB
    actual = after_one - base
    
    print(f"\nSingle vector storage test:")
    print(f"  Expected: {expected:.4f} MB")
    print(f"  Actual: {actual:.2f} MB")
    print(f"  Overhead: {actual/expected:.0f}x")
    
    if actual > 0.5:  # More than 0.5MB for one vector
        print(f"  🔴 MASSIVE OVERHEAD: Single vector using {actual*1024:.0f} KB!")

if __name__ == "__main__":
    analyze_buffer_behavior()
    test_clear_behavior()
    check_vector_storage_locations()
    
    print("\n✅ Buffer memory analysis complete!")