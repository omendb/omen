"""Compression and quantization for OmenDB.

Provides scalar, binary, and product quantization for memory efficiency.
Product Quantization (PQ) achieves state-of-the-art compression ratios.
"""

from .scalar import ScalarQuantizedVector, QuantizedVectorBatch
from .binary import BinaryQuantizedVector
from .product_quantization import (
    PQVector, 
    PQCompressor, 
    PQVectorBatch,
    create_pq32_compressor,
    create_pq16_compressor,
    create_pq64_compressor
)