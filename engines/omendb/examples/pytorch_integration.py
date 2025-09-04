#!/usr/bin/env python3
"""
PyTorch Integration Example
==========================

Demonstrates seamless integration between OmenDB and PyTorch for ML workflows.
Shows how to store and retrieve embeddings from popular models.
"""

import numpy as np
import time
from omendb import DB

# Check for PyTorch
try:
    import torch
    import torch.nn as nn

    TORCH_AVAILABLE = True
    print("✅ PyTorch available")
except ImportError:
    TORCH_AVAILABLE = False
    print("⚠️ PyTorch not available, using NumPy for demonstration")
    print("   Install with: pip install torch")

    # Mock torch with numpy
    class TorchMock:
        float32 = np.float32

        @staticmethod
        def randn(*shape, dtype=None, device=None):
            return np.random.randn(*shape).astype(dtype or np.float32)

        @staticmethod
        def tensor(data, dtype=None):
            return np.array(data, dtype=dtype or np.float32)

        class cuda:
            @staticmethod
            def is_available():
                return False

        class nn:
            class Module:
                pass

            class functional:
                @staticmethod
                def normalize(x, p=2, dim=1):
                    norm = np.linalg.norm(x, ord=p, axis=dim, keepdims=True)
                    return x / np.maximum(norm, 1e-12)

    torch = TorchMock()

# Optional: Use a pre-trained model for embeddings
# pip install transformers
try:
    from transformers import AutoModel, AutoTokenizer

    TRANSFORMERS_AVAILABLE = True
except ImportError:
    TRANSFORMERS_AVAILABLE = False
    print(
        "Note: Install transformers for real embedding examples: pip install transformers"
    )


def create_simple_embeddings(texts, dim=384):
    """Create simple random embeddings for demo purposes."""
    if TORCH_AVAILABLE:
        return torch.randn(len(texts), dim, dtype=torch.float32)
    else:
        return np.random.randn(len(texts), dim).astype(np.float32)


