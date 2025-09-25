#!/usr/bin/env python3
"""
TensorFlow Integration Example
=============================

Demonstrates seamless integration between OmenDB and TensorFlow for ML workflows.
Shows how to store and retrieve embeddings from TensorFlow models.
"""

import numpy as np
import time
from omendb import DB

# Check for TensorFlow
try:
    import tensorflow as tf

    TF_AVAILABLE = True
    print("✅ TensorFlow available")
except ImportError:
    TF_AVAILABLE = False
    print("⚠️ TensorFlow not available, using NumPy for demonstration")
    print("   Install with: pip install tensorflow")

    # Mock tf with numpy
    class TensorFlowMock:
        float32 = np.float32
        float16 = np.float16
        int32 = np.int32

        class random:
            @staticmethod
            def normal(shape, dtype=None):
                return np.random.randn(*shape).astype(dtype or np.float32)

            @staticmethod
            def uniform(shape, minval=0, maxval=1, dtype=None):
                if dtype == np.int32:
                    return np.random.randint(minval, maxval, size=shape, dtype=np.int32)
                return np.random.uniform(minval, maxval, size=shape).astype(
                    dtype or np.float32
                )

        class data:
            class Dataset:
                @staticmethod
                def from_tensor_slices(data):
                    return MockDataset(data)

        class keras:
            class mixed_precision:
                class Policy:
                    def __init__(self, name):
                        self.name = name

                @staticmethod
                def set_global_policy(policy):
                    pass

    class MockDataset:
        def __init__(self, data):
            self.data = data

        def batch(self, batch_size):
            return self

        def __iter__(self):
            # Simple batching for mock
            if isinstance(self.data, dict):
                keys = list(self.data.keys())
                n = len(self.data[keys[0]])
                for i in range(0, n, 100):  # batch size 100
                    batch = {}
                    for k in keys:
                        batch[k] = self.data[k][i : i + 100]
                    yield batch

    tf = TensorFlowMock()

# Optional: TensorFlow Hub for pre-trained models
try:
    import tensorflow_hub as hub

    TF_HUB_AVAILABLE = True
except ImportError:
    TF_HUB_AVAILABLE = False
    print(
        "Note: Install tensorflow-hub for pre-trained models: pip install tensorflow-hub"
    )


