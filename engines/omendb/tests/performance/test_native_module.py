#!/usr/bin/env python3
"""Test direct native module access for FFI validation"""

import sys

sys.path.insert(0, "/Users/nick/github/omendb/omenDB/python")


def test_native_module():
    print("=== Testing Native Module Direct Access ===")

    try:
        # Import the native module directly
        import omendb.native as native

        print("✅ Native module imported successfully")

        # Test basic operations
        print("\n1. Testing add_vector...")
        success = native.add_vector("test_vec_1", [1.0] * 128, {})
        print(f"   add_vector result: {success}")

        print("\n2. Testing search_vectors...")
        results = native.search_vectors([1.0] * 128, 5, {})  # Added empty filter dict
        print(f"   Found {len(results)} results")
        for i, result in enumerate(results[:3]):
            print(f"   [{i}] {result}")

        print("\n3. Testing get_stats...")
        stats = native.info()
        print(f"   Stats: {stats}")

        print("\n✅ Native module access successful!")
        return True

    except Exception as e:
        print(f"❌ Error accessing native module: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    test_native_module()
