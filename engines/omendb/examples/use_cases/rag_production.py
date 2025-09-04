#!/usr/bin/env python3
"""
Production RAG with OmenDB
=========================

A production-ready RAG implementation with best practices:
- Real embeddings (with fallback for demo)
- Efficient document processing
- Metadata filtering
- Error handling
- Performance optimization
- Observability

Works with or without external dependencies.
"""

import time
import json
import hashlib
import logging
from typing import List, Dict, Any, Optional
from dataclasses import dataclass
from omendb import DB

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Try to import optional dependencies
try:
    from sentence_transformers import SentenceTransformer

    EMBEDDINGS_AVAILABLE = True
    logger.info("✅ Using sentence-transformers for embeddings")
except ImportError:
    EMBEDDINGS_AVAILABLE = False
    logger.info("⚠️ sentence-transformers not available, using fallback embeddings")


@dataclass
class Document:
    """Document structure for RAG pipeline."""

    id: str
    title: str
    content: str
    metadata: Dict[str, Any]


@dataclass
class Chunk:
    """Document chunk with metadata."""

    id: str
    text: str
    document_id: str
    metadata: Dict[str, Any]
    embedding: Optional[List[float]] = None


class EmbeddingModel:
    """Embedding model with fallback support."""

    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        self.model_name = model_name
        self.dimension = 384  # Standard for all-MiniLM-L6-v2

        if EMBEDDINGS_AVAILABLE:
            self.model = SentenceTransformer(model_name)
        else:
            self.model = None

    def encode(self, texts: List[str]) -> List[List[float]]:
        """Encode texts to embeddings."""
        if self.model:
            embeddings = self.model.encode(texts)
            return embeddings.tolist()
        else:
            # Fallback: deterministic embeddings
            return [self._fallback_embed(text) for text in texts]

    def _fallback_embed(self, text: str) -> List[float]:
        """Create deterministic embedding from text hash."""
        # Create unique hash for text
        hash_obj = hashlib.sha256(text.encode())
        hash_bytes = hash_obj.digest()

        # Expand to embedding dimension
        embedding = []
        for i in range(self.dimension):
            if i < len(hash_bytes):
                # Use hash bytes
                embedding.append((hash_bytes[i] / 255.0) * 2 - 1)
            else:
                # Deterministic padding based on text length
                val = (len(text) * (i + 1)) % 256
                embedding.append((val / 255.0) * 2 - 1)

        return embedding


class DocumentProcessor:
    """Process documents into searchable chunks."""

    def __init__(self, chunk_size: int = 512, chunk_overlap: int = 128):
        self.chunk_size = chunk_size
        self.chunk_overlap = chunk_overlap

    def process_documents(self, documents: List[Document]) -> List[Chunk]:
        """Process documents into chunks."""
        all_chunks = []

        for doc in documents:
            chunks = self._chunk_document(doc)
            all_chunks.extend(chunks)

        return all_chunks

    def _chunk_document(self, doc: Document) -> List[Chunk]:
        """Chunk a single document."""
        text = doc.content
        words = text.split()
        chunks = []

        # Sliding window chunking
        stride = self.chunk_size - self.chunk_overlap
        for i in range(0, len(words), stride):
            chunk_words = words[i : i + self.chunk_size]
            if len(chunk_words) < self.chunk_size // 4:  # Skip very small chunks
                continue

            chunk_text = " ".join(chunk_words)

            chunk = Chunk(
                id=f"{doc.id}_chunk_{len(chunks)}",
                text=chunk_text,
                document_id=doc.id,
                metadata={
                    **doc.metadata,
                    "chunk_index": len(chunks),
                    "chunk_size": len(chunk_words),
                    "document_title": doc.title,
                },
            )
            chunks.append(chunk)

        logger.info(f"Created {len(chunks)} chunks from document '{doc.title}'")
        return chunks