def benchmark_tensorflow_integration():
    """Benchmark TensorFlow tensor storage and retrieval."""
    print("🚀 TensorFlow Integration Benchmark")
    print("=" * 50)

    # Create database
    db = DB("tensorflow_vectors.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Test different tensor sizes with fixed dimension
    dimension = 128  # Use consistent dimension for OmenDB
    batch_sizes = [100, 1000, 10000]

    print(f"\n📊 Testing {dimension}D embeddings")
    print("-" * 40)

    for batch_size in batch_sizes:
        # Create TensorFlow tensors
        embeddings = tf.random.normal([batch_size, dimension], dtype=tf.float32)
        ids = [f"doc_{i}" for i in range(batch_size)]

        # Time the insertion
        start = time.perf_counter()

        # ALWAYS use batch operations for better performance
        # Convert TensorFlow tensor to NumPy for optimal performance
        if TF_AVAILABLE and hasattr(embeddings, "numpy"):
            # ✅ BEST: Convert TensorFlow to NumPy for zero-copy optimization
            vectors_np = embeddings.numpy()  # TensorFlow to NumPy
            db.add_batch(vectors=vectors_np, ids=ids)
        else:
            # Fallback for mock tf (already NumPy)
            db.add_batch(vectors=embeddings, ids=ids)

        elapsed = time.perf_counter() - start
        rate = batch_size / elapsed

        print(
            f"Batch size: {batch_size:5d} | Rate: {rate:8.1f} vec/s | Time: {elapsed:.3f}s"
        )

        # Test query with TensorFlow tensor
        query_tensor = tf.random.normal([dimension], dtype=tf.float32)

        # Convert to NumPy for query
        if TF_AVAILABLE and hasattr(query_tensor, "numpy"):
            query_np = query_tensor.numpy()
        else:
            query_np = query_tensor

        start = time.perf_counter()
        results = db.search(query_np, limit=10)
        query_time = (time.perf_counter() - start) * 1000

        print(f"Query time: {query_time:.2f}ms for top-10 results")


def text_embedding_example():
    """Example: Text embeddings with TensorFlow."""
    print("\n\n📝 Text Embedding Example")
    print("=" * 50)

    # Sample texts
    texts = [
        "TensorFlow is an open-source machine learning framework",
        "Deep learning models can understand complex patterns",
        "Natural language processing is advancing rapidly",
        "Computer vision helps machines understand images",
        "Neural networks are inspired by biological brains",
        "Machine learning requires good quality data",
        "Python is popular for data science and AI",
        "Vector databases enable fast similarity search",
        "Embeddings capture semantic meaning of text",
        "Transfer learning saves training time and resources",
    ]

    # Create database
    db = DB("tf_text_embeddings.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    if TF_HUB_AVAILABLE:
        print("Using Universal Sentence Encoder from TF Hub...")
        # Load pre-trained text encoder
        embed = hub.load("https://tfhub.dev/google/universal-sentence-encoder/4")

        # Encode texts
        print("Encoding texts...")
        embeddings = embed(texts)

    else:
        print(
            "Using random embeddings for demo (install tensorflow-hub for real embeddings)"
        )
        # Create random embeddings
        embedding_dim = 512
        embeddings = tf.random.normal([len(texts), embedding_dim], dtype=tf.float32)

    # Store embeddings with metadata using batch operations
    print(f"Storing {len(texts)} text embeddings...")

    # Convert TensorFlow tensor to NumPy for optimal performance
    if TF_AVAILABLE and hasattr(embeddings, "numpy"):
        embeddings_np = embeddings.numpy()
    else:
        embeddings_np = embeddings

    # Prepare batch data
    ids = [f"text_{i}" for i in range(len(texts))]
    metadata = [
        {"text": text, "length": str(len(text)), "type": "sentence"} for text in texts
    ]

    # Single batch operation for all embeddings
    db.add_batch(vectors=embeddings_np, ids=ids, metadata=metadata)

    # Perform similarity search
    queries = [
        "How does artificial intelligence work?",
        "What programming language should I learn?",
        "Tell me about neural networks",
    ]

    print("\n🔍 Similarity Search Results:")
    for query in queries:
        print(f"\nQuery: '{query}'")

        if TF_HUB_AVAILABLE:
            query_embedding = embed([query])[0]
        else:
            query_embedding = tf.random.normal([embeddings.shape[1]], dtype=tf.float32)

        # Convert to NumPy for search
        if TF_AVAILABLE and hasattr(query_embedding, "numpy"):
            query_np = query_embedding.numpy()
        else:
            query_np = query_embedding

        # Search
        results = db.search(query_np, limit=3)

        for i, result in enumerate(results):
            text = result.metadata.get("text", "N/A") if result.metadata else "N/A"
            print(f"  {i + 1}. (score: {result.score:.3f}) {text}")


def keras_model_embeddings():
    """Example: Extract and store embeddings from Keras models."""
    print("\n\n🧠 Keras Model Embeddings")
    print("=" * 50)

    if not TF_AVAILABLE:
        print("⚠️ TensorFlow not available, skipping Keras model example")
        print("   Install with: pip install tensorflow")
        return

    # Create a simple CNN for feature extraction
    print("Building simple CNN for feature extraction...")

    model = tf.keras.Sequential(
        [
            tf.keras.layers.Input(shape=(224, 224, 3)),
            tf.keras.layers.Conv2D(32, 3, activation="relu"),
            tf.keras.layers.MaxPooling2D(),
            tf.keras.layers.Conv2D(64, 3, activation="relu"),
            tf.keras.layers.MaxPooling2D(),
            tf.keras.layers.Conv2D(128, 3, activation="relu"),
            tf.keras.layers.GlobalAveragePooling2D(),
            tf.keras.layers.Dense(256, activation="relu", name="embeddings"),
            tf.keras.layers.Dense(10, activation="softmax"),  # 10 classes
        ]
    )

    # Create embedding extractor (output from 'embeddings' layer)
    embedding_model = tf.keras.Model(
        inputs=model.input, outputs=model.get_layer("embeddings").output
    )

    # Create database for image embeddings
    db = DB("keras_embeddings.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Simulate processing batches of images
    num_batches = 10
    batch_size = 32
    total_images = num_batches * batch_size

    print(f"\nProcessing {total_images} simulated images...")

    start_time = time.perf_counter()

    for batch_idx in range(num_batches):
        # Simulate batch of images (random data for demo)
        batch_images = tf.random.normal([batch_size, 224, 224, 3])

        # Extract embeddings
        batch_embeddings = embedding_model(batch_images, training=False)

        # Prepare batch data for OmenDB
        batch_ids = []
        batch_vectors = []
        batch_metadata = []
        for i in range(batch_size):
            img_id = f"img_{batch_idx * batch_size + i}"
            metadata = {
                "batch": str(batch_idx),
                "model": "simple_cnn",
                "embedding_dim": "256",
            }
            batch_ids.append(img_id)
            batch_vectors.append(batch_embeddings[i])
            batch_metadata.append(metadata)

        # Store in database
        db.add_batch(vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata)

    elapsed = time.perf_counter() - start_time
    rate = total_images / elapsed

    print(f"Processed {total_images} images in {elapsed:.2f}s ({rate:.1f} img/s)")

    # Demonstrate fast retrieval
    print("\n🔍 Finding similar images...")
    query_image = tf.random.normal([1, 224, 224, 3])
    query_embedding = embedding_model(query_image, training=False)[0]

    start = time.perf_counter()
    similar = db.search(query_embedding, limit=5)
    query_time = (time.perf_counter() - start) * 1000

    print(f"Found similar images in {query_time:.2f}ms:")
    for result in similar:
        batch = result.metadata.get("batch", "N/A") if result.metadata else "N/A"
        print(f"  - {result.id} from batch {batch} (score: {result.score:.3f})")


def tf_data_pipeline_example():
    """Example: Integration with tf.data pipeline."""
    print("\n\n🔄 TensorFlow Data Pipeline Integration")
    print("=" * 50)

    # Create synthetic dataset
    num_samples = 1000
    embedding_dim = 128

    # Create tf.data.Dataset
    print(f"Creating dataset with {num_samples} samples...")

    # Generate data
    embeddings = tf.random.normal([num_samples, embedding_dim])
    ids = [f"sample_{i}" for i in range(num_samples)]
    labels = tf.random.uniform([num_samples], maxval=10, dtype=tf.int32)

    # Create dataset
    dataset = tf.data.Dataset.from_tensor_slices(
        {"id": ids, "embedding": embeddings, "label": labels}
    )

    # Create database
    db = DB("tf_pipeline.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Process in batches using tf.data
    batch_size = 100
    dataset_batched = dataset.batch(batch_size)

    print("Processing dataset in batches...")
    start = time.perf_counter()

    for batch in dataset_batched:
        # Convert batch to list for OmenDB
        batch_ids = []
        batch_vectors = []
        batch_metadata = []
        for i in range(len(batch["id"])):
            # Handle both real TensorFlow tensors and numpy arrays
            label_val = batch["label"][i]
            if hasattr(label_val, "numpy"):
                label_val = label_val.numpy()

            id_val = batch["id"][i]
            if hasattr(id_val, "numpy"):
                id_val = id_val.numpy()
            if isinstance(id_val, bytes):
                id_val = id_val.decode("utf-8")

            metadata = {"label": str(label_val)}
            batch_ids.append(id_val)
            batch_vectors.append(batch["embedding"][i])
            batch_metadata.append(metadata)

        # Store batch
        db.add_batch(vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata)

    elapsed = time.perf_counter() - start
    rate = num_samples / elapsed

    print(f"Stored {num_samples} embeddings at {rate:.1f} vec/s")

    # Query by label
    print("\n🔍 Querying vectors by label...")
    query_embedding = tf.random.normal([embedding_dim])

    # Find similar vectors with label filter
    all_results = db.search(query_embedding, limit=100)

    # Filter by label (in practice, use db.search with filter clause)
    target_label = "5"
    filtered_results = [
        result
        for result in all_results
        if result.metadata and result.metadata.get("label") == target_label
    ][:5]

    print(f"Found {len(filtered_results)} similar vectors with label={target_label}")


def mixed_precision_example():
    """Example: Working with mixed precision tensors."""
    print("\n\n🎯 Mixed Precision Example")
    print("=" * 50)

    # Enable mixed precision
    policy = tf.keras.mixed_precision.Policy("mixed_float16")
    tf.keras.mixed_precision.set_global_policy(policy)

    print("Mixed precision policy:", policy.name)

    # Create database
    db = DB("mixed_precision.omen")
    db.clear()  # Clear any existing data to avoid dimension conflicts

    # Create float16 embeddings
    num_vectors = 5000
    dim = 256

    # Generate embeddings in float16
    embeddings_f16 = tf.random.normal([num_vectors, dim], dtype=tf.float16)

    # OmenDB automatically handles type conversion
    print(f"Storing {num_vectors} float16 embeddings...")

    start = time.perf_counter()
    batch_ids = [f"vec_{i}" for i in range(num_vectors)]
    batch_vectors = [embeddings_f16[i] for i in range(num_vectors)]
    batch_metadata = [{"dtype": "float16"} for i in range(num_vectors)]
    db.add_batch(vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata)
    elapsed = time.perf_counter() - start

    print(f"Stored at {num_vectors / elapsed:.1f} vec/s")

    # Query with float32 (automatic conversion)
    query_f32 = tf.random.normal([dim], dtype=tf.float32)
    results = db.search(query_f32, limit=5)
    print(f"Query successful, found {len(results)} results")


if __name__ == "__main__":
    print("🔥 OmenDB TensorFlow Integration Examples\n")

    # Run examples
    benchmark_tensorflow_integration()
    text_embedding_example()
    keras_model_embeddings()
    tf_data_pipeline_example()
    mixed_precision_example()

    print("\n\n✅ TensorFlow integration examples complete!")
    print("\n💡 Tips:")
    print("- OmenDB automatically handles TensorFlow tensors")
    print("- Works seamlessly with tf.data pipelines")
    print("- Supports mixed precision (float16/float32)")
    print("- Use batch operations for best performance")
    print("- Consider pre-computing embeddings for production")
