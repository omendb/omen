"""
Advanced Drift Detection System for OmenDB Vector Quality Monitoring.

This module provides sophisticated drift detection algorithms including
statistical tests, machine learning-based detection, and adaptive thresholding
for both embedded and server deployment modes.
"""

from math import sqrt, log, abs, exp
from memory import UnsafePointer
from collections import List, Dict, Optional
from core.vector import Vector
from core.distance import cosine_distance, euclidean_distance
from util.logging import Logger, LogLevel
from time import perf_counter_ns
from .vector_quality import VectorQualityMetrics, DriftDetectionConfig

# Advanced drift detection constants
alias DRIFT_SENSITIVITY_HIGH = Float32(0.05)
alias DRIFT_SENSITIVITY_MEDIUM = Float32(0.15)
alias DRIFT_SENSITIVITY_LOW = Float32(0.30)
alias ADAPTIVE_WINDOW_MIN = 50
alias ADAPTIVE_WINDOW_MAX = 500

struct DriftAlert(Copyable, Movable):
    """
    Alert structure for drift detection events.
    
    Contains detailed information about detected drift including
    severity, affected metrics, and recommended actions.
    """
    var severity: Int                    # Alert severity level
    var drift_type: Int                  # Type of drift detected
    var affected_metrics: List[String]   # Which metrics triggered alert
    var confidence: Float32              # Detection confidence [0,1]
    var timestamp: Int                   # When drift was detected
    var description: String              # Human-readable description
    var recommendations: List[String]    # Suggested actions
    
    # Severity levels
    alias SEVERITY_LOW: Int = 1
    alias SEVERITY_MEDIUM: Int = 2
    alias SEVERITY_HIGH: Int = 3
    alias SEVERITY_CRITICAL: Int = 4
    
    # Drift types
    alias DRIFT_GRADUAL: Int = 1         # Slow, continuous drift
    alias DRIFT_SUDDEN: Int = 2          # Abrupt change
    alias DRIFT_CYCLIC: Int = 3          # Periodic pattern changes
    alias DRIFT_OUTLIER: Int = 4         # Single anomalous batch
    
    fn __init__(out self):
        self.severity = Self.SEVERITY_LOW
        self.drift_type = Self.DRIFT_GRADUAL
        self.affected_metrics = List[String]()
        self.confidence = 0.0
        self.timestamp = perf_counter_ns()
        self.description = ""
        self.recommendations = List[String]()
    
    fn __copyinit__(out self, other: Self):
        self.severity = other.severity
        self.drift_type = other.drift_type
        self.affected_metrics = other.affected_metrics
        self.confidence = other.confidence
        self.timestamp = other.timestamp
        self.description = other.description
        self.recommendations = other.recommendations

struct AdaptiveThreshold(Copyable, Movable):
    """
    Adaptive threshold management for dynamic drift sensitivity.
    
    Automatically adjusts detection sensitivity based on historical
    patterns and data characteristics.
    """
    var base_threshold: Float32
    var current_threshold: Float32
    var adaptation_rate: Float32
    var stability_factor: Float32
    var recent_detections: List[Bool]
    var adjustment_history: List[Float32]
    
    fn __init__(out self, base_threshold: Float32 = DRIFT_SENSITIVITY_MEDIUM):
        self.base_threshold = base_threshold
        self.current_threshold = base_threshold
        self.adaptation_rate = 0.1
        self.stability_factor = 0.95
        self.recent_detections = List[Bool]()
        self.adjustment_history = List[Float32]()
    
    fn __copyinit__(out self, other: Self):
        self.base_threshold = other.base_threshold
        self.current_threshold = other.current_threshold
        self.adaptation_rate = other.adaptation_rate
        self.stability_factor = other.stability_factor
        self.recent_detections = other.recent_detections
        self.adjustment_history = other.adjustment_history
    
    fn update_threshold(mut self, detection_result: Bool, confidence: Float32):
        """Update adaptive threshold based on recent detection patterns."""
        self.recent_detections.append(detection_result)
        
        # Keep recent window for adaptation
        if len(self.recent_detections) > 20:
            var new_detections = List[Bool]()
            for i in range(1, len(self.recent_detections)):
                new_detections.append(self.recent_detections[i])
            self.recent_detections = new_detections
        
        # Calculate false positive rate estimation
        var detection_rate = self._calculate_detection_rate()
        
        # Adapt threshold based on detection patterns
        if detection_rate > 0.3:  # Too many detections, increase threshold
            self.current_threshold *= (1.0 + self.adaptation_rate)
        elif detection_rate < 0.05:  # Too few detections, decrease threshold
            self.current_threshold *= (1.0 - self.adaptation_rate)
        
        # Apply stability factor to prevent oscillation
        self.current_threshold = (self.stability_factor * self.current_threshold + 
                                 (1.0 - self.stability_factor) * self.base_threshold)
        
        # Clamp to reasonable bounds
        self.current_threshold = max(0.01, min(0.5, self.current_threshold))
        
        self.adjustment_history.append(self.current_threshold)
    
    fn get_threshold(self) -> Float32:
        """Get current adaptive threshold."""
        return self.current_threshold
    
    fn _calculate_detection_rate(self) -> Float32:
        """Calculate recent detection rate for threshold adaptation."""
        if len(self.recent_detections) == 0:
            return 0.0
        
        var detection_count = 0
        for i in range(len(self.recent_detections)):
            if self.recent_detections[i]:
                detection_count += 1
        
        return Float32(detection_count) / Float32(len(self.recent_detections))

