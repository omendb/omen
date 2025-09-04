#!/usr/bin/env python3
"""Test graph connectivity issues in DiskANN"""

import omendb
import numpy as np


def test_connectivity():
    """Test graph connectivity as we add vectors"""
    print("TESTING GRAPH CONNECTIVITY")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Use small buffer to ensure DiskANN is used
    db.configure(buffer_size=10)

    dim = 128

    # Add vectors incrementally and test connectivity
    test_points = [10, 20, 50, 100, 200]

    for n in test_points:
        # Generate distinctive vectors (not random - easier to debug)
        vectors = []
        for i in range(n):
            vec = np.zeros(dim, dtype=np.float32)
            # Make each vector distinctive
            vec[i % dim] = 1.0
            vec[(i + 1) % dim] = 0.5
            vec[(i + 2) % dim] = 0.3
            vectors.append(vec)

        # Clear and add vectors
        db.clear()
        for i in range(n):
            db.add(f"vec_{i}", vectors[i])

        # Test connectivity
        correct = 0
        sample_size = min(10, n)
        for i in range(sample_size):
            results = db.search(vectors[i], limit=1)
            if results and results[0].id == f"vec_{i}":
                correct += 1

        accuracy = (correct / sample_size) * 100
        print(f"n={n:3d}: {correct}/{sample_size} correct ({accuracy:.0f}%)")

        if accuracy < 95:
            print(f"  ⚠️ Connectivity breaks at n={n}")

            # Debug: Try to find which vectors are unreachable
            unreachable = []
            for i in range(min(20, n)):
                results = db.search(vectors[i], limit=1)
                if not results or results[0].id != f"vec_{i}":
                    unreachable.append(i)

            if unreachable:
                print(f"  Unreachable vectors: {unreachable[:10]}")

                # Check what they're finding instead
                print("  First 5 mismatches:")
                for idx in unreachable[:5]:
                    results = db.search(vectors[idx], limit=3)
                    if results:
                        found = [r.id for r in results]
                        print(f"    vec_{idx} -> {found}")

    print("\n" + "=" * 60)
    print("TESTING ENTRY POINT UPDATES")
    print("=" * 60)

    # Test if entry point is being updated
    db.clear()

    # Add vectors in batches and check if entry point changes
    n_total = 200
    batch_size = 50

    vectors = np.random.randn(n_total, dim).astype(np.float32)

    for batch_start in range(0, n_total, batch_size):
        batch_end = min(batch_start + batch_size, n_total)

        for i in range(batch_start, batch_end):
            db.add(f"vec_{i}", vectors[i])

        # Test search from first vector (should always be findable)
        results = db.search(vectors[0], limit=1)
        if results and results[0].id == "vec_0":
            print(f"After {batch_end} vectors: vec_0 still findable ✅")
        else:
            found = results[0].id if results else "None"
            print(f"After {batch_end} vectors: vec_0 NOT found (got {found}) ❌")
            print("  Entry point likely disconnected!")


if __name__ == "__main__":
    test_connectivity()
