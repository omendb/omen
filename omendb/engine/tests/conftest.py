"""Pytest configuration for OmenDB tests."""

import pytest
import sys
from pathlib import Path

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent))


@pytest.fixture
def sample_vectors():
    """Provide sample vectors for testing."""
    return [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]


@pytest.fixture
def db_instance():
    """Provide a clean database instance."""
    import omendb

    db = omendb.DB()
    db.clear()
    yield db
    db.clear()
