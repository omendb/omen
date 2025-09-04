"""
Optimized OmenDB Python API with reduced FFI overhead.

Key optimizations:
1. Skip validation for numpy arrays (trust dtype)
2. Direct numpy array passing when possible
3. Cached module imports
4. Fast-path for common cases
"""

import os
from typing import List, Optional, Dict, Union, Tuple, Any
from dataclasses import dataclass
import warnings
import numpy as np  # Import numpy upfront - it's used in 90% of cases

# Import native module
try:
    from omendb import native as _native
except ImportError:
    raise ImportError(
        "Failed to import native module. Ensure OmenDB is properly installed."
    )

# Type definitions
VectorInput = Union[List[float], "np.ndarray"]

# Cache for expensive imports
_torch_module = None
_tf_module = None
_jax_module = None


def _get_torch():
    """Lazy load PyTorch."""
    global _torch_module
    if _torch_module is None:
        try:
            import torch

            _torch_module = torch
        except ImportError:
            pass
    return _torch_module


def _get_tensorflow():
    """Lazy load TensorFlow."""
    global _tf_module
    if _tf_module is None:
        try:
            import tensorflow as tf

            _tf_module = tf
        except ImportError:
            pass
    return _tf_module


def _get_jax():
    """Lazy load JAX."""
    global _jax_module
    if _jax_module is None:
        try:
            import jax

            _jax_module = jax
        except ImportError:
            pass
    return _jax_module


def _convert_to_vector_fast(data: VectorInput) -> Union[List[float], np.ndarray]:
    """
    Convert various input types to vector with optimizations.

    Key optimization: Return numpy arrays directly when possible,
    avoiding expensive .tolist() conversion.
    """

    # Fast path 1: NumPy arrays (most common)
    if isinstance(data, np.ndarray):
        # Trust numpy - no validation needed
        if data.dtype == np.float32:
            return data  # Direct pass, no conversion!
        elif data.dtype in (np.float64, np.int32, np.int64):
            return data.astype(np.float32)  # Convert dtype only
        else:
            return data.astype(np.float32)

    # Fast path 2: Python lists
    if isinstance(data, list):
        # For lists, we still need to validate, but we can optimize
        # by converting to numpy first for batch operations
        if len(data) > 100:  # Threshold for numpy conversion
            return np.array(data, dtype=np.float32)
        else:
            return [float(x) for x in data]

    # PyTorch tensors
    if hasattr(data, "detach") and hasattr(data, "cpu"):
        torch = _get_torch()
        if torch and torch.is_tensor(data):
            # Return numpy array directly, not list
            return data.detach().cpu().numpy().astype(np.float32)

    # JAX arrays
    if hasattr(data, "__array__"):
        jax = _get_jax()
        if jax and isinstance(data, jax.Array):
            # JAX arrays can be converted to numpy efficiently
            return np.array(data, dtype=np.float32)

    # TensorFlow tensors
    if hasattr(data, "numpy") and hasattr(data, "shape"):
        tf = _get_tensorflow()
        if tf and tf.is_tensor(data):
            return data.numpy().astype(np.float32)

    # Fallback for other array-like objects
    if hasattr(data, "tolist"):
        # Convert to numpy for consistency
        return np.array(data.tolist(), dtype=np.float32)

    # Handle other iterables
    if hasattr(data, "__iter__"):
        return np.array([float(x) for x in data], dtype=np.float32)

    raise ValueError(f"Cannot convert {type(data)} to vector")


def _validate_vector_fast(vector: Union[List[float], np.ndarray]) -> None:
    """
    Fast vector validation - trust numpy arrays.
    """
    if isinstance(vector, np.ndarray):
        # Trust numpy - just check shape
        if vector.size == 0:
            raise ValueError("Vector cannot be empty")
        if len(vector.shape) != 1:
            raise ValueError("Vector must be 1-dimensional")
        return  # Skip element validation for numpy

    if isinstance(vector, list):
        if len(vector) == 0:
            raise ValueError("Vector cannot be empty")
        # Only validate lists, not numpy arrays
        if not all(isinstance(x, (int, float)) for x in vector):
            raise ValueError("All vector elements must be numeric")
    else:
        raise ValueError("Vector must be a list or numpy array")


class OptimizedDB:
    """Optimized database interface with reduced FFI overhead."""

    def __init__(self):
        """Initialize optimized database."""
        self._initialized = False
        self._dimension = None

    def _ensure_initialized(self):
        """Lazy initialization."""
        if not self._initialized:
            _native.clear_database()
            self._initialized = True

    def add(
        self, id: str, vector: VectorInput, metadata: Optional[Dict[str, str]] = None
    ) -> bool:
        """
        Optimized add with fast validation.

        Key optimizations:
        1. Skip validation for numpy arrays
        2. Direct numpy passing when possible
        3. Cached conversions
        """
        self._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValueError("Vector ID must be a non-empty string")

        # Fast conversion - may return numpy array
        vector_data = _convert_to_vector_fast(vector)

        # Fast validation - trusts numpy
        _validate_vector_fast(vector_data)

        # Convert numpy to list only if necessary
        if isinstance(vector_data, np.ndarray):
            vector_list = vector_data.tolist()
        else:
            vector_list = vector_data

        # Call native function
        try:
            metadata_dict = metadata or {}
            result = _native.add_vector(id, vector_list, metadata_dict)

            # Track dimension
            if result and self._dimension is None:
                self._dimension = len(vector_list)

            return bool(result)
        except Exception as e:
            if "Dimension mismatch" in str(e):
                raise ValueError(str(e)) from e
            raise RuntimeError(f"Failed to add vector: {e}") from e

    def add_batch_optimized(
        self, vectors: np.ndarray, ids: Optional[List[str]] = None
    ) -> List[str]:
        """
        Ultra-optimized batch add for numpy arrays.

        Bypasses all validation and conversion for maximum speed.
        """
        self._ensure_initialized()

        if not isinstance(vectors, np.ndarray):
            raise TypeError("add_batch_optimized requires numpy array")

        if vectors.dtype != np.float32:
            vectors = vectors.astype(np.float32)

        n_vectors = len(vectors)

        # Generate IDs if needed
        if ids is None:
            ids = [f"vec_{i}" for i in range(n_vectors)]

        # Direct batch call - no conversion needed
        try:
            # The native batch function should accept numpy directly
            result = _native.add_vector_batch(ids, vectors, {})
            return ids
        except Exception as e:
            raise RuntimeError(f"Batch add failed: {e}") from e

    def clear(self):
        """Clear database."""
        _native.clear_database()
        self._initialized = True
        self._dimension = None

    def size(self) -> int:
        """Get number of vectors."""
        return _native.count()

    def search(self, query: VectorInput, limit: int = 10) -> List:
        """Optimized search."""
        self._ensure_initialized()

        # Fast conversion
        query_data = _convert_to_vector_fast(query)

        # Convert to list if numpy
        if isinstance(query_data, np.ndarray):
            query_list = query_data.tolist()
        else:
            query_list = query_data

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
