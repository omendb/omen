"""
Vector Quality Monitoring System for OmenDB.

This module provides real-time vector quality assessment, drift detection,
and adaptive monitoring capabilities for both embedded and server modes.
Designed to maintain <1% performance overhead while providing comprehensive
quality insights.
"""

from math import sqrt, log, abs
from memory import UnsafePointer
from algorithm import vectorize
from collections import List, Dict, Optional
from sys.info import simdwidthof
from core.vector import Vector
from core.distance import cosine_distance, euclidean_distance
from util.logging import Logger, LogLevel
from time import perf_counter_ns

# Quality metric thresholds
alias DEFAULT_DRIFT_THRESHOLD = Float32(0.15)
alias DEFAULT_QUALITY_THRESHOLD = Float32(0.8)
alias DEFAULT_SAMPLE_SIZE = 1000
alias PERFORMANCE_OVERHEAD_TARGET = Float32(0.01)  # <1%

struct VectorQualityMetrics(Copyable, Movable):
    """
    Comprehensive quality metrics for vector collections.
    
    Dual-mode design: Works efficiently for both embedded (1K vectors) 
    and server (1B+ vectors) deployments with configurable sampling.
    """
    var mean_magnitude: Float32
    var std_magnitude: Float32
    var mean_dimensionality_usage: Float32  # Non-zero dimension ratio
    var inter_vector_similarity: Float32    # Average pairwise similarity
    var distribution_entropy: Float32       # Vector distribution diversity
    var quality_score: Float32             # Composite quality score [0,1]
    var sample_size: Int                    # Actual sample size used
    var timestamp: Int                      # When metrics were calculated
    
    fn __init__(out self):
        self.mean_magnitude = 0.0
        self.std_magnitude = 0.0
        self.mean_dimensionality_usage = 0.0
        self.inter_vector_similarity = 0.0
        self.distribution_entropy = 0.0
        self.quality_score = 0.0
        self.sample_size = 0
        self.timestamp = perf_counter_ns()
    
    fn __copyinit__(out self, other: Self):
        self.mean_magnitude = other.mean_magnitude
        self.std_magnitude = other.std_magnitude
        self.mean_dimensionality_usage = other.mean_dimensionality_usage
        self.inter_vector_similarity = other.inter_vector_similarity
        self.distribution_entropy = other.distribution_entropy
        self.quality_score = other.quality_score
        self.sample_size = other.sample_size
        self.timestamp = other.timestamp

struct DriftDetectionConfig(Copyable, Movable):
    """
    Configuration for vector drift detection algorithms.
    
    Supports both statistical and ML-based drift detection with
    configurable sensitivity for different use cases.
    """
    var drift_threshold: Float32           # Sensitivity threshold [0,1]
    var window_size: Int                   # Historical comparison window
    var detection_method: Int              # Statistical vs ML detection
    var confidence_level: Float32          # Statistical confidence [0.9-0.99]
    var adaptive_threshold: Bool           # Auto-adjust based on patterns
    var min_samples: Int                   # Minimum samples for detection
    
    # Detection method constants
    alias STATISTICAL_DETECTION: Int = 1
    alias ML_DETECTION: Int = 2
    alias HYBRID_DETECTION: Int = 3
    
    fn __init__(out self, 
                 drift_threshold: Float32 = DEFAULT_DRIFT_THRESHOLD,
                 window_size: Int = 100,
                 detection_method: Int = Self.STATISTICAL_DETECTION,
                 confidence_level: Float32 = 0.95):
        self.drift_threshold = drift_threshold
        self.window_size = window_size
        self.detection_method = detection_method
        self.confidence_level = confidence_level
        self.adaptive_threshold = True
        self.min_samples = 50
    
    fn __copyinit__(out self, other: Self):
        self.drift_threshold = other.drift_threshold
        self.window_size = other.window_size
        self.detection_method = other.detection_method
        self.confidence_level = other.confidence_level
        self.adaptive_threshold = other.adaptive_threshold
        self.min_samples = other.min_samples

