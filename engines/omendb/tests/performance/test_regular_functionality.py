#!/usr/bin/env python3
"""
Test regular functionality without tiered storage to isolate the issue.
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))
from omendb.api import DB
import numpy as np


def test_regular_functionality():
    """Test regular database functionality without tiered storage."""
    print("=== Testing Regular Functionality (No Tiered Storage) ===")

    # Create database WITHOUT enabling tiered storage
    db = DB()
    db.clear()

    print("1. Adding vectors...")
    for i in range(100):
        vec = np.array([0.1 * i, 0.2 * i, 0.3 * i] + [0.0] * 125, dtype=np.float32)
        result = db.add(f"vec_{i}", vec)
        if not result:
            print(f"❌ Failed to add vector {i}")
            return False

    print("2. Getting stats...")
    stats = db.info()
    print(f"   Vector count: {stats.get('vector_count', 0)}")
    print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")

    print("3. Querying...")
    query_vec = np.array([0.1, 0.2, 0.3] + [0.0] * 125, dtype=np.float32)
    results = db.search(query_vec, 5)
    print(f"   Found {len(results)} results")

    if len(results) != 5:
        print(f"❌ Expected 5 results, got {len(results)}")
        return False

    print("✅ Regular functionality test passed!")
    return True


if __name__ == "__main__":
    test_regular_functionality()
