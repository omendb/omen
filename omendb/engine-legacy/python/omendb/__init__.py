"""
OmenDB Python Bindings

High-performance embedded vector database with native Mojo performance.

Features:
- SIMD-optimized vector operations
- DiskANN algorithm - no rebuilds ever needed
- Multiple distance metrics (L2, cosine, inner product, L2 squared)
- Zero-overhead metrics export
- Cross-platform compatibility

Example:
    >>> from omendb import DB
    >>> db = DB("vectors.omen")
    >>> db.add("doc1", [0.1, 0.2, 0.3])
    >>> results = db.search([0.1, 0.2, 0.3], limit=5)
"""

__version__ = "0.1.2"
__author__ = "nijaru"


# Configure default logging
def _configure_default_logging():
    """Configure default logging settings."""
    try:
        from .logging import configure_logging, LogLevel

        # Configure logging with INFO level by default
        configure_logging(level=LogLevel.INFO, enable_metrics=False)
    except Exception:
        # Silently fail if logging setup fails
        # Core functionality should still work
        pass


# Initialize logging on module import
_configure_default_logging()

# Import main classes directly
from .api import DB, SearchResult
from .exceptions import OmenDBError, DatabaseError, ValidationError

# Import utility functions
from .logging import configure_logging

# Import Arrow integration (adds methods to DB class if available)
try:
    from . import arrow_integration

    _ARROW_INTEGRATION_LOADED = True
except ImportError:
    _ARROW_INTEGRATION_LOADED = False

# Simple, clean exports
__all__ = [
    # Core classes
    "DB",
    "SearchResult",
    # Exceptions
    "OmenDBError",
    "DatabaseError",
    "ValidationError",
    # Utilities
    "configure_logging",
    # Metadata
    "__version__",
    "__author__",
]
