#!/usr/bin/env python3
"""
LangChain integration example for OmenDB.

This example shows how to use OmenDB as a vector store backend
with LangChain for document similarity search and retrieval.
"""

import sys
import os
from typing import List, Optional, Dict, Any

# Add the omendb directory to Python path
root_dir = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(root_dir, "python"))

DEPENDENCIES_AVAILABLE = True
MISSING_DEPS = []

try:
    from langchain.vectorstores.base import VectorStore
    from langchain.schema import Document
    from langchain.embeddings.base import Embeddings

    print("✅ LangChain imports successful")
except ImportError as e:
    print(f"❌ LangChain not available: {e}")
    DEPENDENCIES_AVAILABLE = False
    MISSING_DEPS.append("langchain")

try:
    from sentence_transformers import SentenceTransformer

    print("✅ SentenceTransformers available")
except ImportError as e:
    print(f"❌ SentenceTransformers not available: {e}")
    DEPENDENCIES_AVAILABLE = False
    MISSING_DEPS.append("sentence-transformers")

if not DEPENDENCIES_AVAILABLE:
    print(f"\n🔧 Missing dependencies: {', '.join(MISSING_DEPS)}")
    print("Install with:")
    for dep in MISSING_DEPS:
        print(f"  pip install {dep}")
    print("\nRunning demo with mock dependencies to show OmenDB integration pattern...")

    # Mock the missing dependencies for demonstration
    class MockEmbeddings:
        def embed_documents(self, texts):
            return [[0.1] * 384 for _ in texts]

        def embed_query(self, text):
            return [0.1] * 384

    class MockDocument:
        def __init__(self, page_content, metadata=None):
            self.page_content = page_content
            self.metadata = metadata or {}

    class MockVectorStore:
        pass

    # Use mocks if real dependencies not available
    if "langchain" in MISSING_DEPS:
        VectorStore = MockVectorStore
        Document = MockDocument
        Embeddings = MockEmbeddings

    if "sentence-transformers" in MISSING_DEPS:
        import numpy as np

        class MockSentenceTransformer:
            def __init__(self, model_name):
                self.model_name = model_name

            def encode(self, texts):
                if isinstance(texts, str):
                    texts = [texts]
                # Return numpy arrays like real sentence-transformers
                embeddings = []
                for text in texts:
                    # Create deterministic embeddings based on text hash for consistency
                    import random

                    random.seed(hash(text) % (2**32))
                    embedding = [random.uniform(-0.1, 0.1) for _ in range(384)]
                    embeddings.append(embedding)
                return np.array(embeddings)

        SentenceTransformer = MockSentenceTransformer

# Import OmenDB
try:
    from omendb import DB

    print("✅ OmenDB Python SDK imported")
except ImportError as e:
    print(f"❌ Could not import OmenDB: {e}")
    sys.exit(1)


class SentenceTransformerEmbeddings(Embeddings):
    """Wrapper for SentenceTransformers to work with LangChain."""

    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        """Initialize with a sentence transformer model."""
        self.model = SentenceTransformer(model_name)
        print(f"✅ Loaded embedding model: {model_name}")

    def embed_documents(self, texts: List[str]) -> List[List[float]]:
        """Embed a list of documents."""
        embeddings = self.model.encode(texts)
        # Return NumPy array for better OmenDB performance
        # LangChain can handle NumPy arrays in most cases
        return embeddings

    def embed_query(self, text: str) -> List[float]:
        """Embed a single query text."""
        embedding = self.model.encode([text])
        return embedding[0].tolist()


