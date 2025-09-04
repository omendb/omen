#!/usr/bin/env python3
"""
Working with Data in OmenDB
==========================

Learn all the ways to add, update, search, and manage vectors in OmenDB.
This example covers CRUD operations, batch processing, and metadata filtering.
"""

from omendb import DB
import numpy as np
import time


def main():
    print("📚 Working with Data in OmenDB\n")

    # Create or connect to database
    db = DB("tutorial.omen")
    db.clear()  # Start fresh for tutorial

    # ========================================
    # 1. Adding Vectors (Create)
    # ========================================
    print("1️⃣ Adding Vectors\n")

    # Small batch of vectors (ALWAYS use batch operations)
    print("Adding initial vectors:")
    initial_data = [
        ("user_001", [0.1, 0.2, 0.3, 0.4], {"name": "Alice", "type": "user"}),
        ("user_002", [0.2, 0.3, 0.4, 0.5], {"name": "Bob", "type": "user"}),
        ("user_003", [0.3, 0.4, 0.5, 0.6], {"name": "Charlie", "type": "user"}),
        ("product_001", [0.5, 0.6, 0.7, 0.8], {"name": "Laptop", "type": "product"}),
        ("product_002", [0.6, 0.7, 0.8, 0.9], {"name": "Phone", "type": "product"}),
    ]

    # Extract data for batch operation
    ids = [item[0] for item in initial_data]
    vectors = np.array([item[1] for item in initial_data], dtype=np.float32)
    metadata = [item[2] for item in initial_data]

    db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
    print(f"✅ Added {len(ids)} vectors using batch operation")

    # Large batch addition (demonstrates performance)
    print("\nLarge batch addition:")
    batch_size = 1000
    vectors = np.random.rand(batch_size, 4).astype(np.float32)
    ids = [f"item_{i:04d}" for i in range(batch_size)]
    metadata = [{"batch": 1, "index": i} for i in range(batch_size)]

    start = time.time()
    # IMPORTANT: Pass NumPy array directly, never use .tolist()
    db.add_batch(
        vectors=vectors,  # NumPy array for 1.7x performance
        ids=ids,
        metadata=metadata,
    )
    elapsed = time.time() - start
    print(
        f"✅ Added {batch_size} vectors in {elapsed:.3f}s ({batch_size / elapsed:.0f} vec/s)"
    )
    print("   Note: Using NumPy arrays directly gives 1.7x performance boost!")

    # ========================================
    # 2. Searching Vectors (Read)
    # ========================================
    print("\n\n2️⃣ Searching Vectors\n")

    # Basic search
    print("Basic similarity search:")
    query = np.array([0.15, 0.25, 0.35, 0.45], dtype=np.float32)
    results = db.search(query, limit=3)

    for i, result in enumerate(results, 1):
        name = result.metadata.get("name", result.id) if result.metadata else result.id
        print(f"  {i}. {name} (score: {result.score:.3f})")

    # Search with metadata filtering
    print("\nFiltered search (only users):")
    results = db.search(query, limit=3, filter={"type": "user"})

    for i, result in enumerate(results, 1):
        name = result.metadata.get("name", result.id) if result.metadata else result.id
        print(f"  {i}. {name} (score: {result.score:.3f})")

    # ========================================
    # 3. Updating Vectors (Update)
    # ========================================
    print("\n\n3️⃣ Updating Vectors\n")

    print("Updating a vector:")
    # Update by deleting and re-adding (OmenDB pattern)
    db.delete("user_001")

    # Even for single updates, use batch for consistency
    update_vector = np.array([[0.11, 0.21, 0.31, 0.41]], dtype=np.float32)
    db.add_batch(
        vectors=update_vector,
        ids=["user_001"],
        metadata=[{"name": "Alice", "type": "user", "updated": True}],
    )
    print("✅ Vector updated")

    # Verify update
    query = np.array([0.11, 0.21, 0.31, 0.41], dtype=np.float32)
    results = db.search(query, limit=1)
    if results and results[0].id == "user_001":
        print(f"✅ Verified: {results[0].metadata}")

    # ========================================
    # 4. Deleting Vectors (Delete)
    # ========================================
    print("\n\n4️⃣ Deleting Vectors\n")

    # Delete single vector
    print("Deleting a single vector:")
    success = db.delete("user_002")
    print(f"✅ Deleted vector: {success}")

    # Delete multiple vectors
    print("\nDeleting multiple vectors:")
    ids_to_delete = ["item_0001", "item_0002", "item_0003"]
    for id in ids_to_delete:
        db.delete(id)
    print(f"✅ Deleted {len(ids_to_delete)} vectors")

    # ========================================
    # 5. Database Information
    # ========================================
    print("\n\n5️⃣ Database Information\n")

    info = db.info()
    print("Database stats:")
    print(f"  Total vectors: {info['vector_count']}")
    print(f"  Dimension: {info['dimension']}")
    print(f"  Algorithm: {info['algorithm']}")
    print(f"  Status: {info['status']}")

    # ========================================
    # 6. Advanced Patterns
    # ========================================
    print("\n\n6️⃣ Advanced Patterns\n")

    # Pattern 1: Upsert (update or insert)
    print("Upsert pattern:")

    def upsert(db, id, vector, metadata):
        """Update if exists, insert if not."""
        db.delete(id)  # No error if doesn't exist
        return db.add(id, vector, metadata)

    upsert(db, "user_999", [0.9, 0.9, 0.9, 0.9], {"name": "Test User"})
    print("✅ Upserted vector")

    # Pattern 2: Bulk operations with error handling
    print("\nBulk operations with error handling:")
    vectors_to_add = [
        ("valid_001", [1.0, 2.0, 3.0, 4.0], {"valid": True}),
        ("valid_002", [2.0, 3.0, 4.0, 5.0], {"valid": True}),
        # This would fail with wrong dimension:
        # ("invalid_001", [1.0, 2.0], {"valid": False}),
    ]

    successful = 0
    failed = 0
    for id, vector, metadata in vectors_to_add:
        try:
            if db.add(id, vector, metadata):
                successful += 1
            else:
                failed += 1
        except Exception as e:
            print(f"  ⚠️ Failed to add {id}: {e}")
            failed += 1

    print(f"✅ Added {successful} vectors, {failed} failed")

    # Pattern 3: Pagination for large result sets
    print("\nPagination pattern:")
    all_results = []
    page_size = 100
    offset = 0

    # Note: OmenDB doesn't have built-in pagination,
    # but you can achieve similar results with metadata
    query = [0.5, 0.5, 0.5, 0.5]
    results = db.search(query, limit=page_size * 3)  # Get more than needed

    # Simulate pagination
    for page in range(3):
        start = page * page_size
        end = start + page_size
        page_results = results[start:end]
        if page_results:
            print(f"  Page {page + 1}: {len(page_results)} results")

    # ========================================
    # 7. Best Practices
    # ========================================
    print("\n\n7️⃣ Best Practices\n")

    print("✅ DO:")
    print("  - Use batch operations for bulk inserts")
    print("  - Include meaningful metadata")
    print("  - Use consistent vector dimensions")
    print("  - Handle errors gracefully")

    print("\n❌ DON'T:")
    print("  - Mix vector dimensions in one database")
    print("  - Forget to handle missing IDs in delete")
    print("  - Use huge metadata objects (keep it simple)")

    # Clean up
    print("\n\n🧹 Cleaning up...")
    # db.clear()  # Uncomment to clear all data
    print("✅ Tutorial complete!")


if __name__ == "__main__":
    main()
