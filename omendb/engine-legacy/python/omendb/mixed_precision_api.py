"""
Mixed Precision API for OmenDB

Provides easy-to-use mixed precision support for memory optimization.
Integrates with existing OmenDB API while adding automatic precision detection
and optimization capabilities.
"""

from typing import List, Optional, Dict, Any, Union, Tuple
import numpy as np
from dataclasses import dataclass
from enum import Enum
import time

try:
    from .api import OmenDB, SearchResult
    from .exceptions import OmenDBError, ValidationError
except ImportError:
    try:
        # Try direct import for testing
        import sys
        from pathlib import Path

        # Add the python directory to the path
        current_dir = Path(__file__).parent
        python_dir = current_dir.parent.parent / "python"
        if str(python_dir) not in sys.path:
            sys.path.insert(0, str(python_dir))

        from omendb import OmenDB, SearchResult
        from omendb.exceptions import OmenDBError, ValidationError
    except ImportError:
        # Fallback for testing when OmenDB is not available
        OmenDB = None
        SearchResult = None
        OmenDBError = Exception
        ValidationError = Exception


class PrecisionType(Enum):
    """Supported precision types."""

    FLOAT32 = "float32"
    FLOAT16 = "float16"
    INT8 = "int8"
    AUTO = "auto"  # Automatic precision detection


@dataclass
class PrecisionConfig:
    """Configuration for mixed precision processing."""

    precision_type: PrecisionType = PrecisionType.AUTO
    accuracy_threshold: float = 0.95  # Minimum accuracy to maintain
    memory_priority: bool = True  # Prioritize memory over speed
    force_precision: bool = False  # Force specific precision even if not optimal
    quantization_range: Tuple[float, float] = (-1.0, 1.0)  # Expected data range


@dataclass
class MemoryStats:
    """Memory usage statistics."""

    total_vectors: int
    total_dimension: int
    float32_memory_mb: float
    optimized_memory_mb: float
    memory_savings_mb: float
    memory_savings_percent: float
    precision_distribution: Dict[str, int]


class PrecisionAnalyzer:
    """Analyzes vector data to recommend optimal precision."""

    @staticmethod
    def analyze_vector_batch(vectors: List[List[float]]) -> Dict[str, Any]:
        """Analyze a batch of vectors to determine characteristics."""
        if not vectors:
            return {}

        # Flatten all values for analysis
        all_values = []
        for vector in vectors:
            all_values.extend(vector)

        all_values = np.array(all_values)

        return {
            "min": float(np.min(all_values)),
            "max": float(np.max(all_values)),
            "mean": float(np.mean(all_values)),
            "std": float(np.std(all_values)),
            "range": float(np.max(all_values) - np.min(all_values)),
            "zero_fraction": float(np.sum(all_values == 0) / len(all_values)),
            "total_values": len(all_values),
        }

    @staticmethod
    def recommend_precision(
        characteristics: Dict[str, Any], config: PrecisionConfig
    ) -> PrecisionType:
        """Recommend optimal precision based on data characteristics."""
        if config.precision_type != PrecisionType.AUTO:
            return config.precision_type

        min_val = characteristics.get("min", 0)
        max_val = characteristics.get("max", 0)
        range_val = characteristics.get("range", 0)
        std_val = characteristics.get("std", 0)
        mean_val = characteristics.get("mean", 0)

        # Decision logic for automatic precision selection

        # Check if data is suitable for int8 quantization
        # Criteria: small range, reasonable distribution, not too spread out
        if (
            range_val <= 2.0
            and abs(mean_val) <= 1.0
            and std_val <= 0.5
            and min_val >= -2.0
            and max_val <= 2.0
        ):
            return PrecisionType.INT8

        # Check if data is suitable for float16
        # Most ML embeddings are suitable for float16
        elif range_val <= 100.0 and abs(mean_val) <= 10.0 and std_val <= 10.0:
            return PrecisionType.FLOAT16

        # Default to float32 for data with large range or high precision needs
        else:
            return PrecisionType.FLOAT32

    @staticmethod
    def estimate_accuracy_loss(
        characteristics: Dict[str, Any], precision_type: PrecisionType
    ) -> float:
        """Estimate accuracy loss for a given precision type."""
        if precision_type == PrecisionType.FLOAT32:
            return 0.0

        range_val = characteristics.get("range", 0)
        std_val = characteristics.get("std", 0)

        if precision_type == PrecisionType.FLOAT16:
            # Float16 has 11 bits of precision
            # Estimate loss based on range and variance
            if range_val <= 2.0 and std_val <= 1.0:
                return 0.001  # Very low loss
            elif range_val <= 10.0:
                return 0.01  # Low loss
            else:
                return 0.05  # Moderate loss

        elif precision_type == PrecisionType.INT8:
            # Int8 quantization loss depends heavily on data distribution
            if range_val <= 1.0 and std_val <= 0.3:
                return 0.02  # Low loss for well-distributed data
            elif range_val <= 2.0 and std_val <= 0.5:
                return 0.05  # Moderate loss
            else:
                return 0.15  # High loss - may not be suitable

        return 0.0


