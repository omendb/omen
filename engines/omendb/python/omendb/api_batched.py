"""
Auto-batching API wrapper for OmenDB.

This wrapper automatically batches individual add operations
to reduce FFI overhead. Instead of calling native code for
each vector, it accumulates them and flushes in batches.
"""

import numpy as np
from typing import List, Optional, Dict, Union, Any
from dataclasses import dataclass
import threading
import time

# Import native module
try:
    from omendb import native as _native
except ImportError:
    raise ImportError(
        "Failed to import native module. Ensure OmenDB is properly installed."
    )


class AutoBatchDB:
    """
    Database interface with automatic batching.

    Individual add() calls are accumulated and automatically
    flushed when:
    1. Batch size threshold is reached
    2. Time threshold is exceeded
    3. Explicitly flushed
    4. Search is performed
    """

    def __init__(self, batch_size: int = 100, batch_timeout: float = 0.1):
        """
        Initialize auto-batching database.

        Args:
            batch_size: Number of vectors to accumulate before auto-flush
            batch_timeout: Maximum time (seconds) before auto-flush
        """
        self._batch_size = batch_size
        self._batch_timeout = batch_timeout

        # Batch accumulator
        self._pending_ids = []
        self._pending_vectors = []
        self._pending_metadata = []
        self._last_flush = time.time()

        # Thread safety
        self._lock = threading.Lock()

        # State
        self._initialized = False
        self._dimension = None

        # Stats
        self._total_adds = 0
        self._total_flushes = 0

    def _ensure_initialized(self):
        """Lazy initialization."""
        if not self._initialized:
            _native.clear_database()
            self._initialized = True

    def _flush_batch(self) -> int:
        """
        Flush pending batch to native storage.

        Returns:
            Number of vectors flushed
        """
        if not self._pending_ids:
            return 0

        # Convert pending vectors to numpy array for batch operation
        try:
            # Stack vectors into 2D array
            vectors_array = np.array(self._pending_vectors, dtype=np.float32)

            # Prepare metadata
            metadata_list = self._pending_metadata or [{}] * len(self._pending_ids)

            # Call native batch function
            _native.add_vector_batch(self._pending_ids, vectors_array, metadata_list)

            count = len(self._pending_ids)

            # Clear pending
            self._pending_ids.clear()
            self._pending_vectors.clear()
            self._pending_metadata.clear()

            # Update stats
            self._total_flushes += 1
            self._last_flush = time.time()

            return count

        except Exception as e:
            # On error, clear pending to avoid retrying bad data
            self._pending_ids.clear()
            self._pending_vectors.clear()
            self._pending_metadata.clear()
            raise RuntimeError(f"Batch flush failed: {e}") from e

    def add(
        self,
        id: str,
        vector: Union[List[float], np.ndarray],
        metadata: Optional[Dict[str, str]] = None,
    ) -> bool:
        """
        Add a vector (automatically batched).

        This method accumulates vectors and flushes them in batches
        to reduce FFI overhead.
        """
        self._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValueError("Vector ID must be a non-empty string")

        with self._lock:
            # Convert vector to list if numpy
            if isinstance(vector, np.ndarray):
                vector_list = vector.tolist()
            else:
                vector_list = list(vector)

            # Track dimension
            if self._dimension is None:
                self._dimension = len(vector_list)
            elif len(vector_list) != self._dimension:
                raise ValueError(
                    f"Dimension mismatch: expected {self._dimension}, got {len(vector_list)}"
                )

            # Add to pending
            self._pending_ids.append(id)
            self._pending_vectors.append(vector_list)
            self._pending_metadata.append(metadata or {})

            self._total_adds += 1

            # Check if we should flush
            should_flush = False

            # Condition 1: Batch size reached
            if len(self._pending_ids) >= self._batch_size:
                should_flush = True

            # Condition 2: Timeout exceeded
            elif time.time() - self._last_flush > self._batch_timeout:
                should_flush = True

            # Flush if needed
            if should_flush:
                self._flush_batch()

            return True

    def flush(self) -> int:
        """
        Manually flush pending batch.

        Returns:
            Number of vectors flushed
        """
        with self._lock:
            return self._flush_batch()

    def search(self, query: Union[List[float], np.ndarray], limit: int = 10) -> List:
        """
        Search for similar vectors.

        Note: Flushes pending batch before searching to ensure
        all vectors are searchable.
        """
        self._ensure_initialized()

        # Flush pending vectors before search
        with self._lock:
            self._flush_batch()

        # Convert query to list
        if isinstance(query, np.ndarray):
            query_list = query.tolist()
        else:
            query_list = list(query)

        # Search
        results = _native.search_vectors(query_list, limit, {})

        # Convert results
        output = []
        for r in results:
            output.append(
                {
                    "id": r["id"],
                    "score": r.get("score", 0.0),
                    "metadata": r.get("metadata", {}),
                }
            )

        return output

    def clear(self):
        """Clear database and pending batch."""
        with self._lock:
            # Clear pending
            self._pending_ids.clear()
            self._pending_vectors.clear()
            self._pending_metadata.clear()

            # Clear database
            _native.clear_database()
            self._initialized = True
            self._dimension = None

            # Reset stats
            self._total_adds = 0
            self._total_flushes = 0
            self._last_flush = time.time()

    def size(self) -> int:
        """
        Get total number of vectors (including pending).
        """
        with self._lock:
            db_size = _native.count()
            pending_size = len(self._pending_ids)
            return db_size + pending_size

    def stats(self) -> Dict[str, Any]:
        """
        Get batching statistics.
        """
        with self._lock:
            return {
                "total_adds": self._total_adds,
                "total_flushes": self._total_flushes,
                "pending_count": len(self._pending_ids),
                "average_batch_size": self._total_adds / max(1, self._total_flushes),
                "batch_size_config": self._batch_size,
                "batch_timeout_config": self._batch_timeout,
            }

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit - flush pending."""
        self.flush()
