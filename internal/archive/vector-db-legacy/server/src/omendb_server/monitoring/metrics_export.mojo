"""
Metrics Export System for OmenDB Vector Quality Monitoring.

This module provides dashboard-ready metrics export capabilities including
JSON serialization, time-series data formatting, and real-time streaming
for both embedded and server deployment modes.
"""

from collections import List, Dict, Optional
from util.logging import Logger, LogLevel
from time import perf_counter_ns
from .vector_quality import VectorQualityMetrics
from .drift_detection import DriftAlert
from .performance_monitor import PerformanceMetrics
from .adaptive_integration import IndexOptimizationStrategy

# Export format constants
alias EXPORT_FORMAT_JSON = 1
alias EXPORT_FORMAT_PROMETHEUS = 2
alias EXPORT_FORMAT_CSV = 3
alias EXPORT_FORMAT_INFLUX = 4

struct MetricsSnapshot(Copyable, Movable):
    """
    Complete snapshot of monitoring metrics at a point in time.
    
    Aggregates all monitoring data into a single exportable structure
    suitable for dashboard consumption and time-series analysis.
    """
    var timestamp: Int                          # Snapshot timestamp
    var quality_metrics: VectorQualityMetrics   # Current quality metrics
    var drift_alerts: List[DriftAlert]          # Recent drift alerts
    var performance_metrics: Dict[String, Float32]  # Performance summary
    var adaptation_recommendations: List[String]    # Current recommendations
    var system_health: Dict[String, Float32]    # Overall system health
    var metadata: Dict[String, String]          # Additional metadata
    
    fn __init__(out self):
        self.timestamp = perf_counter_ns()
        self.quality_metrics = VectorQualityMetrics()
        self.drift_alerts = List[DriftAlert]()
        self.performance_metrics = Dict[String, Float32]()
        self.adaptation_recommendations = List[String]()
        self.system_health = Dict[String, Float32]()
        self.metadata = Dict[String, String]()
    
    fn __copyinit__(out self, other: Self):
        self.timestamp = other.timestamp
        self.quality_metrics = other.quality_metrics
        self.drift_alerts = other.drift_alerts
        self.performance_metrics = other.performance_metrics
        self.adaptation_recommendations = other.adaptation_recommendations
        self.system_health = other.system_health
        self.metadata = other.metadata