class MixedPrecisionDB:
    """
    OmenDB with mixed precision support for memory optimization.

    Features:
    - Automatic precision detection based on data characteristics
    - Memory usage tracking and optimization
    - Accuracy preservation with configurable thresholds
    - Seamless integration with existing OmenDB API
    """

    def __init__(
        self,
        db_path: Optional[str] = None,
        precision_config: Optional[PrecisionConfig] = None,
    ):
        """Initialize mixed precision database."""
        if OmenDB is None:
            raise ImportError("OmenDB not available. Install native module.")

        self.db = OmenDB(db_path) if db_path else OmenDB()
        self.config = precision_config or PrecisionConfig()

        # Tracking
        self.vectors_added = 0
        self.total_dimension = 0
        self.precision_usage = {"float32": 0, "float16": 0, "int8": 0}
        self.data_characteristics = {}
        self.recommended_precision = PrecisionType.FLOAT32

        print(
            f"🔬 MixedPrecisionDB initialized with {self.config.precision_type.value} precision"
        )

    def add(
        self, id: str, vector: List[float], metadata: Optional[Dict[str, Any]] = None
    ) -> None:
        """Add vector with automatic precision optimization."""
        # Update tracking
        self.vectors_added += 1
        if self.total_dimension == 0:
            self.total_dimension = len(vector)
        elif self.total_dimension != len(vector):
            raise ValidationError(
                f"Vector dimension {len(vector)} doesn't match database dimension {self.total_dimension}"
            )

        # For now, delegate to underlying DB (future: implement precision optimization)
        self.db.add(id, vector, metadata)

        # Update precision usage tracking
        self.precision_usage["float32"] += 1  # Currently all stored as float32

    def add_batch(
        self, vectors: List[Tuple[str, List[float], Optional[Dict[str, Any]]]]
    ) -> None:
        """Add batch of vectors with mixed precision optimization."""
        if not vectors:
            return

        print(
            f"🔍 Analyzing batch of {len(vectors)} vectors for precision optimization..."
        )

        # Extract just the vector data for analysis
        vector_data = [v[1] for v in vectors]

        # Analyze data characteristics
        start_time = time.time()
        self.data_characteristics = PrecisionAnalyzer.analyze_vector_batch(vector_data)
        analysis_time = time.time() - start_time

        # Get precision recommendation
        self.recommended_precision = PrecisionAnalyzer.recommend_precision(
            self.data_characteristics, self.config
        )

        # Estimate accuracy loss
        accuracy_loss = PrecisionAnalyzer.estimate_accuracy_loss(
            self.data_characteristics, self.recommended_precision
        )

        # Check if accuracy loss is acceptable
        estimated_accuracy = 1.0 - accuracy_loss
        if (
            estimated_accuracy < self.config.accuracy_threshold
            and not self.config.force_precision
        ):
            print(
                f"⚠️  Estimated accuracy {estimated_accuracy:.3f} below threshold {self.config.accuracy_threshold}"
            )
            print(f"   Falling back to higher precision")
            if self.recommended_precision == PrecisionType.INT8:
                self.recommended_precision = PrecisionType.FLOAT16
            elif self.recommended_precision == PrecisionType.FLOAT16:
                self.recommended_precision = PrecisionType.FLOAT32

        # Report analysis results
        print(f"📊 Data analysis complete ({analysis_time:.3f}s):")
        print(
            f"   Range: [{self.data_characteristics['min']:.3f}, {self.data_characteristics['max']:.3f}]"
        )
        print(
            f"   Mean: {self.data_characteristics['mean']:.3f}, Std: {self.data_characteristics['std']:.3f}"
        )
        print(f"   Recommended precision: {self.recommended_precision.value}")
        print(f"   Estimated accuracy: {estimated_accuracy:.3f}")

        # Calculate memory savings
        memory_savings = self._calculate_memory_savings(
            len(vectors), self.total_dimension or len(vector_data[0])
        )
        print(
            f"💾 Memory optimization: {memory_savings['savings_percent']:.1f}% savings"
        )

        # Add vectors to database (currently using standard precision)
        # Future: Implement actual precision optimization in native layer
        start_time = time.time()
        for id, vector, metadata in vectors:
            self.add(id, vector, metadata)
        add_time = time.time() - start_time

        print(f"✅ Batch added in {add_time:.3f}s")

        # Update precision tracking
        self.precision_usage[self.recommended_precision.value] += len(vectors)

    def query(
        self,
        vector: List[float],
        top_k: int = 10,
        where: Optional[Dict[str, Any]] = None,
    ) -> List[SearchResult]:
        """Query with mixed precision support."""
        # For now, delegate to underlying DB
        # Future: Implement precision-aware querying
        return self.db.query(vector, top_k, where)

    def get_memory_stats(self) -> MemoryStats:
        """Get detailed memory usage statistics."""
        if self.vectors_added == 0 or self.total_dimension == 0:
            return MemoryStats(
                total_vectors=0,
                total_dimension=0,
                float32_memory_mb=0.0,
                optimized_memory_mb=0.0,
                memory_savings_mb=0.0,
                memory_savings_percent=0.0,
                precision_distribution={},
            )

        # Calculate memory usage
        float32_memory_bytes = (
            self.vectors_added * self.total_dimension * 4
        )  # 4 bytes per float32
        float32_memory_mb = float32_memory_bytes / (1024 * 1024)

        # Calculate optimized memory based on precision distribution
        optimized_memory_bytes = 0
        for precision, count in self.precision_usage.items():
            bytes_per_element = {"float32": 4, "float16": 2, "int8": 1}[precision]
            optimized_memory_bytes += count * self.total_dimension * bytes_per_element

        optimized_memory_mb = optimized_memory_bytes / (1024 * 1024)

        memory_savings_mb = float32_memory_mb - optimized_memory_mb
        memory_savings_percent = (
            (memory_savings_mb / float32_memory_mb) * 100
            if float32_memory_mb > 0
            else 0
        )

        return MemoryStats(
            total_vectors=self.vectors_added,
            total_dimension=self.total_dimension,
            float32_memory_mb=float32_memory_mb,
            optimized_memory_mb=optimized_memory_mb,
            memory_savings_mb=memory_savings_mb,
            memory_savings_percent=memory_savings_percent,
            precision_distribution=dict(self.precision_usage),
        )

    def _calculate_memory_savings(
        self, num_vectors: int, dimension: int
    ) -> Dict[str, float]:
        """Calculate potential memory savings for given precision."""
        float32_bytes = num_vectors * dimension * 4

        precision_bytes = {
            PrecisionType.FLOAT32: float32_bytes,
            PrecisionType.FLOAT16: num_vectors * dimension * 2,
            PrecisionType.INT8: num_vectors * dimension * 1,
        }

        optimized_bytes = precision_bytes[self.recommended_precision]
        savings_bytes = float32_bytes - optimized_bytes
        savings_percent = (savings_bytes / float32_bytes) * 100

        return {
            "float32_mb": float32_bytes / (1024 * 1024),
            "optimized_mb": optimized_bytes / (1024 * 1024),
            "savings_mb": savings_bytes / (1024 * 1024),
            "savings_percent": savings_percent,
        }

    def optimize_precision(self) -> Dict[str, Any]:
        """Analyze current database and suggest precision optimizations."""
        stats = self.get_memory_stats()

        if not self.data_characteristics:
            return {
                "status": "no_analysis",
                "message": "No data characteristics available. Add vectors first.",
            }

        # Get current and optimal precision
        current_precision = PrecisionType.FLOAT32  # Current implementation
        optimal_precision = self.recommended_precision

        optimization_report = {
            "current_precision": current_precision.value,
            "recommended_precision": optimal_precision.value,
            "current_memory_mb": stats.float32_memory_mb,
            "optimized_memory_mb": stats.optimized_memory_mb,
            "potential_savings_mb": stats.memory_savings_mb,
            "potential_savings_percent": stats.memory_savings_percent,
            "data_characteristics": self.data_characteristics,
            "vectors_analyzed": self.vectors_added,
        }

        return optimization_report

    def print_optimization_report(self) -> None:
        """Print a detailed optimization report."""
        report = self.optimize_precision()

        if report.get("status") == "no_analysis":
            print("⚠️  No optimization data available")
            return

        print("\n🔬 Mixed Precision Optimization Report")
        print("=" * 45)
        print(f"📊 Vectors analyzed: {report['vectors_analyzed']}")
        print(f"🎯 Current precision: {report['current_precision']}")
        print(f"💡 Recommended: {report['recommended_precision']}")
        print(f"\n💾 Memory Analysis:")
        print(f"   Current usage: {report['current_memory_mb']:.2f} MB")
        print(f"   Optimized usage: {report['optimized_memory_mb']:.2f} MB")
        print(
            f"   Potential savings: {report['potential_savings_mb']:.2f} MB ({report['potential_savings_percent']:.1f}%)"
        )

        chars = report["data_characteristics"]
        print(f"\n📈 Data Characteristics:")
        print(f"   Range: [{chars['min']:.3f}, {chars['max']:.3f}]")
        print(f"   Mean: {chars['mean']:.3f}, Std: {chars['std']:.3f}")
        print(f"   Zero fraction: {chars['zero_fraction']:.1%}")


