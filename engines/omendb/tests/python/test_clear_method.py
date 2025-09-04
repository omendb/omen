#!/usr/bin/env python3
"""Comprehensive tests for the clear() method."""

import sys
import os
import pytest
import numpy as np

# Add parent directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))

import omendb
from omendb import DB


class TestClearMethod:
    """Test suite for the clear() method."""

    def setup_method(self):
        """Set up test fixtures - ensure clean state before each test."""
        # Reset database state for test isolation
        db = DB()
        try:
            db.clear()
        except Exception:
            pass  # clear() might fail in some edge cases

    def test_clear_empty_database(self):
        """Test clearing an empty database."""
        db = DB()

        # Clear empty DB should work
        result = db.clear()
        assert result == True
        assert db.count() == 0

    def test_clear_basic_functionality(self):
        """Test basic clear functionality."""
        db = DB()

        # Add some vectors
        db.add("vec1", [0.1, 0.2, 0.3])
        db.add("vec2", [0.4, 0.5, 0.6])
        assert db.count() == 2

        # Clear database
        result = db.clear()
        assert result == True
        assert db.count() == 0

        # Should be able to add new vectors
        db.add("vec3", [0.7, 0.8, 0.9])
        assert db.count() == 1

        # Search should work
        results = db.search([0.7, 0.8, 0.9], limit=1)
        assert len(results) == 1
        assert results[0].id == "vec3"

    def test_clear_with_metadata(self):
        """Test clearing vectors with metadata."""
        db = DB()

        # Add vectors with metadata
        db.add("v1", [1.0, 2.0, 3.0], {"type": "test", "value": "1"})
        db.add("v2", [4.0, 5.0, 6.0], {"type": "test", "value": "2"})

        # Clear
        db.clear()
        assert db.count() == 0

        # Add new vector with metadata
        db.add("v3", [7.0, 8.0, 9.0], {"type": "new", "value": "3"})

        # Check metadata works after clear
        results = db.search([7.0, 8.0, 9.0], limit=1)
        assert results[0].metadata["type"] == "new"

    def test_clear_dimension_reset(self):
        """Test that dimension is properly reset after clear."""
        db = DB()

        # Add 3D vectors
        db.add("v1_3d", [0.1, 0.2, 0.3])
        info = db.info()
        assert info.get("dimension") == 3

        # Clear and add 128D vectors
        db.clear()
        db.add("v1_128d", np.random.rand(128).tolist())
        info = db.info()
        assert info.get("dimension") == 128

        # Clear and add 64D vectors
        db.clear()
        db.add("v1_64d", np.random.rand(64).tolist())
        info = db.info()
        assert info.get("dimension") == 64

    def test_clear_large_dataset(self):
        """Test clearing a large dataset."""
        db = DB()

        # Add many vectors
        dim = 128
        num_vectors = 1000

        for i in range(num_vectors):
            db.add(f"vec_{i}", np.random.rand(dim).tolist())

        assert db.count() == num_vectors

        # Clear should handle large datasets
        result = db.clear()
        assert result == True
        assert db.count() == 0

        # Should be able to add after clearing large dataset
        db.add("new_vec", np.random.rand(dim).tolist())
        assert db.count() == 1

    def test_multiple_clear_cycles(self):
        """Test multiple add/clear cycles."""
        db = DB()

        # Multiple cycles of add and clear
        for i in range(5):
            # Add a vector
            db.add(f"vec_{i}", [float(i), float(i + 1), float(i + 2)])
            assert db.count() == 1

            # Clear
            result = db.clear()
            assert result == True
            assert db.count() == 0

    @pytest.mark.skip(
        reason="Collections API disabled in v0.1.0 due to Mojo language limitations"
    )
    def test_clear_collections(self):
        """Test clear on collections."""
        # Collections API is disabled in v0.1.0 due to Mojo module-level variable limitations
        # This test will be re-enabled when Collections are supported
        pass

    def test_clear_memory_efficiency(self):
        """Test that clear properly releases memory."""
        db = DB()

        # Add many large vectors
        dim = 1024
        num_vectors = 100

        for i in range(num_vectors):
            db.add(f"large_vec_{i}", np.random.rand(dim).tolist())

        # Get memory usage before clear
        info_before = db.info()
        memory_before = info_before.get("memory_usage_mb", 0)

        # Clear
        db.clear()

        # Memory should be significantly reduced
        info_after = db.info()
        memory_after = info_after.get("memory_usage_mb", 0)

        # After clear, memory should be minimal
        assert memory_after < memory_before * 0.1  # Less than 10% of original


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
