"""Test dimension validation in OmenDB.

Tests for dimension mismatches that can occur in real-world usage:
- Switching embedding models
- Loading databases with different dimensions
- Mixing vectors of different dimensions
"""

import pytest
import numpy as np
import tempfile
import os
from omendb import DB, ValidationError


class TestDimensionValidation:
    """Test dimension handling and validation."""

    def test_consistent_dimensions_work(self):
        """Test that vectors with same dimension work correctly."""
        db = DB()

        # Add 128D vectors
        db.add("vec1", [1.0] * 128)
        db.add("vec2", [2.0] * 128)

        # Query with 128D vector should work
        results = db.search([1.0] * 128, limit=2)
        assert len(results) == 2

    def test_dimension_mismatch_on_add(self):
        """Test adding vectors with different dimensions."""
        db = DB()

        # First vector sets dimension
        assert db.add("vec1", [1.0] * 128)

        # Adding different dimension should fail
        with pytest.raises(ValidationError, match="Dimension mismatch"):
            db.add("vec2", [1.0] * 256)

    def test_dimension_mismatch_on_query(self):
        """Test querying with wrong dimension."""
        db = DB()

        # Add 128D vector
        db.add("vec1", [1.0] * 128)

        # Query with 256D should fail
        with pytest.raises(ValidationError, match="Dimension mismatch"):
            db.search([1.0] * 256, limit=1)

    def test_empty_db_accepts_any_dimension(self):
        """Test that empty DB accepts first vector of any dimension."""
        db = DB()

        # Should accept any dimension when empty
        results = db.search([1.0] * 512, limit=1)
        assert len(results) == 0  # No results, but no error

        # First add sets dimension
        assert db.add("vec1", [1.0] * 384)

        # Now only 384D should work
        assert db.add("vec2", [2.0] * 384)
        with pytest.raises(ValidationError):
            db.add("vec3", [3.0] * 128)

    def test_dimension_persisted_across_reload(self):
        """Test dimension validation with persisted database."""
        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Create DB with 256D vectors
            db1 = DB(db_path)
            db1.add("vec1", [1.0] * 256)
            db1.add("vec2", [2.0] * 256)
            db1.save()

            # Reload database
            db2 = DB(db_path)

            # Should enforce 256D
            assert db2.add("vec3", [3.0] * 256)

            # Should reject other dimensions
            with pytest.raises(ValidationError, match="Dimension mismatch"):
                db2.add("vec4", [4.0] * 128)

            with pytest.raises(ValidationError, match="Dimension mismatch"):
                db2.search([1.0] * 512, limit=1)
        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)

    def test_batch_add_dimension_validation(self):
        """Test dimension validation in batch operations."""
        db = DB()

        # Add first vector to set dimension
        db.add("vec1", [1.0] * 128)

        # Batch with mixed dimensions should fail
        batch_data = [
            ("vec2", [2.0] * 128, None),  # Correct dimension
            ("vec3", [3.0] * 256, None),  # Wrong dimension
            ("vec4", [4.0] * 128, None),  # Correct dimension
        ]

        with pytest.raises(ValidationError, match="Dimension mismatch"):
            db.add_batch(batch_data)

    def test_real_world_embedding_switch(self):
        """Test real-world scenario: switching embedding models."""
        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Day 1: Using all-MiniLM-L6-v2 (384 dimensions)
            db = DB(db_path)
            db.add("doc1", np.random.randn(384).tolist())
            db.add("doc2", np.random.randn(384).tolist())
            db.save()

            # Day 2: Switch to text-embedding-ada-002 (1536 dimensions)
            db2 = DB(db_path)

            # This should fail with clear error
            with pytest.raises(ValidationError) as exc_info:
                db2.add("doc3", np.random.randn(1536).tolist())

            # Error should be helpful
            assert "384D" in str(exc_info.value)
            assert "1536D" in str(exc_info.value)
        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)

    def test_dimension_in_stats(self):
        """Test that stats() reports the expected dimension."""
        db = DB()

        # Empty DB should not have dimension
        stats = db.info()
        assert "expected_dimension" not in stats or stats["expected_dimension"] is None

        # After adding vector, should report dimension
        db.add("vec1", [1.0] * 512)
        stats = db.info()
        assert stats.get("expected_dimension") == 512

    def test_numpy_array_dimension_validation(self):
        """Test dimension validation with NumPy arrays."""
        db = DB()

        # Add with NumPy array
        db.add("vec1", np.ones(768))

        # Query with wrong dimension NumPy array
        with pytest.raises(ValidationError, match="Dimension mismatch"):
            db.search(np.ones(384), limit=1)

    @pytest.mark.skipif(not hasattr(pytest, "torch"), reason="PyTorch not installed")
    def test_torch_tensor_dimension_validation(self):
        """Test dimension validation with PyTorch tensors."""
        import torch

        db = DB()

        # Add with torch tensor
        db.add("vec1", torch.ones(256))

        # Query with wrong dimension tensor
        with pytest.raises(ValidationError, match="Dimension mismatch"):
            db.search(torch.ones(512), limit=1)


if __name__ == "__main__":
    # Run tests
    pytest.main([__file__, "-v"])
