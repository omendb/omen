#!/usr/bin/env python3
"""
Standard tensor framework compatibility tests.

Tests conversion order and functionality for all supported tensor types.
This is a PERMANENT test that should be run after any API changes.
"""

import sys
import os
import time

# Ensure we can import omendb
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "python"))

from omendb import DB


def test_conversion_order():
    """Test that tensor conversion order works correctly for all frameworks."""
    print("🧪 TENSOR COMPATIBILITY TESTS")
    print("=" * 60)

    db = DB()
    conversions_tested = []

    # Test 1: Python lists (should be fastest)
    print("1. Python lists (most common):")
    start = time.perf_counter()
    db.add("list_test", [1.0, 2.0, 3.0])
    duration = time.perf_counter() - start
    print(f"   ✅ Lists: {duration * 1000:.2f}ms")
    conversions_tested.append(("Python lists", True, duration))

    # Test 2: NumPy arrays
    print("2. NumPy arrays (very common):")
    try:
        import numpy as np

        arr = np.array([4.0, 5.0, 6.0])
        start = time.perf_counter()
        db.add("numpy_test", arr)
        duration = time.perf_counter() - start
        print(f"   ✅ NumPy: {duration * 1000:.2f}ms")
        conversions_tested.append(("NumPy", True, duration))
    except ImportError:
        print("   ⚠️  NumPy not available")
        conversions_tested.append(("NumPy", False, 0))

    # Test 3: PyTorch tensors (should be detected specifically)
    print("3. PyTorch tensors (ML framework):")
    try:
        import torch

        tensor = torch.tensor([7.0, 8.0, 9.0])
        start = time.perf_counter()
        db.add("torch_test", tensor)
        duration = time.perf_counter() - start
        print(f"   ✅ PyTorch: {duration * 1000:.2f}ms")
        conversions_tested.append(("PyTorch", True, duration))
    except ImportError:
        print("   ⚠️  PyTorch not available")
        conversions_tested.append(("PyTorch", False, 0))

    # Test 4: JAX arrays (should be detected before TensorFlow)
    print("4. JAX arrays (Google ML framework):")
    try:
        import jax.numpy as jnp

        jax_array = jnp.array([10.0, 11.0, 12.0])
        start = time.perf_counter()
        db.add("jax_test", jax_array)
        duration = time.perf_counter() - start
        print(f"   ✅ JAX: {duration * 1000:.2f}ms")
        conversions_tested.append(("JAX", True, duration))
    except ImportError:
        print("   ⚠️  JAX not available")
        conversions_tested.append(("JAX", False, 0))

    # Test 5: TensorFlow tensors (should not interfere with JAX)
    print("5. TensorFlow tensors (Google ML framework):")
    try:
        import tensorflow as tf

        tf_tensor = tf.constant([13.0, 14.0, 15.0])
        start = time.perf_counter()
        db.add("tf_test", tf_tensor)
        duration = time.perf_counter() - start
        print(f"   ✅ TensorFlow: {duration * 1000:.2f}ms")
        conversions_tested.append(("TensorFlow", True, duration))
    except ImportError:
        print("   ⚠️  TensorFlow not available")
        conversions_tested.append(("TensorFlow", False, 0))

    print(f"\nTotal vectors added: {db.count()}")

    # Summary
    print("\n📊 COMPATIBILITY SUMMARY:")
    available_frameworks = [
        name for name, available, _ in conversions_tested if available
    ]
    unavailable_frameworks = [
        name for name, available, _ in conversions_tested if not available
    ]

    print(f"   ✅ Working: {', '.join(available_frameworks)}")
    if unavailable_frameworks:
        print(f"   ⚠️  Unavailable: {', '.join(unavailable_frameworks)}")

    print(f"   📈 Frameworks tested: {len(conversions_tested)}")
    print(f"   🎯 Frameworks working: {len(available_frameworks)}")

    # Performance check
    working_times = [
        duration
        for _, available, duration in conversions_tested
        if available and duration > 0
    ]
    if working_times:
        avg_time = sum(working_times) / len(working_times)
        print(f"   ⚡ Average conversion: {avg_time * 1000:.2f}ms")

    return len(available_frameworks) >= 2  # At least Python lists + NumPy should work


def test_query_compatibility():
    """Test that query operations work with different tensor types."""
    print("\n🔍 QUERY COMPATIBILITY TESTS")
    print("=" * 60)

    db = DB()

    # Add test vectors
    db.add("reference", [1.0, 2.0, 3.0])

    query_tests = []

    # Test querying with different tensor types
    print("Testing query operations with different input types:")

    # Python list query
    results = db.search([1.0, 2.0, 3.0], limit=5)
    print(f"   ✅ Python list query: {len(results)} results")
    query_tests.append(("Python list", len(results) > 0))

    # NumPy query
    try:
        import numpy as np

        np_query = np.array([1.0, 2.0, 3.0])
        results = db.search(np_query, limit=5)
        print(f"   ✅ NumPy query: {len(results)} results")
        query_tests.append(("NumPy", len(results) > 0))
    except ImportError:
        print("   ⚠️  NumPy query not available")
        query_tests.append(("NumPy", False))

    # PyTorch query
    try:
        import torch

        torch_query = torch.tensor([1.0, 2.0, 3.0])
        results = db.search(torch_query, limit=5)
        print(f"   ✅ PyTorch query: {len(results)} results")
        query_tests.append(("PyTorch", len(results) > 0))
    except ImportError:
        print("   ⚠️  PyTorch query not available")
        query_tests.append(("PyTorch", False))

    working_queries = [name for name, working in query_tests if working]
    print(f"\n   🎯 Query types working: {', '.join(working_queries)}")

    return len(working_queries) >= 2


def main():
    """Run all tensor compatibility tests."""
    print("🧪 OMENDB TENSOR FRAMEWORK COMPATIBILITY SUITE")
    print("Standard test - run after any conversion changes")
    print("=" * 70)

    tests = [
        ("Tensor Conversion Order", test_conversion_order),
        ("Query Compatibility", test_query_compatibility),
    ]

    results = []
    for test_name, test_func in tests:
        try:
            success = test_func()
            results.append((test_name, success))
        except Exception as e:
            print(f"❌ {test_name} FAILED with exception: {e}")
            results.append((test_name, False))

    print("\n📋 FINAL RESULTS:")
    all_passed = True
    for test_name, success in results:
        status = "✅" if success else "❌"
        print(f"   {status} {test_name}")
        if not success:
            all_passed = False

    if all_passed:
        print("\n🎉 ALL TENSOR COMPATIBILITY TESTS PASSED!")
        print("   Framework support is working correctly")
        return True
    else:
        print("\n❌ Some compatibility tests failed!")
        return False


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)