class RAGPipeline:
    """Production-ready RAG pipeline."""

    def __init__(
        self,
        db_path: str = "production_rag.omen",
        embedding_model: Optional[EmbeddingModel] = None,
        chunk_processor: Optional[DocumentProcessor] = None,
    ):
        self.db = DB(db_path)
        self.embedding_model = embedding_model or EmbeddingModel()
        self.chunk_processor = chunk_processor or DocumentProcessor()

        # Performance metrics
        self.metrics = {
            "documents_processed": 0,
            "chunks_created": 0,
            "queries_processed": 0,
            "avg_query_time": 0,
        }

    def index_documents(self, documents: List[Document], batch_size: int = 100):
        """Index documents with batching for performance."""
        start_time = time.time()

        # Process documents into chunks
        logger.info(f"Processing {len(documents)} documents...")
        chunks = self.chunk_processor.process_documents(documents)

        # Generate embeddings in batches
        logger.info(f"Generating embeddings for {len(chunks)} chunks...")
        for i in range(0, len(chunks), batch_size):
            batch = chunks[i : i + batch_size]
            texts = [chunk.text for chunk in batch]

            # Generate embeddings
            embeddings = self.embedding_model.encode(texts)

            # Prepare batch data
            ids = []
            vectors = []
            metadata_list = []

            for chunk, embedding in zip(batch, embeddings):
                ids.append(chunk.id)
                vectors.append(embedding)
                metadata_list.append(
                    {
                        "text": chunk.text,
                        "document_id": chunk.document_id,
                        **chunk.metadata,
                    }
                )

            # Batch insert
            self.db.add_batch(vectors=vectors, ids=ids, metadata=metadata_list)

            logger.info(
                f"Indexed batch {i // batch_size + 1}/{(len(chunks) + batch_size - 1) // batch_size}"
            )

        # Update metrics
        elapsed = time.time() - start_time
        self.metrics["documents_processed"] += len(documents)
        self.metrics["chunks_created"] += len(chunks)

        logger.info(
            f"✅ Indexed {len(chunks)} chunks in {elapsed:.2f}s ({len(chunks) / elapsed:.1f} chunks/s)"
        )

        return len(chunks)

    def search(
        self, query: str, limit: int = 5, filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """Search for relevant chunks."""
        start_time = time.time()

        # Generate query embedding
        query_embedding = self.embedding_model.encode([query])[0]

        # Search with optional filters
        results = self.db.search(query_embedding, limit=limit, filter=filters)

        # Process results
        processed_results = []
        for result in results:
            processed_results.append(
                {
                    "id": result.id,
                    "score": result.score,
                    "text": result.metadata.get("text", ""),
                    "document_id": result.metadata.get("document_id", ""),
                    "document_title": result.metadata.get("document_title", ""),
                    "metadata": result.metadata,
                }
            )

        # Update metrics
        elapsed = time.time() - start_time
        self.metrics["queries_processed"] += 1
        self.metrics["avg_query_time"] = (
            self.metrics["avg_query_time"] * (self.metrics["queries_processed"] - 1)
            + elapsed
        ) / self.metrics["queries_processed"]

        logger.info(
            f"Query completed in {elapsed * 1000:.2f}ms, found {len(results)} results"
        )

        return processed_results

    def get_context(self, query: str, max_tokens: int = 2000) -> str:
        """Get context for LLM prompt."""
        results = self.search(query, limit=10)

        context_parts = []
        token_count = 0

        for result in results:
            text = result["text"]
            # Rough token estimation (1 token ≈ 4 chars)
            estimated_tokens = len(text) // 4

            if token_count + estimated_tokens > max_tokens:
                # Truncate if needed
                remaining_tokens = max_tokens - token_count
                text = text[: remaining_tokens * 4]
                context_parts.append(text)
                break

            context_parts.append(text)
            token_count += estimated_tokens

        return "\n\n---\n\n".join(context_parts)

    def get_metrics(self) -> Dict[str, Any]:
        """Get pipeline metrics."""
        db_info = self.db.info()
        return {
            **self.metrics,
            "total_chunks": db_info["vector_count"],
            "dimension": db_info["dimension"],
            "algorithm": db_info["algorithm"],
        }


def main():
    """Demonstrate production RAG pipeline."""
    print("🚀 Production RAG Pipeline with OmenDB\n")

    # Sample documents (in production, load from database/files)
    documents = [
        Document(
            id="doc1",
            title="OmenDB Performance Guide",
            content="""OmenDB achieves exceptional performance through several key optimizations. 
            First, it uses SIMD instructions for vectorized operations, enabling parallel processing 
            of vector comparisons. Second, the automatic algorithm switching between brute force 
            and HNSW ensures optimal performance at any scale. For small datasets under 5,000 vectors, 
            brute force search provides perfect accuracy with minimal overhead. As datasets grow, 
            HNSW indexing kicks in automatically, maintaining sub-millisecond query times even with 
            millions of vectors. The batch API supports ingestion rates exceeding 99,000 vectors 
            per second, making it ideal for real-time applications. Memory efficiency is achieved 
            through optional quantization, reducing memory usage by 4x with minimal accuracy loss.""",
            metadata={"category": "performance", "version": "1.0"},
        ),
        Document(
            id="doc2",
            title="Vector Search Best Practices",
            content="""When implementing vector search, several best practices ensure optimal results. 
            Choose embedding models that match your use case - all-MiniLM-L6-v2 offers a good balance 
            of speed and quality for general text, while specialized models may work better for 
            domain-specific content. Normalize your vectors before insertion to ensure consistent 
            similarity scores. Use metadata filtering to narrow search scope and improve relevance. 
            For large documents, implement sliding window chunking with overlap to preserve context 
            across chunk boundaries. Monitor your search quality using metrics like recall@k and 
            precision@k. Consider implementing a feedback loop to continuously improve result quality 
            based on user interactions.""",
            metadata={"category": "best_practices", "version": "1.0"},
        ),
        Document(
            id="doc3",
            title="RAG Architecture Patterns",
            content="""Retrieval-Augmented Generation combines the power of vector search with 
            large language models. The basic pattern involves: 1) Chunking documents into 
            manageable pieces, 2) Generating embeddings for each chunk, 3) Storing embeddings 
            in a vector database, 4) Converting queries to embeddings, 5) Retrieving relevant 
            chunks, and 6) Using retrieved context with an LLM to generate responses. Advanced 
            patterns include hybrid search combining vector and keyword matching, re-ranking 
            results using cross-encoders, implementing query expansion for better recall, and 
            using multiple embedding models for different content types. Consider implementing 
            caching strategies for frequently accessed content and query results.""",
            metadata={"category": "architecture", "version": "1.0"},
        ),
    ]

    # Initialize pipeline
    pipeline = RAGPipeline()

    # Clear database for demo
    pipeline.db.clear()

    # Index documents
    print("📚 Indexing documents...")
    pipeline.index_documents(documents)

    # Example queries
    queries = [
        "How can I optimize vector search performance?",
        "What are the key components of a RAG system?",
        "Tell me about embedding model selection",
    ]

    print("\n🔍 Running queries...")
    for query in queries:
        print(f"\n❓ Query: '{query}'")

        # Search
        results = pipeline.search(query, limit=3)

        print("📄 Results:")
        for i, result in enumerate(results, 1):
            print(f"   {i}. {result['document_title']} (score: {result['score']:.3f})")
            print(f"      {result['text'][:100]}...")

        # Get context for LLM
        context = pipeline.get_context(query)
        print(f"\n🤖 Context for LLM ({len(context)} chars):")
        print(f"   {context[:200]}...")

    # Demonstrate filtered search
    print("\n\n🎯 Filtered Search Example:")
    query = "performance optimization"
    print(f"Query: '{query}' (filtering for category='performance')")

    results = pipeline.search(query, limit=3, filters={"category": "performance"})

    print(f"Found {len(results)} results with filter applied")

    # Show metrics
    print("\n\n📊 Pipeline Metrics:")
    metrics = pipeline.get_metrics()
    for key, value in metrics.items():
        print(f"   {key}: {value}")

    print("\n✅ Production RAG pipeline demo complete!")


if __name__ == "__main__":
    main()
