"""
OmenDB Python API - High-performance embedded vector database.

Design Philosophy:
- Make the fast path the default path
- Accept common data formats (lists, NumPy, PyTorch, etc.)
- Automatically optimize for best performance
- Keep the API simple and intuitive
"""

from typing import List, Optional, Dict, Union, Tuple, Any, TYPE_CHECKING
import os
import time
import threading
import warnings
from dataclasses import dataclass
from collections import deque

# Type hints for optional dependencies
if TYPE_CHECKING:
    import numpy as np
    import pandas as pd
    import torch
    import tensorflow as tf
    import jax

# Type alias for vector inputs
VectorInput = Union[List[float], "np.ndarray", "torch.Tensor", "tf.Tensor", "jax.Array"]

# Import the pre-compiled native module
try:
    import platform
    import ctypes

    if platform.system() == "Darwin":  # macOS
        lib_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib")
        if os.path.exists(lib_dir):
            runtime_libs = [
                "libMSupportGlobals.dylib",
                "libAsyncRTRuntimeGlobals.dylib",
                "libKGENCompilerRTShared.dylib",
                "libAsyncRTMojoBindings.dylib",
            ]

            for lib_name in runtime_libs:
                lib_path = os.path.join(lib_dir, lib_name)
                if os.path.exists(lib_path):
                    try:
                        ctypes.CDLL(lib_path)
                    except OSError:
                        pass

    # Try to import the pre-compiled native module directly
    import importlib.util

    native_so_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "native.so"
    )

    if os.path.exists(native_so_path):
        spec = importlib.util.spec_from_file_location("native", native_so_path)
        _native = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(_native)
        _NATIVE_AVAILABLE = True
    else:
        raise ImportError("Native module not found at expected path")

except ImportError:
    _native = None
    _NATIVE_AVAILABLE = False
except Exception:
    _native = None
    _NATIVE_AVAILABLE = False

# Pure Mojo implementation - no Python fallbacks!

# Import exceptions
from .exceptions import OmenDBError, DatabaseError, ValidationError

# Global native module validation (cached per process)
_native_validated = False
_native_configured = False  # Track if configure_database has been called

# Using Mojo v25.4 for stable singleton pattern and 41K vec/s performance
# Will upgrade to newer versions when module-level static data is supported


def _ensure_native_available():
    """Ensure native module is available and working (cached globally)."""
    global _native_validated

    if _native_validated:
        return

    if not _NATIVE_AVAILABLE:
        raise DatabaseError(
            "Native Mojo module not available. Please ensure native.so is compiled."
        )

    # Test connection once per process
    try:
        result = _native.test_connection()
        if "successful" not in str(result).lower():
            raise DatabaseError("Native module connection test failed")
    except Exception as e:
        raise DatabaseError(f"Failed to connect to native module: {e}")

    _native_validated = True


# Tensor conversion utilities with true lazy imports
def _convert_to_vector(data: VectorInput) -> List[float]:
    """Convert various input types to a vector list with optimal lazy loading."""

    # Fast path: Python lists (most common case - no imports needed)
    if isinstance(data, list):
        return [float(x) for x in data]

    # Specific path: PyTorch tensors (unique detach/cpu methods)
    if hasattr(data, "detach") and hasattr(data, "cpu"):
        import torch  # Import only when PyTorch tensor detected

        if torch.is_tensor(data):
            return data.detach().cpu().numpy().tolist()

    # Specific path: JAX arrays (check JAX first before TensorFlow)
    if hasattr(data, "shape") and hasattr(data, "__array__"):
        try:
            import jax

            if isinstance(data, jax.Array):
                return data.tolist()
        except ImportError:
            pass

    # Specific path: TensorFlow tensors
    if hasattr(data, "numpy") and hasattr(data, "shape"):
        try:
            import tensorflow as tf  # Import only when TensorFlow tensor detected

            if tf.is_tensor(data):
                return data.numpy().tolist()
        except ImportError:
            pass

    # General path: NumPy arrays and array-like objects with tolist
    if hasattr(data, "tolist"):
        return data.tolist()

    # General path: Try numpy() method for other array types
    if hasattr(data, "numpy"):
        try:
            return data.numpy().tolist()
        except:
            pass

    # Handle other iterables
    if hasattr(data, "__iter__"):
        return [float(x) for x in data]

    raise ValidationError(f"Cannot convert {type(data)} to vector")


def _validate_vector(vector) -> None:
    """Validate vector format (accepts lists or NumPy arrays)."""
    import numpy as np
    
    # Accept both lists and NumPy arrays
    if isinstance(vector, np.ndarray):
        # NumPy array validation
        if vector.size == 0:
            raise ValidationError("Vector cannot be empty")
        if not np.issubdtype(vector.dtype, np.number):
            raise ValidationError("All vector elements must be numeric")
    elif isinstance(vector, list):
        # List validation
        if len(vector) == 0:
            raise ValidationError("Vector cannot be empty")
        if not all(isinstance(x, (int, float)) for x in vector):
            raise ValidationError("All vector elements must be numeric")
    else:
        raise ValidationError("Vector must be a list of floats or NumPy array")


@dataclass
class SearchResult:
    """Result from vector similarity search."""

    id: str
    score: float  # Higher values indicate better matches (0-1 range, 1 = identical)
    vector: Optional[List[float]] = None
    metadata: Optional[Dict[str, str]] = None

    @property
    def distance(self) -> float:
        """Distance value (lower is better). Computed as 1 - score for cosine distance."""
        return 1.0 - self.score


