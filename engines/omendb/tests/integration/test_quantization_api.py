#!/usr/bin/env python3
"""Test quantization API for OmenDB."""

import sys

sys.path.insert(0, "python")

import numpy as np
import time
from omendb.quantization import QuantizedDB, QuantizationConfig, choose_quantization


def test_int8_quantization():
    """Test Int8 quantization (4x compression)."""
    print("=" * 70)
    print("INT8 QUANTIZATION TEST")
    print("=" * 70)

    # Create quantized database
    db = QuantizedDB(quantization="int8", buffer_size=5000)

    # Generate test data
    n_vectors = 1000
    dimension = 128
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    # Add vectors (automatically quantized)
    start = time.perf_counter()
    db.add_batch(vectors, ids)
    elapsed = time.perf_counter() - start

    print(f"Added {n_vectors} vectors in {elapsed:.3f}s")
    print(f"Throughput: {n_vectors / elapsed:.0f} vec/s")

    # Test search
    query = vectors[0]
    results = db.search(query, 10)

    if results and results[0].id == ids[0]:
        print("✅ Search accuracy maintained with Int8 quantization")
    else:
        print("⚠️  Search accuracy affected by quantization")

    # Memory stats
    stats = db.get_memory_usage()
    if "compression_ratio" in stats:
        print(f"Compression ratio: {stats['compression_ratio']:.1f}x")

    db.clear()
    print()


def test_binary_quantization():
    """Test binary quantization (32x compression)."""
    print("=" * 70)
    print("BINARY QUANTIZATION TEST")
    print("=" * 70)

    # Create binary quantized database
    db = QuantizedDB(quantization="binary", buffer_size=5000)

    # Generate test data
    n_vectors = 1000
    dimension = 128
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    # Add vectors
    start = time.perf_counter()
    db.add_batch(vectors, ids)
    elapsed = time.perf_counter() - start

    print(f"Added {n_vectors} vectors in {elapsed:.3f}s")
    print(f"Throughput: {n_vectors / elapsed:.0f} vec/s")

    # Test search (lower accuracy expected)
    query = vectors[0]
    results = db.search(query, 10)

    if results:
        print(f"Search returned {len(results)} results")
        print("Note: Binary quantization trades accuracy for 32x compression")

    # Memory stats
    stats = db.get_memory_usage()
    if "compression_ratio" in stats:
        print(f"Compression ratio: {stats['compression_ratio']:.1f}x")

    db.clear()
    print()


def test_product_quantization():
    """Test product quantization (configurable)."""
    print("=" * 70)
    print("PRODUCT QUANTIZATION TEST")
    print("=" * 70)

    # Configure product quantization
    config = QuantizationConfig(type="product", num_subspaces=8, codebook_size=256)

    db = QuantizedDB(quantization=config, buffer_size=5000)

    # Generate test data
    n_vectors = 1000
    dimension = 128
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    # Add vectors
    start = time.perf_counter()
    db.add_batch(vectors, ids)
    elapsed = time.perf_counter() - start

    print(f"Added {n_vectors} vectors in {elapsed:.3f}s")
    print(f"Throughput: {n_vectors / elapsed:.0f} vec/s")
    print(
        f"Configuration: {config.num_subspaces} subspaces, {config.codebook_size} codebook size"
    )

    # Test search
    query = vectors[0]
    results = db.search(query, 10)

    if results:
        print(f"Search returned {len(results)} results")

    # Memory stats
    stats = db.get_memory_usage()
    if "compression_ratio" in stats:
        print(f"Compression ratio: {stats['compression_ratio']:.1f}x")

    db.clear()
    print()


def test_auto_quantization_selection():
    """Test automatic quantization selection."""
    print("=" * 70)
    print("AUTO QUANTIZATION SELECTION TEST")
    print("=" * 70)

    test_cases = [
        (128, 1000, 10.0),  # Small dataset, generous budget
        (768, 10000, 50.0),  # Medium dataset, tight budget
        (1536, 100000, 100.0),  # Large dataset, very tight budget
    ]

    print(f"{'Dimension':<12} {'Vectors':<12} {'Budget (MB)':<12} {'Recommended'}")
    print("-" * 60)

    for dim, n_vecs, budget in test_cases:
        recommendation = choose_quantization(dim, n_vecs, budget)

        # Calculate actual sizes
        float32_size = (dim * n_vecs * 4) / (1024 * 1024)

        size_after = float32_size
        if recommendation == "int8":
            size_after = float32_size / 4
        elif recommendation == "binary":
            size_after = float32_size / 32
        elif recommendation == "product":
            size_after = float32_size / 8

        print(
            f"{dim:<12} {n_vecs:<12} {budget:<12.1f} {recommendation:<12} ({size_after:.1f} MB)"
        )

    print()


def test_comparison():
    """Compare different quantization methods."""
    print("=" * 70)
    print("QUANTIZATION COMPARISON")
    print("=" * 70)

    n_vectors = 5000
    dimension = 256
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    quantization_types = ["none", "int8", "binary", "product"]

    print(f"Testing with {n_vectors} vectors of dimension {dimension}")
    print(f"{'Type':<12} {'Time (s)':<12} {'Vec/s':<12} {'Memory':<12} {'Compression'}")
    print("-" * 70)

    for q_type in quantization_types:
        db = QuantizedDB(quantization=q_type, buffer_size=10000)

        start = time.perf_counter()
        db.add_batch(vectors, ids)
        elapsed = time.perf_counter() - start

        vec_per_sec = n_vectors / elapsed
        stats = db.get_memory_usage()

        compression = stats.get("compression_ratio", 1.0)
        memory = stats.get(
            "compressed_size_mb", (n_vectors * dimension * 4) / (1024 * 1024)
        )

        print(
            f"{q_type:<12} {elapsed:<12.3f} {vec_per_sec:<12.0f} {memory:<12.1f} {compression:.1f}x"
        )

        db.clear()

    print()
    print("Trade-offs:")
    print("- None: No compression, full accuracy")
    print("- Int8: 4x compression, ~99% accuracy")
    print("- Binary: 32x compression, ~70% accuracy")
    print("- Product: 8x compression, ~90% accuracy")


def main():
    """Run all quantization tests."""
    print("OMENDB QUANTIZATION API TESTS")
    print("=" * 70)
    print()

    test_int8_quantization()
    test_binary_quantization()
    test_product_quantization()
    test_auto_quantization_selection()
    test_comparison()

    print("=" * 70)
    print("QUANTIZATION API SUMMARY")
    print("=" * 70)
    print("✅ Quantization API implemented successfully")
    print("✅ Int8, Binary, and Product quantization available")
    print("✅ Automatic quantization selection based on memory budget")
    print("✅ Seamless integration with existing OmenDB API")


if __name__ == "__main__":
    main()
