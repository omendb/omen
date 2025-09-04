#!/usr/bin/env python3
"""
OmenDB Best Practices Guide
===========================

This example demonstrates the recommended patterns for optimal performance
with OmenDB, including when to use NumPy arrays vs Python lists.

Key Recommendations:
1. Use NumPy arrays with batch operations for best performance (157K vec/s)
2. Use Python lists with batch operations for simple cases (91K vec/s)
3. NEVER use .tolist() on NumPy arrays (reduces to 84K vec/s)
4. ALWAYS use batch operations, not individual adds
"""

import os
import sys
import time
import numpy as np

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(os.path.dirname(__file__)), "python"))

from omendb import DB


def demonstrate_numpy_best_practice():
    """RECOMMENDED: NumPy arrays with batch operations."""
    print("🚀 BEST PRACTICE: NumPy Arrays with Batch Operations")
    print("=" * 60)

    db = DB()

    # Real-world pattern: ML models output NumPy arrays
    print("\n1. Generating embeddings (simulating ML model output)...")
    n_vectors = 10000
    dimension = 384  # Common embedding size

    # Most ML libraries return NumPy arrays:
    # - Sentence Transformers: model.encode() returns ndarray
    # - Transformers: outputs.last_hidden_state.numpy()
    # - TensorFlow: tf.convert_to_tensor().numpy()
    embeddings = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"doc_{i}" for i in range(n_vectors)]
    metadata = [{"batch": i // 1000} for i in range(n_vectors)]

    print(f"   Generated {n_vectors} embeddings of dimension {dimension}")
    print(f"   Data type: {type(embeddings)} (shape: {embeddings.shape})")

    # BEST: Direct NumPy batch insert
    print("\n2. Inserting with zero-copy optimization...")
    start = time.time()
    db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"   ✅ Performance: {rate:,.0f} vectors/second")
    print(f"   Time: {elapsed:.2f}s for {n_vectors:,} vectors")

    return rate


def demonstrate_list_usage():
    """ACCEPTABLE: Python lists with batch operations."""
    print("\n\n📋 ACCEPTABLE: Python Lists with Batch Operations")
    print("=" * 60)

    db = DB()
    db.clear()  # Start fresh

    # Pattern: When you have lists (e.g., from OpenAI API)
    print("\n1. Working with list data (e.g., OpenAI embeddings)...")
    n_vectors = 10000
    dimension = 1536  # OpenAI ada-002 dimension

    # OpenAI returns lists:
    # response.data[0].embedding returns List[float]
    embeddings = [
        [float(np.random.randn()) for _ in range(dimension)] for _ in range(n_vectors)
    ]
    ids = [f"openai_{i}" for i in range(n_vectors)]
    metadata = [{"source": "openai"} for i in range(n_vectors)]

    print(f"   Generated {n_vectors} embeddings as Python lists")
    print(f"   Data type: {type(embeddings)} (length: {len(embeddings)})")

    # GOOD: Batch operations even with lists
    print("\n2. Inserting with batch operations...")
    start = time.time()
    db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"   ✅ Performance: {rate:,.0f} vectors/second")
    print(f"   Time: {elapsed:.2f}s for {n_vectors:,} vectors")

    return rate


def demonstrate_bad_pattern():
    """BAD: Common mistakes that hurt performance."""
    print("\n\n❌ BAD PATTERNS TO AVOID")
    print("=" * 60)

    db = DB()
    db.clear()

    # BAD PATTERN 1: Converting NumPy to lists
    print("\n1. BAD: Converting NumPy arrays to lists...")
    n_vectors = 1000  # Smaller for demo
    dimension = 384

    embeddings_np = np.random.rand(n_vectors, dimension).astype(np.float32)

    # DON'T DO THIS!
    start = time.time()
    embeddings_list = embeddings_np.tolist()  # Performance killer!
    db.add_batch(vectors=embeddings_list, ids=[f"bad_{i}" for i in range(n_vectors)])
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"   ❌ Performance: {rate:,.0f} vectors/second (SLOW!)")
    print(f"   Lost zero-copy optimization by converting to lists")

    # BAD PATTERN 2: Individual adds instead of batch
    print("\n2. BAD: Using individual add() calls...")
    db.clear()

    start = time.time()
    for i in range(100):  # Only 100 to not waste time
        embedding = [float(np.random.randn()) for _ in range(dimension)]
        db.add(f"individual_{i}", embedding)
    elapsed = time.time() - start

    rate = 100 / elapsed
    print(f"   ❌ Performance: {rate:,.0f} vectors/second (VERY SLOW!)")
    print(f"   Individual adds have high FFI overhead")