def benchmark_pytorch_integration():
    """Benchmark PyTorch tensor storage and retrieval."""
    print("🚀 PyTorch Integration Benchmark")
    print("=" * 50)

    # Create database
    db = DB("pytorch_vectors.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Test different tensor sizes with fixed dimension
    dimension = 128  # Use consistent dimension for OmenDB
    batch_sizes = [100, 1000, 10000]

    print(f"\n📊 Testing {dimension}D embeddings")
    print("-" * 40)

    for batch_size in batch_sizes:
        # Create PyTorch tensors
        embeddings = torch.randn(batch_size, dimension, dtype=torch.float32)
        ids = [f"doc_{i}" for i in range(batch_size)]

        # Time the insertion
        start = time.perf_counter()

        # ALWAYS use batch operations for better performance
        # Convert PyTorch tensor to NumPy for optimal performance
        if TORCH_AVAILABLE and hasattr(embeddings, "numpy"):
            # ✅ BEST: Convert PyTorch to NumPy for zero-copy optimization
            vectors_np = embeddings.numpy()  # PyTorch to NumPy
            db.add_batch(vectors=vectors_np, ids=ids)
        else:
            # Fallback for mock torch (already NumPy)
            db.add_batch(vectors=embeddings, ids=ids)

        elapsed = time.perf_counter() - start
        rate = batch_size / elapsed

        print(
            f"Batch size: {batch_size:5d} | Rate: {rate:8.1f} vec/s | Time: {elapsed:.3f}s"
        )

        # Test query with PyTorch tensor
        query_tensor = torch.randn(dimension, dtype=torch.float32)

        start = time.perf_counter()
        results = db.search(query_tensor, limit=10)
        query_time = (time.perf_counter() - start) * 1000

        print(f"Query time: {query_time:.2f}ms for top-10 results")


def semantic_search_example():
    """Example: Semantic search with sentence embeddings."""
    print("\n\n🔍 Semantic Search Example")
    print("=" * 50)

    # Sample documents
    documents = [
        "The quick brown fox jumps over the lazy dog",
        "Machine learning is transforming artificial intelligence",
        "Python is a versatile programming language",
        "Neural networks can learn complex patterns",
        "Vector databases enable efficient similarity search",
        "Deep learning requires large amounts of data",
        "Natural language processing helps computers understand text",
        "Computer vision enables machines to see and understand images",
        "Reinforcement learning trains agents through rewards",
        "Transfer learning reuses knowledge from pre-trained models",
    ]

    # Create database
    db = DB("semantic_search.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    if TRANSFORMERS_AVAILABLE:
        print("Using real sentence embeddings from sentence-transformers...")
        # Use a small, fast model
        from transformers import AutoModel, AutoTokenizer

        model_name = "sentence-transformers/all-MiniLM-L6-v2"
        tokenizer = AutoTokenizer.from_pretrained(model_name)
        model = AutoModel.from_pretrained(model_name)
        model.eval()

        def encode_texts(texts):
            """Encode texts to embeddings using transformer model."""
            with torch.no_grad():
                inputs = tokenizer(
                    texts, padding=True, truncation=True, return_tensors="pt"
                )
                outputs = model(**inputs)
                # Mean pooling
                embeddings = outputs.last_hidden_state.mean(dim=1)
                # Normalize
                embeddings = torch.nn.functional.normalize(embeddings, p=2, dim=1)
            return embeddings

        # Encode documents
        print("Encoding documents...")
        doc_embeddings = encode_texts(documents)

    else:
        print(
            "Using random embeddings for demo (install transformers for real embeddings)"
        )
        doc_embeddings = create_simple_embeddings(documents, dim=384)

    # Store in database with metadata
    print(f"Storing {len(documents)} documents...")
    for i, (doc, embedding) in enumerate(zip(documents, doc_embeddings)):
        metadata = {"text": doc, "doc_id": f"doc_{i}", "length": str(len(doc))}
        db.add(f"doc_{i}", embedding, metadata)

    # Perform semantic search
    queries = [
        "How do machines learn?",
        "Programming languages for AI",
        "Animal movements",
    ]

    print("\n🔍 Semantic Search Results:")
    for query in queries:
        print(f"\nQuery: '{query}'")

        if TRANSFORMERS_AVAILABLE:
            query_embedding = encode_texts([query])[0]
        else:
            query_embedding = create_simple_embeddings([query], dim=384)[0]

        # Search
        results = db.search(query_embedding, limit=3)

        for i, result in enumerate(results):
            # Access result attributes
            doc_text = result.metadata.get("text", "N/A") if result.metadata else "N/A"
            print(f"  {i + 1}. (score: {result.score:.3f}) {doc_text}")


def model_embedding_cache():
    """Example: Cache embeddings from a vision model."""
    print("\n\n🖼️ Vision Model Embedding Cache")
    print("=" * 50)

    # Simulate image embeddings (in practice, use ResNet, CLIP, etc.)
    num_images = 1000
    embedding_dim = 2048  # ResNet-style dimensions

    print(f"Creating embedding cache for {num_images} images...")

    # Create database for embeddings
    db = DB("image_embeddings.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Simulate batch processing of images
    batch_size = 100
    total_time = 0

    for batch_start in range(0, num_images, batch_size):
        batch_end = min(batch_start + batch_size, num_images)
        batch_count = batch_end - batch_start

        # Simulate model inference (in practice, use actual model)
        image_embeddings = torch.randn(batch_count, embedding_dim, dtype=torch.float32)

        # Create batch data
        batch_ids = [f"img_{batch_start + i}" for i in range(batch_count)]
        batch_metadata = [
            {
                "path": f"/images/img_{batch_start + i}.jpg",
                "size": "224x224",
                "model": "resnet50",
            }
            for i in range(batch_count)
        ]

        # Store embeddings
        start = time.perf_counter()
        # ✅ CORRECT: Convert PyTorch tensor to NumPy for zero-copy
        if TORCH_AVAILABLE and hasattr(image_embeddings, "numpy"):
            vectors_np = image_embeddings.numpy()
            db.add_batch(vectors=vectors_np, ids=batch_ids, metadata=batch_metadata)
        else:
            db.add_batch(
                vectors=image_embeddings, ids=batch_ids, metadata=batch_metadata
            )
        elapsed = time.perf_counter() - start
        total_time += elapsed

        if batch_start == 0:
            print(f"First batch ({batch_count} images): {elapsed:.3f}s")

    print(f"Total time to cache {num_images} embeddings: {total_time:.3f}s")
    print(f"Average rate: {num_images / total_time:.1f} embeddings/second")

    # Demonstrate fast retrieval
    print("\n🔍 Fast similarity search on cached embeddings:")
    query_embedding = torch.randn(embedding_dim, dtype=torch.float32)

    start = time.perf_counter()
    similar_images = db.search(query_embedding, limit=10)
    query_time = (time.perf_counter() - start) * 1000

    print(f"Found 10 most similar images in {query_time:.2f}ms")
    for result in similar_images[:5]:
        path = result.metadata.get("path", "N/A") if result.metadata else "N/A"
        print(f"  - {result.id}: {path} (score: {result.score:.3f})")


def gpu_tensor_example():
    """Example: Working with GPU tensors."""
    if not torch.cuda.is_available():
        print("\n\n⚠️  GPU not available, skipping GPU tensor example")
        return

    print("\n\n🎮 GPU Tensor Integration")
    print("=" * 50)

    db = DB("gpu_vectors.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Create GPU tensors
    num_vectors = 10000
    dim = 512

    print(f"Creating {num_vectors} vectors on GPU...")
    gpu_embeddings = torch.randn(num_vectors, dim, device="cuda", dtype=torch.float32)

    # OmenDB automatically handles GPU→CPU transfer
    print("Transferring and storing vectors...")
    start = time.perf_counter()

    batch_ids = [f"gpu_vec_{i}" for i in range(num_vectors)]
    batch_metadata = [{} for _ in range(num_vectors)]

    # ✅ CORRECT: Transfer GPU tensor to CPU NumPy for best performance
    vectors_cpu = gpu_embeddings.cpu().numpy()
    db.add_batch(vectors=vectors_cpu, ids=batch_ids, metadata=batch_metadata)

    elapsed = time.perf_counter() - start
    rate = num_vectors / elapsed

    print(f"Stored {num_vectors} GPU vectors at {rate:.1f} vec/s")

    # Query with GPU tensor
    query_gpu = torch.randn(dim, device="cuda", dtype=torch.float32)
    results = db.search(query_gpu, limit=5)
    print(f"Query completed, found {len(results)} results")


if __name__ == "__main__":
    print("🔥 OmenDB PyTorch Integration Examples\n")

    # Run examples
    benchmark_pytorch_integration()
    semantic_search_example()
    model_embedding_cache()
    gpu_tensor_example()

    print("\n\n✅ PyTorch integration examples complete!")
    print("\n💡 Performance Tips:")
    print(
        "- Convert PyTorch tensors to NumPy for optimal performance (1.8x improvement)"
    )
    print("- Batch operations achieve 90K vec/s (lists) or 158K vec/s (NumPy)")
    print("- For GPU tensors: Use tensor.cpu().numpy() for efficient transfer")
    print("- Direct array passing avoids conversion overhead")
