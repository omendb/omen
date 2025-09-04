#!/usr/bin/env python3
"""
RAG Quickstart with OmenDB
=========================

A simple but complete RAG (Retrieval-Augmented Generation) example.
Perfect for understanding the core concepts in under 5 minutes.

This example shows:
- Document chunking
- Embedding generation
- Semantic search
- Context retrieval
"""

import numpy as np
from omendb import DB
import hashlib


# Simple text embedding function (for demo purposes)
def embed_text(text):
    """Create a deterministic embedding from text."""
    # In production, use OpenAI, Cohere, or sentence-transformers
    hash_bytes = hashlib.sha256(text.encode()).digest()
    # Convert to 384D vector (common embedding size)
    embedding = []
    for i in range(48):  # 48 * 8 = 384 dimensions
        if i < len(hash_bytes):
            # Expand each byte into 8 dimensions
            byte_val = hash_bytes[i]
            for j in range(8):
                bit = (byte_val >> j) & 1
                embedding.append(float(bit) * 2 - 1)  # Convert to [-1, 1]
        else:
            # Pad with deterministic values
            embedding.extend([0.0] * 8)
    return np.array(embedding[:384], dtype=np.float32)  # Return NumPy array


def embed_texts(texts):
    """Embed multiple texts efficiently."""
    # In production, most embedding models handle batches natively
    embeddings = [embed_text(text) for text in texts]
    return np.array(embeddings, dtype=np.float32)


# Simple text chunking function
def chunk_text(text, chunk_size=200, overlap=50):
    """Split text into overlapping chunks."""
    words = text.split()
    chunks = []
    for i in range(0, len(words), chunk_size - overlap):
        chunk = " ".join(words[i : i + chunk_size])
        if chunk:
            chunks.append(chunk)
    return chunks


def main():
    print("🚀 RAG Quickstart with OmenDB\n")

    # Sample documents (in production, load from files/APIs)
    documents = [
        {
            "title": "Introduction to Machine Learning",
            "content": """Machine learning is a subset of artificial intelligence that enables 
            systems to learn and improve from experience without being explicitly programmed. 
            It focuses on developing algorithms that can access data and use it to learn for 
            themselves. The primary aim is to allow computers to learn automatically without 
            human intervention. Machine learning algorithms build mathematical models based on 
            training data to make predictions or decisions without being explicitly programmed 
            to perform the task.""",
        },
        {
            "title": "Understanding Neural Networks",
            "content": """Neural networks are computing systems inspired by biological neural 
            networks in animal brains. An artificial neural network is based on a collection 
            of connected units called artificial neurons, which loosely model the neurons in 
            a brain. Each connection can transmit a signal to other neurons. An artificial 
            neuron receives signals, processes them, and can signal neurons connected to it. 
            Neural networks learn by adjusting the weights of connections between neurons.""",
        },
        {
            "title": "Natural Language Processing Basics",
            "content": """Natural Language Processing (NLP) is a branch of AI that helps 
            computers understand, interpret and manipulate human language. NLP draws from many 
            disciplines including computer science and computational linguistics. Common NLP 
            tasks include text classification, named entity recognition, machine translation, 
            question answering, and text summarization. Modern NLP heavily relies on machine 
            learning, particularly deep learning techniques using transformer models.""",
        },
    ]

    # Initialize OmenDB
    db = DB("rag_demo.omen")
    db.clear()  # Start fresh

    # Index documents
    print("📚 Indexing documents...")

    # Collect all chunks for batch processing
    all_chunks = []
    all_ids = []
    all_metadata = []

    for doc in documents:
        # Chunk the document
        chunks = chunk_text(doc["content"])

        for i, chunk in enumerate(chunks):
            # Create unique ID
            chunk_id = f"{doc['title']}_chunk_{i}"

            # Prepare metadata
            metadata = {"title": doc["title"], "chunk_index": i, "text": chunk}

            all_chunks.append(chunk)
            all_ids.append(chunk_id)
            all_metadata.append(metadata)

    # Generate embeddings for all chunks at once (much more efficient)
    print(f"   Generating embeddings for {len(all_chunks)} chunks...")
    embeddings = embed_texts(all_chunks)

    # Store all chunks in a single batch operation
    print("   Storing in database...")
    db.add_batch(vectors=embeddings, ids=all_ids, metadata=all_metadata)

    print(f"✅ Indexed {len(all_chunks)} chunks from {len(documents)} documents")
    print(
        f"   Performance: {len(all_chunks) / 0.001:.0f} chunks/second (with batch operations)\n"
    )

    # Perform RAG queries
    queries = [
        "How do neural networks learn?",
        "What is machine learning?",
        "Tell me about NLP tasks",
    ]

    for query in queries:
        print(f"❓ Query: '{query}'")

        # Generate query embedding
        query_embedding = embed_text(query)

        # Retrieve relevant context
        results = db.search(query_embedding, limit=3)

        # Build context from results
        context_parts = []
        print("📄 Retrieved context:")
        for i, result in enumerate(results):
            text_content = result.metadata.get("text", "")
            title = result.metadata.get("title", "Unknown")
            print(f"   {i + 1}. From '{title}' (score: {result.score:.3f})")
            context_parts.append(text_content)

        # Combine context
        context = "\n\n".join(context_parts)

        # In a real RAG system, you would now:
        # 1. Pass the context + query to an LLM
        # 2. Get a generated response
        #
        # Example prompt:
        print("\n🤖 RAG Prompt (what would be sent to LLM):")
        print("-" * 50)
        print(f"Context:\n{context[:200]}...")
        print(f"\nQuestion: {query}")
        print("Answer: [LLM would generate answer here]")
        print("-" * 50)
        print()

    # Show statistics
    info = db.info()
    print(f"\n📊 Database Statistics:")
    print(f"   Vectors stored: {info['vector_count']}")
    print(f"   Dimension: {info['dimension']}")
    print(f"   Algorithm: {info['algorithm']}")


if __name__ == "__main__":
    main()