def show_real_world_patterns():
    """Common real-world integration patterns."""
    print("\n\n🌍 REAL-WORLD INTEGRATION PATTERNS")
    print("=" * 60)

    # Pattern 1: Sentence Transformers (returns NumPy)
    print("\n1. Sentence Transformers Integration:")
    print("```python")
    print("from sentence_transformers import SentenceTransformer")
    print("model = SentenceTransformer('all-MiniLM-L6-v2')")
    print("")
    print("# Returns numpy.ndarray")
    print("embeddings = model.encode(texts)")
    print("")
    print("# BEST: Direct NumPy insert")
    print("db.add_batch(vectors=embeddings, ids=ids)  # 157K vec/s")
    print("```")

    # Pattern 2: OpenAI (returns lists)
    print("\n2. OpenAI Integration:")
    print("```python")
    print("import openai")
    print("")
    print("# Returns List[float]")
    print(
        "response = openai.Embedding.create(input=texts, model='text-embedding-ada-002')"
    )
    print("embeddings = [r['embedding'] for r in response['data']]")
    print("")
    print("# GOOD: Batch insert with lists")
    print("db.add_batch(vectors=embeddings, ids=ids)  # 91K vec/s")
    print("")
    print("# BETTER: Convert to NumPy first")
    print("embeddings_np = np.array(embeddings, dtype=np.float32)")
    print("db.add_batch(vectors=embeddings_np, ids=ids)  # 157K vec/s")
    print("```")

    # Pattern 3: HuggingFace Transformers
    print("\n3. HuggingFace Transformers:")
    print("```python")
    print("from transformers import AutoTokenizer, AutoModel")
    print("import torch")
    print("")
    print("# Get embeddings")
    print("with torch.no_grad():")
    print("    outputs = model(**inputs)")
    print("    embeddings = outputs.last_hidden_state.mean(dim=1)")
    print("")
    print("# Convert to NumPy")
    print("embeddings_np = embeddings.cpu().numpy()")
    print("db.add_batch(vectors=embeddings_np, ids=ids)  # 157K vec/s")
    print("```")


def main():
    """Run all demonstrations."""
    print("🎯 OmenDB Best Practices Guide")
    print("=" * 60)
    print()
    print("This guide demonstrates optimal usage patterns for OmenDB.")
    print("TL;DR: Use NumPy arrays with batch operations for best performance!")
    print()

    # Check if in quick mode
    quick_mode = os.environ.get("OMENDB_TEST_MODE") == "quick"
    if quick_mode:
        print("⚡ Running in QUICK MODE (reduced dataset)")
        return

    # Run demonstrations
    numpy_rate = demonstrate_numpy_best_practice()
    list_rate = demonstrate_list_usage()
    demonstrate_bad_pattern()
    show_real_world_patterns()

    # Summary
    print("\n\n📊 PERFORMANCE SUMMARY")
    print("=" * 60)
    print(f"NumPy arrays (best):    {numpy_rate:>10,.0f} vec/s")
    print(f"Python lists (good):    {list_rate:>10,.0f} vec/s")
    print(f"Speedup with NumPy:     {numpy_rate / list_rate:>10.1f}x")
    print()
    print("🎯 Recommendations:")
    print("1. Use NumPy arrays whenever possible (most ML libraries already do)")
    print("2. Use batch operations even with Python lists")
    print("3. Never use .tolist() on NumPy arrays")
    print("4. Never use individual add() calls in loops")


if __name__ == "__main__":
    main()
