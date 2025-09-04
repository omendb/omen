#!/usr/bin/env python3
"""
OpenAI Embeddings Integration Example for OmenDB
================================================

This example demonstrates using real OpenAI embeddings with OmenDB for
production-grade semantic search and retrieval-augmented generation (RAG).

Features:
- Real OpenAI text-embedding-ada-002 embeddings (1536D)
- Batch operations with NumPy conversion for 1.7x performance
- Document chunking for large texts
- Semantic search with metadata filtering
- Cost optimization with caching
- Production error handling

Requirements:
    pip install openai

Usage:
    export OPENAI_API_KEY="your-api-key-here"
    python examples/integrations/openai_embeddings_example.py
"""

import sys
import os
import time
import hashlib
import json
from typing import List, Dict, Any, Optional
from pathlib import Path

# Add python directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "python"))

# Check for OpenAI dependency
OPENAI_AVAILABLE = True
try:
    import openai

    print("✅ OpenAI library available")
except ImportError:
    print("❌ OpenAI library not available")
    print("Install with: pip install openai")
    OPENAI_AVAILABLE = False

# Import OmenDB
try:
    from omendb import DB

    print("✅ OmenDB imported successfully")
except ImportError as e:
    print(f"❌ Could not import OmenDB: {e}")
    sys.exit(1)


class OpenAIEmbeddingCache:
    """Simple file-based cache for OpenAI embeddings to reduce API costs."""

    def __init__(self, cache_dir: str = "embeddings_cache"):
        """Initialize cache with directory."""
        self.cache_dir = Path(cache_dir)
        self.cache_dir.mkdir(exist_ok=True)
        self.cache_file = self.cache_dir / "embeddings.json"

        # Load existing cache
        self.cache = {}
        if self.cache_file.exists():
            try:
                with open(self.cache_file, "r") as f:
                    self.cache = json.load(f)
                print(f"✅ Loaded {len(self.cache)} cached embeddings")
            except Exception as e:
                print(f"⚠️ Could not load cache: {e}")

    def _get_cache_key(self, text: str) -> str:
        """Generate cache key for text."""
        return hashlib.md5(text.encode()).hexdigest()

    def get(self, text: str) -> Optional[List[float]]:
        """Get embedding from cache."""
        key = self._get_cache_key(text)
        return self.cache.get(key)

    def put(self, text: str, embedding: List[float]):
        """Store embedding in cache."""
        key = self._get_cache_key(text)
        self.cache[key] = embedding

        # Save to file
        try:
            with open(self.cache_file, "w") as f:
                json.dump(self.cache, f)
        except Exception as e:
            print(f"⚠️ Could not save cache: {e}")

    def stats(self) -> Dict[str, Any]:
        """Get cache statistics."""
        return {
            "cached_embeddings": len(self.cache),
            "cache_file": str(self.cache_file),
            "cache_size_mb": self.cache_file.stat().st_size / 1024 / 1024
            if self.cache_file.exists()
            else 0,
        }


