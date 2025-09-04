"""
Core components for OmenDB.

This package contains the core data structures and algorithms used by OmenDB,
including high-performance vector operations, distance calculations, and
sparse vector support for hybrid search scenarios.
"""

from .vector import (
    Vector,
    Float32Vector,
    Float64Vector,
    DefaultVector,
    zeros,
    ones,
    random_vector,
    from_list,
    benchmark_dot_product,
)

from .distance import (
    DistanceMetrics,
    BM25Plus,
    TfIdfScorer,
    ScoreFusion,
    DistanceUtils,
    benchmark_distance_calculations,
)

from .metadata import *

from .sparse_map import SparseMap

# Type aliases for convenience
alias DenseVector = DefaultVector

# Version information
alias VERSION = "0.1.0-alpha"