struct TimeSeriesData(Copyable, Movable):
    """
    Time-series data container for historical metrics tracking.
    
    Efficiently stores and manages historical metrics data with
    configurable retention and aggregation policies.
    """
    var timestamps: List[Int]                   # Timestamp sequence
    var quality_scores: List[Float32]           # Quality score history
    var drift_detections: List[Bool]            # Drift detection history
    var performance_values: Dict[String, List[Float32]]  # Performance metrics
    var retention_hours: Int                    # Data retention period
    var aggregation_interval_ms: Int            # Aggregation interval
    
    fn __init__(out self, retention_hours: Int = 24, aggregation_interval_ms: Int = 60000):
        self.timestamps = List[Int]()
        self.quality_scores = List[Float32]()
        self.drift_detections = List[Bool]()
        self.performance_values = Dict[String, List[Float32]]()
        self.retention_hours = retention_hours
        self.aggregation_interval_ms = aggregation_interval_ms
    
    fn __copyinit__(out self, other: Self):
        self.timestamps = other.timestamps
        self.quality_scores = other.quality_scores
        self.drift_detections = other.drift_detections
        self.performance_values = other.performance_values
        self.retention_hours = other.retention_hours
        self.aggregation_interval_ms = other.aggregation_interval_ms
    
    fn add_data_point(mut self, snapshot: MetricsSnapshot):
        """Add new data point to time series."""
        self.timestamps.append(snapshot.timestamp)
        self.quality_scores.append(snapshot.quality_metrics.quality_score)
        self.drift_detections.append(len(snapshot.drift_alerts) > 0)
        
        # Add performance metrics
        for key in snapshot.performance_metrics:
            var value = snapshot.performance_metrics[key]
            if key not in self.performance_values:
                self.performance_values[key] = List[Float32]()
            self.performance_values[key].append(value)
        
        # Maintain retention policy
        self._enforce_retention()
    
    fn get_aggregated_data(self, metric_name: String, window_minutes: Int) -> List[Float32]:
        """Get aggregated data for specified metric and time window."""
        var aggregated = List[Float32]()
        
        if metric_name == "quality_score":
            aggregated = self._aggregate_values(self.quality_scores, window_minutes)
        elif metric_name in self.performance_values:
            aggregated = self._aggregate_values(self.performance_values[metric_name], window_minutes)
        
        return aggregated
    
    fn _enforce_retention(mut self):
        """Enforce data retention policy."""
        var current_time = perf_counter_ns()
        var retention_ns = Int(self.retention_hours) * 3600 * 1000000000  # Convert to ns
        var cutoff_time = current_time - retention_ns
        
        # Find first index to keep
        var keep_start = 0
        for i in range(len(self.timestamps)):
            if self.timestamps[i] >= cutoff_time:
                keep_start = i
                break
        
        if keep_start > 0:
            self._trim_data(keep_start)
    
    fn _trim_data(mut self, start_index: Int):
        """Trim data arrays starting from specified index."""
        var new_timestamps = List[Int]()
        var new_quality_scores = List[Float32]()
        var new_drift_detections = List[Bool]()
        
        for i in range(start_index, len(self.timestamps)):
            new_timestamps.append(self.timestamps[i])
            new_quality_scores.append(self.quality_scores[i])
            new_drift_detections.append(self.drift_detections[i])
        
        self.timestamps = new_timestamps
        self.quality_scores = new_quality_scores
        self.drift_detections = new_drift_detections
        
        # Trim performance values
        for key in self.performance_values:
            var new_values = List[Float32]()
            var old_values = self.performance_values[key]
            
            for i in range(start_index, min(len(old_values), len(self.timestamps))):
                new_values.append(old_values[i])
            
            self.performance_values[key] = new_values
    
    fn _aggregate_values(self, values: List[Float32], window_minutes: Int) -> List[Float32]:
        """Aggregate values over specified time window."""
        var aggregated = List[Float32]()
        var window_ns = Int(window_minutes) * 60 * 1000000000  # Convert to ns
        
        if len(values) == 0 or len(self.timestamps) == 0:
            return aggregated
        
        var current_time = self.timestamps[len(self.timestamps) - 1]
        var window_start = current_time - window_ns
        
        # Simple moving average aggregation
        var sum = Float32(0.0)
        var count = 0
        
        for i in range(len(self.timestamps)):
            if self.timestamps[i] >= window_start and i < len(values):
                sum += values[i]
                count += 1
        
        if count > 0:
            aggregated.append(sum / Float32(count))
        
        return aggregated

