#!/usr/bin/env python3
"""
Memory Safety Test Suite

Tests for memory safety, particularly around the clear() method which
may have segmentation fault issues. These tests verify that OmenDB
handles memory management correctly in various scenarios.
"""

import sys
import os
import pytest
import gc
import time
import numpy as np

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

from omendb import DB


class TestMemorySafety:
    """Test suite for memory safety and clear() method."""

    def test_clear_empty_database_safety(self):
        """Test that clearing an empty database doesn't segfault."""
        db = DB()

        # Clear empty database multiple times
        for _ in range(5):
            try:
                result = db.clear()
                assert result == True, "clear() should return True"
                assert db.count() == 0, "Database should be empty after clear"
            except Exception as e:
                pytest.fail(f"clear() on empty database failed: {e}")

    def test_clear_after_single_vector_safety(self):
        """Test clearing database with single vector."""
        db = DB()

        try:
            # Add single vector
            db.clear()  # Start clean
            db.add("test_vec", [1.0, 2.0, 3.0])
            assert db.count() == 1

            # Clear should work safely
            result = db.clear()
            assert result == True
            assert db.count() == 0

            # Should be able to add again after clear
            db.add("new_vec", [4.0, 5.0, 6.0])
            assert db.count() == 1

        except Exception as e:
            pytest.fail(f"Single vector clear test failed: {e}")

    def test_clear_after_many_vectors_safety(self):
        """Test clearing database with many vectors."""
        db = DB()

        try:
            db.clear()  # Start clean

            # Add many vectors
            n_vectors = 1000
            for i in range(n_vectors):
                db.add(f"vec_{i}", [float(i), float(i + 1), float(i + 2)])

            assert db.count() == n_vectors

            # Clear large dataset
            result = db.clear()
            assert result == True
            assert db.count() == 0

            # Memory should be released (test by adding new vectors)
            db.add("after_clear", [1.0, 2.0, 3.0])
            assert db.count() == 1

        except Exception as e:
            pytest.fail(f"Many vectors clear test failed: {e}")

    def test_clear_with_high_dimensional_vectors(self):
        """Test clearing with high-dimensional vectors (potential memory issues)."""
        db = DB()

        try:
            db.clear()  # Start clean

            # Add high-dimensional vectors
            dim = 1536  # OpenAI embedding size
            n_vectors = 100

            for i in range(n_vectors):
                vector = np.random.randn(dim).astype(np.float32).tolist()
                db.add(f"high_dim_{i}", vector)

            assert db.count() == n_vectors

            # Clear high-dimensional data
            result = db.clear()
            assert result == True
            assert db.count() == 0

            # Should be able to add different dimension after clear
            db.add("low_dim", [1.0, 2.0, 3.0])  # 3D after 1536D
            info = db.info()
            assert info.get("dimension") == 3

        except Exception as e:
            pytest.fail(f"High-dimensional clear test failed: {e}")

    def test_repeated_clear_cycles_safety(self):
        """Test repeated add/clear cycles for memory leaks."""
        db = DB()

        try:
            # Multiple add/clear cycles
            for cycle in range(10):
                db.clear()  # Start each cycle clean

                # Add vectors
                for i in range(100):
                    db.add(
                        f"cycle_{cycle}_vec_{i}", [float(i), float(i + 1), float(i + 2)]
                    )

                assert db.count() == 100

                # Clear
                result = db.clear()
                assert result == True
                assert db.count() == 0

                # Force garbage collection
                gc.collect()

        except Exception as e:
            pytest.fail(f"Repeated clear cycles failed at some point: {e}")

    def test_clear_with_metadata_safety(self):
        """Test clearing vectors with complex metadata."""
        db = DB()

        try:
            db.clear()  # Start clean

            # Add vectors with various metadata
            complex_metadata = {
                "text": "This is a long text field with various characters: !@#$%^&*()",
                "numbers": [1, 2, 3, 4, 5],
                "nested": {"key": "value", "number": 42},
                "unicode": "Hello 世界 🌍",
            }

            for i in range(50):
                metadata = complex_metadata.copy()
                metadata["id"] = i
                db.add(
                    f"meta_vec_{i}", [float(i), float(i + 1), float(i + 2)], metadata
                )

            assert db.count() == 50

            # Clear with complex metadata
            result = db.clear()
            assert result == True
            assert db.count() == 0

        except Exception as e:
            pytest.fail(f"Complex metadata clear test failed: {e}")

    def test_clear_during_search_operations(self):
        """Test clearing database while search operations might be cached."""
        db = DB()

        try:
            db.clear()  # Start clean

            # Add vectors and perform searches to potentially cache results
            vectors = []
            for i in range(500):
                vector = [float(i), float(i + 1), float(i + 2)]
                vectors.append(vector)
                db.add(f"search_vec_{i}", vector)

            # Perform several searches (might cache results internally)
            for i in range(0, len(vectors), 50):
                results = db.search(vectors[i], limit=10)
                assert len(results) > 0

            # Now clear (potential issue if search results are cached)
            result = db.clear()
            assert result == True
            assert db.count() == 0

            # Search on empty database should return empty results
            empty_results = db.search([1.0, 2.0, 3.0], limit=10)
            assert len(empty_results) == 0

        except Exception as e:
            pytest.fail(f"Clear during search operations failed: {e}")

    def test_clear_with_different_algorithms(self):
        """Test clear with both brute force and HNSW algorithms."""
        db = DB()

        try:
            # Test 1: Clear with brute force (< 5K vectors)
            db.clear()

            for i in range(1000):  # Under migration threshold
                db.add(f"bf_vec_{i}", [float(i), float(i + 1), float(i + 2)])

            assert db.count() == 1000
            result = db.clear()
            assert result == True
            assert db.count() == 0

            # Test 2: Clear with HNSW (> 5K vectors)
            vectors = np.random.randn(6000, 128).astype(np.float32)
            ids = [f"hnsw_vec_{i}" for i in range(6000)]

            # Use batch add to quickly get to HNSW
            db.add_batch(vectors=vectors, ids=ids)
            assert db.count() == 6000

            # Clear HNSW database
            result = db.clear()
            assert result == True
            assert db.count() == 0

        except Exception as e:
            pytest.fail(f"Clear with different algorithms failed: {e}")

    @pytest.mark.stress
    def test_clear_stress_test(self):
        """Stress test for clear() method with large datasets."""
        db = DB()

        try:
            db.clear()  # Start clean

            # Large dataset stress test
            n_vectors = 10000
            dim = 256

            print(f"Adding {n_vectors} vectors of dimension {dim}...")

            # Generate large dataset
            vectors = np.random.randn(n_vectors, dim).astype(np.float32)
            ids = [f"stress_vec_{i}" for i in range(n_vectors)]

            # Add in batches for speed
            batch_size = 1000
            for i in range(0, n_vectors, batch_size):
                end_idx = min(i + batch_size, n_vectors)
                batch_vectors = vectors[i:end_idx]
                batch_ids = ids[i:end_idx]

                for j, (vec, vec_id) in enumerate(zip(batch_vectors, batch_ids)):
                    db.add(vec_id, vec.tolist())

            assert db.count() == n_vectors
            print(f"Successfully added {n_vectors} vectors")

            # Clear large dataset
            print("Clearing large dataset...")
            start_time = time.time()
            result = db.clear()
            clear_time = time.time() - start_time

            assert result == True
            assert db.count() == 0
            print(f"Clear completed in {clear_time:.2f} seconds")

            # Verify memory is released by adding new vectors
            db.add("post_stress", [1.0, 2.0, 3.0])
            assert db.count() == 1

        except Exception as e:
            pytest.fail(f"Stress test failed: {e}")

    def test_memory_doesnt_grow_unbounded(self):
        """Test that repeated operations don't cause unbounded memory growth."""
        db = DB()

        try:
            # Get initial memory usage
            initial_info = db.info()
            initial_memory = initial_info.get("memory_usage_mb", 0)

            # Repeated add/clear cycles
            for cycle in range(5):
                db.clear()

                # Add moderate number of vectors
                for i in range(200):
                    db.add(
                        f"mem_test_{cycle}_{i}", [float(i), float(i + 1), float(i + 2)]
                    )

                # Check memory hasn't grown excessively
                cycle_info = db.info()
                cycle_memory = cycle_info.get("memory_usage_mb", 0)

                # Memory should not grow more than 10x from initial
                assert cycle_memory < initial_memory + 50, (
                    f"Memory grew too much: {cycle_memory}MB"
                )

                # Clear for next cycle
                db.clear()

        except Exception as e:
            pytest.fail(f"Memory growth test failed: {e}")


if __name__ == "__main__":
    # Run memory safety tests
    pytest.main([__file__, "-v", "-s"])
