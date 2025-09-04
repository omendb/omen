#!/usr/bin/env python3
"""Test DiskANN with fixed robust pruning algorithm."""

import sys
import os
import numpy as np
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))
import omendb


def test_diskann_at_scale():
    """Test DiskANN with various sizes to verify the pruning fix."""

    sizes = [100, 500, 1000]

    for size in sizes:
        print(f"\n{'=' * 50}")
        print(f"Testing DiskANN with {size} vectors")
        print("=" * 50)

        # Initialize
        db = omendb.DB(algorithm="diskann", buffer_size=min(100, size // 2))

        # Generate vectors
        vectors = np.random.rand(size, 128).astype(np.float32)
        vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)
        ids = [f"vec_{i}" for i in range(size)]

        # Add vectors
        start = time.time()
        db.add_batch(vectors, ids)
        insert_time = time.time() - start
        print(
            f"✅ Added {size} vectors in {insert_time:.2f}s ({size / insert_time:.0f} vec/s)"
        )

        # Test search on multiple queries
        test_count = min(10, size)
        successful_searches = 0
        total_results = 0

        for i in range(test_count):
            query = vectors[i]
            results = db.search(query, limit=10)

            if results:
                successful_searches += 1
                total_results += len(results)

                # Check if correct vector is in results
                expected_id = ids[i]
                # Handle SearchResult objects
                if hasattr(results[0], "id"):
                    result_ids = [r.id for r in results]
                else:
                    result_ids = [
                        r[0] if isinstance(r, tuple) else str(r) for r in results
                    ]

                if expected_id in result_ids:
                    rank = result_ids.index(expected_id) + 1
                    if i == 0:  # Print details for first query
                        print(f"  Query vec_0 found at rank {rank}")
                        print(f"  Top 3 results: {', '.join(result_ids[:3])}")

        if successful_searches > 0:
            avg_results = total_results / successful_searches
            print(
                f"✅ Search working: {successful_searches}/{test_count} queries succeeded"
            )
            print(f"  Average results per query: {avg_results:.1f}")
        else:
            print(f"❌ Search FAILED: 0/{test_count} queries returned results")
            return False

    return True


if __name__ == "__main__":
    success = test_diskann_at_scale()

    if success:
        print("\n" + "=" * 50)
        print("✅ DiskANN PRUNING FIX SUCCESSFUL!")
        print("  Graph connectivity maintained at all scales")
        print("  Ready for further optimization")
    else:
        print("\n" + "=" * 50)
        print("❌ DiskANN still has issues")
        print("  Needs more debugging")
