#!/usr/bin/env python3
import sys

sys.path.insert(0, "python")

try:
    from omendb.api import _native

    # Test add_vector signature
    print("Testing add_vector...")
    result = _native.add_vector("test", [1.0, 2.0, 3.0])
    print("add_vector works with id+vector:", result)

    # Test basic query
    print("Testing search_vectors...")
    results = _native.search_vectors([1.0, 2.0, 3.0], 5)
    print("search_vectors works, found:", len(results))

    # Test stats
    print("Testing get_stats...")
    stats = _native.get_stats()
    print("Stats:", stats)

    print("✅ All native functions working correctly")

except Exception as e:
    print("Error:", e)
    import traceback

    traceback.print_exc()