class OmenDBVectorStore(VectorStore):
    """LangChain VectorStore implementation using OmenDB."""

    def __init__(
        self,
        database_path: str = "langchain_vectors.omen",
        embedding_function: Optional[Embeddings] = None,
    ):
        """Initialize OmenDB vector store."""
        self.database_path = database_path
        self.embedding_function = embedding_function or SentenceTransformerEmbeddings()

        # Initialize OmenDB
        self.db = DB(database_path)

        # Get dimension from embedding model
        sample_embedding = self.embedding_function.embed_query("test")
        self.dimension = len(sample_embedding)

        print(f"✅ Initialized OmenDB vector store")
        print(f"   Database: {database_path}")
        print(f"   Dimension: {self.dimension}")

    def add_texts(
        self,
        texts: List[str],
        metadatas: Optional[List[Dict[str, Any]]] = None,
        **kwargs,
    ) -> List[str]:
        """Add texts to the vector store."""
        print(f"Adding {len(texts)} texts to vector store...")

        # Generate embeddings
        embeddings = self.embedding_function.embed_documents(texts)

        # Generate IDs
        ids = [f"doc_{i}" for i in range(len(texts))]

        # Prepare metadata with text included
        final_metadatas = []
        for i, text in enumerate(texts):
            metadata = {}
            if metadatas and i < len(metadatas):
                metadata.update(metadatas[i])
            metadata["text"] = text
            final_metadatas.append(metadata)

        # Use batch operation for better performance
        # With our custom embeddings, this is NumPy array (optimal)
        self.db.add_batch(
            vectors=embeddings,  # numpy.ndarray for 1.7x performance
            ids=ids,
            metadata=final_metadatas,
        )
        print(f"  ✅ Added {len(texts)} documents in batch")

        return ids

    def similarity_search(self, query: str, k: int = 4, **kwargs) -> List[Document]:
        """Search for similar documents."""
        print(f"Searching for '{query}' (k={k})...")

        # Generate query embedding
        query_embedding = self.embedding_function.embed_query(query)

        # Search in OmenDB
        results = self.db.search(query_embedding, limit=k)

        # Convert results to LangChain Documents
        documents = []
        for result in results:
            # Extract metadata if available
            metadata = {
                "doc_id": result.id,
                "source": "omendb",
                "similarity": result.score,
            }
            if result.metadata:
                # Use the stored text and other metadata
                page_content = result.metadata.get("text", f"Document {result.id}")
                metadata.update(result.metadata)
            else:
                page_content = f"Document {result.id} (retrieved from OmenDB)"

            doc = Document(page_content=page_content, metadata=metadata)
            documents.append(doc)

        print(f"  ✅ Found {len(documents)} similar documents")
        return documents

    def similarity_search_with_score(
        self, query: str, k: int = 4, **kwargs
    ) -> List[tuple]:
        """Search with similarity scores."""
        # Generate query embedding
        query_embedding = self.embedding_function.embed_query(query)

        # Search in OmenDB
        results = self.db.search(query_embedding, limit=k)

        # Convert results to LangChain Documents with real scores
        document_score_pairs = []
        for result in results:
            # Extract metadata if available
            metadata = {"doc_id": result.id, "source": "omendb"}
            if result.metadata:
                page_content = result.metadata.get("text", f"Document {result.id}")
                metadata.update(result.metadata)
            else:
                page_content = f"Document {result.id} (retrieved from OmenDB)"

            doc = Document(page_content=page_content, metadata=metadata)
            # Use real similarity score from OmenDB
            document_score_pairs.append((doc, result.score))

        return document_score_pairs

    @classmethod
    def from_texts(
        cls,
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        **kwargs,
    ) -> "OmenDBVectorStore":
        """Create vector store from list of texts."""
        database_path = kwargs.get("database_path", "langchain_vectors.omen")
        store = cls(database_path=database_path, embedding_function=embedding)
        store.add_texts(texts, metadatas)
        return store


def main():
    """Demonstrate LangChain + OmenDB integration."""
    print("🚀 LangChain + OmenDB Integration Example")
    print("=" * 50)

    # Sample documents
    documents = [
        "Machine learning is a subset of artificial intelligence.",
        "Vector databases store high-dimensional embeddings efficiently.",
        "LangChain provides a framework for building LLM applications.",
        "OmenDB is an embedded vector database optimized for performance.",
        "Retrieval-augmented generation combines search with language models.",
    ]

    # Create embeddings
    embeddings = SentenceTransformerEmbeddings("all-MiniLM-L6-v2")

    # Create vector store
    print("\n📊 Creating vector store...")
    vector_store = OmenDBVectorStore.from_texts(
        texts=documents, embedding=embeddings, database_path="langchain_demo.omen"
    )

    # Test similarity search
    print("\n🔍 Testing similarity search...")
    query = "What is vector database?"
    results = vector_store.similarity_search(query, k=3)

    print(f"\nQuery: '{query}'")
    print("Results:")
    for i, doc in enumerate(results, 1):
        print(f"  {i}. {doc.page_content}")

    # Test with scores
    print("\n📈 Testing search with scores...")
    results_with_scores = vector_store.similarity_search_with_score(query, k=3)

    print(f"\nQuery: '{query}'")
    print("Results with scores:")
    for i, (doc, score) in enumerate(results_with_scores, 1):
        print(f"  {i}. Score: {score:.3f} - {doc.page_content}")

    print("\n✅ LangChain integration example completed successfully!")
    print("\nThis demonstrates:")
    print("- OmenDB as a LangChain VectorStore backend")
    if DEPENDENCIES_AVAILABLE:
        print("- Real embedding model integration (all-MiniLM-L6-v2)")
    else:
        print(
            "- Mock embedding model (install sentence-transformers for real embeddings)"
        )
    print("- Document storage and similarity search")
    print("- Production-ready ML framework integration")

    if not DEPENDENCIES_AVAILABLE:
        print(f"\n📝 To get full functionality, install missing dependencies:")
        for dep in MISSING_DEPS:
            print(f"  pip install {dep}")
        print("Then re-run this example for real LangChain integration.")


if __name__ == "__main__":
    main()