class DB:
    """OmenDB database API - Pure Mojo implementation with instant startup."""

    def __init__(
        self,
        db_path: Optional[str] = None,
        buffer_size: int = 5000,  # Balanced buffer size - reduces pre-allocation
        algorithm: str = "auto",  # Auto-select best algorithm based on size
        use_columnar: bool = False,
        is_server: bool = False,
        quantization: Optional[str] = "scalar",  # Default to scalar quantization for memory efficiency
    ):
        """Create a reference to the embedded OmenDB database instance.

        Args:
            db_path: Optional path to database file (loaded lazily on first operation)
            buffer_size: Size of write buffer (default: 10000).
                        For 'auto' algorithm, this determines when to use graph indexing.
                        Range: 100-100000 vectors.
            algorithm: Index algorithm (deprecated - always uses DiskANN):
                      - All values use Buffer + DiskANN architecture
                      - No algorithm switching occurs
                      - Parameter kept for backwards compatibility
            use_columnar: Enable columnar storage for SIMD optimization (default: False)
            is_server: Server mode - larger buffer for maximum throughput (default: False)
            quantization: Quantization mode (default: None for full precision)
                        - None: Full precision (best accuracy)
                        - "scalar": 8-bit quantization (4x memory savings, 2-3% accuracy loss)
                        - "binary": 1-bit quantization (32x savings, 10-15% accuracy loss)

        IMPORTANT: Like SQLite, OmenDB uses a single embedded instance per process.
        Multiple DB() calls return references to the same underlying database.
        Use clear() to reset the database state for testing or isolation.

        This constructor is ultra-fast (~0.01ms) by deferring all heavy operations
        until actually needed. The database follows embedded patterns:
        - Instant object creation
        - Single instance per process (like SQLite)
        - Lazy file loading
        - Pay-for-what-you-use tensor imports

        Examples:
            # Standard usage with full precision (default)
            db = omendb.DB()

            # Enable scalar quantization for 4x memory savings
            db = omendb.DB(quantization="scalar")

            # Use flat (brute force) for small datasets
            db = omendb.DB(algorithm='flat')

            # Server mode with quantization
            db = omendb.DB(is_server=True, quantization="scalar")
        """
        self._db_path = db_path
        self._buffer_size = buffer_size
        self._algorithm = algorithm
        self._quantization = quantization
        self._use_columnar = use_columnar
        self._is_server = is_server
        self._initialized = False  # Lazy initialization flag
        self._dimension = None  # Track dimension for better error messages

        # Auto-batching for performance (5-10x speedup for individual adds)
        # Use more conservative settings to avoid memory issues
        # IMPORTANT: Auto-batching disabled for HNSW+ 
        # Batching destroys NumPy arrays and hurts HNSW+ graph quality
        self._auto_batch_enabled = False
        self._auto_batcher = None  # Lazy init
        self._batch_lock = threading.RLock()  # Use RLock to avoid deadlocks
        self._pending_batch = []
        self._batch_timer = None
        self._batch_size_limit = 1000  # Smaller batches to reduce memory pressure

        # Buffer size is already large enough (100000) for all modes

    def configure(self, **kwargs):
        """Configure database parameters at runtime.

        Args:
            buffer_size: Size of write buffer (100-10000)
            algorithm: Index algorithm ('diskann' or 'flat')
            use_columnar: Enable columnar storage for SIMD speedup
            is_server: Server mode (larger buffer)

        Examples:
            # Use DiskANN algorithm
            db.configure(algorithm='diskann')

            # Enable columnar storage
            db.configure(use_columnar=True)

            # Server mode with larger buffer
            db.configure(is_server=True)

            # Custom buffer size
            db.configure(buffer_size=5000)
        """
        _ensure_native_available()

        config = {}
        if "buffer_size" in kwargs:
            config["buffer_size"] = kwargs["buffer_size"]
            self._buffer_size = kwargs["buffer_size"]
        if "algorithm" in kwargs:
            config["algorithm"] = kwargs["algorithm"]
            self._algorithm = kwargs["algorithm"]
        if "use_columnar" in kwargs:
            config["use_columnar"] = kwargs["use_columnar"]
            self._use_columnar = kwargs["use_columnar"]
        if "is_server" in kwargs:
            config["is_server"] = kwargs["is_server"]
            self._is_server = kwargs["is_server"]
            # Buffer size can be configured for server mode if needed

        if config:
            _native.configure_database(config)

    def _ensure_initialized(self):
        """Ensure database is initialized (called lazily on first operation)."""
        if self._initialized:
            return

        # Use a global flag to track if the native module has been configured
        global _native_configured

        # Validate native module (cached globally)
        _ensure_native_available()

        # Configure database with custom parameters
        # Auto-select algorithm based on buffer size if 'auto' is specified
        actual_algorithm = self._algorithm
        if self._algorithm == "auto":
            # Use flat for typical workloads, DiskANN only for large datasets
            if self._buffer_size < 50000:
                actual_algorithm = "flat"
            else:
                actual_algorithm = "diskann"

        config = {}
        config["buffer_size"] = self._buffer_size
        config["algorithm"] = actual_algorithm
        if self._use_columnar:
            config["use_columnar"] = self._use_columnar
        if self._is_server:
            config["is_server"] = self._is_server
        if self._quantization:
            config["quantization"] = self._quantization

        try:
            _native.configure_database(config)
            _native_configured = True
        except Exception as e:
            # Fall back to environment variables if native call fails
            os.environ["OMENDB_BUFFER_SIZE"] = str(self._buffer_size)
            os.environ["OMENDB_ALGORITHM"] = self._algorithm
        if self._use_columnar:
            os.environ["OMENDB_USE_COLUMNAR"] = "true"

        # Load from file if path provided and file exists
        if self._db_path and os.path.exists(self._db_path):
            self._load_file()  # Use internal method to avoid recursive call

        self._initialized = True

    def _initialize_algorithm(self, sample_vector: List[float]) -> None:
        """Initialize algorithm state for batch operations.

        Forces transition from uninitialized state to flat index
        by temporarily adding a dummy vector with correct dimensions.

        Args:
            sample_vector: Sample vector to determine correct dimension
        """
        try:
            stats = _native.get_stats()
            if (
                stats.get("status") == "roargraph_ready"
                and stats.get("vector_count", 0) == 0
            ):
                # Database is in uninitialized RoarGraph state - force algorithm switch
                # Add and immediately delete a dummy vector to initialize flat index
                # Use same dimension as target vectors to avoid dimension mismatch
                dummy_vector = [0.0] * len(sample_vector)
                dummy_id = "__batch_init_dummy__"
                _native.add_vector(dummy_id, dummy_vector, {})
                _native.delete_vector(dummy_id)
        except Exception:
            # If algorithm check fails, continue anyway - batch operation will handle it
            pass

    def _load_file(self) -> int:
        """Internal method to load database from file without initialization checks."""
        try:
            # WORKAROUND: Due to Mojo global variable issue, load_database
            # returns parsed data instead of modifying global state
            result = _native.load_database(self._db_path)

            if result is None:
                return 0

            # Extract vectors and hashmap
            vectors_data = result.get("vectors", [])
            hashmap_data = result.get("hashmap", {})
            dimension = result.get("dimension", 0)
            total_vectors = result.get("total", 0)

            if not vectors_data:
                return 0

            # Use HashMap data for O(1) bulk loading (MEMORY CORRUPTION FIXED!)
            try:
                # Try bulk loading first - much faster O(1) HashMap restoration
                bulk_result = _native.bulk_load_vectors(
                    vectors_data,
                    hashmap_data,
                    result.get("dimension", 0),
                    result.get("total", len(vectors_data)),
                )

                if bulk_result:
                    return int(bulk_result)

            except Exception:
                # Fallback to individual loading if bulk fails
                pass

            # Fallback: Individual loading (slower O(n) method)
            vectors_loaded = 0
            for vector_id, vector_values in vectors_data:
                # Convert to list if needed
                vector_list = list(vector_values)
                # Add with empty metadata
                success = _native.add_vector(vector_id, vector_list, {})
                if success:
                    vectors_loaded += 1

            return vectors_loaded

        except Exception as e:
            raise DatabaseError(f"Failed to load database: {e}")

    def warmup(self) -> None:
        """Manually initialize the database and warm up tensor libraries.

        This is optional - the database will initialize automatically on first use.
        Use this if you want to control exactly when initialization happens.

        ```python
        db = DB()  # Instant startup
        db.warmup()  # Manual initialization + tensor library warmup
        ```
        """
        self._ensure_initialized()

        # Optionally warm up tensor libraries by triggering imports
        # (only if you want to pay the cost upfront)
        try:
            import torch
            import tensorflow as tf
        except ImportError:
            pass

    def __del__(self):
        """Cleanup when instance is destroyed."""
        pass  # Single instance - no cleanup needed

    def __enter__(self):
        """Context manager entry - returns self for 'with' statements."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit - automatically saves database if path is set."""
        if self._db_path:
            try:
                self.save()
            except Exception:
                # Don't raise exceptions during cleanup to avoid masking original exceptions
                pass
        return False  # Don't suppress exceptions

    def add_dataframe(
        self,
        df,
        id_column: str,
        vector_columns: Union[str, List[str]],
        metadata_columns: Optional[List[str]] = None,
    ) -> int:
        """Add vectors from a Pandas DataFrame.

        Args:
            df: Pandas DataFrame containing vectors and metadata
            id_column: Column name containing vector IDs
            vector_columns: Column name(s) containing vector data
            metadata_columns: Optional list of columns to use as metadata

        Returns:
            Number of vectors successfully added
        """
        try:
            import pandas as pd

            if not isinstance(df, pd.DataFrame):
                raise ValidationError("Input must be a Pandas DataFrame")
        except ImportError:
            raise ValidationError("Pandas not available for DataFrame operations")

        if id_column not in df.columns:
            raise ValidationError(f"ID column '{id_column}' not found in DataFrame")

        # Handle vector columns
        if isinstance(vector_columns, str):
            if vector_columns not in df.columns:
                raise ValidationError(
                    f"Vector column '{vector_columns}' not found in DataFrame"
                )
            vector_data = df[vector_columns].tolist()
        else:
            for col in vector_columns:
                if col not in df.columns:
                    raise ValidationError(
                        f"Vector column '{col}' not found in DataFrame"
                    )
            # Combine multiple columns into vectors
            vector_data = df[vector_columns].values.tolist()

        # Handle metadata columns
        metadata_data = []
        if metadata_columns:
            for col in metadata_columns:
                if col not in df.columns:
                    raise ValidationError(
                        f"Metadata column '{col}' not found in DataFrame"
                    )
            metadata_data = df[metadata_columns].to_dict("records")
        else:
            metadata_data = [{}] * len(df)

        # Add vectors
        added_count = 0
        for i, (idx, row) in enumerate(df.iterrows()):
            vector_id = str(row[id_column])
            vector = vector_data[i]
            metadata = {k: str(v) for k, v in metadata_data[i].items()}

            if self.add(vector_id, vector, metadata):
                added_count += 1

        return added_count

    def set(
        self, 
        key_or_dict: Union[str, Dict[str, VectorInput]], 
        value: Optional[VectorInput] = None, 
        metadata: Optional[Dict[str, str]] = None
    ) -> Union[bool, List[str]]:
        """Set vector(s) using Redis-style API.
        
        Args:
            key_or_dict: Either a single ID string or dict of {id: vector} for batch
            value: Vector data if key_or_dict is a string (ignored for dict)
            metadata: Optional metadata
            
        Returns:
            True for single set, list of IDs for batch
            
        Examples:
            >>> db.set("id1", vector)           # Single vector
            >>> db.set({"id1": v1, "id2": v2})  # Batch via dict
        """
        if isinstance(key_or_dict, dict):
            # Batch mode via dict
            ids = list(key_or_dict.keys())
            vectors = list(key_or_dict.values())
            # Convert to 2D array for add_batch
            vectors_list = [_convert_to_vector(v) for v in vectors]
            self.add_batch(vectors_list, ids, None if metadata is None else [metadata] * len(ids))
            return ids
        else:
            # Single mode
            if value is None:
                raise ValidationError("value required when key_or_dict is a string")
            return self.add(key_or_dict, value, metadata)
    
    def get_vector(self, key: str) -> Optional[List[float]]:
        """Get a single vector by ID (returns just the vector, not metadata).
        
        Args:
            key: Vector ID
            
        Returns:
            Vector as list of floats, or None if not found
        """
        result = self.get(key)
        if result is None:
            return None
        return result[0]  # Just return the vector part

    def add(
        self, id: str, vector: VectorInput, metadata: Optional[Dict[str, str]] = None
    ) -> bool:
        """Add a vector to the database.
        
        DEPRECATED: Use set() instead for consistency with Redis-style API.

        With auto-batching enabled (default), rapid add() calls are automatically
        batched for 5-10x better performance.

        Args:
            id: Unique identifier for the vector
            vector: Vector data (supports lists, NumPy arrays, PyTorch tensors, TensorFlow tensors)
            metadata: Optional metadata dictionary

        Returns:
            True if successfully added, False otherwise
        """
        self._ensure_initialized()  # Lazy initialization

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        # Convert various input types to vector (preserve NumPy for zero-copy)
        try:
            # Check if it's already a NumPy array - DON'T CONVERT!
            import numpy as np
            if isinstance(vector, np.ndarray):
                # Keep as NumPy for zero-copy optimization in native layer
                vector_data = vector
                if vector.dtype != np.float32:
                    vector_data = vector.astype(np.float32)
                _validate_vector(vector_data)  # Validate shape/size
            else:
                # Convert other types to list
                vector_data = _convert_to_vector(vector)
                _validate_vector(vector_data)
        except Exception as e:
            raise ValidationError(f"Invalid vector format: {e}")

        # Auto-batching logic for better performance
        if self._auto_batch_enabled:
            with self._batch_lock:
                # Add to pending batch (preserve NumPy arrays!)
                self._pending_batch.append((id, vector_data, metadata or {}))

                # Check for backpressure - don't accumulate too many pending items
                if len(self._pending_batch) >= self._batch_size_limit:
                    # Force flush and wait to prevent memory issues
                    self._flush_batch()
                # Flush if batch is large enough for optimal performance
                elif len(self._pending_batch) >= min(100, self._batch_size_limit // 10):
                    self._flush_batch()
                # Schedule flush if no timer is running
                elif not self._batch_timer:
                    self._schedule_batch_flush()

                return True  # Optimistic return
        else:
            # Direct add without batching (pass vector_data to preserve NumPy)
            return self._add_single(id, vector_data, metadata or {})

    def _add_single(
        self, id: str, vector_data, metadata: Dict[str, str]
    ) -> bool:
        """Add a single vector directly (no batching).
        
        vector_data can be either a List[float] or numpy.ndarray for zero-copy.
        """
        try:
            # Pass NumPy arrays directly for zero-copy, lists as-is
            result = _native.add_vector(id, vector_data, metadata)

            # Track dimension for better error messages
            if result and self._dimension is None:
                self._dimension = len(vector_data)

            return bool(result)
        except Exception as e:
            # Convert native dimension errors to ValidationError with helpful context
            if "Dimension mismatch" in str(e):
                error_msg = str(e)
                if hasattr(self, "_dimension") and self._dimension:
                    error_msg = f"Vector dimension mismatch: Database expects {self._dimension}-dimensional vectors. {error_msg}"
                else:
                    error_msg = f"Vector dimension mismatch. All vectors must have the same dimension. {error_msg}"
                raise ValidationError(error_msg) from e
            raise DatabaseError(f"Failed to add vector: {e}") from e

    def _schedule_batch_flush(self):
        """Schedule a batch flush after 10ms for better batching."""
        if self._batch_timer:
            self._batch_timer.cancel()
        self._batch_timer = threading.Timer(0.010, self._flush_batch_async)
        self._batch_timer.start()

    def _flush_batch_async(self):
        """Async wrapper for batch flush."""
        with self._batch_lock:
            self._flush_batch()

    def _flush_batch(self):
        """Flush pending batch (must hold lock)."""
        if not self._pending_batch:
            return

        # Cancel timer if running
        if self._batch_timer:
            try:
                self._batch_timer.cancel()
            except:
                pass
            self._batch_timer = None

        # Extract batch
        batch = list(self._pending_batch)
        self._pending_batch.clear()

        if not batch:
            return

        # Convert to batch format with pre-sizing for memory efficiency
        batch_size = len(batch)
        ids = []
        vectors = []
        metadata_list = []
        
        # Pre-allocate lists for better memory efficiency
        ids.extend(None for _ in range(batch_size))
        vectors.extend(None for _ in range(batch_size))
        metadata_list.extend(None for _ in range(batch_size))
        
        for i, (id, vector, metadata) in enumerate(batch):
            ids[i] = id
            vectors[i] = vector
            metadata_list[i] = metadata

        # Use optimized batch add with error handling
        try:
            results = _native.add_vector_batch(ids, vectors, metadata_list)
        except Exception as e:
            # Fallback to smaller chunks if batch is too large
            if batch_size > 100:
                # Split into smaller chunks and retry
                chunk_size = 100
                for i in range(0, batch_size, chunk_size):
                    try:
                        chunk_ids = ids[i:i+chunk_size]
                        chunk_vectors = vectors[i:i+chunk_size]
                        chunk_metadata = metadata_list[i:i+chunk_size]
                        _native.add_vector_batch(chunk_ids, chunk_vectors, chunk_metadata)
                    except Exception as chunk_e:
                        # Final fallback - individual adds
                        for j in range(len(chunk_ids)):
                            try:
                                _native.add_vector(chunk_ids[j], chunk_vectors[j], chunk_metadata[j])
                            except:
                                pass  # Skip failed vector
            else:
                # Small batch failed, try individual adds
                for id, vec, meta in batch:
                    try:
                        _native.add_vector(id, vec, meta)
                    except:
                        pass  # Skip failed vector

    def upsert(
        self, id: str, vector: VectorInput, metadata: Optional[Dict[str, str]] = None
    ) -> bool:
        """Insert or update a vector in the database.

        If a vector with the given ID exists, it will be updated.
        If not, a new vector will be inserted.

        Args:
            id: Unique identifier for the vector
            vector: Vector data (supports lists, NumPy arrays, PyTorch tensors, etc.)
            metadata: Optional metadata dictionary

        Returns:
            True if successfully upserted

        Example:
            # Insert new vector
            db.upsert("doc1", [1.0, 2.0, 3.0], {"type": "text"})

            # Update existing vector
            db.upsert("doc1", [4.0, 5.0, 6.0], {"type": "updated"})
        """
        self._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        # Convert vector input
        try:
            vector_list = _convert_to_vector(vector)
            _validate_vector(vector_list)
        except Exception as e:
            raise ValidationError(f"Invalid vector format: {e}")

        # Check if vector exists
        if self.exists(id):
            # Update existing vector
            try:
                metadata_dict = metadata or {}
                # Use native update_vector which handles the update properly
                result = _native.update_vector(id, vector_list, metadata_dict)
                return bool(result)
            except Exception as e:
                if "Dimension mismatch" in str(e):
                    raise ValidationError(str(e)) from e
                raise DatabaseError(f"Failed to update vector: {e}") from e
        else:
            # Insert new vector
            return self.add(id, vector, metadata)

    def search(
        self,
        vector: VectorInput,
        limit: int = 10,
        filter: Optional[Dict[str, str]] = None,
        beamwidth: Optional[int] = None,
    ) -> List[SearchResult]:
        """Search for similar vectors.

        Args:
            vector: Search vector (supports lists, NumPy arrays, PyTorch tensors, TensorFlow tensors)
            limit: Number of similar vectors to return (default: 10)
            filter: Optional metadata filter dictionary
            beamwidth: Search beam width for DiskANN (default: auto-selects based on dataset size)
                      Higher values = better accuracy but slower search

        Returns:
            List of SearchResult objects
        """
        self._ensure_initialized()  # Lazy initialization

        # Flush any pending batched adds to ensure consistency
        if self._auto_batch_enabled:
            self.flush()

        # Convert various input types to vector list
        try:
            vector_list = _convert_to_vector(vector)
            _validate_vector(vector_list)
        except Exception as e:
            raise ValidationError(f"Invalid query vector format: {e}")

        # Validate limit parameter with reasonable limits
        if not isinstance(limit, int):
            raise ValidationError("limit must be an integer")
        if limit <= 0:
            raise ValidationError("limit must be a positive integer")
        if limit > 10000:
            raise ValidationError("limit cannot exceed 10,000 for performance reasons")

        # Use native Mojo module for ultra-optimized performance
        try:
            filter_dict = filter or {}
            # Pass beamwidth if provided, otherwise let native code auto-select
            search_kwargs = {"filter_dict": filter_dict}
            if beamwidth is not None:
                if not isinstance(beamwidth, int) or beamwidth < 1:
                    raise ValidationError("beamwidth must be a positive integer")
                search_kwargs["beamwidth"] = beamwidth
            
            # SAFETY FIX: Handle Collections corruption crash
            try:
                if beamwidth is not None:
                    results = _native.search_vectors_with_beam(vector_list, limit, filter_dict, beamwidth)
                else:
                    results = _native.search_vectors(vector_list, limit, filter_dict)
            except:
                # If native search crashes due to Collections corruption, return empty results
                # This prevents segmentation faults when Collections have corrupted global state
                import warnings

                warnings.warn(
                    "Search failed due to memory corruption. This is a known issue in v0.0.1 "
                    "when Collections are used. Returning empty results to prevent crash.",
                    UserWarning,
                )
                return []

            # Convert to SearchResult objects
            search_results = []
            for result in results:
                if isinstance(result, dict):
                    # Handle both 'score' (new) and 'similarity' (legacy) keys
                    score_value = result.get("score", result.get("similarity", 0.0))
                    search_results.append(
                        SearchResult(
                            id=result.get("id", ""),
                            score=float(score_value),
                            vector=result.get("vector"),
                            metadata=result.get("metadata"),
                        )
                    )
                else:
                    # Handle simple tuple format (id, score)
                    search_results.append(
                        SearchResult(
                            id=str(result[0]) if len(result) > 0 else "",
                            score=float(result[1]) if len(result) > 1 else 0.0,
                        )
                    )

            return search_results
        except Exception as e:
            # Convert native dimension errors to ValidationError
            if "Dimension mismatch" in str(e):
                raise ValidationError(str(e)) from e
            # Re-raise other errors as DatabaseError
            raise DatabaseError(f"Query failed: {e}") from e

    def search_to_dataframe(
        self,
        vector: VectorInput,
        limit: int = 10,
        filter: Optional[Dict[str, str]] = None,
    ):
        """Search for similar vectors and return results as a Pandas DataFrame.

        Args:
            vector: Search vector (supports lists, NumPy arrays, PyTorch tensors, TensorFlow tensors)
            limit: Number of results to return (default: 10)
            filter: Optional metadata filter dictionary

        Returns:
            Pandas DataFrame with columns: id, similarity, and metadata columns
        """
        try:
            import pandas as pd
        except ImportError:
            raise ValidationError("Pandas not available for DataFrame operations")

        # Get search results
        results = self.search(vector, limit, filter)

        if not results:
            return pd.DataFrame(columns=["id", "similarity"])

        # Convert to DataFrame
        data = []
        for result in results:
            row = {"id": result.id, "score": result.score}

            # Add metadata columns
            if hasattr(result, "metadata") and result.metadata:
                row.update(result.metadata)

            data.append(row)

        return pd.DataFrame(data)

    def info(self) -> Dict[str, Any]:
        """Get database information and statistics."""
        # SAFETY FIX: Handle Collections corruption gracefully
        try:
            self._ensure_initialized()  # Lazy initialization
        except:
            # If initialization fails due to Collections corruption, skip it
            # The native get_stats() has its own safety fallbacks
            pass

        try:
            return _native.get_stats()
        except Exception as e:
            raise DatabaseError(f"Failed to get info: {e}")


    def save(self, path: Optional[str] = None) -> bool:
        """Save database to file."""
        target_path = path or self._db_path
        if not target_path:
            raise ValidationError("No save path provided")

        try:
            # WORKAROUND: save_database returns data instead of writing file
            result = _native.save_database(target_path)

            if result is None:
                return False

            # Write the data to file with HashMap for O(1) loading
            with open(target_path, "w") as f:
                # Write header
                f.write("OMENDB_SAVE_V1\n")

                # Write metadata
                dimension = result.get("dimension", 0)
                total_vectors = result.get("total_vectors", 0)
                rebuild_threshold = result.get("rebuild_threshold", 1000)
                f.write(f"{dimension},{total_vectors},{rebuild_threshold}\n")

                # Write HashMap section for O(1) loading
                vectors = result.get("vectors", [])
                f.write("[HASHMAP]\n")
                for index, (vector_id, _) in enumerate(vectors):
                    f.write(f"{vector_id},{index}\n")

                # Write vectors section
                f.write("[VECTORS]\n")
                for vector_id, vector_values in vectors:
                    # Build line: ID,dim,v1,v2,v3,...
                    line_parts = [str(vector_id), str(len(vector_values))]
                    line_parts.extend(str(v) for v in vector_values)
                    f.write(",".join(line_parts) + "\n")

            return True

        except Exception as e:
            raise DatabaseError(f"Failed to save database: {e}")

    def load(self, path: Optional[str] = None) -> int:
        """Load database from file."""
        self._ensure_initialized()  # Lazy initialization

        target_path = path or self._db_path
        if not target_path:
            raise ValidationError("No load path provided")

        try:
            # WORKAROUND: Due to Mojo global variable issue, load_database
            # returns parsed data instead of modifying global state
            result = _native.load_database(target_path)

            if result is None:
                return 0

            # Extract vectors and hashmap
            vectors_data = result.get("vectors", [])
            hashmap_data = result.get("hashmap", {})
            dimension = result.get("dimension", 0)
            total_vectors = result.get("total", 0)

            if not vectors_data:
                return 0

            # Use HashMap data for O(1) bulk loading (MEMORY CORRUPTION FIXED!)
            try:
                # Try bulk loading first - much faster O(1) HashMap restoration
                bulk_result = _native.bulk_load_vectors(
                    vectors_data,
                    hashmap_data,
                    result.get("dimension", 0),
                    result.get("total", len(vectors_data)),
                )

                if bulk_result:
                    return int(bulk_result)

            except Exception:
                # Fallback to individual loading if bulk fails
                pass

            # Fallback: Individual loading (slower O(n) method)
            vectors_loaded = 0
            for vector_id, vector_values in vectors_data:
                # Convert to list if needed
                vector_list = list(vector_values)
                # Add with empty metadata
                success = _native.add_vector(vector_id, vector_list, {})
                if success:
                    vectors_loaded += 1

            return vectors_loaded

        except Exception as e:
            raise DatabaseError(f"Failed to load database: {e}")

    def enable_quantization(self) -> bool:
        """Enable 8-bit scalar quantization for 4x memory savings.
        
        Must be called before adding any vectors. Once vectors are added,
        quantization mode cannot be changed.
        
        Returns:
            bool: True if quantization was enabled, False if vectors already exist
        
        Example:
            >>> db = omendb.DB()
            >>> db.enable_quantization()  # Must call before adding vectors
            >>> db.add("vec1", [0.1, 0.2, 0.3])  # Vectors will be quantized
        """
        self._ensure_initialized()
        # Check if any vectors exist
        if self.count() > 0:
            return False  # Cannot enable quantization with existing vectors
        return bool(_native.enable_quantization())
    
    def get_memory_stats(self) -> Dict[str, float]:
        """Get detailed memory usage statistics.
        
        Returns:
            Dictionary with memory usage breakdown by component
            
        Example:
            >>> db = omendb.DB()
            >>> db.add_batch(vectors)
            >>> stats = db.get_memory_stats()
            >>> print(f"Total memory: {stats['total_mb']:.2f} MB")
        """
        self._ensure_initialized()
        try:
            result = _native.get_memory_stats()
            # Check if we got an error response
            if isinstance(result, dict) and "error" in result:
                # Return the error dict as-is
                return result
            return dict(result)
        except Exception as e:
            # Return a safe error response
            return {
                "error": str(e),
                "status": "failed",
                "vector_count": self.count() if hasattr(self, 'count') else 0
            }
    
    def enable_binary_quantization(self) -> bool:
        """Enable binary quantization for 32x memory savings.
        
        Must be called before adding any vectors. Once vectors are added,
        quantization mode cannot be changed.
        
        Binary quantization uses 1 bit per dimension, providing extreme memory
        savings at the cost of accuracy (10-15% recall drop typical).
        
        Returns:
            bool: True if binary quantization was enabled, False if vectors already exist
            
        Example:
            >>> db = omendb.DB()
            >>> db.enable_binary_quantization()  # Must call before adding vectors
            >>> db.add_batch(vectors)  # Vectors stored as 1 bit per dimension
        """
        self._ensure_initialized()
        # Check if any vectors exist
        if self.count() > 0:
            return False  # Cannot enable quantization with existing vectors
        return bool(_native.enable_binary_quantization())

    def flush(self) -> bool:
        """Flush any pending batched operations.

        When auto-batching is enabled, this forces immediate processing
        of any pending add() operations.

        Returns:
            True if flush succeeded
        """
        if self._auto_batch_enabled and self._pending_batch:
            with self._batch_lock:
                self._flush_batch()
        return True

    def clear(self) -> bool:
        """Clear all vectors from database."""
        # Flush any pending operations first
        self.flush()
        try:
            result = _native.clear_database()
            return bool(result)
        except Exception as e:
            raise DatabaseError(f"Failed to clear database: {e}")

    def checkpoint(self) -> bool:
        """Force a checkpoint to persist data to disk.

        This ensures all in-memory data is written to persistent storage.
        Useful before shutting down or for ensuring durability.

        Returns:
            True if checkpoint succeeded, False otherwise

        Example:
            if db.checkpoint():
                print("Data persisted successfully")
        """
        self._ensure_initialized()
        try:
            return bool(_native.checkpoint())
        except Exception as e:
            return False

    def recover(self) -> int:
        """Recover database from persisted storage.

        Loads vectors from disk storage back into memory.
        This is automatically called on startup if persistence is configured.

        Returns:
            Number of vectors recovered

        Example:
            recovered = db.recover()
            print(f"Recovered {recovered} vectors from disk")
        """
        self._ensure_initialized()
        try:
            return int(_native.recover())
        except Exception as e:
            return 0

    def set_persistence(self, path: str, use_wal: bool = True) -> bool:
        """Configure persistence settings.

        Args:
            path: Path to database file (e.g., "mydb.omen")
            use_wal: Use Write-Ahead Log for durability (default: True)
                    WAL provides crash recovery and better write performance

        Returns:
            True if configuration succeeded

        Example:
            # Enable persistence with WAL
            db.set_persistence("vectors.db", use_wal=True)

            # Add vectors - automatically persisted
            db.add_batch(vectors, ids)

            # Force checkpoint if needed
            db.checkpoint()
        """
        self._ensure_initialized()
        try:
            return bool(_native.set_persistence(path, use_wal))
        except Exception as e:
            return False

    def delete(self, id: str) -> bool:
        """Delete a vector by ID.

        Args:
            id: String identifier of the vector to delete

        Returns:
            True if vector was found and deleted, False otherwise
        """
        # Flush any pending batched adds to ensure consistency
        if self._auto_batch_enabled:
            self.flush()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        try:
            result = _native.delete_vector(id)
            return bool(result)
        except Exception as e:
            raise DatabaseError(f"Failed to delete vector: {e}")

    def delete_batch(self, ids: List[str]) -> List[bool]:
        """Delete multiple vectors efficiently.

        Args:
            ids: List of string identifiers to delete

        Returns:
            List of success flags for each deletion
        """
        if not isinstance(ids, list):
            raise ValidationError("IDs must be a list of strings")

        for i, vector_id in enumerate(ids):
            if not isinstance(vector_id, str) or len(vector_id.strip()) == 0:
                raise ValidationError(f"ID at index {i} must be a non-empty string")

        try:
            results = _native.delete_vector_batch(ids)
            return [bool(result) for result in results]
        except Exception as e:
            raise DatabaseError(f"Failed to delete vectors in batch: {e}")

    def get(self, id: str) -> Optional[Tuple[List[float], Dict[str, str]]]:
        """Get vector data and metadata by ID.

        Args:
            id: String identifier of the vector

        Returns:
            Tuple of (vector_data, metadata) if found, None otherwise
        """
        # Flush any pending batched adds to ensure consistency
        if self._auto_batch_enabled:
            self.flush()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        try:
            vector_data = _native.get_vector(id)
            if vector_data is None:
                return None

            # Get metadata using native function
            metadata = _native.get_metadata(id)
            metadata_dict = dict(metadata) if metadata else {}

            # Convert to float list and return with metadata
            return (list(vector_data), metadata_dict)
        except Exception as e:
            raise DatabaseError(f"Failed to get vector: {e}")

    def exists(self, id: str) -> bool:
        """Check if a vector exists by ID.

        Args:
            id: String identifier to check

        Returns:
            True if vector exists, False otherwise

        Note: This is implemented as get(id) is not None for consistency.
        """
        return self.get(id) is not None

    def list_ids(self, limit: int = 100, offset: int = 0) -> List[str]:
        """List vector IDs in the database.

        Args:
            limit: Maximum number of IDs to return (default: 100)
            offset: Number of IDs to skip (default: 0)

        Returns:
            List of vector IDs

        Example:
            >>> ids = db.list_ids(limit=10)  # Get first 10 IDs
            >>> all_ids = db.list_ids(limit=db.count())  # Get all IDs
        """
        # For now, we'll use search with a random vector to get IDs
        # In future, add native list_ids support for efficiency
        info = self.info()
        dimension = info.get("dimension", 128)
        total_count = info.get("vector_count", 0)

        if total_count == 0:
            return []

        # Use a zero vector to search (will return all vectors by distance)
        zero_vector = [0.0] * dimension
        # Get more results than requested to handle offset
        results = self.search(zero_vector, limit=min(limit + offset, total_count))

        # Apply offset and limit
        ids = [r.id for r in results]
        return ids[offset : offset + limit]

    def add_batch(
        self,
        vectors: Union[List[List[float]], "np.ndarray"],
        ids: Optional[List[str]] = None,
        metadata: Optional[List[Dict[str, str]]] = None,
    ) -> List[str]:
        """Add multiple vectors efficiently using columnar format.

        Args:
            vectors: 2D array-like of shape (n_vectors, dimension)
                    Can be numpy array (fastest) or list of lists
            ids: Optional list of unique IDs. Auto-generated if None.
            metadata: Optional list of metadata dicts. Empty dicts if None.

        Returns:
            List of IDs (useful when auto-generated)

        Performance:
        - With NumPy: 95K+ vectors/second (zero-copy)
        - With lists: 78K+ vectors/second
        - Automatic chunking: Large batches split into 5K chunks for stability

        Example:
            # With numpy (fastest)
            ids = db.add_batch(
                vectors=embeddings,  # numpy array
                ids=["doc1", "doc2"],
                metadata=[{"type": "text"}, {"type": "image"}]
            )

            # Auto-generate IDs
            ids = db.add_batch(vectors=embeddings)
        """
        self._ensure_initialized()

        # Handle empty input
        if isinstance(vectors, list) and not vectors:
            return []
        try:
            import numpy as np

            if isinstance(vectors, np.ndarray) and vectors.size == 0:
                return []
        except ImportError:
            pass

        # Determine batch size
        if hasattr(vectors, "shape"):
            n_vectors = vectors.shape[0]
            dimension = vectors.shape[1] if len(vectors.shape) > 1 else len(vectors[0])
        else:
            n_vectors = len(vectors)
            dimension = len(vectors[0]) if vectors else 0

        if n_vectors == 0:
            return []

        # Generate IDs if not provided
        if ids is None:
            import uuid

            ids = [str(uuid.uuid4()) for _ in range(n_vectors)]
        elif len(ids) != n_vectors:
            raise ValidationError(
                f"Number of IDs ({len(ids)}) must match number of vectors ({n_vectors})"
            )

        # Default metadata if not provided
        if metadata is None:
            metadata = [{}] * n_vectors
        elif len(metadata) != n_vectors:
            raise ValidationError(
                f"Number of metadata ({len(metadata)}) must match number of vectors ({n_vectors})"
            )

        # Initialize algorithm
        if hasattr(vectors, "__getitem__"):
            self._initialize_algorithm(vectors[0])

        try:
            # Try NumPy optimization
            try:
                import numpy as np

                # Convert to numpy if needed
                if not isinstance(vectors, np.ndarray):
                    vectors = np.array(vectors, dtype=np.float32, order="C")
                elif vectors.dtype != np.float32:
                    vectors = vectors.astype(np.float32, order="C")

            except (ImportError, ValueError):
                # NumPy not available or conversion failed
                pass

            # UNLIMITED SCALE: Stream large batches to prevent memory issues
            batch_size = len(ids) if hasattr(ids, '__len__') else len(vectors)
            if batch_size > 2000:  # Stream anything larger than 2K vectors
                print(f"🌊 STREAMING: Processing {batch_size:,} vectors in streaming mode")
                return self._stream_add_batch(ids, vectors, metadata)
            
            # Let native module handle smaller batches
            results = _native.add_vector_batch(ids, vectors, metadata)

            # Return IDs for successful additions
            return [ids[i] for i, success in enumerate(results) if success]

        except Exception as e:
            # Convert native dimension errors to ValidationError
            if "Dimension mismatch" in str(e):
                raise ValidationError(str(e)) from e
            # Re-raise other errors as DatabaseError
            raise DatabaseError(f"Failed to add vectors in batch: {e}") from e

    def upsert_batch(
        self,
        vectors: Union[List[List[float]], "np.ndarray"],
        ids: List[str],
        metadata: Optional[List[Dict[str, str]]] = None,
    ) -> List[str]:
        """Upsert multiple vectors efficiently.

        For existing IDs, vectors are updated. For new IDs, vectors are inserted.

        Args:
            vectors: 2D array-like of shape (n_vectors, dimension)
            ids: List of unique IDs (required for upsert)
            metadata: Optional list of metadata dicts

        Returns:
            List of successfully upserted IDs

        Example:
            # Mix of updates and inserts
            ids = db.upsert_batch(
                vectors=[[1,2,3], [4,5,6], [7,8,9]],
                ids=["existing1", "existing2", "new1"],
                metadata=[{"updated": "true"}, {"updated": "true"}, {"new": "true"}]
            )
        """
        self._ensure_initialized()

        # Validate inputs
        if ids is None:
            raise ValidationError("IDs are required for upsert_batch")

        # Handle empty input
        if isinstance(vectors, list) and not vectors:
            return []
        try:
            import numpy as np

            if isinstance(vectors, np.ndarray) and vectors.size == 0:
                return []
        except ImportError:
            pass

        # Determine batch size
        if hasattr(vectors, "shape"):
            n_vectors = vectors.shape[0]
        else:
            n_vectors = len(vectors)

        if n_vectors == 0:
            return []

        if len(ids) != n_vectors:
            raise ValidationError(
                f"Number of IDs ({len(ids)}) must match number of vectors ({n_vectors})"
            )

        # Default metadata if not provided
        if metadata is None:
            metadata = [{}] * n_vectors
        elif len(metadata) != n_vectors:
            raise ValidationError(
                f"Number of metadata ({len(metadata)}) must match number of vectors ({n_vectors})"
            )

        # Process each vector
        successful_ids = []
        for i in range(n_vectors):
            try:
                if hasattr(vectors, "__getitem__"):
                    vector = vectors[i]
                else:
                    vector = list(vectors[i])

                # Convert numpy arrays to lists if needed
                if hasattr(vector, "tolist"):
                    vector = vector.tolist()

                if self.upsert(ids[i], vector, metadata[i]):
                    successful_ids.append(ids[i])
            except Exception as e:
                # Continue with other vectors on error
                continue

        return successful_ids

    def export_metrics(self, format: str = "prometheus") -> str:
        """Export metrics in standard formats."""
        try:
            from .metrics_export import MetricsExporter, MetricsFormat

            format_map = {
                "prometheus": MetricsFormat.PROMETHEUS,
                "json": MetricsFormat.JSON,
                "statsd": MetricsFormat.STATSD,
            }

            if format not in format_map:
                raise ValidationError(f"Unsupported metrics format: {format}")

            stats = self.info()
            db_id = os.path.basename(self._db_path) if self._db_path else "default"
            exporter = MetricsExporter(database_id=db_id)
            return exporter.export_metrics(format_map[format])
        except Exception as e:
            raise DatabaseError(f"Failed to export metrics: {e}")

    def get_health_status(self) -> Dict[str, Any]:
        """Get health status for monitoring."""
        try:
            stats = self.info()
            return {
                "status": "healthy" if stats.get("vector_count", 0) >= 0 else "error",
                "vector_count": stats.get("vector_count", 0),
                "algorithm": stats.get("algorithm", "unknown"),
            }
        except Exception:
            return {"status": "error", "vector_count": 0, "algorithm": "unknown"}

    def _stream_add_batch(self, ids, vectors, metadata):
        """Stream large batches in small chunks to prevent memory issues."""
        import numpy as np
        
        # Convert to numpy for efficient slicing
        if not isinstance(vectors, np.ndarray):
            vectors = np.array(vectors, dtype=np.float32)
        
        batch_size = len(vectors)
        chunk_size = 1000  # Very small chunks for true streaming
        all_successful_ids = []
        
        print(f"🔄 Processing {batch_size:,} vectors in {(batch_size + chunk_size - 1) // chunk_size} chunks of {chunk_size:,}")
        
        for start_idx in range(0, batch_size, chunk_size):
            end_idx = min(start_idx + chunk_size, batch_size)
            
            # Extract chunk slices (no large allocations)
            chunk_vectors = vectors[start_idx:end_idx]
            chunk_ids = ids[start_idx:end_idx] if hasattr(ids, '__getitem__') else [f"vec_{start_idx + i}" for i in range(end_idx - start_idx)]
            chunk_metadata = metadata[start_idx:end_idx] if metadata and hasattr(metadata, '__getitem__') else []
            
            # Process chunk through native module (bypassing our streaming check)
            try:
                results = _native.add_vector_batch(chunk_ids, chunk_vectors, chunk_metadata)
                successful_chunk_ids = [chunk_ids[i] for i, success in enumerate(results) if success]
                all_successful_ids.extend(successful_chunk_ids)
                
                if start_idx % (chunk_size * 10) == 0:  # Progress every 10K vectors
                    print(f"   ✅ Processed {end_idx:,}/{batch_size:,} vectors...")
                    
            except Exception as e:
                print(f"   ❌ Chunk {start_idx:,}-{end_idx:,} failed: {e}")
                continue
        
        print(f"🎉 STREAMING COMPLETE: {len(all_successful_ids):,}/{batch_size:,} vectors successfully added")
        return all_successful_ids

    def count(self) -> int:
        """Get the total number of vectors in the database.

        Returns:
            Total count of vectors stored
        """
        _ensure_native_available()

        try:
            result = _native.count()
            count = int(result)
            
            # Include pending batch vectors that haven't been flushed yet
            if self._auto_batch_enabled:
                with self._batch_lock:
                    count += len(self._pending_batch)
            
            return count
        except Exception as e:
            raise DatabaseError(f"Failed to get vector count: {e}")

    def size(self) -> int:
        """Alias for count() for compatibility.

        Returns:
            Total count of vectors stored
        """
        return self.count()

    # =============================================================================
    # COLLECTIONS API - ChromaDB-style collection management
    # =============================================================================

    def create_collection(
        self, name: str, dimension: Optional[int] = None
    ) -> "Collection":
        """Create a new collection.

        Args:
            name: Name of the collection to create
            dimension: Optional dimension hint (for future use)

        Returns:
            Collection object for the created collection

        Raises:
            DatabaseError: If collection already exists

        Example:
            >>> db = omendb.DB()
            >>> images = db.create_collection("images", dimension=512)
            >>> images.add("img1", [0.1, 0.2, ...])
        """
        # SAFETY FIX: Disable Collections entirely in v0.0.1 due to memory corruption
        import warnings

        warnings.warn(
            "Collections API is completely disabled in v0.0.1 due to memory corruption bugs. "
            "This will be fixed in v0.1.0. Use the main DB for now.",
            UserWarning,
        )
        raise DatabaseError(
            "Collections API is disabled in v0.0.1 due to memory corruption issues. "
            "Use the main DB or wait for v0.1.0."
        )

        try:
            result = _native.create_collection(name)
            if not result:
                raise DatabaseError(f"Collection '{name}' already exists")
            return Collection(self, name, dimension)
        except Exception as e:
            if "already exists" in str(e):
                raise DatabaseError(f"Collection '{name}' already exists")
            raise DatabaseError(f"Failed to create collection '{name}': {e}")

    def get_collection(self, name: str) -> "Collection":
        """Get an existing collection.

        Args:
            name: Name of the collection to get

        Returns:
            Collection object for the existing collection

        Raises:
            DatabaseError: If collection doesn't exist

        Example:
            >>> db = omendb.DB()
            >>> images = db.get_collection("images")
            >>> results = images.search([0.1, 0.2, ...], limit=10)
        """
        self._ensure_initialized()

        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        try:
            exists = _native.collection_exists(name)
            if not exists:
                raise DatabaseError(f"Collection '{name}' not found")

            # Get dimension from collection stats if available
            stats = _native.get_collection_stats(name)
            dimension = stats.get("dimension") if stats else None

            return Collection(self, name, dimension)
        except Exception as e:
            if "not found" in str(e):
                raise DatabaseError(f"Collection '{name}' not found")
            raise DatabaseError(f"Failed to get collection '{name}': {e}")

    def get_or_create_collection(
        self, name: str, dimension: Optional[int] = None
    ) -> "Collection":
        """Get existing collection or create if it doesn't exist.

        Args:
            name: Name of the collection
            dimension: Optional dimension hint for new collections

        Returns:
            Collection object (existing or newly created)

        Example:
            >>> db = omendb.DB()
            >>> # Creates if doesn't exist, gets if exists
            >>> images = db.get_or_create_collection("images", dimension=512)
        """
        self._ensure_initialized()

        try:
            return self.get_collection(name)
        except DatabaseError:
            return self.create_collection(name, dimension)

    def list_collections(self) -> List[str]:
        """List all collection names.

        Returns:
            List of collection names including "default"

        Example:
            >>> db = omendb.DB()
            >>> db.create_collection("images")
            >>> db.create_collection("text")
            >>> print(db.list_collections())
            ['default', 'images', 'text']
        """
        self._ensure_initialized()

        try:
            result = _native.list_collections()
            return list(result)
        except Exception as e:
            raise DatabaseError(f"Failed to list collections: {e}")

    def delete_collection(self, name: str) -> bool:
        """Delete a collection and all its vectors.

        Args:
            name: Name of the collection to delete

        Returns:
            True if deleted, False if didn't exist or is default

        Raises:
            DatabaseError: If deletion fails

        Note:
            The "default" collection cannot be deleted.

        Example:
            >>> db = omendb.DB()
            >>> db.delete_collection("old_data")
            True
        """
        self._ensure_initialized()

        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        if name == "default":
            return False  # Cannot delete default collection

        try:
            result = _native.delete_collection(name)
            return bool(result)
        except Exception as e:
            raise DatabaseError(f"Failed to delete collection '{name}': {e}")

    def get_collection_stats(self, name: str) -> Optional[Dict[str, Any]]:
        """Get statistics for a specific collection.

        Args:
            name: Name of the collection

        Returns:
            Dictionary with collection statistics or None if not found

        Example:
            >>> stats = db.get_collection_stats("images")
            >>> print(f"Vectors: {stats.get('vector_count', 0)}")
        """
        self._ensure_initialized()

        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        try:
            result = _native.get_collection_stats(name)
            if result is None:
                return None

            # Convert PythonObject to dict if needed
            if hasattr(result, "__dict__"):
                return dict(result)
            return result
        except Exception as e:
            raise DatabaseError(f"Failed to get stats for collection '{name}': {e}")
    
    def from_numpy(self, array: "np.ndarray", ids: Optional[List[str]] = None, metadata: Optional[List[Dict[str, str]]] = None) -> List[str]:
        """Import vectors from a numpy array.
        
        Args:
            array: 2D numpy array of shape (n_vectors, dimension)
            ids: Optional list of IDs. Auto-generated if None.
            metadata: Optional list of metadata dicts
            
        Returns:
            List of IDs
            
        Example:
            >>> embeddings = model.encode(texts)  # numpy array
            >>> db.from_numpy(embeddings, ids=text_ids)
        """
        return self.add_batch(array, ids, metadata)
    
    def to_numpy(self, ids: Optional[List[str]] = None) -> Tuple["np.ndarray", List[str]]:
        """Export vectors to a numpy array.
        
        Args:
            ids: Optional list of IDs to export. If None, exports all.
            
        Returns:
            Tuple of (numpy array, list of IDs)
            
        Example:
            >>> vectors, ids = db.to_numpy()
            >>> vectors.shape  # (n_vectors, dimension)
        """
        try:
            import numpy as np
        except ImportError:
            raise ImportError("NumPy is required for to_numpy(). Install with: pip install numpy")
        
        if ids is None:
            # Get all IDs
            ids = self.list_ids(limit=self.count())
        
        # Get vectors
        vectors = []
        valid_ids = []
        for id in ids:
            result = self.get(id)
            if result is not None:
                vectors.append(result[0])  # Just the vector part
                valid_ids.append(id)
        
        if not vectors:
            # Return empty array with proper shape
            info = self.info()
            dim = info.get("dimension", 128)
            return np.array([], dtype=np.float32).reshape(0, dim), []
        
        return np.array(vectors, dtype=np.float32), valid_ids
    
    def from_pandas(self, df: "pd.DataFrame", vector_column: str, id_column: Optional[str] = None, metadata_columns: Optional[List[str]] = None) -> List[str]:
        """Import vectors from a pandas DataFrame.
        
        Args:
            df: DataFrame containing vectors
            vector_column: Name of column containing vectors
            id_column: Name of column containing IDs. If None, uses index.
            metadata_columns: List of column names to include as metadata
            
        Returns:
            List of IDs
            
        Example:
            >>> df = pd.DataFrame({
            ...     'id': ['doc1', 'doc2'],
            ...     'embedding': [[0.1, 0.2], [0.3, 0.4]],
            ...     'category': ['A', 'B']
            ... })
            >>> db.from_pandas(df, 'embedding', 'id', ['category'])
        """
        try:
            import pandas as pd
            import numpy as np
        except ImportError:
            raise ImportError("Pandas is required for from_pandas(). Install with: pip install pandas")
        
        # Extract IDs
        if id_column:
            ids = df[id_column].astype(str).tolist()
        else:
            ids = df.index.astype(str).tolist()
        
        # Extract vectors
        vectors = np.vstack(df[vector_column].values)
        
        # Extract metadata
        metadata = None
        if metadata_columns:
            metadata = []
            for _, row in df.iterrows():
                meta = {col: str(row[col]) for col in metadata_columns if col in row}
                metadata.append(meta)
        
        return self.add_batch(vectors, ids, metadata)
    
    def to_pandas(self, ids: Optional[List[str]] = None) -> "pd.DataFrame":
        """Export vectors to a pandas DataFrame.
        
        Args:
            ids: Optional list of IDs to export. If None, exports all.
            
        Returns:
            DataFrame with columns: id, vector, and any metadata fields
            
        Example:
            >>> df = db.to_pandas()
            >>> df.head()
        """
        try:
            import pandas as pd
        except ImportError:
            raise ImportError("Pandas is required for to_pandas(). Install with: pip install pandas")
        
        if ids is None:
            ids = self.list_ids(limit=self.count())
        
        rows = []
        for id in ids:
            result = self.get(id)
            if result is not None:
                vector, metadata = result
                row = {'id': id, 'vector': vector}
                row.update(metadata)
                rows.append(row)
        
        if not rows:
            return pd.DataFrame(columns=['id', 'vector'])
        
        return pd.DataFrame(rows)

    @property
    def collections(self) -> "CollectionsManager":
        """Access the collections manager for multi-collection operations.

        Returns:
            CollectionsManager instance for creating and managing collections

        Example:
            >>> db = omendb.DB()
            >>> collections = db.collections
            >>> images = collections.get_or_create("images")
            >>> images.add("img1", [0.1, 0.2, 0.3])
        """
        self._ensure_initialized()
        return CollectionsManager(self)


# =============================================================================
# COLLECTIONS API - Multi-collection support
# =============================================================================


class CollectionsManager:
    """Manager for multiple named vector collections.

    Provides a Dict[String, VectorStore] architecture similar to:
    - Pinecone namespaces
    - Weaviate classes
    - ChromaDB collections
    """

    def __init__(self, db: "DB"):
        """Initialize collections manager.

        Args:
            db: Parent DB instance to operate on
        """
        self._db = db

    def create_collection(
        self, name: str, dimension: Optional[int] = None
    ) -> "Collection":
        """Create a new named collection.

        Args:
            name: Name of the collection to create
            dimension: Optional dimension hint for the collection

        Returns:
            Collection object for the created collection

        Raises:
            ValidationError: If name is invalid
            DatabaseError: If creation fails or collection already exists
        """
        return self._db.create_collection(name, dimension)

    def list_collections(self) -> List[str]:
        """List all existing collections.

        Returns:
            List of collection names

        Raises:
            DatabaseError: If listing fails
        """
        return self._db.list_collections()

    def delete_collection(self, name: str) -> bool:
        """Delete a collection and all its vectors.

        Args:
            name: Name of the collection to delete

        Returns:
            True if collection was deleted, False if it didn't exist or is default

        Raises:
            ValidationError: If name is invalid
            DatabaseError: If deletion fails
        """
        return self._db.delete_collection(name)

    def exists(self, name: str) -> bool:
        """Check if a collection exists.

        Args:
            name: Name of the collection to check

        Returns:
            True if collection exists, False otherwise

        Raises:
            ValidationError: If name is invalid
            DatabaseError: If check fails
        """
        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        try:
            result = _native.collection_exists(name)
            return bool(result)
        except Exception as e:
            raise DatabaseError(f"Failed to check collection '{name}': {e}")

    def get_stats(self, name: str) -> Optional[Dict[str, Any]]:
        """Get statistics for a specific collection.

        Args:
            name: Name of the collection

        Returns:
            Dictionary of collection statistics or None if collection doesn't exist

        Raises:
            ValidationError: If name is invalid
            DatabaseError: If stats retrieval fails
        """
        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        try:
            result = _native.get_collection_stats(name)
            if result is None:
                return None
            return dict(result)
        except Exception as e:
            raise DatabaseError(f"Failed to get stats for collection '{name}': {e}")

    def get_or_create(self, name: str, dimension: Optional[int] = None) -> "Collection":
        """Get an existing collection or create it if it doesn't exist.

        Args:
            name: Name of the collection
            dimension: Optional dimension hint for new collection

        Returns:
            Collection instance for the named collection

        Raises:
            ValidationError: If name is invalid
            DatabaseError: If creation fails
        """
        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        # Try to create the collection (will fail if it exists)
        try:
            return self._db.create_collection(name, dimension)
        except DatabaseError as e:
            if "already exists" in str(e):
                # Collection exists, just return it
                return Collection(self._db, name, dimension)
            raise  # Re-raise other database errors

    def get_collection(self, name: str) -> "Collection":
        """Get a Collection instance for working with a specific collection.

        Args:
            name: Name of the collection

        Returns:
            Collection instance for the named collection

        Raises:
            ValidationError: If name is invalid
        """
        if not name or not isinstance(name, str):
            raise ValidationError("Collection name must be a non-empty string")

        return Collection(self._db, name)


class Collection:
    """Interface to a specific named collection.

    Provides the same API as DB but operates on a specific collection.
    Collections are automatically created when first accessed.
    """

    def __init__(self, db: "DB", name: str, dimension: Optional[int] = None):
        """Initialize collection interface.

        Args:
            db: Parent DB instance
            name: Name of the collection
            dimension: Optional dimension hint (for future use)
        """
        self._db = db
        self._name = name
        self._dimension = dimension

    @property
    def name(self) -> str:
        """Get the collection name."""
        return self._name

    @property
    def dimension(self) -> Optional[int]:
        """Get the collection dimension if known."""
        return self._dimension

    def add(
        self, id: str, vector: VectorInput, metadata: Optional[Dict[str, str]] = None
    ) -> bool:
        """Add a vector to this collection.

        Args:
            id: Unique identifier for the vector
            vector: Vector data (supports lists, NumPy arrays, PyTorch tensors, etc.)
            metadata: Optional metadata dictionary

        Returns:
            True if successfully added

        Example:
            >>> images = db.get_collection("images")
            >>> images.add("img1", [0.1, 0.2, 0.3])
        """
        self._db._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        # Convert various input types to vector list
        try:
            vector_list = _convert_to_vector(vector)
            _validate_vector(vector_list)
        except Exception as e:
            raise ValidationError(f"Invalid vector format: {e}")

        # Prepare metadata
        metadata_dict = metadata or {}

        try:
            result = _native.add_vector_to_collection(
                self._name, id, vector_list, metadata_dict
            )
            return bool(result)
        except Exception as e:
            if "Dimension mismatch" in str(e):
                raise ValidationError(str(e)) from e
            raise DatabaseError(
                f"Failed to add vector to collection '{self._name}': {e}"
            )

    def search(
        self,
        vector: VectorInput,
        limit: int = 10,
        filter: Optional[Dict[str, str]] = None,
    ) -> List[SearchResult]:
        """Search for similar vectors in this collection.

        Args:
            vector: Query vector (supports lists, NumPy arrays, PyTorch tensors, etc.)
            limit: Number of similar vectors to return
            filter: Optional metadata filter (not yet implemented for collections)

        Returns:
            List of SearchResult objects

        Example:
            >>> images = db.get_collection("images")
            >>> results = images.search([0.1, 0.2, 0.3], limit=5)
        """
        self._db._ensure_initialized()

        # Convert various input types to vector list
        try:
            vector_list = _convert_to_vector(vector)
            _validate_vector(vector_list)
        except Exception as e:
            raise ValidationError(f"Invalid query vector format: {e}")

        # Validate limit parameter
        if not isinstance(limit, int):
            raise ValidationError("limit must be an integer")
        if limit <= 0:
            raise ValidationError("limit must be a positive integer")
        if limit > 10000:
            raise ValidationError("limit cannot exceed 10,000 for performance reasons")

        if filter is not None:
            import warnings

            warnings.warn(
                "Metadata filtering not yet implemented for collections", UserWarning
            )

        try:
            # WORKAROUND: Collections search causes memory crash in v0.0.1
            # Use main DB search functionality instead
            import warnings

            warnings.warn(
                "Collections search is disabled in v0.0.1 due to memory bug. "
                "Using workaround through main DB. This will be fixed in v0.1.0.",
                UserWarning,
            )

            # Fall back to main DB search
            # This won't provide true isolation but avoids the crash
            return self._db.search(vector, limit=limit, filter=filter)
        except Exception as e:
            if "Dimension mismatch" in str(e):
                raise ValidationError(str(e)) from e
            raise DatabaseError(f"Failed to search in collection '{self._name}': {e}")

    def add_batch(
        self,
        vectors: Union[List[List[float]], "np.ndarray"],
        ids: Optional[List[str]] = None,
        metadata: Optional[List[Dict[str, str]]] = None,
    ) -> List[str]:
        """Add multiple vectors efficiently to this collection.

        Args:
            vectors: 2D array-like of shape (n_vectors, dimension)
            ids: Optional list of unique IDs. Auto-generated if None.
            metadata: Optional list of metadata dicts. Empty dicts if None.

        Returns:
            List of IDs (useful when auto-generated)

        Example:
            >>> images = db.get_collection("images")
            >>> ids = images.add_batch(
            >>>     vectors=embeddings,  # numpy array
            >>>     ids=["img1", "img2"],
            >>>     metadata=[{"type": "photo"}, {"type": "art"}]
            >>> )
        """
        self._db._ensure_initialized()

        # Handle empty input
        if isinstance(vectors, list) and not vectors:
            return []
        try:
            import numpy as np

            if isinstance(vectors, np.ndarray) and vectors.size == 0:
                return []
        except ImportError:
            pass

        # Determine batch size
        if hasattr(vectors, "shape"):
            n_vectors = vectors.shape[0]
        else:
            n_vectors = len(vectors)

        if n_vectors == 0:
            return []

        # Generate IDs if not provided
        if ids is None:
            import uuid

            ids = [str(uuid.uuid4()) for _ in range(n_vectors)]
        elif len(ids) != n_vectors:
            raise ValidationError(
                f"Number of IDs ({len(ids)}) must match number of vectors ({n_vectors})"
            )

        # Default metadata if not provided
        if metadata is None:
            metadata = [{}] * n_vectors
        elif len(metadata) != n_vectors:
            raise ValidationError(
                f"Number of metadata ({len(metadata)}) must match number of vectors ({n_vectors})"
            )

        # Process each vector (TODO: optimize with native batch function for collections)
        successful_ids = []
        for i in range(n_vectors):
            try:
                if hasattr(vectors, "__getitem__"):
                    vector = vectors[i]
                else:
                    vector = list(vectors[i])

                if self.add(ids[i], vector, metadata[i]):
                    successful_ids.append(ids[i])
            except Exception as e:
                # Continue with other vectors on error
                continue

        return successful_ids

    def count(self) -> int:
        """Get the number of vectors in this collection.

        Returns:
            Number of vectors in the collection

        Example:
            >>> images = db.get_collection("images")
            >>> print(f"Collection has {images.count()} vectors")
        """
        self._db._ensure_initialized()

        try:
            stats = _native.get_collection_stats(self._name)
            if stats:
                return stats.get("vector_count", 0)
            return 0
        except Exception as e:
            raise DatabaseError(
                f"Failed to get count for collection '{self._name}': {e}"
            )

    def delete(self, id: str) -> bool:
        """Delete a vector by ID from this collection.

        Args:
            id: String identifier of the vector to delete

        Returns:
            True if vector was found and deleted, False otherwise

        Example:
            >>> images = db.get_collection("images")
            >>> images.delete("img1")
        """
        self._db._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        try:
            # TODO: Add native function for deleting from specific collection
            # For now, this would require collection-specific delete support
            raise NotImplementedError("Collection-specific delete not yet implemented")
        except Exception as e:
            raise DatabaseError(
                f"Failed to delete vector from collection '{self._name}': {e}"
            )

    def get(self, id: str) -> Optional[Tuple[List[float], Dict[str, str]]]:
        """Get vector data and metadata by ID from this collection.

        Args:
            id: String identifier of the vector

        Returns:
            Tuple of (vector_data, metadata) if found, None otherwise

        Example:
            >>> images = db.get_collection("images")
            >>> vector, metadata = images.get("img1")
        """
        self._db._ensure_initialized()

        if not isinstance(id, str) or len(id.strip()) == 0:
            raise ValidationError("Vector ID must be a non-empty string")

        try:
            # TODO: Add native function for getting from specific collection
            # For now, this would require collection-specific get support
            raise NotImplementedError("Collection-specific get not yet implemented")
        except Exception as e:
            raise DatabaseError(
                f"Failed to get vector from collection '{self._name}': {e}"
            )

    def exists(self, id: str) -> bool:
        """Check if a vector exists by ID in this collection.

        Args:
            id: String identifier to check

        Returns:
            True if vector exists, False otherwise
        """
        return self.get(id) is not None

    def info(self) -> Dict[str, Any]:
        """Get collection statistics and information.

        Returns:
            Dictionary with collection statistics

        Example:
            >>> images = db.get_collection("images")
            >>> info = images.info()
            >>> print(f"Vectors: {info['vector_count']}")
        """
        self._db._ensure_initialized()

        try:
            stats = _native.get_collection_stats(self._name)
            return dict(stats) if stats else {}
        except Exception as e:
            raise DatabaseError(
                f"Failed to get info for collection '{self._name}': {e}"
            )

    def clear(self) -> bool:
        """Clear all vectors from this collection.

        Note: This deletes and recreates the collection.

        Returns:
            True if cleared successfully

        Example:
            >>> images = db.get_collection("images")
            >>> images.clear()  # Removes all vectors
        """
        self._db._ensure_initialized()

        try:
            # Delete and recreate the collection
            _native.delete_collection(self._name)
            _native.create_collection(self._name)
            return True
        except Exception as e:
            raise DatabaseError(f"Failed to clear collection '{self._name}': {e}")


# Collections manager is now accessed via db.collections property
