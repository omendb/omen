#!/usr/bin/env python3
"""
OmenDB Embedded Quickstart Example

This example demonstrates the basic usage of OmenDB's embedded mode.
OmenDB embedded provides "SQLite for vectors" - a single-file vector database
that runs directly in your application without external dependencies.

Prerequisites:
- Python 3.8+
- OmenDB (run from project directory with pixi environment)

Usage:
    python quickstart.py
"""

import os
import sys
import random
from typing import List

# Add the python directory to path for development
current_dir = os.path.dirname(os.path.abspath(__file__))
python_dir = os.path.join(os.path.dirname(os.path.dirname(current_dir)), "python")
sys.path.insert(0, python_dir)

from omendb import DB


def main():
    """Demonstrate basic embedded vector database operations."""

    print("🚀 OmenDB Embedded Quickstart")
    print("=" * 40)

    # 1. Create/open embedded database (single file, SQLite-style)
    print("\n1. Creating embedded database...")
    db_path = "quickstart_vectors.omen"

    # Clean up any existing database
    if os.path.exists(db_path):
        os.remove(db_path)

    print(f"   Database path: {db_path}")

    # 2. Add vectors with metadata
    print("\n2. Adding sample vectors...")
    sample_data = [
        ("doc1", [0.1, 0.2, 0.3, 0.4]),
        ("doc2", [0.2, 0.3, 0.4, 0.5]),
        ("doc3", [0.8, 0.7, 0.6, 0.5]),
        ("doc4", [0.1, 0.8, 0.2, 0.9]),
        ("doc5", [0.3, 0.1, 0.4, 0.2]),
    ]

    with DB(db_path) as db:
        # Add vectors individually (current API)
        inserted_count = 0
        for doc_id, vector in sample_data:
            success = db.add(doc_id, vector)
            if success:
                inserted_count += 1
                print(f"   ✅ Added {doc_id}: {vector}")
            else:
                print(f"   ❌ Failed to add {doc_id}")

        print(f"   Added {inserted_count} vectors")

        # 3. Basic vector similarity search
        print("\n3. Vector similarity search...")
        query_vector = [0.15, 0.25, 0.35, 0.45]  # Similar to AI documents
        results = db.search(query_vector, limit=3)

        print(f"   Query: {query_vector}")
        print("   Top 3 most similar vectors:")
        for i, result in enumerate(results, 1):
            print(f"   {i}. ID: {result.id}, Score: {result.score:.3f}")

        # 4. Test similarity calculation
        print("\n4. Test similarity calculation...")
        vec1 = [1.0, 0.0, 0.0, 0.0]
        vec2 = [1.0, 0.0, 0.0, 0.0]  # Identical
        vec3 = [0.0, 1.0, 0.0, 0.0]  # Orthogonal

        # Add test vectors and query for similarity
        db.add("test_identical", vec2)
        db.add("test_orthogonal", vec3)

        # Query for all vectors to see similarity scores
        all_results = db.search(vec1, limit=10)

        # Find the similarities
        identical_sim = 0
        orthogonal_sim = 0
        for result in all_results:
            if result.id == "test_identical":
                identical_sim = result.score
            elif result.id == "test_orthogonal":
                orthogonal_sim = result.score

        print(f"   Identical vectors score: {identical_sim:.3f}")
        print(f"   Orthogonal vectors score: {orthogonal_sim:.3f}")

        # 5. Database statistics
        print("\n5. Database statistics...")
        stats = db.info()
        print(f"   Vector count: {stats.get('vector_count', 0)}")
        print(f"   Dimension: {stats.get('current_dimension', 'auto-detected')}")
        print(f"   Status: {stats.get('status', 'unknown')}")
        print(f"   Features: {stats.get('features', 'basic')}")

    # 6. Test persistence
    print("\n6. Test persistence...")
    print("   Database closed with auto-save")

    # Reopen and verify data persisted
    print("   Reopening database to verify persistence...")
    with DB(db_path) as db2:
        # Test that we can still search
        test_query = [0.2, 0.3, 0.4, 0.5]
        results2 = db2.search(test_query, limit=2)

        print(f"   Found {len(results2)} documents after reopening")
        for i, result in enumerate(results2, 1):
            print(f"   {i}. ID: {result.id}, Score: {result.score:.3f}")

        # Show final stats
        final_stats = db2.info()
        print(f"   Reopened database has {final_stats.get('vector_count', 0)} vectors")

    print("\n✅ Quickstart completed successfully!")
    print("\nNext steps:")
    print("- Try the main quickstart: examples/getting_started/quickstart.py")
    print("- Explore performance testing: test/scale/test_performance_scaling.py")
    print("- Build your vector-powered application!")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"❌ Error: {e}")
        print("Make sure you're in the project directory with pixi environment active")
