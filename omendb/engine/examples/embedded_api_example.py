"""
OmenDB Embedded Example

This example demonstrates how to use the OmenDB embedded database
for high-performance vector operations in applications.
"""

import numpy as np
import logging
import time
from typing import List, Dict
import sys
import os

# Add the python directory to the path for development
current_dir = os.path.dirname(os.path.abspath(__file__))
python_dir = os.path.join(os.path.dirname(current_dir), "python")
sys.path.insert(0, python_dir)

from omendb import DB

# Configure logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("ClientExample")


def create_random_vector(dim: int) -> List[float]:
    """Create a random normalized vector."""
    vector = np.random.randn(dim).astype(np.float32)
    vector = vector / np.linalg.norm(vector)
    return vector.tolist()


def create_test_data(count: int, dim: int) -> List[Dict]:
    """Create test data for batch insertion."""
    categories = ["electronics", "books", "clothing", "food"]
    ratings = ["1", "2", "3", "4", "5"]

    items = []
    for i in range(count):
        item = {
            "id": f"item_{i}",
            "vector": create_random_vector(dim),
            "metadata": {
                "category": categories[i % len(categories)],
                "rating": ratings[i % len(ratings)],
                "timestamp": str(int(time.time() - i * 100)),
                "in_stock": str(i % 3 == 0),  # Every third item is in stock
            },
        }
        items.append(item)

    return items


def main():
    """Run the embedded database example."""
    logger.info("Starting OmenDB Embedded Example")

    # Create embedded database
    db = DB("client_example.omen")
    logger.info("Created OmenDB embedded database")

    # Vector dimension for this example
    dim = 128

    # Example 1: Insert a single vector
    logger.info("\nExample 1: Insert a single vector")

    vector_id = "example_vec_1"
    vector = create_random_vector(dim)
    metadata = {
        "category": "electronics",
        "name": "Smart TV",
        "price": "499.99",
        "rating": "4.5",
        "in_stock": "true",
    }

    success = db.add(vector_id, vector, metadata)
    logger.info(f"Insert result: success={success}")

    # Example 2: Get the vector
    logger.info("\nExample 2: Get the vector")

    # Note: OmenDB embedded API doesn't have a get() method
    # We can verify by searching for exact match
    search_results = db.search(vector, limit=1)
    if search_results and search_results[0].id == vector_id:
        logger.info(
            f"Found vector {search_results[0].id} with score: {search_results[0].score}"
        )
        logger.info(f"Vector dimension: {len(vector)}")
    else:
        logger.warning("Failed to find vector")

    # Example 3: Update the vector
    logger.info("\nExample 3: Update the vector")

    updated_metadata = {
        "price": "449.99",  # Price reduced
        "sale": "true",  # New field
    }

    # OmenDB embedded API doesn't have update - we can add with new metadata
    success = db.add(vector_id, vector, {**metadata, **updated_metadata})
    logger.info(f"Update (re-add) result: success={success}")

    # Verify the update by searching again
    search_results = db.search(vector, limit=1)
    if search_results and search_results[0].id == vector_id:
        logger.info(f"Update verified - found vector with ID: {search_results[0].id}")

    # Example 4: Batch insert
    logger.info("\nExample 4: Batch insert")

    batch_items = create_test_data(10, dim)
    # Convert batch_items to the format expected by add_batch
    batch_ids = [item["id"] for item in batch_items]
    batch_vectors = [item["vector"] for item in batch_items]
    batch_metadata = [item["metadata"] for item in batch_items]

    result_ids = db.add_batch(
        vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata
    )

    logger.info(f"Batch insert completed")
    logger.info(f"Inserted {len(result_ids)} vectors successfully")

    # Example 5: Search
    logger.info("\nExample 5: Search")

    # Create a query vector
    query_vector = create_random_vector(dim)

    # Simple search
    results = db.search(query_vector, limit=5)
    logger.info(f"Found {len(results)} results")

    for i, result in enumerate(results):
        logger.info(f"Result {i + 1}: id={result.id}, score={result.score}")
        if result.metadata:
            logger.info(f"  Metadata: {result.metadata}")

    # Example 6: Filtered search
    logger.info("\nExample 6: Filtered search")

    # OmenDB embedded API uses filter dict format
    filter_dict = {"category": "electronics", "in_stock": "true"}

    results = db.search(query_vector, limit=5, filter=filter_dict)
    logger.info(f"Found {len(results)} results matching filters")

    for i, result in enumerate(results):
        logger.info(f"Result {i + 1}: id={result.id}, score={result.score}")
        if result.metadata:
            logger.info(f"  Metadata: {result.metadata}")

    # Example 7: Count vectors
    logger.info("\nExample 7: Count vectors")

    # Get total count from database info
    stats = db.info()
    count = stats.get("vector_count", 0)
    logger.info(f"Total vector count: {count}")

    # Note: OmenDB embedded API doesn't support filtered counting
    # We could approximate by searching and counting results
    electronics_results = db.search(
        query_vector, limit=1000, filter={"category": "electronics"}
    )
    logger.info(f"Electronics category approximate count: {len(electronics_results)}")

    # Example 8: Delete a vector
    logger.info("\nExample 8: Delete a vector")

    success = db.delete(vector_id)
    logger.info(f"Delete result: success={success}")

    # Verify deletion by searching
    search_results = db.search(vector, limit=1)
    vector_found = any(r.id == vector_id for r in search_results)
    if not vector_found:
        logger.info("Vector successfully deleted")
    else:
        logger.warning("Vector still exists after deletion attempt")

    logger.info("\nEmbedded database example completed successfully!")


if __name__ == "__main__":
    main()
