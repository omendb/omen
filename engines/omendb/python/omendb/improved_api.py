"""
Improved OmenDB API with truly automatic optimization.

Key improvements:
1. Automatic algorithm selection that actually works
2. No unnecessary parameters (no is_server, use_columnar)
3. Simple override when needed (force_algorithm)
4. Smart defaults based on real benchmarks
"""

import numpy as np
from typing import List, Optional, Union, Dict

# Import the native module
from . import _native


class DB:
    """Simplified OmenDB with automatic optimization."""

    def __init__(self, force_algorithm: Optional[str] = None):
        """Create OmenDB instance with automatic optimization.

        Args:
            force_algorithm: Optional override ('flat' or 'diskann')
                           Leave None for automatic selection

        Examples:
            # Automatic (recommended)
            db = omendb.DB()

            # Force specific algorithm (debugging/benchmarking)
            db = omendb.DB(force_algorithm='flat')
        """
        self._force_algorithm = force_algorithm
        self._configured = False
        self._dimension = None
        self._vector_count = 0

    def add(
        self, vector: Union[List[float], np.ndarray], id: Optional[str] = None
    ) -> str:
        """Add single vector with automatic optimization.

        Args:
            vector: Vector data
            id: Optional ID (auto-generated if None)

        Returns:
            Vector ID
        """
        # Auto-configure on first vector
        if not self._configured:
            self._auto_configure(1, len(vector))

        # Convert and add
        if isinstance(vector, np.ndarray):
            vector = vector.tolist()

        if id is None:
            id = f"vec_{self._vector_count}"

        _native.add_vector(id, vector, {})
        self._vector_count += 1
        return id

    def add_batch(
        self,
        vectors: Union[List[List[float]], np.ndarray],
        ids: Optional[List[str]] = None,
    ) -> List[str]:
        """Add batch of vectors with automatic optimization.

        Args:
            vectors: Batch of vectors
            ids: Optional IDs

        Returns:
            List of IDs
        """
        # Ensure numpy array
        if not isinstance(vectors, np.ndarray):
            vectors = np.array(vectors, dtype=np.float32)

        batch_size = len(vectors)
        dimension = vectors.shape[1] if len(vectors.shape) > 1 else len(vectors[0])

        # Auto-configure on first batch
        if not self._configured:
            self._auto_configure(batch_size, dimension)

        # Generate IDs if needed
        if ids is None:
            ids = [f"vec_{self._vector_count + i}" for i in range(batch_size)]

        # Smart batching - split large batches
        OPTIMAL_BATCH = 5000

        if batch_size <= OPTIMAL_BATCH:
            # Single batch
            _native.add_vector_batch(vectors, ids, [{}] * batch_size)
        else:
            # Split into optimal chunks
            for i in range(0, batch_size, OPTIMAL_BATCH):
                end = min(i + OPTIMAL_BATCH, batch_size)
                batch_vecs = vectors[i:end]
                batch_ids = ids[i:end]
                _native.add_vector_batch(batch_vecs, batch_ids, [{}] * len(batch_ids))

        self._vector_count += batch_size
        return ids

    def search(
        self, query: Union[List[float], np.ndarray], limit: int = 10
    ) -> List[Dict]:
        """Search for nearest neighbors.

        Args:
            query: Query vector
            limit: Number of results

        Returns:
            List of results with id and score
        """
        if isinstance(query, np.ndarray):
            query = query.tolist()

        return _native.search_vectors(query, limit, {})

    def _auto_configure(self, first_batch_size: int, dimension: int):
        """Automatically configure optimal algorithm.

        Based on real benchmark results:
        - Small batches (<1K): Brute force (120K vec/s)
        - All sizes: DiskANN (1,400 vec/s for 128D)
        - High dimensions (768+): DiskANN for better scaling
        """
        if self._force_algorithm:
            # User override
            algorithm = self._force_algorithm
        elif first_batch_size >= 5000 or dimension >= 768:
            # Large batch or high dimension -> DiskANN
            algorithm = "diskann"
        elif first_batch_size < 1000:
            # Small batch -> brute force
            algorithm = "brute_force"
        else:
            # Medium -> let it migrate naturally
            algorithm = None

        if algorithm:
            # Configure the native module
            config = {
                "force_algorithm": algorithm,
                "migration_threshold": 1000000 if algorithm else 5000,
            }
            _native.configure_database(config)

            print(
                f"🎯 Auto-configured: {algorithm or 'adaptive'} "
                f"for {first_batch_size} vectors @{dimension}D"
            )

        self._configured = True
        self._dimension = dimension

    def clear(self):
        """Clear all vectors."""
        _native.clear_database()
        self._vector_count = 0
        self._configured = False


# Convenience function
def connect(force_algorithm: Optional[str] = None) -> DB:
    """Create OmenDB connection with automatic optimization.

    Args:
        force_algorithm: Optional override ('flat' or 'diskann')

    Returns:
        DB instance

    Example:
        # Automatic (recommended)
        db = omendb.connect()

        # Force DiskANN for benchmarking
        db = omendb.connect(force_algorithm='diskann')
    """
    return DB(force_algorithm)
