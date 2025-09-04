#!/usr/bin/env python3
"""Debug the get() issue with auto-batching."""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

import omendb
import random


def generate_vector(dim=128):
    """Generate random vector."""
    return [random.random() for _ in range(dim)]


def main():
    print("Testing get() with auto-batching")
    print("=" * 50)

    db = omendb.DB(algorithm="diskann")
    db.clear()

    # Ensure auto-batching is enabled
    db._auto_batch_enabled = True
    print(f"Auto-batch enabled: {db._auto_batch_enabled}")

    # Add a vector
    test_vec = generate_vector()
    print(f"\nAdding vector 'test_1'...")
    db.add("test_1", test_vec)
    print(f"Pending batch size after add: {len(db._pending_batch)}")

    # Try to get it
    print(f"\nGetting vector 'test_1'...")
    print(f"Pending batch size before get: {len(db._pending_batch)}")
    result = db.get("test_1")
    print(f"Pending batch size after get: {len(db._pending_batch)}")

    if result is None:
        print("❌ Vector not found!")
        print("\nTrying manual flush...")
        db.flush()
        result = db.get("test_1")
        if result is not None:
            print("✅ Vector found after manual flush")
        else:
            print("❌ Vector still not found after manual flush")
    else:
        print("✅ Vector found!")

    # Test with multiple vectors
    print("\n" + "=" * 50)
    print("Testing with multiple vectors...")
    db.clear()

    for i in range(5):
        db.add(f"vec_{i}", generate_vector())
        print(f"Added vec_{i}, pending: {len(db._pending_batch)}")

    print("\nSearching for vectors...")
    for i in range(5):
        result = db.get(f"vec_{i}")
        if result is None:
            print(f"❌ vec_{i} not found")
        else:
            print(f"✅ vec_{i} found")


if __name__ == "__main__":
    main()
