#!/usr/bin/env python3
"""Debug buffer behavior"""

import omendb
import numpy as np


def test_buffer_debug():
    """Test to see if buffer is actually being used"""
    print("DEBUGGING BUFFER BEHAVIOR")
    print("=" * 60)

    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=100)

    # Add vectors and check stats
    dim = 128

    print("\nAdding vectors and checking buffer status:")
    for i in range(150):
        vec = np.random.randn(dim).astype(np.float32)
        db.add(f"vec_{i}", vec)

        if i in [0, 50, 99, 100, 149]:
            stats = db.info()
            buffer_size = stats.get("buffer_size", 0)
            main_size = stats.get("main_index_size", 0)
            print(
                f"After {i + 1:3d} vectors: buffer={buffer_size}, main_index={main_size}"
            )

    print("\nFinal database stats:")
    final_stats = db.info()
    for key, value in final_stats.items():
        if "buffer" in key.lower() or "index" in key.lower() or "vector" in key.lower():
            print(f"  {key}: {value}")


if __name__ == "__main__":
    test_buffer_debug()