class OpenAIVectorStore:
    """Production-ready vector store using OpenAI embeddings and OmenDB."""

    def __init__(
        self,
        database_path: str = "openai_vectors.omen",
        api_key: Optional[str] = None,
        model: str = "text-embedding-ada-002",
        enable_cache: bool = True,
    ):
        """
        Initialize OpenAI + OmenDB vector store.

        Args:
            database_path: Path to OmenDB database file
            api_key: OpenAI API key (or set OPENAI_API_KEY env var)
            model: OpenAI embedding model to use
            enable_cache: Whether to cache embeddings to reduce API costs
        """
        self.database_path = database_path
        self.model = model
        self.enable_cache = enable_cache

        # Initialize OpenAI client
        if OPENAI_AVAILABLE:
            self.client = openai.OpenAI(api_key=api_key or os.getenv("OPENAI_API_KEY"))
            if not self.client.api_key:
                raise ValueError(
                    "OpenAI API key required. Set OPENAI_API_KEY environment variable."
                )
        else:
            self.client = None
            print("🔧 OpenAI not available - using mock embeddings for demonstration")

        # Initialize cache
        self.cache = OpenAIEmbeddingCache() if enable_cache else None

        # Initialize OmenDB
        self.db = DB(database_path)

        # Track API usage
        self.api_calls = 0
        self.cache_hits = 0

        print(f"✅ Initialized OpenAI vector store")
        print(f"   Database: {database_path}")
        print(f"   Model: {model}")
        print(f"   Cache: {'enabled' if enable_cache else 'disabled'}")

    def _get_embedding(self, text: str) -> List[float]:
        """Get embedding for text with caching."""
        # Check cache first
        if self.cache:
            cached = self.cache.get(text)
            if cached:
                self.cache_hits += 1
                return cached

        # Get from OpenAI API or mock
        if self.client:
            try:
                response = self.client.embeddings.create(input=text, model=self.model)
                embedding = response.data[0].embedding
                self.api_calls += 1
            except Exception as e:
                print(f"❌ OpenAI API error: {e}")
                raise
        else:
            # Mock embedding for demonstration (1536D like OpenAI)
            import random

            random.seed(hash(text) % (2**32))
            embedding = [random.uniform(-0.01, 0.01) for _ in range(1536)]
            print(f"🔧 Using mock embedding for: {text[:50]}...")

        # Cache the result
        if self.cache:
            self.cache.put(text, embedding)

        return embedding

    def _get_embeddings_batch(self, texts: List[str]) -> List[List[float]]:
        """Get embeddings for multiple texts efficiently."""
        embeddings = []

        # Check cache for all texts first
        uncached_texts = []
        uncached_indices = []

        for i, text in enumerate(texts):
            if self.cache:
                cached = self.cache.get(text)
                if cached:
                    embeddings.append(cached)
                    self.cache_hits += 1
                    continue

            # Need to fetch this one
            embeddings.append(None)  # Placeholder
            uncached_texts.append(text)
            uncached_indices.append(i)

        # Fetch uncached embeddings
        if uncached_texts:
            if self.client:
                try:
                    # OpenAI API supports batch requests
                    response = self.client.embeddings.create(
                        input=uncached_texts, model=self.model
                    )

                    for i, embedding_data in enumerate(response.data):
                        embedding = embedding_data.embedding
                        original_index = uncached_indices[i]
                        embeddings[original_index] = embedding

                        # Cache the result
                        if self.cache:
                            self.cache.put(uncached_texts[i], embedding)

                    self.api_calls += 1

                except Exception as e:
                    print(f"❌ OpenAI API batch error: {e}")
                    raise
            else:
                # Mock embeddings
                for i, text in enumerate(uncached_texts):
                    import random

                    random.seed(hash(text) % (2**32))
                    embedding = [random.uniform(-0.01, 0.01) for _ in range(1536)]
                    original_index = uncached_indices[i]
                    embeddings[original_index] = embedding

        return embeddings

    def add_documents(
        self,
        documents: List[str],
        metadatas: Optional[List[Dict[str, Any]]] = None,
        batch_size: int = 100,
    ) -> List[str]:
        """
        Add documents to the vector store.

        Args:
            documents: List of document texts
            metadatas: Optional list of metadata dicts
            batch_size: Batch size for OpenAI API calls

        Returns:
            List of document IDs
        """
        print(f"📄 Adding {len(documents)} documents...")

        doc_ids = []

        # Process in batches to manage API limits
        for i in range(0, len(documents), batch_size):
            batch_docs = documents[i : i + batch_size]
            batch_metadata = metadatas[i : i + batch_size] if metadatas else None

            print(
                f"   Processing batch {i // batch_size + 1}/{(len(documents) + batch_size - 1) // batch_size}..."
            )

            # Get embeddings for batch
            embeddings = self._get_embeddings_batch(batch_docs)

            # Convert to NumPy for better performance (1.7x speedup)
            import numpy as np

            embeddings_np = np.array(embeddings, dtype=np.float32)

            # Prepare batch data
            batch_ids = [f"doc_{i + j:05d}" for j in range(len(batch_docs))]
            batch_meta = []

            for j, doc in enumerate(batch_docs):
                metadata = {"text": doc}
                if batch_metadata and j < len(batch_metadata):
                    metadata.update(batch_metadata[j])
                batch_meta.append(metadata)

            # BEST PRACTICE: Use batch operations with NumPy
            try:
                self.db.add_batch(
                    vectors=embeddings_np, ids=batch_ids, metadata=batch_meta
                )
                doc_ids.extend(batch_ids)
                print(f"     ✅ Added {len(batch_ids)} documents using batch operation")
            except Exception as e:
                print(f"     ❌ Error adding batch: {e}")

        print(f"   Total added: {len(doc_ids)} documents")
        return doc_ids

    def search(
        self, query: str, k: int = 5, where: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Search for similar documents.

        Args:
            query: Search query text
            k: Number of results to return
            where: Optional metadata filter

        Returns:
            List of search results with metadata
        """
        print(f"🔍 Searching for: '{query}' (k={k})")

        # Get query embedding
        query_embedding = self._get_embedding(query)

        # Search in OmenDB
        try:
            results = self.db.search(query_embedding, limit=k, filter=where)

            # Format results
            formatted_results = []
            for result in results:
                result_dict = {
                    "id": result.id,
                    "similarity": result.score,
                    "text": result.metadata.get("text", "") if result.metadata else "",
                    "metadata": result.metadata or {},
                }
                formatted_results.append(result_dict)

            print(f"   ✅ Found {len(formatted_results)} results")
            return formatted_results

        except Exception as e:
            print(f"   ❌ Search failed: {e}")
            return []

    def get_usage_stats(self) -> Dict[str, Any]:
        """Get API usage and performance statistics."""
        db_stats = self.db.info()
        cache_stats = self.cache.stats() if self.cache else {}

        return {
            "api_calls": self.api_calls,
            "cache_hits": self.cache_hits,
            "cache_hit_rate": self.cache_hits / (self.api_calls + self.cache_hits)
            if (self.api_calls + self.cache_hits) > 0
            else 0,
            "database_stats": db_stats,
            "cache_stats": cache_stats,
        }


def chunk_text(text: str, chunk_size: int = 500, overlap: int = 50) -> List[str]:
    """
    Split text into overlapping chunks for better retrieval.

    Args:
        text: Text to chunk
        chunk_size: Target chunk size in characters
        overlap: Overlap between chunks in characters

    Returns:
        List of text chunks
    """
    if len(text) <= chunk_size:
        return [text]

    chunks = []
    start = 0

    while start < len(text):
        end = start + chunk_size

        # Try to break at sentence boundary
        if end < len(text):
            # Look for sentence ending within last 100 chars
            search_start = max(start, end - 100)
            for i in range(end, search_start, -1):
                if text[i] in ".!?":
                    end = i + 1
                    break

        chunk = text[start:end].strip()
        if chunk:
            chunks.append(chunk)

        start = end - overlap

        if start >= len(text):
            break

    return chunks


def demonstrate_openai_integration():
    """Demonstrate OpenAI + OmenDB integration."""
    print("🚀 OpenAI + OmenDB Integration Demo")
    print("=" * 50)

    # Check API key
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key and OPENAI_AVAILABLE:
        print("⚠️ No OPENAI_API_KEY found. Set it to use real OpenAI embeddings:")
        print("  export OPENAI_API_KEY='your-api-key-here'")
        print("Running with mock embeddings for demonstration...")

    # Initialize vector store
    vector_store = OpenAIVectorStore(
        database_path="openai_demo.omen", enable_cache=True
    )

    # Sample documents about AI and technology
    documents = [
        "OpenAI's GPT models have revolutionized natural language processing by using transformer architecture and massive scale training.",
        "Vector databases like OmenDB enable efficient similarity search by storing high-dimensional embeddings and using optimized indexing algorithms.",
        "Retrieval-augmented generation (RAG) combines the power of large language models with external knowledge retrieval for more accurate responses.",
        "Machine learning embeddings capture semantic meaning by representing words, sentences, or documents as dense vectors in high-dimensional space.",
        "The transformer architecture introduced attention mechanisms that allow models to focus on relevant parts of input sequences.",
        "Semantic search goes beyond keyword matching by understanding the meaning and context of queries and documents.",
        "Large language models like GPT-4 can perform complex reasoning tasks by leveraging patterns learned from vast amounts of text data.",
        "Vector similarity search uses metrics like cosine similarity to find the most relevant documents for a given query.",
        "Fine-tuning pre-trained models on specific domains can significantly improve performance for specialized tasks.",
        "Prompt engineering involves crafting effective inputs to guide language models toward desired outputs and behaviors.",
    ]

    # Add metadata
    metadatas = [
        {"category": "nlp", "topic": "gpt", "difficulty": "intermediate"},
        {"category": "databases", "topic": "vector_search", "difficulty": "beginner"},
        {"category": "ai", "topic": "rag", "difficulty": "advanced"},
        {"category": "ml", "topic": "embeddings", "difficulty": "beginner"},
        {"category": "nlp", "topic": "transformers", "difficulty": "intermediate"},
        {"category": "search", "topic": "semantic", "difficulty": "beginner"},
        {"category": "ai", "topic": "reasoning", "difficulty": "advanced"},
        {"category": "search", "topic": "similarity", "difficulty": "intermediate"},
        {"category": "ml", "topic": "fine_tuning", "difficulty": "intermediate"},
        {"category": "ai", "topic": "prompting", "difficulty": "beginner"},
    ]

    # Add documents to vector store
    doc_ids = vector_store.add_documents(documents, metadatas)

    # Test various search queries
    queries = [
        "How do vector databases work?",
        "What is retrieval augmented generation?",
        "Tell me about transformer models",
        "How can I improve language model performance?",
    ]

    print(f"\n🔍 Testing search with {len(queries)} queries...")
    for i, query in enumerate(queries, 1):
        print(f"\n{i}. Query: '{query}'")
        results = vector_store.search(query, k=3)

        for j, result in enumerate(results, 1):
            print(
                f"   {j}. Score: {result['similarity']:.3f} | {result['metadata'].get('topic', 'N/A')}"
            )
            print(f"      {result['text'][:80]}...")

    # Test metadata filtering
    print(f"\n🏷️ Testing metadata filtering...")
    advanced_results = vector_store.search(
        "artificial intelligence concepts", k=5, where={"difficulty": "advanced"}
    )

    print(f"Advanced AI topics ({len(advanced_results)} results):")
    for result in advanced_results:
        print(f"  - {result['metadata'].get('topic', 'N/A')}: {result['text'][:60]}...")

    # Show usage statistics
    print(f"\n📊 Usage Statistics:")
    stats = vector_store.get_usage_stats()
    print(f"   API calls: {stats['api_calls']}")
    print(f"   Cache hits: {stats['cache_hits']}")
    print(f"   Cache hit rate: {stats['cache_hit_rate']:.1%}")
    print(f"   Documents in DB: {stats['database_stats'].get('vector_count', 0)}")

    if stats["cache_stats"]:
        cache_stats = stats["cache_stats"]
        print(f"   Cached embeddings: {cache_stats['cached_embeddings']}")
        print(f"   Cache size: {cache_stats['cache_size_mb']:.2f} MB")

    # Demonstrate document chunking for large texts
    print(f"\n📄 Document Chunking Demo:")
    large_text = """
    Artificial intelligence (AI) is intelligence demonstrated by machines, in contrast to the natural intelligence displayed by humans and animals. Leading AI textbooks define the field as the study of "intelligent agents": any device that perceives its environment and takes actions that maximize its chance of successfully achieving its goals. Colloquially, the term "artificial intelligence" is often used to describe machines that mimic "cognitive" functions that humans associate with the human mind, such as "learning" and "problem solving".
    
    The scope of AI is disputed: as machines become increasingly capable, tasks considered to require "intelligence" are often removed from the definition of AI, a phenomenon known as the AI effect. A quip in Tesler's Theorem says "AI is whatever hasn't been done yet." For instance, optical character recognition is frequently excluded from things considered to be AI, having become a routine technology.
    
    Modern machine learning techniques are at the core of AI. Problems for AI applications include reasoning, knowledge representation, planning, learning, natural language processing, perception, and the ability to move and manipulate objects. General intelligence is among the field's long-term goals.
    """

    chunks = chunk_text(large_text.strip(), chunk_size=200, overlap=50)
    print(f"   Split {len(large_text)} chars into {len(chunks)} chunks:")
    for i, chunk in enumerate(chunks):
        print(f"     Chunk {i + 1}: {len(chunk)} chars - {chunk[:50]}...")

    print(f"\n✅ OpenAI integration demo completed!")
    print(f"This demonstrates:")
    print(f"- Real OpenAI embeddings (1536D) with production caching")
    print(f"- Semantic search with metadata filtering")
    print(f"- Cost optimization through embedding caching")
    print(f"- Document chunking for large texts")
    print(f"- Production error handling and usage tracking")

    if not OPENAI_AVAILABLE or not api_key:
        print(f"\n📝 For full functionality with real OpenAI embeddings:")
        print(f"   1. pip install openai")
        print(f"   2. export OPENAI_API_KEY='your-api-key'")
        print(f"   3. Re-run this example")


if __name__ == "__main__":
    if not OPENAI_AVAILABLE:
        print("Running demo with mock embeddings...")

    demonstrate_openai_integration()