struct MetricsExporter(Copyable, Movable):
    """
    Comprehensive metrics export system for dashboard integration.
    
    Dual-mode design:
    - Embedded: Lightweight export with minimal memory footprint
    - Server: Full-featured export with multiple format support
    
    Features:
    - Multiple export formats (JSON, Prometheus, CSV, InfluxDB)
    - Real-time streaming capabilities
    - Time-series data aggregation
    - Dashboard-ready formatting
    """
    var logger: Logger
    var time_series: TimeSeriesData
    var current_snapshot: MetricsSnapshot
    var export_config: Dict[String, String]
    var streaming_enabled: Bool
    var export_interval_ms: Int
    var last_export_time: Int
    
    fn __init__(out self, export_interval_ms: Int = 60000):
        self.logger = Logger("MetricsExporter")
        self.time_series = TimeSeriesData()
        self.current_snapshot = MetricsSnapshot()
        self.export_config = Dict[String, String]()
        self.streaming_enabled = False
        self.export_interval_ms = export_interval_ms
        self.last_export_time = 0
        
        # Default configuration
        self.export_config["format"] = "json"
        self.export_config["include_historical"] = "true"
        self.export_config["aggregation_window"] = "5"  # minutes
    
    fn __copyinit__(out self, other: Self):
        self.logger = other.logger
        self.time_series = other.time_series
        self.current_snapshot = other.current_snapshot
        self.export_config = other.export_config
        self.streaming_enabled = other.streaming_enabled
        self.export_interval_ms = other.export_interval_ms
        self.last_export_time = other.last_export_time
    
    fn update_metrics(mut self, 
                      quality_metrics: VectorQualityMetrics,
                      drift_alerts: List[DriftAlert],
                      performance_metrics: Dict[String, Float32],
                      recommendations: List[String]):
        """Update current metrics snapshot."""
        self.current_snapshot.timestamp = perf_counter_ns()
        self.current_snapshot.quality_metrics = quality_metrics
        self.current_snapshot.drift_alerts = drift_alerts
        self.current_snapshot.performance_metrics = performance_metrics
        self.current_snapshot.adaptation_recommendations = recommendations
        
        # Calculate system health
        self.current_snapshot.system_health = self._calculate_system_health()
        
        # Add to time series
        self.time_series.add_data_point(self.current_snapshot)
        
        # Check if export is due
        if self._should_export():
            self._perform_export()
    
    fn export_current_metrics(self, format_type: Int = EXPORT_FORMAT_JSON) -> String:
        """Export current metrics in specified format."""
        if format_type == EXPORT_FORMAT_JSON:
            return self._export_json()
        elif format_type == EXPORT_FORMAT_PROMETHEUS:
            return self._export_prometheus()
        elif format_type == EXPORT_FORMAT_CSV:
            return self._export_csv()
        elif format_type == EXPORT_FORMAT_INFLUX:
            return self._export_influx()
        else:
            return self._export_json()  # Default to JSON
    
    fn export_time_series(self, metric_name: String, window_minutes: Int = 60) -> String:
        """Export time-series data for specific metric."""
        var aggregated_data = self.time_series.get_aggregated_data(metric_name, window_minutes)
        
        # Create time-series JSON export
        var result = "{\n"
        result += "  \"metric_name\": \"" + metric_name + "\",\n"
        result += "  \"window_minutes\": " + str(window_minutes) + ",\n"
        result += "  \"timestamp\": " + str(perf_counter_ns()) + ",\n"
        result += "  \"data_points\": " + str(len(aggregated_data)) + ",\n"
        result += "  \"values\": ["
        
        for i in range(len(aggregated_data)):
            if i > 0:
                result += ", "
            result += str(aggregated_data[i])
        
        result += "]\n}"
        
        return result
    
    fn get_dashboard_summary(self) -> Dict[String, Float32]:
        """Get summary metrics optimized for dashboard display."""
        var summary = Dict[String, Float32]()
        
        # Current status
        summary["current_quality_score"] = self.current_snapshot.quality_metrics.quality_score
        summary["drift_alerts_count"] = Float32(len(self.current_snapshot.drift_alerts))
        summary["recommendations_count"] = Float32(len(self.current_snapshot.adaptation_recommendations))
        
        # System health
        for key in self.current_snapshot.system_health:
            summary[key] = self.current_snapshot.system_health[key]
        
        # Performance metrics
        for key in self.current_snapshot.performance_metrics:
            summary[key] = self.current_snapshot.performance_metrics[key]
        
        # Historical trends (simplified)
        if len(self.time_series.quality_scores) > 1:
            var recent_avg = self._calculate_recent_average("quality_score", 5)
            summary["quality_trend"] = recent_avg - self.current_snapshot.quality_metrics.quality_score
        
        return summary
    
    fn configure_export(mut self, config: Dict[String, String]):
        """Configure export settings."""
        for key in config:
            self.export_config[key] = config[key]
        
        self.logger.log(LogLevel.INFO, "Export configuration updated")
    
    fn enable_streaming(mut self, enabled: Bool):
        """Enable or disable real-time streaming."""
        self.streaming_enabled = enabled
        
        if enabled:
            self.logger.log(LogLevel.INFO, "Real-time metrics streaming enabled")
        else:
            self.logger.log(LogLevel.INFO, "Real-time metrics streaming disabled")
    
    # Private export format implementations
    
    fn _export_json(self) -> String:
        """Export metrics in JSON format."""
        var json = "{\n"
        json += "  \"timestamp\": " + str(self.current_snapshot.timestamp) + ",\n"
        json += "  \"quality_metrics\": {\n"
        json += "    \"quality_score\": " + str(self.current_snapshot.quality_metrics.quality_score) + ",\n"
        json += "    \"mean_magnitude\": " + str(self.current_snapshot.quality_metrics.mean_magnitude) + ",\n"
        json += "    \"inter_vector_similarity\": " + str(self.current_snapshot.quality_metrics.inter_vector_similarity) + ",\n"
        json += "    \"distribution_entropy\": " + str(self.current_snapshot.quality_metrics.distribution_entropy) + ",\n"
        json += "    \"sample_size\": " + str(self.current_snapshot.quality_metrics.sample_size) + "\n"
        json += "  },\n"
        
        json += "  \"drift_alerts\": [\n"
        for i in range(len(self.current_snapshot.drift_alerts)):
            if i > 0:
                json += ",\n"
            var alert = self.current_snapshot.drift_alerts[i]
            json += "    {\n"
            json += "      \"severity\": " + str(alert.severity) + ",\n"
            json += "      \"confidence\": " + str(alert.confidence) + ",\n"
            json += "      \"drift_type\": " + str(alert.drift_type) + ",\n"
            json += "      \"description\": \"" + alert.description + "\"\n"
            json += "    }"
        json += "\n  ],\n"
        
        json += "  \"performance_metrics\": {\n"
        var perf_count = 0
        for key in self.current_snapshot.performance_metrics:
            if perf_count > 0:
                json += ",\n"
            json += "    \"" + key + "\": " + str(self.current_snapshot.performance_metrics[key])
            perf_count += 1
        json += "\n  },\n"
        
        json += "  \"system_health\": {\n"
        var health_count = 0
        for key in self.current_snapshot.system_health:
            if health_count > 0:
                json += ",\n"
            json += "    \"" + key + "\": " + str(self.current_snapshot.system_health[key])
            health_count += 1
        json += "\n  },\n"
        
        json += "  \"recommendations\": [\n"
        for i in range(len(self.current_snapshot.adaptation_recommendations)):
            if i > 0:
                json += ",\n"
            json += "    \"" + self.current_snapshot.adaptation_recommendations[i] + "\""
        json += "\n  ]\n"
        
        json += "}"
        
        return json
    
    fn _export_prometheus(self) -> String:
        """Export metrics in Prometheus format."""
        var prometheus = ""
        var timestamp_ms = str(self.current_snapshot.timestamp // 1000000)  # Convert to ms
        
        # Quality metrics
        prometheus += "# HELP omendb_quality_score Vector quality score\n"
        prometheus += "# TYPE omendb_quality_score gauge\n"
        prometheus += "omendb_quality_score " + str(self.current_snapshot.quality_metrics.quality_score) + " " + timestamp_ms + "\n\n"
        
        prometheus += "# HELP omendb_vector_similarity Inter-vector similarity\n"
        prometheus += "# TYPE omendb_vector_similarity gauge\n"
        prometheus += "omendb_vector_similarity " + str(self.current_snapshot.quality_metrics.inter_vector_similarity) + " " + timestamp_ms + "\n\n"
        
        # Drift alerts
        prometheus += "# HELP omendb_drift_alerts_total Total number of drift alerts\n"
        prometheus += "# TYPE omendb_drift_alerts_total counter\n"
        prometheus += "omendb_drift_alerts_total " + str(len(self.current_snapshot.drift_alerts)) + " " + timestamp_ms + "\n\n"
        
        # Performance metrics
        for key in self.current_snapshot.performance_metrics:
            var metric_name = "omendb_" + key.replace(".", "_").replace("-", "_")
            prometheus += "# HELP " + metric_name + " " + key + "\n"
            prometheus += "# TYPE " + metric_name + " gauge\n"
            prometheus += metric_name + " " + str(self.current_snapshot.performance_metrics[key]) + " " + timestamp_ms + "\n\n"
        
        return prometheus
    
    fn _export_csv(self) -> String:
        """Export metrics in CSV format."""
        var csv = "timestamp,quality_score,mean_magnitude,inter_vector_similarity,distribution_entropy,drift_alerts_count\n"
        
        csv += str(self.current_snapshot.timestamp) + ","
        csv += str(self.current_snapshot.quality_metrics.quality_score) + ","
        csv += str(self.current_snapshot.quality_metrics.mean_magnitude) + ","
        csv += str(self.current_snapshot.quality_metrics.inter_vector_similarity) + ","
        csv += str(self.current_snapshot.quality_metrics.distribution_entropy) + ","
        csv += str(len(self.current_snapshot.drift_alerts)) + "\n"
        
        return csv
    
    fn _export_influx(self) -> String:
        """Export metrics in InfluxDB line protocol format."""
        var influx = ""
        var timestamp_ns = str(self.current_snapshot.timestamp)
        
        # Quality metrics measurement
        influx += "quality_metrics"
        influx += " quality_score=" + str(self.current_snapshot.quality_metrics.quality_score)
        influx += ",mean_magnitude=" + str(self.current_snapshot.quality_metrics.mean_magnitude)
        influx += ",inter_vector_similarity=" + str(self.current_snapshot.quality_metrics.inter_vector_similarity)
        influx += ",distribution_entropy=" + str(self.current_snapshot.quality_metrics.distribution_entropy)
        influx += " " + timestamp_ns + "\n"
        
        # Drift alerts measurement
        influx += "drift_alerts"
        influx += " count=" + str(len(self.current_snapshot.drift_alerts))
        influx += " " + timestamp_ns + "\n"
        
        # Performance measurements
        for key in self.current_snapshot.performance_metrics:
            influx += "performance"
            influx += ",metric=" + key
            influx += " value=" + str(self.current_snapshot.performance_metrics[key])
            influx += " " + timestamp_ns + "\n"
        
        return influx
    
    # Private helper methods
    
    fn _calculate_system_health(self) -> Dict[String, Float32]:
        """Calculate overall system health metrics."""
        var health = Dict[String, Float32]()
        
        # Overall health score (0-1)
        var health_score = self.current_snapshot.quality_metrics.quality_score
        
        # Drift health (inverse of alert severity)
        var drift_health = Float32(1.0)
        for i in range(len(self.current_snapshot.drift_alerts)):
            var alert = self.current_snapshot.drift_alerts[i]
            drift_health *= (1.0 - alert.confidence * 0.25)  # Reduce by 25% per alert
        
        # Performance health
        var performance_health = Float32(1.0)
        if "current_overhead_percent" in self.current_snapshot.performance_metrics:
            var overhead = self.current_snapshot.performance_metrics["current_overhead_percent"]
            performance_health = max(0.0, 1.0 - overhead / 5.0)  # Full health up to 5% overhead
        
        # Composite health
        health["overall_health"] = (health_score + drift_health + performance_health) / 3.0
        health["quality_health"] = health_score
        health["drift_health"] = drift_health
        health["performance_health"] = performance_health
        
        return health
    
    fn _should_export(self) -> Bool:
        """Check if export is due based on interval."""
        var current_time = perf_counter_ns()
        var export_interval_ns = Int(self.export_interval_ms) * 1000000  # Convert to ns
        
        return (current_time - self.last_export_time) >= export_interval_ns
    
    fn _perform_export(mut self):
        """Perform scheduled export."""
        if self.streaming_enabled:
            var exported_data = self.export_current_metrics()
            self.logger.log(LogLevel.DEBUG, "Metrics exported: " + str(len(exported_data)) + " bytes")
        
        self.last_export_time = perf_counter_ns()
    
    fn _calculate_recent_average(self, metric_name: String, window_minutes: Int) -> Float32:
        """Calculate recent average for specified metric."""
        var aggregated = self.time_series.get_aggregated_data(metric_name, window_minutes)
        
        if len(aggregated) == 0:
            return 0.0
        
        var sum = Float32(0.0)
        for i in range(len(aggregated)):
            sum += aggregated[i]
        
        return sum / Float32(len(aggregated))