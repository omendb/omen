#!/usr/bin/env python3
"""
NumPy Optimization Regression Test
==================================

Ensures that NumPy arrays perform better than Python lists for batch operations.
This test MUST pass for releases to ensure we maintain the optimization.

Expected Performance:
- Python lists: ~90,000 vec/s
- NumPy arrays: ~158,000 vec/s (1.8x faster)
"""

import time
import numpy as np
import pytest
from omendb import DB


class TestNumpyOptimization:
    """Test suite to ensure NumPy optimization is working."""

    def setup_method(self):
        """Set up test database."""
        self.db = DB()
        self.dimension = 128
        self.test_size = 10000  # Large enough to measure performance

    def teardown_method(self):
        """Clean up after tests."""
        if hasattr(self, "db"):
            self.db.clear()

    def test_numpy_faster_than_lists(self):
        """Verify NumPy arrays are faster than Python lists."""
        # Generate test data
        numpy_vectors = np.random.rand(self.test_size, self.dimension).astype(
            np.float32
        )
        ids = [f"vec_{i}" for i in range(self.test_size)]
        metadata = [{} for _ in range(self.test_size)]

        # Test 1: Python lists (converting from NumPy)
        self.db.clear()
        list_vectors = [numpy_vectors[i].tolist() for i in range(self.test_size)]

        start = time.perf_counter()
        self.db.add_batch(vectors=list_vectors, ids=ids, metadata=metadata)
        list_time = time.perf_counter() - start
        list_rate = self.test_size / list_time

        # Test 2: NumPy arrays (direct)
        self.db.clear()

        start = time.perf_counter()
        self.db.add_batch(vectors=numpy_vectors, ids=ids, metadata=metadata)
        numpy_time = time.perf_counter() - start
        numpy_rate = self.test_size / numpy_time

        # Calculate speedup
        speedup = numpy_rate / list_rate

        # Print results for debugging
        print(f"\nPerformance Results:")
        print(f"  Python lists: {list_rate:,.0f} vec/s")
        print(f"  NumPy arrays: {numpy_rate:,.0f} vec/s")
        print(f"  Speedup: {speedup:.2f}x")

        # Assertions
        assert numpy_time < list_time, "NumPy should be faster than lists"
        assert speedup > 1.5, (
            f"NumPy should be at least 1.5x faster, got {speedup:.2f}x"
        )

        # Performance thresholds (conservative to avoid flaky tests)
        assert list_rate > 50000, f"List performance too low: {list_rate:.0f} vec/s"
        assert numpy_rate > 80000, f"NumPy performance too low: {numpy_rate:.0f} vec/s"

    def test_numpy_query_performance(self):
        """Verify NumPy arrays work correctly for queries."""
        # Add some vectors
        vectors = np.random.rand(1000, self.dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(1000)]
        self.db.add_batch(vectors=vectors, ids=ids, metadata=[{} for _ in range(1000)])

        # Query with NumPy array
        query_numpy = np.random.rand(self.dimension).astype(np.float32)

        start = time.perf_counter()
        results_numpy = self.db.search(query_numpy, limit=10)
        numpy_query_time = time.perf_counter() - start

        # Query with list
        query_list = query_numpy.tolist()

        start = time.perf_counter()
        results_list = self.db.search(query_list, limit=10)
        list_query_time = time.perf_counter() - start

        # Both should return same results
        assert len(results_numpy) == len(results_list)
        assert results_numpy[0].id == results_list[0].id

        # Query times should be similar (no major optimization expected here)
        print(f"\nQuery Performance:")
        print(f"  NumPy query: {numpy_query_time * 1000:.2f}ms")
        print(f"  List query: {list_query_time * 1000:.2f}ms")

    def test_batch_size_scaling(self):
        """Test that NumPy optimization scales with batch size."""
        batch_sizes = [100, 1000, 5000]
        speedups = []

        for size in batch_sizes:
            # Generate test data
            numpy_vectors = np.random.rand(size, self.dimension).astype(np.float32)
            list_vectors = [numpy_vectors[i].tolist() for i in range(size)]
            ids = [f"vec_{i}" for i in range(size)]
            metadata = [{} for _ in range(size)]

            # Test lists
            self.db.clear()
            start = time.perf_counter()
            self.db.add_batch(vectors=list_vectors, ids=ids, metadata=metadata)
            list_time = time.perf_counter() - start

            # Test NumPy
            self.db.clear()
            start = time.perf_counter()
            self.db.add_batch(vectors=numpy_vectors, ids=ids, metadata=metadata)
            numpy_time = time.perf_counter() - start

            speedup = list_time / numpy_time
            speedups.append(speedup)

            print(f"\nBatch size {size}: {speedup:.2f}x speedup")

        # Speedup should be consistent across batch sizes
        assert all(s > 1.3 for s in speedups), (
            "NumPy should be faster at all batch sizes"
        )

        # Speedup should increase with larger batches
        assert speedups[-1] >= speedups[0], "Speedup should improve with larger batches"


if __name__ == "__main__":
    # Run the test
    test = TestNumpyOptimization()
    test.setup_method()

    try:
        print("Running NumPy optimization regression tests...")
        test.test_numpy_faster_than_lists()
        test.test_numpy_query_performance()
        test.test_batch_size_scaling()
        print("\n✅ All tests passed! NumPy optimization is working correctly.")
    except AssertionError as e:
        print(f"\n❌ Test failed: {e}")
        raise
    finally:
        test.teardown_method()
