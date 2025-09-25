"""
OmenDB Compression Package.

Provides compression algorithms for vector storage including binary quantization,
scalar quantization, and product quantization for memory-efficient vector storage.
"""

# Import main compression interfaces
from .binary_quantization import BinaryQuantizedVector, BinaryQuantizer, CompressionStats
from .compression_manager import CompressedVector, CompressionManager, create_embedded_compression_manager, create_server_compression_manager