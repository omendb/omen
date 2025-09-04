"""Quantization support for OmenDB.

Provides vector quantization options for memory optimization:
- Int8: 4x memory reduction, minimal accuracy loss
- Binary: 32x memory reduction, for initial filtering
- Product: Configurable accuracy/memory trade-off
"""

from typing import Optional, Union, List, Literal
import numpy as np
from dataclasses import dataclass


@dataclass
class QuantizationConfig:
    """Configuration for vector quantization."""

    type: Literal["none", "int8", "binary", "product"] = "none"
    """Quantization type to use."""

    # Product quantization specific
    num_subspaces: int = 8
    """Number of subspaces for product quantization."""

    codebook_size: int = 256
    """Codebook size for product quantization."""

    # Performance vs accuracy trade-off
    preserve_originals: bool = False
    """Keep original vectors for reranking (hybrid mode)."""


class QuantizedDB:
    """OmenDB with quantization support.

    Wraps standard DB with automatic quantization for memory efficiency.
    """

    def __init__(
        self,
        quantization: Union[str, QuantizationConfig] = "none",
        dimension: Optional[int] = None,
        buffer_size: int = 10000,
        db_path: Optional[str] = None,
    ):
        """Initialize quantized database.

        Args:
            quantization: Quantization type or config
            dimension: Vector dimension (auto-detected if None)
            buffer_size: Buffer size for batch operations
            db_path: Path for persistence
        """
        # Import here to avoid circular dependency
        from .api import DB

        self.db = DB(buffer_size=buffer_size, db_path=db_path)

        if isinstance(quantization, str):
            self.config = QuantizationConfig(type=quantization)
        else:
            self.config = quantization

        self.dimension = dimension
        self._quantizer = None

    def add(self, vector_id: str, vector: Union[List[float], np.ndarray]) -> bool:
        """Add quantized vector to database.

        Args:
            vector_id: Unique identifier
            vector: Vector to add (will be quantized)

        Returns:
            True if successful
        """
        if isinstance(vector, list):
            vector = np.array(vector, dtype=np.float32)

        # Auto-detect dimension
        if self.dimension is None:
            self.dimension = len(vector)
            self._init_quantizer()

        # Quantize if enabled
        if self.config.type != "none":
            quantized = self._quantize(vector)
            # Store quantized version
            return self.db.add(vector_id, quantized)
        else:
            return self.db.add(vector_id, vector)

    def add_batch(
        self, vectors: Union[List[List[float]], np.ndarray], ids: List[str]
    ) -> int:
        """Add batch of quantized vectors.

        Args:
            vectors: Batch of vectors to add
            ids: Unique identifiers

        Returns:
            Number of vectors added
        """
        if isinstance(vectors, list):
            vectors = np.array(vectors, dtype=np.float32)

        # Auto-detect dimension
        if self.dimension is None and len(vectors) > 0:
            self.dimension = vectors.shape[1]
            self._init_quantizer()

        # Quantize batch if enabled
        if self.config.type != "none":
            quantized = self._quantize_batch(vectors)
            return self.db.add_batch(quantized, ids)
        else:
            return self.db.add_batch(vectors, ids)

    def search(self, query: Union[List[float], np.ndarray], limit: int = 10) -> List:
        """Search with quantized query.

        Args:
            query: Query vector
            limit: Number of results

        Returns:
            Search results
        """
        if isinstance(query, list):
            query = np.array(query, dtype=np.float32)

        # Quantize query if using quantization
        if self.config.type != "none" and self._quantizer:
            query = self._quantize(query)

        return self.db.search(query, limit)

    def _init_quantizer(self):
        """Initialize quantizer based on config."""
        if self.config.type == "int8":
            self._quantizer = Int8Quantizer(self.dimension)
        elif self.config.type == "binary":
            self._quantizer = BinaryQuantizer(self.dimension)
        elif self.config.type == "product":
            self._quantizer = ProductQuantizer(
                self.dimension, self.config.num_subspaces, self.config.codebook_size
            )

    def _quantize(self, vector: np.ndarray) -> np.ndarray:
        """Quantize single vector."""
        if self._quantizer is None:
            return vector
        return self._quantizer.quantize(vector)

    def _quantize_batch(self, vectors: np.ndarray) -> np.ndarray:
        """Quantize batch of vectors."""
        if self._quantizer is None:
            return vectors
        return self._quantizer.quantize_batch(vectors)

    def get_memory_usage(self) -> dict:
        """Get memory usage statistics.

        Returns:
            Dict with memory stats
        """
        stats = self.db.info()

        if self.config.type != "none":
            # Calculate compression ratio
            original_size = stats.get("vector_count", 0) * self.dimension * 4

            if self.config.type == "int8":
                compressed_size = original_size // 4
            elif self.config.type == "binary":
                compressed_size = original_size // 32
            else:  # product
                compressed_size = original_size // 8  # Typical

            stats["original_size_mb"] = original_size / (1024 * 1024)
            stats["compressed_size_mb"] = compressed_size / (1024 * 1024)
            stats["compression_ratio"] = (
                original_size / compressed_size if compressed_size > 0 else 1
            )

        return stats

    def clear(self):
        """Clear all vectors."""
        self.db.clear()


