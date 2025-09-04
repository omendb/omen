#!/usr/bin/env python3
"""
OmenDB Python Bindings Demo

Demonstrates the Python interface to the embedded vector database,
showcasing key features like vector insertion, search, and metadata.
"""

import os
import sys
import tempfile
from pathlib import Path

# Add the python package to path for development
project_root = Path(__file__).parent.parent.parent
python_package = project_root / "python"
sys.path.insert(0, str(python_package))

try:
    # Import modern API (vectors are plain Python lists)
    from omendb import DB

    print("Using modern Mojo-Python bindings")
    # Note: Vectors are List[float], Metadata is dict[str, str]
    binding_mode = "modern"
except ImportError as e1:
    print(f"Error: Could not import OmenDB bindings")
    print(f"Import error: {e1}")
    print("Please build the Python bindings first or check installation.")
    sys.exit(1)


def demo_basic_operations():
    """Demonstrate basic database operations."""
    print("\\n=== Basic Operations Demo ===")

    # Create temporary database file
    with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as temp_file:
        db_path = temp_file.name

    try:
        # Create database (dimension auto-detected from first vector)
        with DB(db_path) as db:
            print(f"Created database: {db_path}")

            # Insert some vectors (dimension auto-detected)
            dimension = 128
            vectors = [
                ("doc1", [0.1] * dimension),
                ("doc2", [0.2] * dimension),
                ("doc3", [0.15] * dimension),
            ]

            for vector_id, data in vectors:
                success = db.add(vector_id, data)
                print(f"Added {vector_id}: {success}")

            # Search for similar vectors
            query = [0.12] * dimension
            results = db.search(query, limit=3)
            print(f"Found {len(results)} search results")
            for result in results:
                print(f"  {result.id}: similarity {result.score:.3f}")

            # Get database statistics
            stats = db.info()
            print(f"Database stats: {stats}")

            print("Database will auto-save on close")

    finally:
        # Clean up
        try:
            os.unlink(db_path)
            print(f"Cleaned up {db_path}")
        except:
            pass


def demo_vector_operations():
    """Demonstrate vector operations."""
    print("\\n=== Vector Operations Demo ===")

    # Create vectors (just Python lists)
    vec1 = [1.0, 2.0, 3.0, 4.0]
    vec2 = [2.0, 3.0, 4.0, 5.0]

    print(f"Vector 1: {vec1} - dimension {len(vec1)}")
    print(f"Vector 2: {vec2} - dimension {len(vec2)}")

    # Access elements (standard Python list operations)
    print(f"vec1[0] = {vec1[0]}")
    print(f"vec1 as list: {vec1}")  # Already a list!

    # Test similarity with database
    try:
        from omendb import DB

        # Calculate similarity manually using dot product
        import numpy as np

        vec1_np = np.array(vec1)
        vec2_np = np.array(vec2)
        similarity = np.dot(vec1_np, vec2_np) / (
            np.linalg.norm(vec1_np) * np.linalg.norm(vec2_np)
        )
        print(f"Cosine similarity: {similarity:.3f}")
    except Exception as e:
        print(f"Similarity calculation error: {e}")


def demo_metadata_operations():
    """Demonstrate metadata operations."""
    print("\\n=== Metadata Operations Demo ===")

    # Metadata is just a Python dictionary
    metadata = {"author": "John Doe", "category": "research", "year": "2024"}

    print(f"Created metadata: {metadata}")

    # Set additional values (standard dict operations)
    metadata["language"] = "en"
    metadata["tags"] = "AI,ML,vectors"

    # Check contents (standard dict operations)
    print(f"Author: {metadata.get('author')}")
    print(f"Contains 'category': {'category' in metadata}")
    print(f"Tags: {metadata['tags']}")

    # Note: In current API, metadata is handled as dict[str, str]
    print("Note: Metadata is passed as dict when adding vectors to database")


def demo_error_handling():
    """Demonstrate error handling."""
    print("\\n=== Error Handling Demo ===")

    try:
        # Try to create vector with empty data
        []  # Empty list
    except ValueError as e:
        print(f"Caught expected error: {e}")

    try:
        # Try invalid database path (if not in stub mode)
        # Try invalid database path
        db = DB("/invalid/path/to/database.omen")
    except Exception as e:
        print(f"Caught database error: {e}")


def demo_performance():
    """Demonstrate performance with larger datasets."""
    print("\\n=== Performance Demo ===")

    import time
    import random

    with tempfile.NamedTemporaryFile(suffix="_perf.omen", delete=False) as temp_file:
        db_path = temp_file.name

    try:
        db = DB(db_path)
        db.clear()  # Clear any existing data to avoid dimension conflicts

        # Set up database
        dimension = 256
        # Note: DB handles dimension automatically based on first vector

        # Insert multiple vectors
        num_vectors = 100
        print(f"Inserting {num_vectors} vectors of dimension {dimension}...")

        start_time = time.time()

        for i in range(num_vectors):
            vector_id = f"vec_{i:04d}"
            data = [random.random() for _ in range(dimension)]
            metadata = {"id": str(i), "batch": str(i // 10)}

            db.add(vector_id, data, metadata)

            if (i + 1) % 20 == 0:
                print(f"  Inserted {i + 1} vectors...")

        insert_time = time.time() - start_time
        print(f"Insertion completed in {insert_time:.2f} seconds")
        print(f"Rate: {num_vectors / insert_time:.1f} vectors/second")

        # Perform searches
        num_searches = 10
        print(f"\\nPerforming {num_searches} search queries...")

        start_time = time.time()

        for i in range(num_searches):
            query = [random.random() for _ in range(dimension)]
            results = db.search(query, limit=5)

        search_time = time.time() - start_time
        print(f"Search completed in {search_time:.2f} seconds")
        print(f"Rate: {num_searches / search_time:.1f} searches/second")

        # Show final stats
        print(f"\\nFinal database stats:")
        print(db.info())

    finally:
        try:
            os.unlink(db_path)
        except:
            pass


def main():
    """Main demo function."""
    print("OmenDB Python Bindings Demo")
    print("=" * 40)
    print(f"Binding mode: {binding_mode}")

    # Run demonstrations
    demo_basic_operations()
    demo_vector_operations()
    demo_metadata_operations()
    demo_error_handling()
    demo_performance()

    print("\\n=== Demo Complete ===")
    print("\\nNext steps:")
    print("1. Build the native Mojo library for full functionality")
    print("2. Install the package: pip install -e python/")
    print("3. Use in your applications!")


if __name__ == "__main__":
    main()