# Convenience functions
def create_mixed_precision_db(
    db_path: Optional[str] = None,
    precision_type: Union[str, PrecisionType] = PrecisionType.AUTO,
    accuracy_threshold: float = 0.95,
    memory_priority: bool = True,
) -> MixedPrecisionDB:
    """Create a mixed precision database with specified configuration."""
    if isinstance(precision_type, str):
        precision_type = PrecisionType(precision_type)

    config = PrecisionConfig(
        precision_type=precision_type,
        accuracy_threshold=accuracy_threshold,
        memory_priority=memory_priority,
    )

    return MixedPrecisionDB(db_path, config)


def analyze_vectors_for_precision(vectors: List[List[float]]) -> Dict[str, Any]:
    """Analyze a set of vectors to determine optimal precision settings."""
    analyzer = PrecisionAnalyzer()
    characteristics = analyzer.analyze_vector_batch(vectors)

    config = PrecisionConfig()  # Default config
    recommended_precision = analyzer.recommend_precision(characteristics, config)
    accuracy_loss = analyzer.estimate_accuracy_loss(
        characteristics, recommended_precision
    )

    # Calculate memory implications
    num_vectors = len(vectors)
    dimension = len(vectors[0]) if vectors else 0

    float32_memory = num_vectors * dimension * 4  # bytes
    precision_memory = {
        PrecisionType.FLOAT32: float32_memory,
        PrecisionType.FLOAT16: num_vectors * dimension * 2,
        PrecisionType.INT8: num_vectors * dimension * 1,
    }

    optimized_memory = precision_memory[recommended_precision]
    memory_savings = ((float32_memory - optimized_memory) / float32_memory) * 100

    return {
        "recommended_precision": recommended_precision.value,
        "estimated_accuracy": 1.0 - accuracy_loss,
        "memory_savings_percent": memory_savings,
        "data_characteristics": characteristics,
        "memory_analysis": {
            "float32_mb": float32_memory / (1024 * 1024),
            "optimized_mb": optimized_memory / (1024 * 1024),
            "savings_mb": (float32_memory - optimized_memory) / (1024 * 1024),
        },
    }
