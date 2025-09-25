"""Utilities for OmenDB.

Performance optimization, memory management, and profiling tools.
"""

from .memory_pool import MemoryPool, VectorMemoryPool, reset_global_pool
from .metrics import (
    DatabaseMetrics,
    MetricsSnapshot, 
    OperationTimer,
    init_metrics,
    get_global_metrics,
    record_query_timing,
    record_insert,
    record_error
)
from .types import (
    # Constants
    DEFAULT_BUFFER_SIZE,
    MAX_VECTOR_DIM,
    DEFAULT_BEAM_WIDTH,
    DEFAULT_MAX_EDGES,
    # Type definitions
    AlgorithmType,
    StorageType,
    QuantizationType,
    SearchMode,
    DistanceMetric,
    # Result types
    SearchResult,
    IndexStats,
    DatabaseError
)
from .validation import (
    # Vector validation
    validate_vector_dimension,
    validate_vector_size,
    validate_k_parameter,
    # Python conversions
    python_list_to_float32,
    float32_list_to_python,
    extract_numpy_data,
    # String validation
    validate_vector_id,
    validate_collection_name,
    validate_file_path,
    # Metadata validation
    validate_metadata,
    # Numeric validation
    validate_positive_int,
    validate_range,
    clamp_value
)