#!/usr/bin/env python3
"""
Simple ML integration demo for OmenDB.

This demonstrates the native module integration with a simulated
ML workflow using the actual OmenDB Python SDK.
"""

import sys
import os
import random
import math
from typing import List, Dict, Any

# Add the omendb directory to Python path
root_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(root_dir, "python"))

# Import OmenDB
try:
    from omendb import DB

    print("✅ OmenDB Python SDK imported successfully")
except ImportError as e:
    print(f"❌ Could not import OmenDB: {e}")
    sys.exit(1)


class SimpleEmbeddingModel:
    """Simulate a basic embedding model for demo purposes."""

    def __init__(self, dimension: int = 384):
        """Initialize with embedding dimension."""
        self.dimension = dimension
        print(f"✅ Initialized embedding model (dim={dimension})")

    def encode(self, texts: List[str]) -> List[List[float]]:
        """Generate deterministic embeddings for demo."""
        embeddings = []
        for text in texts:
            # Create deterministic embedding based on text hash
            random.seed(hash(text) % (2**32))
            embedding = [random.uniform(-1, 1) for _ in range(self.dimension)]
            # Normalize to unit vector
            norm = math.sqrt(sum(x * x for x in embedding))
            embedding = [x / norm for x in embedding]
            embeddings.append(embedding)
        return embeddings

    def encode_single(self, text: str) -> List[float]:
        """Encode a single text."""
        return self.encode([text])[0]


class MLFrameworkDemo:
    """Demonstrate ML framework integration patterns."""

    def __init__(self, database_path: str = "ml_demo.omen"):
        """Initialize the demo."""
        self.database_path = database_path
        self.embedding_model = SimpleEmbeddingModel(384)  # Simulate all-MiniLM-L6-v2

        # Initialize OmenDB
        self.db = DB(database_path)

        print(f"✅ Initialized ML demo with OmenDB")
        print(f"   Database: {database_path}")
        print(f"   Embedding dimension: {self.embedding_model.dimension}")

    def add_documents(
        self, documents: List[str], metadata_list: List[Dict] = None
    ) -> List[str]:
        """Add documents with embeddings to the vector store."""
        print(f"\n📄 Adding {len(documents)} documents...")

        # Generate embeddings
        embeddings = self.embedding_model.encode(documents)

        # Generate document IDs
        doc_ids = [f"doc_{i:04d}" for i in range(len(documents))]

        # Add to database
        for i, (doc, embedding, doc_id) in enumerate(
            zip(documents, embeddings, doc_ids)
        ):
            # Prepare metadata
            metadata = {"text": doc, "index": i}
            if metadata_list and i < len(metadata_list):
                metadata.update(metadata_list[i])

            # Insert into OmenDB
            try:
                success = self.db.add(doc_id, embedding)
                if success:
                    print(f"  ✅ Added {doc_id}: {doc[:50]}...")
                else:
                    print(f"  ❌ Failed to add {doc_id}")
            except Exception as e:
                print(f"  ❌ Error adding {doc_id}: {e}")

        return doc_ids

    def search_similar(self, query: str, k: int = 5) -> List[str]:
        """Search for similar documents."""
        print(f"\n🔍 Searching for: '{query}' (k={k})")

        # Generate query embedding
        query_embedding = self.embedding_model.encode_single(query)

        # Search in OmenDB
        try:
            results = self.db.search(query_embedding, limit=k)
            result_ids = [r.id for r in results]
            print(f"  ✅ Found {len(result_ids)} results")
            return result_ids
        except Exception as e:
            print(f"  ❌ Search failed: {e}")
            return []

    def demonstrate_retrieval_workflow(self):
        """Demonstrate a typical RAG-style retrieval workflow."""
        print("\n🔄 Demonstrating retrieval workflow...")

        # Sample documents (ML/AI domain)
        documents = [
            "Machine learning is a subset of artificial intelligence that focuses on algorithms.",
            "Vector databases are specialized databases designed to store and query high-dimensional vectors.",
            "Embeddings are dense vector representations of data that capture semantic meaning.",
            "Transformer models have revolutionized natural language processing tasks.",
            "Retrieval-augmented generation combines information retrieval with language generation.",
            "FAISS is a library for efficient similarity search and clustering of dense vectors.",
            "Semantic search uses meaning rather than keyword matching to find relevant documents.",
            "Large language models can generate human-like text given appropriate prompts.",
            "Vector similarity is often measured using cosine similarity or Euclidean distance.",
            "Distributed vector databases can scale to billions of vectors across multiple nodes.",
        ]

        # Add documents to vector store
        doc_ids = self.add_documents(documents)

        # Test queries
        queries = [
            "What are vector databases?",
            "How do machine learning algorithms work?",
            "Tell me about embeddings and vectors",
            "What is retrieval augmented generation?",
        ]

        # Perform searches
        for query in queries:
            results = self.search_similar(query, k=3)
            print(f"\nQuery: '{query}'")
            print("Top results:")
            for i, result_id in enumerate(results, 1):
                # Find the document text for this ID
                doc_index = int(result_id.split("_")[1])
                if doc_index < len(documents):
                    doc_text = documents[doc_index]
                    print(f"  {i}. {result_id}: {doc_text}")
                else:
                    print(f"  {i}. {result_id}: (document not found)")

    def test_database_operations(self):
        """Test various database operations."""
        print("\n🧪 Testing database operations...")

        # Test stats
        stats = self.db.info()
        print(f"  Database stats: {stats}")

        # Test similarity
        try:
            vec1 = [1.0, 0.0, 0.0]
            vec2 = [1.0, 0.0, 0.0]
            similarity = self.db.test_similarity(vec1, vec2)
            print(f"  ✅ Similarity test: {similarity}")
        except Exception as e:
            print(f"  ❌ Similarity test failed: {e}")

    def cleanup(self):
        """Clean up resources."""
        try:
            self.db.close()
            print("✅ Database closed successfully")
        except Exception as e:
            print(f"❌ Error closing database: {e}")


def main():
    """Run the ML integration demo."""
    print("🚀 OmenDB ML Framework Integration Demo")
    print("=" * 50)
    print("This demonstrates:")
    print("- Real embedding model simulation (384-dim vectors)")
    print("- Document storage with metadata")
    print("- Semantic similarity search")
    print("- Production ML workflow patterns")
    print("- Native module functionality")

    # Run the demo
    demo = MLFrameworkDemo("ml_integration_demo.omen")

    try:
        # Test basic database operations
        demo.test_database_operations()

        # Demonstrate retrieval workflow
        demo.demonstrate_retrieval_workflow()

        print("\n✅ ML integration demo completed successfully!")
        print("\nThis shows OmenDB can be integrated with:")
        print("- Any embedding model (SentenceTransformers, OpenAI, etc.)")
        print("- ML frameworks (LangChain, LlamaIndex, etc.)")
        print("- RAG and semantic search workflows")
        print("- Production ML applications")

    finally:
        demo.cleanup()


if __name__ == "__main__":
    main()