struct VectorQualityMonitor[dtype: DType = DType.float32](Copyable, Movable):
    """
    Real-time vector quality monitoring and drift detection system.
    
    Dual-mode implementation:
    - Embedded: Low-latency, memory-efficient monitoring for <50MB footprint
    - Server: Scalable monitoring for billions of vectors with sampling
    
    Performance: <1% overhead, adaptive sampling, configurable sensitivity
    """
    var logger: Logger
    var drift_config: DriftDetectionConfig
    var quality_threshold: Float32
    var sample_size: Int
    var historical_metrics: List[VectorQualityMetrics]
    var performance_overhead: Float32
    var monitoring_enabled: Bool
    
    fn __init__(out self, 
                 drift_config: DriftDetectionConfig = DriftDetectionConfig(),
                 quality_threshold: Float32 = DEFAULT_QUALITY_THRESHOLD,
                 sample_size: Int = DEFAULT_SAMPLE_SIZE):
        self.logger = Logger("VectorQualityMonitor")
        self.drift_config = drift_config
        self.quality_threshold = quality_threshold
        self.sample_size = sample_size
        self.historical_metrics = List[VectorQualityMetrics]()
        self.performance_overhead = 0.0
        self.monitoring_enabled = True
    
    fn __copyinit__(out self, other: Self):
        self.logger = other.logger
        self.drift_config = other.drift_config
        self.quality_threshold = other.quality_threshold
        self.sample_size = other.sample_size
        self.historical_metrics = other.historical_metrics
        self.performance_overhead = other.performance_overhead
        self.monitoring_enabled = other.monitoring_enabled
    
    fn calculate_quality_metrics(self, vectors: List[Vector[dtype]]) -> VectorQualityMetrics:
        """
        Calculate comprehensive quality metrics for vector collection.
        
        Uses statistical sampling for large collections to maintain
        performance targets while providing accurate quality assessment.
        """
        var start_time = perf_counter_ns()
        var metrics = VectorQualityMetrics()
        
        if not self.monitoring_enabled or len(vectors) == 0:
            return metrics
        
        # Adaptive sampling for large collections (server mode)
        var effective_sample_size = min(len(vectors), self.sample_size)
        var sample_indices = self._generate_sample_indices(len(vectors), effective_sample_size)
        
        # Calculate magnitude statistics
        var magnitudes = self._calculate_magnitudes(vectors, sample_indices)
        metrics.mean_magnitude = self._calculate_mean(magnitudes)
        metrics.std_magnitude = self._calculate_std(magnitudes, metrics.mean_magnitude)
        
        # Calculate dimensionality usage (sparsity measure)
        metrics.mean_dimensionality_usage = self._calculate_dimensionality_usage(vectors, sample_indices)
        
        # Calculate inter-vector similarity (diversity measure)
        metrics.inter_vector_similarity = self._calculate_inter_vector_similarity(vectors, sample_indices)
        
        # Calculate distribution entropy (clustering measure)
        metrics.distribution_entropy = self._calculate_distribution_entropy(vectors, sample_indices)
        
        # Composite quality score
        metrics.quality_score = self._calculate_composite_quality_score(metrics)
        
        metrics.sample_size = effective_sample_size
        metrics.timestamp = perf_counter_ns()
        
        # Track performance overhead
        var end_time = perf_counter_ns()
        var monitoring_time = Float32(end_time - start_time) / 1e9  # Convert to seconds
        self._update_performance_overhead(monitoring_time)
        
        self.logger.log(LogLevel.DEBUG, "Quality metrics calculated: quality_score=" + 
                       str(metrics.quality_score) + ", sample_size=" + str(effective_sample_size))
        
        return metrics
    
    fn detect_drift(mut self, current_metrics: VectorQualityMetrics) -> Bool:
        """
        Detect vector drift using statistical analysis of quality metrics.
        
        Returns True if significant drift is detected based on configured
        sensitivity and historical baseline comparison.
        """
        if len(self.historical_metrics) < self.drift_config.min_samples:
            self.historical_metrics.append(current_metrics)
            return False
        
        var drift_detected = False
        
        if self.drift_config.detection_method == DriftDetectionConfig.STATISTICAL_DETECTION:
            drift_detected = self._detect_statistical_drift(current_metrics)
        elif self.drift_config.detection_method == DriftDetectionConfig.ML_DETECTION:
            drift_detected = self._detect_ml_drift(current_metrics)
        else:  # HYBRID_DETECTION
            var stat_drift = self._detect_statistical_drift(current_metrics)
            var ml_drift = self._detect_ml_drift(current_metrics)
            drift_detected = stat_drift or ml_drift
        
        # Add to historical tracking
        self.historical_metrics.append(current_metrics)
        
        # Maintain window size
        if len(self.historical_metrics) > self.drift_config.window_size:
            # Remove oldest metrics to maintain window
            var new_metrics = List[VectorQualityMetrics]()
            for i in range(1, len(self.historical_metrics)):
                new_metrics.append(self.historical_metrics[i])
            self.historical_metrics = new_metrics
        
        if drift_detected:
            self.logger.log(LogLevel.WARNING, "Vector drift detected: quality_score=" + 
                           str(current_metrics.quality_score))
        
        return drift_detected
    
    fn get_quality_recommendations(self, metrics: VectorQualityMetrics) -> List[String]:
        """
        Generate actionable recommendations based on quality metrics.
        
        Provides specific guidance for improving vector quality and
        addressing detected issues.
        """
        var recommendations = List[String]()
        
        if metrics.quality_score < self.quality_threshold:
            recommendations.append("Overall quality below threshold (" + 
                                 str(self.quality_threshold) + ")")
        
        if metrics.mean_dimensionality_usage < 0.3:
            recommendations.append("High sparsity detected - consider dimensionality reduction")
        
        if metrics.inter_vector_similarity > 0.9:
            recommendations.append("Vectors are highly similar - check for duplicates or data quality")
        
        if metrics.distribution_entropy < 2.0:
            recommendations.append("Low diversity in vector distribution - consider data augmentation")
        
        if metrics.std_magnitude > metrics.mean_magnitude:
            recommendations.append("High magnitude variance - consider normalization")
        
        if len(recommendations) == 0:
            recommendations.append("Vector quality metrics within acceptable ranges")
        
        return recommendations
    
    fn get_performance_overhead(self) -> Float32:
        """Return current performance overhead as percentage."""
        return self.performance_overhead * 100.0
    
    fn is_overhead_acceptable(self) -> Bool:
        """Check if performance overhead is within target (<1%)."""
        return self.performance_overhead <= PERFORMANCE_OVERHEAD_TARGET
    
    # Private helper methods
    
    fn _generate_sample_indices(self, total_size: Int, sample_size: Int) -> List[Int]:
        """Generate evenly distributed sample indices for large collections."""
        var indices = List[Int]()
        
        if sample_size >= total_size:
            for i in range(total_size):
                indices.append(i)
        else:
            var step = total_size // sample_size
            for i in range(0, total_size, step):
                if len(indices) < sample_size:
                    indices.append(i)
        
        return indices
    
    fn _calculate_magnitudes(self, vectors: List[Vector[dtype]], indices: List[Int]) -> List[Float32]:
        """Calculate vector magnitudes for sampled vectors."""
        var magnitudes = List[Float32]()
        
        for i in range(len(indices)):
            var idx = indices[i]
            if idx < len(vectors):
                var magnitude = vectors[idx].magnitude()
                magnitudes.append(magnitude)
        
        return magnitudes
    
    fn _calculate_mean(self, values: List[Float32]) -> Float32:
        """Calculate mean of Float32 values."""
        if len(values) == 0:
            return 0.0
        
        var sum = Float32(0.0)
        for i in range(len(values)):
            sum += values[i]
        
        return sum / Float32(len(values))
    
    fn _calculate_std(self, values: List[Float32], mean: Float32) -> Float32:
        """Calculate standard deviation of Float32 values."""
        if len(values) <= 1:
            return 0.0
        
        var sum_sq_diff = Float32(0.0)
        for i in range(len(values)):
            var diff = values[i] - mean
            sum_sq_diff += diff * diff
        
        var variance = sum_sq_diff / Float32(len(values) - 1)
        return sqrt(variance)
    
    fn _calculate_dimensionality_usage(self, vectors: List[Vector[dtype]], indices: List[Int]) -> Float32:
        """Calculate average dimensionality usage (non-zero dimension ratio)."""
        if len(indices) == 0:
            return 0.0
        
        var total_usage = Float32(0.0)
        
        for i in range(len(indices)):
            var idx = indices[i]
            if idx < len(vectors):
                var vector = vectors[idx]
                var non_zero_count = 0
                
                for d in range(vector.dim):
                    if abs(vector.data[d]) > 1e-8:  # Consider very small values as zero
                        non_zero_count += 1
                
                var usage = Float32(non_zero_count) / Float32(vector.dim)
                total_usage += usage
        
        return total_usage / Float32(len(indices))
    
    fn _calculate_inter_vector_similarity(self, vectors: List[Vector[dtype]], indices: List[Int]) -> Float32:
        """Calculate average pairwise cosine similarity for sampled vectors."""
        if len(indices) < 2:
            return 0.0
        
        var total_similarity = Float32(0.0)
        var pair_count = 0
        var max_pairs = min(100, (len(indices) * (len(indices) - 1)) // 2)  # Limit for performance
        
        for i in range(len(indices)):
            for j in range(i + 1, len(indices)):
                if pair_count >= max_pairs:
                    break
                
                var idx1 = indices[i]
                var idx2 = indices[j]
                
                if idx1 < len(vectors) and idx2 < len(vectors):
                    # Use 1 - cosine_distance to get similarity
                    var similarity = 1.0 - cosine_distance(vectors[idx1], vectors[idx2])
                    total_similarity += similarity
                    pair_count += 1
            
            if pair_count >= max_pairs:
                break
        
        return total_similarity / Float32(pair_count) if pair_count > 0 else 0.0
    
    fn _calculate_distribution_entropy(self, vectors: List[Vector[dtype]], indices: List[Int]) -> Float32:
        """Calculate Shannon entropy of vector distribution in reduced space."""
        if len(indices) < 10:  # Need minimum samples for entropy calculation
            return 0.0
        
        # Simple binning approach for entropy calculation
        var num_bins = min(20, len(indices) // 5)
        var bin_counts = List[Int]()
        
        for i in range(num_bins):
            bin_counts.append(0)
        
        # Calculate magnitude range for binning
        var magnitudes = self._calculate_magnitudes(vectors, indices)
        var min_mag = magnitudes[0]
        var max_mag = magnitudes[0]
        
        for i in range(len(magnitudes)):
            min_mag = min(min_mag, magnitudes[i])
            max_mag = max(max_mag, magnitudes[i])
        
        var range_mag = max_mag - min_mag
        if range_mag < 1e-8:
            return 0.0  # All vectors have same magnitude
        
        # Bin the magnitudes
        for i in range(len(magnitudes)):
            var bin_idx = Int((magnitudes[i] - min_mag) / range_mag * Float32(num_bins - 1))
            bin_idx = max(0, min(num_bins - 1, bin_idx))
            bin_counts[bin_idx] += 1
        
        # Calculate entropy
        var entropy = Float32(0.0)
        var total_count = len(magnitudes)
        
        for i in range(num_bins):
            if bin_counts[i] > 0:
                var prob = Float32(bin_counts[i]) / Float32(total_count)
                entropy -= prob * log(prob)
        
        return entropy
    
    fn _calculate_composite_quality_score(self, metrics: VectorQualityMetrics) -> Float32:
        """Calculate composite quality score from individual metrics."""
        # Weighted combination of quality indicators
        var dimensionality_score = min(1.0, metrics.mean_dimensionality_usage / 0.8)
        var diversity_score = max(0.0, 1.0 - metrics.inter_vector_similarity)
        var entropy_score = min(1.0, metrics.distribution_entropy / 4.0)
        var magnitude_consistency = 1.0 / (1.0 + metrics.std_magnitude / max(metrics.mean_magnitude, 1e-8))
        
        # Weighted average (adjust weights based on use case)
        var composite_score = (0.3 * dimensionality_score + 
                              0.3 * diversity_score + 
                              0.2 * entropy_score + 
                              0.2 * magnitude_consistency)
        
        return max(0.0, min(1.0, composite_score))
    
    fn _detect_statistical_drift(self, current_metrics: VectorQualityMetrics) -> Bool:
        """Detect drift using statistical tests on quality metrics."""
        if len(self.historical_metrics) < 10:
            return False
        
        # Calculate baseline statistics from historical data
        var historical_quality_scores = List[Float32]()
        for i in range(len(self.historical_metrics)):
            historical_quality_scores.append(self.historical_metrics[i].quality_score)
        
        var baseline_mean = self._calculate_mean(historical_quality_scores)
        var baseline_std = self._calculate_std(historical_quality_scores, baseline_mean)
        
        # Z-score test for drift detection
        var z_score = abs(current_metrics.quality_score - baseline_mean) / max(baseline_std, 1e-8)
        var critical_value = Float32(2.0)  # ~95% confidence
        
        if self.drift_config.confidence_level > 0.95:
            critical_value = Float32(2.58)  # ~99% confidence
        
        return z_score > critical_value and abs(current_metrics.quality_score - baseline_mean) > self.drift_config.drift_threshold
    
    fn _detect_ml_drift(self, current_metrics: VectorQualityMetrics) -> Bool:
        """Detect drift using ML-based pattern recognition (simplified implementation)."""
        if len(self.historical_metrics) < 20:
            return False
        
        # Simple trend analysis for ML-based detection
        var recent_window = min(10, len(self.historical_metrics))
        var recent_scores = List[Float32]()
        
        var start_idx = len(self.historical_metrics) - recent_window
        for i in range(start_idx, len(self.historical_metrics)):
            recent_scores.append(self.historical_metrics[i].quality_score)
        
        var recent_mean = self._calculate_mean(recent_scores)
        var trend_threshold = self.drift_config.drift_threshold * 0.5
        
        # Detect significant trend changes
        return abs(current_metrics.quality_score - recent_mean) > trend_threshold
    
    fn _update_performance_overhead(mut self, monitoring_time: Float32):
        """Update running average of performance overhead."""
        # Exponential moving average for overhead tracking
        var alpha = Float32(0.1)  # Smoothing factor
        
        if self.performance_overhead == 0.0:
            self.performance_overhead = monitoring_time
        else:
            self.performance_overhead = alpha * monitoring_time + (1.0 - alpha) * self.performance_overhead
        
        # Log warning if overhead exceeds target
        if not self.is_overhead_acceptable():
            self.logger.log(LogLevel.WARNING, "Performance overhead exceeds target: " + 
                           str(self.get_performance_overhead()) + "%")