#!/usr/bin/env python3
"""Profile memory usage by component with quantization."""

import numpy as np
import sys
import gc
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def profile_with_quantization():
    """Profile memory with different quantization modes."""
    
    print("Component Memory Profiling (100K vectors)")
    print("=" * 60)
    
    vector_count = 100000
    dimension = 128
    
    # Generate test data once
    print("Generating test data...")
    vectors = np.random.rand(vector_count, dimension).astype(np.float32)
    
    modes = [
        ("Normal (Float32)", None, None),
        ("Scalar Quantization (Int8)", "scalar", None),
        ("Binary Quantization (1-bit)", "binary", None),
    ]
    
    for mode_name, quant_type, _ in modes:
        print(f"\n{mode_name}:")
        print("-" * 40)
        
        # Create DB
        db = omendb.DB()
        
        # Enable quantization if needed
        if quant_type == "scalar":
            db.enable_quantization()
            print("  Scalar quantization enabled")
        elif quant_type == "binary":
            db.enable_binary_quantization()
            print("  Binary quantization enabled")
        
        # Add vectors in batches
        batch_size = 10000
        for i in range(0, vector_count, batch_size):
            batch = vectors[i:i+batch_size]
            ids = [f"vec_{j}" for j in range(i, i+len(batch))]
            db.add_batch(batch, ids=ids)
            if (i + batch_size) % 50000 == 0:
                print(f"  Added {i+batch_size:,} vectors...")
        
        # Get memory stats
        try:
            stats = db.get_memory_stats()
            print(f"\n  Component breakdown:")
            # Convert keys for display
            display_stats = {
                "vectors": stats.get("vectors_mb", 0),
                "graph": stats.get("graph_mb", 0),
                "metadata": stats.get("metadata_mb", 0),
                "buffer": stats.get("buffer_mb", 0),
                "index": stats.get("index_mb", 0),
                "total": stats.get("total_mb", 0)
            }
            
            for key, value_mb in display_stats.items():
                if key != "total" and value_mb > 0:
                    print(f"    {key:20s}: {value_mb:8.2f} MB")
            
            total = display_stats["total"]
            print(f"    {'TOTAL':20s}: {total:8.2f} MB")
            print(f"    Per vector: {(total * 1024) / vector_count:.2f} KB" if vector_count > 0 else "")
        except Exception as e:
            print(f"  Could not get memory stats: {e}")
        
        # Clean up
        del db
        gc.collect()
    
    # Calculate theoretical minimums
    print("\n" + "=" * 60)
    print("THEORETICAL MINIMUM MEMORY")
    print("=" * 60)
    
    print(f"\nFor {vector_count:,} vectors @ {dimension} dimensions:")
    
    # Vectors
    float32_size = vector_count * dimension * 4 / (1024 * 1024)
    int8_size = vector_count * dimension * 1 / (1024 * 1024)
    binary_size = vector_count * dimension / 8 / (1024 * 1024)
    
    print(f"\nVector storage:")
    print(f"  Float32: {float32_size:.2f} MB")
    print(f"  Int8:    {int8_size:.2f} MB")
    print(f"  Binary:  {binary_size:.2f} MB")
    
    # Graph (R=48)
    R = 48
    graph_size = vector_count * R * 4 / (1024 * 1024)  # 4 bytes per index
    print(f"\nGraph structure (R={R}):")
    print(f"  Edges:   {graph_size:.2f} MB")
    
    # IDs (string vs int)
    string_id_size = vector_count * 50 / (1024 * 1024)  # ~50 bytes per string
    int_id_size = vector_count * 8 / (1024 * 1024)  # 8 bytes per int64
    
    print(f"\nID storage:")
    print(f"  String IDs: {string_id_size:.2f} MB")
    print(f"  Int IDs:    {int_id_size:.2f} MB")
    
    # Metadata (Dict overhead)
    metadata_size = vector_count * 100 / (1024 * 1024)  # ~100 bytes overhead
    print(f"\nMetadata overhead:")
    print(f"  Dict structures: {metadata_size:.2f} MB")
    
    # Total theoretical
    print(f"\nTheoretical totals:")
    print(f"  Float32 + graph: {float32_size + graph_size:.2f} MB")
    print(f"  Int8 + graph:    {int8_size + graph_size:.2f} MB")
    print(f"  Binary + graph:  {binary_size + graph_size:.2f} MB")

def analyze_graph_structure():
    """Analyze the actual graph structure in detail."""
    
    print("\n" + "=" * 60)
    print("GRAPH STRUCTURE ANALYSIS")
    print("=" * 60)
    
    # Create a small graph to analyze
    db = omendb.DB()
    
    # Add exactly 1000 vectors
    vectors = np.random.rand(1000, 128).astype(np.float32)
    for i in range(1000):
        db.add(f"vec_{i}", vectors[i])
    
    # Force flush to build graph
    if hasattr(db, 'flush'):
        db.flush()
    
    # Get stats
    stats = db.get_stats()
    
    print(f"\nGraph statistics (1000 vectors):")
    print(f"  Total vectors: {stats.get('total_vectors', 0)}")
    print(f"  Buffer size: {stats.get('buffer_size', 0)}")
    print(f"  Index algorithm: {stats.get('index_algorithm', 'unknown')}")
    
    # Theoretical graph size
    R = 48  # From DiskANN
    avg_degree = R // 2  # Assume average degree is half of max
    
    edges_forward = 1000 * avg_degree
    edges_reverse = edges_forward  # Bidirectional
    total_edges = edges_forward + edges_reverse
    
    print(f"\nEdge analysis:")
    print(f"  Max degree (R): {R}")
    print(f"  Assumed avg degree: {avg_degree}")
    print(f"  Forward edges: {edges_forward:,}")
    print(f"  Reverse edges: {edges_reverse:,}")
    print(f"  Total edges: {total_edges:,}")
    print(f"  Memory (4 bytes/edge): {total_edges * 4 / 1024:.2f} KB")
    
    del db
    gc.collect()

if __name__ == "__main__":
    profile_with_quantization()
    analyze_graph_structure()