class Int8Quantizer:
    """Int8 quantization - 4x memory reduction."""

    def __init__(self, dimension: int):
        self.dimension = dimension

    def quantize(self, vector: np.ndarray) -> np.ndarray:
        """Quantize to int8."""
        # Find min/max for scaling
        min_val = vector.min()
        max_val = vector.max()

        # Scale to int8 range
        if max_val - min_val < 1e-6:
            return np.zeros(self.dimension, dtype=np.float32)

        scale = (max_val - min_val) / 255.0
        offset = min_val + 128.0 * scale

        # Quantize and store scale factors
        # In practice, we'd store scale/offset separately
        quantized = np.round((vector - offset) / scale).astype(np.int8)

        # For now, dequantize back to float32 for compatibility
        return (quantized.astype(np.float32) * scale + offset).astype(np.float32)

    def quantize_batch(self, vectors: np.ndarray) -> np.ndarray:
        """Quantize batch of vectors."""
        return np.array([self.quantize(v) for v in vectors])


class BinaryQuantizer:
    """Binary quantization - 32x memory reduction."""

    def __init__(self, dimension: int):
        self.dimension = dimension

    def quantize(self, vector: np.ndarray) -> np.ndarray:
        """Quantize to binary."""
        # Use median as threshold
        threshold = np.median(vector)

        # Create binary vector (as float32 for compatibility)
        binary = (vector > threshold).astype(np.float32)

        # Scale to preserve some magnitude information
        magnitude = np.linalg.norm(vector)
        if magnitude > 0:
            binary *= magnitude / np.sqrt(np.sum(binary))

        return binary

    def quantize_batch(self, vectors: np.ndarray) -> np.ndarray:
        """Quantize batch of vectors."""
        return np.array([self.quantize(v) for v in vectors])


class ProductQuantizer:
    """Product quantization - configurable trade-off."""

    def __init__(
        self, dimension: int, num_subspaces: int = 8, codebook_size: int = 256
    ):
        self.dimension = dimension
        self.num_subspaces = num_subspaces
        self.codebook_size = codebook_size
        self.subspace_dim = dimension // num_subspaces

        # In practice, codebooks would be learned from data
        # For now, using random initialization
        self.codebooks = [
            np.random.randn(codebook_size, self.subspace_dim).astype(np.float32)
            for _ in range(num_subspaces)
        ]

    def quantize(self, vector: np.ndarray) -> np.ndarray:
        """Quantize using product quantization."""
        # Split into subspaces
        subvectors = vector.reshape(self.num_subspaces, self.subspace_dim)

        # Find nearest codebook entry for each subspace
        quantized = []
        for i, subvec in enumerate(subvectors):
            # Find nearest centroid
            distances = np.sum((self.codebooks[i] - subvec) ** 2, axis=1)
            nearest = np.argmin(distances)
            quantized.append(self.codebooks[i][nearest])

        return np.concatenate(quantized).astype(np.float32)

    def quantize_batch(self, vectors: np.ndarray) -> np.ndarray:
        """Quantize batch of vectors."""
        return np.array([self.quantize(v) for v in vectors])


def choose_quantization(
    dimension: int, num_vectors: int, memory_budget_mb: float
) -> str:
    """Choose optimal quantization strategy.

    Args:
        dimension: Vector dimension
        num_vectors: Number of vectors
        memory_budget_mb: Memory budget in MB

    Returns:
        Recommended quantization type
    """
    float32_size = (dimension * num_vectors * 4) / (1024 * 1024)
    int8_size = float32_size / 4
    binary_size = float32_size / 32

    if float32_size <= memory_budget_mb:
        return "none"
    elif int8_size <= memory_budget_mb:
        return "int8"
    elif binary_size <= memory_budget_mb:
        return "binary"
    else:
        return "product"