struct AdvancedDriftDetector[dtype: DType = DType.float32](Copyable, Movable):
    """
    Advanced drift detection system with multiple algorithms and adaptive thresholding.
    
    Supports:
    - Statistical drift detection (KS test, Z-score, trend analysis)
    - Machine learning-based detection (pattern recognition)
    - Adaptive threshold management
    - Multi-metric drift analysis
    - Performance-aware operation (<1% overhead)
    """
    var logger: Logger
    var config: DriftDetectionConfig
    var adaptive_threshold: AdaptiveThreshold
    var historical_metrics: List[VectorQualityMetrics]
    var recent_alerts: List[DriftAlert]
    var detection_statistics: Dict[String, Float32]
    var performance_tracker: Dict[String, Float32]
    
    fn __init__(out self, config: DriftDetectionConfig = DriftDetectionConfig()):
        self.logger = Logger("AdvancedDriftDetector")
        self.config = config
        self.adaptive_threshold = AdaptiveThreshold(config.drift_threshold)
        self.historical_metrics = List[VectorQualityMetrics]()
        self.recent_alerts = List[DriftAlert]()
        self.detection_statistics = Dict[String, Float32]()
        self.performance_tracker = Dict[String, Float32]()
        
        # Initialize statistics tracking
        self.detection_statistics["total_detections"] = 0.0
        self.detection_statistics["false_positives"] = 0.0
        self.detection_statistics["detection_rate"] = 0.0
    
    fn __copyinit__(out self, other: Self):
        self.logger = other.logger
        self.config = other.config
        self.adaptive_threshold = other.adaptive_threshold
        self.historical_metrics = other.historical_metrics
        self.recent_alerts = other.recent_alerts
        self.detection_statistics = other.detection_statistics
        self.performance_tracker = other.performance_tracker
    
    fn detect_comprehensive_drift(mut self, current_metrics: VectorQualityMetrics) -> Optional[DriftAlert]:
        """
        Comprehensive drift detection using multiple algorithms and metrics.
        
        Returns detailed drift alert if significant drift is detected,
        None otherwise. Uses adaptive thresholding and confidence scoring.
        """
        var start_time = perf_counter_ns()
        
        if len(self.historical_metrics) < self.config.min_samples:
            self.historical_metrics.append(current_metrics)
            return Optional[DriftAlert]()
        
        var alert = DriftAlert()
        var drift_detected = False
        var max_confidence = Float32(0.0)
        
        # Multi-algorithm drift detection
        var statistical_result = self._detect_statistical_drift_advanced(current_metrics)
        var ml_result = self._detect_ml_drift_advanced(current_metrics)
        var trend_result = self._detect_trend_drift(current_metrics)
        
        # Combine results with confidence weighting
        if statistical_result.confidence > self.adaptive_threshold.get_threshold():
            drift_detected = True
            max_confidence = max(max_confidence, statistical_result.confidence)
            alert.affected_metrics.append("statistical_metrics")
        
        if ml_result.confidence > self.adaptive_threshold.get_threshold():
            drift_detected = True
            max_confidence = max(max_confidence, ml_result.confidence)
            alert.affected_metrics.append("pattern_metrics")
        
        if trend_result.confidence > self.adaptive_threshold.get_threshold():
            drift_detected = True
            max_confidence = max(max_confidence, trend_result.confidence)
            alert.affected_metrics.append("trend_metrics")
        
        # Update adaptive threshold
        self.adaptive_threshold.update_threshold(drift_detected, max_confidence)
        
        # Create detailed alert if drift detected
        if drift_detected:
            alert.confidence = max_confidence
            alert.severity = self._calculate_severity(max_confidence)
            alert.drift_type = self._classify_drift_type(current_metrics)
            alert.description = self._generate_drift_description(alert)
            alert.recommendations = self._generate_drift_recommendations(alert, current_metrics)
            
            self.recent_alerts.append(alert)
            self._update_detection_statistics(True)
            
            self.logger.log(LogLevel.WARNING, "Advanced drift detected: confidence=" + 
                           str(max_confidence) + ", type=" + str(alert.drift_type))
        else:
            self._update_detection_statistics(False)
        
        # Maintain historical window
        self.historical_metrics.append(current_metrics)
        if len(self.historical_metrics) > self.config.window_size:
            var new_metrics = List[VectorQualityMetrics]()
            for i in range(1, len(self.historical_metrics)):
                new_metrics.append(self.historical_metrics[i])
            self.historical_metrics = new_metrics
        
        # Track performance
        var end_time = perf_counter_ns()
        var detection_time = Float32(end_time - start_time) / 1e9
        self._update_performance_tracking("drift_detection", detection_time)
        
        return Optional[DriftAlert](alert) if drift_detected else Optional[DriftAlert]()
    
    fn get_detection_statistics(self) -> Dict[String, Float32]:
        """Get comprehensive detection statistics for monitoring."""
        return self.detection_statistics
    
    fn get_recent_alerts(self, limit: Int = 10) -> List[DriftAlert]:
        """Get recent drift alerts up to specified limit."""
        var alerts = List[DriftAlert]()
        var start_idx = max(0, len(self.recent_alerts) - limit)
        
        for i in range(start_idx, len(self.recent_alerts)):
            alerts.append(self.recent_alerts[i])
        
        return alerts
    
    fn reset_detection_state(mut self):
        """Reset detection state for clean start."""
        self.historical_metrics = List[VectorQualityMetrics]()
        self.recent_alerts = List[DriftAlert]()
        self.adaptive_threshold = AdaptiveThreshold(self.config.drift_threshold)
        self.detection_statistics["total_detections"] = 0.0
        self.detection_statistics["false_positives"] = 0.0
        self.detection_statistics["detection_rate"] = 0.0
    
    # Private detection algorithms
    
    struct DetectionResult(Copyable, Movable):
        var detected: Bool
        var confidence: Float32
        var details: String
        
        fn __init__(out self, detected: Bool = False, confidence: Float32 = 0.0, details: String = ""):
            self.detected = detected
            self.confidence = confidence
            self.details = details
        
        fn __copyinit__(out self, other: Self):
            self.detected = other.detected
            self.confidence = other.confidence
            self.details = other.details
    
    fn _detect_statistical_drift_advanced(self, current_metrics: VectorQualityMetrics) -> DetectionResult:
        """Advanced statistical drift detection using multiple tests."""
        var result = DetectionResult()
        
        if len(self.historical_metrics) < 10:
            return result
        
        # Kolmogorov-Smirnov test approximation for quality scores
        var ks_confidence = self._kolmogorov_smirnov_test(current_metrics)
        
        # Z-score test with Bonferroni correction for multiple metrics
        var z_confidence = self._multi_metric_z_test(current_metrics)
        
        # Welch's t-test for mean differences
        var t_confidence = self._welch_t_test(current_metrics)
        
        # Combine statistical tests
        var combined_confidence = max(ks_confidence, max(z_confidence, t_confidence))
        
        result.detected = combined_confidence > self.config.drift_threshold
        result.confidence = combined_confidence
        result.details = "Statistical tests: KS=" + str(ks_confidence) + 
                        ", Z=" + str(z_confidence) + ", T=" + str(t_confidence)
        
        return result
    
    fn _detect_ml_drift_advanced(self, current_metrics: VectorQualityMetrics) -> DetectionResult:
        """ML-based drift detection using pattern analysis."""
        var result = DetectionResult()
        
        if len(self.historical_metrics) < 20:
            return result
        
        # Pattern-based detection using time series analysis
        var pattern_confidence = self._detect_pattern_anomaly(current_metrics)
        
        # Ensemble outlier detection
        var outlier_confidence = self._detect_multivariate_outlier(current_metrics)
        
        # Temporal sequence analysis
        var sequence_confidence = self._analyze_temporal_sequence(current_metrics)
        
        # Combine ML-based methods
        var combined_confidence = (0.4 * pattern_confidence + 
                                  0.3 * outlier_confidence + 
                                  0.3 * sequence_confidence)
        
        result.detected = combined_confidence > self.config.drift_threshold
        result.confidence = combined_confidence
        result.details = "ML tests: Pattern=" + str(pattern_confidence) + 
                        ", Outlier=" + str(outlier_confidence) + 
                        ", Sequence=" + str(sequence_confidence)
        
        return result
    
    fn _detect_trend_drift(self, current_metrics: VectorQualityMetrics) -> DetectionResult:
        """Trend-based drift detection for gradual changes."""
        var result = DetectionResult()
        
        if len(self.historical_metrics) < 15:
            return result
        
        # Linear trend analysis
        var trend_strength = self._calculate_trend_strength()
        
        # Change point detection
        var changepoint_confidence = self._detect_change_point(current_metrics)
        
        # Volatility analysis
        var volatility_confidence = self._analyze_volatility_change(current_metrics)
        
        var combined_confidence = max(trend_strength, max(changepoint_confidence, volatility_confidence))
        
        result.detected = combined_confidence > self.config.drift_threshold
        result.confidence = combined_confidence
        result.details = "Trend analysis: Strength=" + str(trend_strength) + 
                        ", Change=" + str(changepoint_confidence) + 
                        ", Volatility=" + str(volatility_confidence)
        
        return result
    
    # Statistical test implementations
    
    fn _kolmogorov_smirnov_test(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Simplified KS test for distribution comparison."""
        # Collect historical quality scores
        var historical_scores = List[Float32]()
        for i in range(len(self.historical_metrics)):
            historical_scores.append(self.historical_metrics[i].quality_score)
        
        # Simple distribution comparison (approximation)
        var hist_mean = self._calculate_mean(historical_scores)
        var hist_std = self._calculate_std(historical_scores, hist_mean)
        
        if hist_std < 1e-8:
            return 0.0
        
        var z_score = abs(current_metrics.quality_score - hist_mean) / hist_std
        return min(1.0, z_score / 3.0)  # Normalize to [0,1]
    
    fn _multi_metric_z_test(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Multi-metric Z-test with Bonferroni correction."""
        var max_z = Float32(0.0)
        var num_tests = 5  # Number of metrics tested
        
        # Test multiple metrics
        var quality_z = self._z_score_metric("quality_score", current_metrics.quality_score)
        var magnitude_z = self._z_score_metric("mean_magnitude", current_metrics.mean_magnitude)
        var similarity_z = self._z_score_metric("inter_vector_similarity", current_metrics.inter_vector_similarity)
        var entropy_z = self._z_score_metric("distribution_entropy", current_metrics.distribution_entropy)
        var usage_z = self._z_score_metric("dimensionality_usage", current_metrics.mean_dimensionality_usage)
        
        max_z = max(quality_z, max(magnitude_z, max(similarity_z, max(entropy_z, usage_z))))
        
        # Apply Bonferroni correction for multiple testing
        var corrected_threshold = 2.0 * sqrt(Float32(num_tests))
        return min(1.0, max_z / corrected_threshold)
    
    fn _welch_t_test(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Welch's t-test for mean difference (simplified)."""
        if len(self.historical_metrics) < 5:
            return 0.0
        
        var recent_scores = List[Float32]()
        var start_idx = max(0, len(self.historical_metrics) - 10)
        
        for i in range(start_idx, len(self.historical_metrics)):
            recent_scores.append(self.historical_metrics[i].quality_score)
        
        var recent_mean = self._calculate_mean(recent_scores)
        var recent_std = self._calculate_std(recent_scores, recent_mean)
        
        if recent_std < 1e-8:
            return 0.0
        
        var t_stat = abs(current_metrics.quality_score - recent_mean) / (recent_std / sqrt(Float32(len(recent_scores))))
        return min(1.0, t_stat / 3.0)
    
    # ML-based detection methods
    
    fn _detect_pattern_anomaly(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Pattern anomaly detection using local outlier factor approximation."""
        var window_size = min(20, len(self.historical_metrics))
        var start_idx = len(self.historical_metrics) - window_size
        
        var distances = List[Float32]()
        
        # Calculate distances to recent neighbors
        for i in range(start_idx, len(self.historical_metrics)):
            var hist = self.historical_metrics[i]
            var distance = self._calculate_metric_distance(current_metrics, hist)
            distances.append(distance)
        
        if len(distances) == 0:
            return 0.0
        
        # Local outlier factor approximation
        var mean_distance = self._calculate_mean(distances)
        var std_distance = self._calculate_std(distances, mean_distance)
        
        if std_distance < 1e-8:
            return 0.0
        
        var outlier_score = (mean_distance - min(distances[0], distances[len(distances)-1])) / std_distance
        return min(1.0, max(0.0, outlier_score / 2.0))
    
    fn _detect_multivariate_outlier(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Multivariate outlier detection using Mahalanobis distance approximation."""
        # Simplified multivariate analysis
        var feature_scores = List[Float32]()
        
        feature_scores.append(self._z_score_metric("quality_score", current_metrics.quality_score))
        feature_scores.append(self._z_score_metric("mean_magnitude", current_metrics.mean_magnitude))
        feature_scores.append(self._z_score_metric("inter_vector_similarity", current_metrics.inter_vector_similarity))
        
        # Calculate combined score (simplified Mahalanobis)
        var combined_score = Float32(0.0)
        for i in range(len(feature_scores)):
            combined_score += feature_scores[i] * feature_scores[i]
        
        combined_score = sqrt(combined_score / Float32(len(feature_scores)))
        return min(1.0, combined_score / 3.0)
    
    fn _analyze_temporal_sequence(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Temporal sequence analysis for drift patterns."""
        if len(self.historical_metrics) < 10:
            return 0.0
        
        # Analyze recent trend vs current value
        var recent_window = min(5, len(self.historical_metrics))
        var trend_scores = List[Float32]()
        
        var start_idx = len(self.historical_metrics) - recent_window
        for i in range(start_idx, len(self.historical_metrics)):
            trend_scores.append(self.historical_metrics[i].quality_score)
        
        var trend_mean = self._calculate_mean(trend_scores)
        var prediction_error = abs(current_metrics.quality_score - trend_mean)
        
        # Normalize prediction error
        var trend_std = self._calculate_std(trend_scores, trend_mean)
        if trend_std < 1e-8:
            return 0.0
        
        return min(1.0, prediction_error / (2.0 * trend_std))
    
    # Helper methods
    
    fn _calculate_trend_strength(self) -> Float32:
        """Calculate trend strength using linear regression approximation."""
        if len(self.historical_metrics) < 10:
            return 0.0
        
        var scores = List[Float32]()
        for i in range(len(self.historical_metrics)):
            scores.append(self.historical_metrics[i].quality_score)
        
        # Simple linear trend calculation
        var n = Float32(len(scores))
        var sum_x = n * (n - 1.0) / 2.0
        var sum_y = self._calculate_mean(scores) * n
        var sum_xy = Float32(0.0)
        var sum_x2 = n * (n - 1.0) * (2.0 * n - 1.0) / 6.0
        
        for i in range(len(scores)):
            sum_xy += Float32(i) * scores[i]
        
        var slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x)
        return min(1.0, abs(slope) * 10.0)  # Scale for detection
    
    fn _detect_change_point(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Simple change point detection."""
        if len(self.historical_metrics) < 20:
            return 0.0
        
        var mid_point = len(self.historical_metrics) // 2
        
        # Compare first half vs second half statistics
        var first_half = List[Float32]()
        var second_half = List[Float32]()
        
        for i in range(mid_point):
            first_half.append(self.historical_metrics[i].quality_score)
        
        for i in range(mid_point, len(self.historical_metrics)):
            second_half.append(self.historical_metrics[i].quality_score)
        
        var first_mean = self._calculate_mean(first_half)
        var second_mean = self._calculate_mean(second_half)
        
        var change_magnitude = abs(current_metrics.quality_score - second_mean) / max(abs(first_mean - second_mean), 1e-8)
        return min(1.0, change_magnitude)
    
    fn _analyze_volatility_change(self, current_metrics: VectorQualityMetrics) -> Float32:
        """Analyze changes in volatility patterns."""
        if len(self.historical_metrics) < 15:
            return 0.0
        
        var recent_window = min(10, len(self.historical_metrics))
        var start_idx = len(self.historical_metrics) - recent_window
        
        var recent_scores = List[Float32]()
        for i in range(start_idx, len(self.historical_metrics)):
            recent_scores.append(self.historical_metrics[i].quality_score)
        
        var recent_std = self._calculate_std(recent_scores, self._calculate_mean(recent_scores))
        
        # Compare with historical volatility
        var all_scores = List[Float32]()
        for i in range(len(self.historical_metrics)):
            all_scores.append(self.historical_metrics[i].quality_score)
        
        var historical_std = self._calculate_std(all_scores, self._calculate_mean(all_scores))
        
        if historical_std < 1e-8:
            return 0.0
        
        var volatility_change = abs(recent_std - historical_std) / historical_std
        return min(1.0, volatility_change)
    
    fn _z_score_metric(self, metric_name: String, current_value: Float32) -> Float32:
        """Calculate Z-score for a specific metric."""
        var values = List[Float32]()
        
        for i in range(len(self.historical_metrics)):
            var hist = self.historical_metrics[i]
            if metric_name == "quality_score":
                values.append(hist.quality_score)
            elif metric_name == "mean_magnitude":
                values.append(hist.mean_magnitude)
            elif metric_name == "inter_vector_similarity":
                values.append(hist.inter_vector_similarity)
            elif metric_name == "distribution_entropy":
                values.append(hist.distribution_entropy)
            elif metric_name == "dimensionality_usage":
                values.append(hist.mean_dimensionality_usage)
        
        if len(values) == 0:
            return 0.0
        
        var mean_val = self._calculate_mean(values)
        var std_val = self._calculate_std(values, mean_val)
        
        if std_val < 1e-8:
            return 0.0
        
        return abs(current_value - mean_val) / std_val
    
    fn _calculate_metric_distance(self, m1: VectorQualityMetrics, m2: VectorQualityMetrics) -> Float32:
        """Calculate distance between two metric sets."""
        var quality_diff = abs(m1.quality_score - m2.quality_score)
        var magnitude_diff = abs(m1.mean_magnitude - m2.mean_magnitude)
        var similarity_diff = abs(m1.inter_vector_similarity - m2.inter_vector_similarity)
        var entropy_diff = abs(m1.distribution_entropy - m2.distribution_entropy)
        var usage_diff = abs(m1.mean_dimensionality_usage - m2.mean_dimensionality_usage)
        
        # Euclidean distance in normalized metric space
        return sqrt(quality_diff*quality_diff + magnitude_diff*magnitude_diff + 
                   similarity_diff*similarity_diff + entropy_diff*entropy_diff + 
                   usage_diff*usage_diff)
    
    fn _calculate_severity(self, confidence: Float32) -> Int:
        """Calculate alert severity based on confidence."""
        if confidence > 0.8:
            return DriftAlert.SEVERITY_CRITICAL
        elif confidence > 0.6:
            return DriftAlert.SEVERITY_HIGH
        elif confidence > 0.4:
            return DriftAlert.SEVERITY_MEDIUM
        else:
            return DriftAlert.SEVERITY_LOW
    
    fn _classify_drift_type(self, current_metrics: VectorQualityMetrics) -> Int:
        """Classify the type of drift detected."""
        # Simple heuristic-based classification
        var trend_strength = self._calculate_trend_strength()
        
        if trend_strength > 0.7:
            return DriftAlert.DRIFT_GRADUAL
        elif len(self.historical_metrics) > 5:
            var recent_std = self._calculate_recent_volatility()
            if recent_std > 0.5:
                return DriftAlert.DRIFT_SUDDEN
            else:
                return DriftAlert.DRIFT_OUTLIER
        else:
            return DriftAlert.DRIFT_OUTLIER
    
    fn _generate_drift_description(self, alert: DriftAlert) -> String:
        """Generate human-readable drift description."""
        var severity_str = "Low"
        if alert.severity == DriftAlert.SEVERITY_HIGH:
            severity_str = "High"
        elif alert.severity == DriftAlert.SEVERITY_CRITICAL:
            severity_str = "Critical"
        elif alert.severity == DriftAlert.SEVERITY_MEDIUM:
            severity_str = "Medium"
        
        var type_str = "gradual"
        if alert.drift_type == DriftAlert.DRIFT_SUDDEN:
            type_str = "sudden"
        elif alert.drift_type == DriftAlert.DRIFT_OUTLIER:
            type_str = "outlier"
        elif alert.drift_type == DriftAlert.DRIFT_CYCLIC:
            type_str = "cyclic"
        
        return severity_str + " " + type_str + " drift detected with " + str(Int(alert.confidence * 100)) + "% confidence"
    
    fn _generate_drift_recommendations(self, alert: DriftAlert, metrics: VectorQualityMetrics) -> List[String]:
        """Generate actionable recommendations for detected drift."""
        var recommendations = List[String]()
        
        if alert.severity >= DriftAlert.SEVERITY_HIGH:
            recommendations.append("Immediate investigation required - check data pipeline")
            recommendations.append("Review recent model updates or data source changes")
        
        if alert.drift_type == DriftAlert.DRIFT_SUDDEN:
            recommendations.append("Check for data ingestion anomalies or system changes")
        elif alert.drift_type == DriftAlert.DRIFT_GRADUAL:
            recommendations.append("Consider model retraining or adaptation")
            recommendations.append("Monitor trend continuation over next few batches")
        
        if metrics.quality_score < 0.5:
            recommendations.append("Vector quality degraded - validate data preprocessing")
        
        recommendations.append("Increase monitoring frequency temporarily")
        
        return recommendations
    
    fn _calculate_recent_volatility(self) -> Float32:
        """Calculate recent volatility for drift classification."""
        if len(self.historical_metrics) < 5:
            return 0.0
        
        var recent_scores = List[Float32]()
        var start_idx = max(0, len(self.historical_metrics) - 5)
        
        for i in range(start_idx, len(self.historical_metrics)):
            recent_scores.append(self.historical_metrics[i].quality_score)
        
        var mean_score = self._calculate_mean(recent_scores)
        return self._calculate_std(recent_scores, mean_score)
    
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
    
    fn _update_detection_statistics(mut self, detected: Bool):
        """Update detection statistics for monitoring."""
        self.detection_statistics["total_detections"] += 1.0
        
        if detected:
            var current_rate = self.detection_statistics["detection_rate"]
            var total = self.detection_statistics["total_detections"]
            self.detection_statistics["detection_rate"] = (current_rate * (total - 1.0) + 1.0) / total
        
        # Simple false positive estimation (would need ground truth in practice)
        # This is a placeholder for more sophisticated false positive tracking
    
    fn _update_performance_tracking(mut self, operation: String, duration: Float32):
        """Update performance tracking for overhead monitoring."""
        var key = operation + "_avg_time"
        
        if key in self.performance_tracker:
            var current_avg = self.performance_tracker[key]
            self.performance_tracker[key] = 0.9 * current_avg + 0.1 * duration
        else:
            self.performance_tracker[key] = duration