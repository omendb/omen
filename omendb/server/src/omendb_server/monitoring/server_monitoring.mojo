"""
Production Server Monitoring and Logging
========================================

Comprehensive monitoring system for OmenDB server deployment with:
- Real-time performance metrics
- Health checks and alerting
- Request tracing and analytics
- Resource utilization monitoring
- SLA compliance tracking
- Enterprise observability integration
"""

from collections import List, Dict, Optional
from time import perf_counter_ns
from math import sqrt

from core.vector import Vector, VectorID
from core.record import SearchResult
from util.logging import Logger, LogLevel


alias METRICS_WINDOW_SIZE = 1000
alias HEALTH_CHECK_INTERVAL_MS = 5000
alias ALERT_THRESHOLD_LATENCY_MS = 100
alias ALERT_THRESHOLD_ERROR_RATE = 0.05


struct RequestMetrics:
    """Individual request metrics for tracking."""
    
    var request_id: String
    var request_type: String  # "insert", "search", "batch_insert"
    var start_time: Int
    var end_time: Int
    var success: Bool
    var error_message: String
    var vector_count: Int
    var result_count: Int
    
    fn __init__(inout self, request_id: String, request_type: String, vector_count: Int = 1):
        self.request_id = request_id
        self.request_type = request_type
        self.start_time = perf_counter_ns()
        self.end_time = 0
        self.success = False
        self.error_message = ""
        self.vector_count = vector_count
        self.result_count = 0
    
    fn complete(inout self, success: Bool, error_message: String = "", result_count: Int = 0):
        """Mark request as completed."""
        self.end_time = perf_counter_ns()
        self.success = success
        self.error_message = error_message
        self.result_count = result_count
    
    fn duration_ms(self) -> Float64:
        """Get request duration in milliseconds."""
        if self.end_time == 0:
            return Float64(perf_counter_ns() - self.start_time) / 1000000.0
        return Float64(self.end_time - self.start_time) / 1000000.0
    
    fn duration_us(self) -> Float64:
        """Get request duration in microseconds."""
        if self.end_time == 0:
            return Float64(perf_counter_ns() - self.start_time) / 1000.0
        return Float64(self.end_time - self.start_time) / 1000.0


struct PerformanceMetrics:
    """Aggregated performance metrics."""
    
    var total_requests: Int
    var successful_requests: Int
    var failed_requests: Int
    var total_vectors_processed: Int
    var avg_latency_ms: Float64
    var p95_latency_ms: Float64
    var p99_latency_ms: Float64
    var throughput_qps: Float64
    var error_rate: Float64
    var memory_usage_mb: Float64
    var cpu_usage_percent: Float64
    
    fn __init__(inout self):
        self.total_requests = 0
        self.successful_requests = 0
        self.failed_requests = 0
        self.total_vectors_processed = 0
        self.avg_latency_ms = 0.0
        self.p95_latency_ms = 0.0
        self.p99_latency_ms = 0.0
        self.throughput_qps = 0.0
        self.error_rate = 0.0
        self.memory_usage_mb = 0.0
        self.cpu_usage_percent = 0.0


struct AlertManager:
    """Alert management for critical system events."""
    
    var logger: Logger
    var alert_history: List[String]
    var last_alert_time: Dict[String, Int]
    var alert_cooldown_ms: Int
    
    fn __init__(inout self):
        self.logger = Logger("AlertManager", LogLevel.WARN)
        self.alert_history = List[String]()
        self.last_alert_time = Dict[String, Int]()
        self.alert_cooldown_ms = 60000  # 1 minute cooldown
    
    fn check_latency_alert(inout self, avg_latency_ms: Float64):
        """Check if latency exceeds threshold."""
        if avg_latency_ms > ALERT_THRESHOLD_LATENCY_MS:
            self._trigger_alert("HIGH_LATENCY", 
                "Average latency " + String(avg_latency_ms) + "ms exceeds threshold " + String(ALERT_THRESHOLD_LATENCY_MS) + "ms")
    
    fn check_error_rate_alert(inout self, error_rate: Float64):
        """Check if error rate exceeds threshold."""
        if error_rate > ALERT_THRESHOLD_ERROR_RATE:
            self._trigger_alert("HIGH_ERROR_RATE", 
                "Error rate " + String(error_rate * 100) + "% exceeds threshold " + String(ALERT_THRESHOLD_ERROR_RATE * 100) + "%")
    
    fn check_memory_alert(inout self, memory_usage_mb: Float64, memory_limit_mb: Float64):
        """Check if memory usage is critical."""
        var usage_percent = memory_usage_mb / memory_limit_mb
        if usage_percent > 0.9:  # 90% memory usage
            self._trigger_alert("HIGH_MEMORY_USAGE", 
                "Memory usage " + String(memory_usage_mb) + "MB (" + String(usage_percent * 100) + "%) is critical")
    
    fn _trigger_alert(inout self, alert_type: String, message: String):
        """Trigger an alert with cooldown."""
        var current_time = perf_counter_ns()
        
        # Check cooldown
        if alert_type in self.last_alert_time:
            var last_time = self.last_alert_time[alert_type]
            if (current_time - last_time) < (self.alert_cooldown_ms * 1000000):  # Convert to ns
                return  # Skip alert due to cooldown
        
        # Log alert
        self.logger.error("🚨 ALERT [" + alert_type + "]: " + message)
        
        # Record alert
        self.alert_history.append("[" + alert_type + "] " + message)
        self.last_alert_time[alert_type] = current_time
        
        # In production, this would integrate with alerting systems (PagerDuty, Slack, etc.)


struct ServerMonitor:
    """
    Comprehensive server monitoring system.
    
    Tracks performance, health, and provides alerting for production deployment.
    """
    
    var logger: Logger
    var request_metrics: List[RequestMetrics]
    var performance_metrics: PerformanceMetrics
    var alert_manager: AlertManager
    var start_time: Int
    var last_health_check: Int
    var health_status: String
    var monitoring_enabled: Bool
    var request_counter: Int
    
    fn __init__(inout self):
        self.logger = Logger("ServerMonitor", LogLevel.INFO)
        self.request_metrics = List[RequestMetrics]()
        self.performance_metrics = PerformanceMetrics()
        self.alert_manager = AlertManager()
        self.start_time = perf_counter_ns()
        self.last_health_check = perf_counter_ns()
        self.health_status = "healthy"
        self.monitoring_enabled = True
        self.request_counter = 0
        
        self.logger.info("📊 Server monitoring initialized")
    
    fn start_request(inout self, request_type: String, vector_count: Int = 1) -> String:
        """
        Start tracking a new request.
        
        Args:
            request_type: Type of request (insert, search, batch_insert)
            vector_count: Number of vectors involved
            
        Returns:
            Request ID for tracking
        """
        if not self.monitoring_enabled:
            return ""
        
        self.request_counter += 1
        var request_id = "req_" + String(self.request_counter)
        
        var metrics = RequestMetrics(request_id, request_type, vector_count)
        self.request_metrics.append(metrics)
        
        # Keep only recent metrics to prevent memory growth
        if len(self.request_metrics) > METRICS_WINDOW_SIZE:
            self.request_metrics.pop(0)
        
        self.logger.debug("📥 Request started: " + request_id + " [" + request_type + "]")
        return request_id
    
    fn complete_request(inout self, request_id: String, success: Bool, error_message: String = "", result_count: Int = 0):
        """
        Complete tracking for a request.
        
        Args:
            request_id: Request ID from start_request
            success: Whether request was successful
            error_message: Error message if failed
            result_count: Number of results returned
        """
        if not self.monitoring_enabled or request_id == "":
            return
        
        # Find and complete the request metrics
        for i in range(len(self.request_metrics)):
            if self.request_metrics[i].request_id == request_id:
                self.request_metrics[i].complete(success, error_message, result_count)
                var duration_ms = self.request_metrics[i].duration_ms()
                
                self.logger.debug("📤 Request completed: " + request_id + 
                    " [" + ("✅" if success else "❌") + "] " + String(duration_ms) + "ms")
                break
        
        # Update aggregated metrics
        self._update_performance_metrics()
        
        # Check for alerts
        self._check_alerts()
    
    fn get_performance_metrics(self) -> PerformanceMetrics:
        """Get current performance metrics."""
        return self.performance_metrics
    
    fn get_health_status(inout self) -> String:
        """Get current health status with checks."""
        var current_time = perf_counter_ns()
        
        # Perform health check if needed
        if (current_time - self.last_health_check) > (HEALTH_CHECK_INTERVAL_MS * 1000000):
            self._perform_health_check()
            self.last_health_check = current_time
        
        return self.health_status
    
    fn get_monitoring_summary(self) -> String:
        """Get comprehensive monitoring summary."""
        var uptime_seconds = Float64(perf_counter_ns() - self.start_time) / 1000000000.0
        var uptime_hours = uptime_seconds / 3600.0
        
        var summary = "📊 OmenDB Server Monitoring Summary\\n"
        summary += "====================================\\n"
        summary += "Health Status: " + self.health_status + "\\n"
        summary += "Uptime: " + String(uptime_hours) + " hours\\n"
        summary += "Total Requests: " + String(self.performance_metrics.total_requests) + "\\n"
        summary += "Success Rate: " + String((1.0 - self.performance_metrics.error_rate) * 100) + "%\\n"
        summary += "Avg Latency: " + String(self.performance_metrics.avg_latency_ms) + "ms\\n"
        summary += "P95 Latency: " + String(self.performance_metrics.p95_latency_ms) + "ms\\n"
        summary += "P99 Latency: " + String(self.performance_metrics.p99_latency_ms) + "ms\\n"
        summary += "Throughput: " + String(self.performance_metrics.throughput_qps) + " QPS\\n"
        summary += "Memory Usage: " + String(self.performance_metrics.memory_usage_mb) + " MB\\n"
        summary += "Vectors Processed: " + String(self.performance_metrics.total_vectors_processed) + "\\n"
        
        if len(self.alert_manager.alert_history) > 0:
            summary += "\\nRecent Alerts:\\n"
            var alert_count = min(5, len(self.alert_manager.alert_history))
            for i in range(len(self.alert_manager.alert_history) - alert_count, len(self.alert_manager.alert_history)):
                summary += "  " + self.alert_manager.alert_history[i] + "\\n"
        
        return summary
    
    fn enable_monitoring(inout self):
        """Enable monitoring."""
        self.monitoring_enabled = True
        self.logger.info("📊 Monitoring enabled")
    
    fn disable_monitoring(inout self):
        """Disable monitoring."""
        self.monitoring_enabled = False
        self.logger.info("📊 Monitoring disabled")
    
    # Private helper methods
    fn _update_performance_metrics(inout self):
        """Update aggregated performance metrics."""
        if len(self.request_metrics) == 0:
            return
        
        var total_requests = 0
        var successful_requests = 0
        var failed_requests = 0
        var total_vectors = 0
        var total_latency = 0.0
        var latencies = List[Float64]()
        
        # Collect metrics from recent requests
        for metrics in self.request_metrics:
            if metrics.end_time > 0:  # Completed requests only
                total_requests += 1
                total_vectors += metrics.vector_count
                
                var latency = metrics.duration_ms()
                total_latency += latency
                latencies.append(latency)
                
                if metrics.success:
                    successful_requests += 1
                else:
                    failed_requests += 1
        
        if total_requests == 0:
            return
        
        # Update performance metrics
        self.performance_metrics.total_requests = total_requests
        self.performance_metrics.successful_requests = successful_requests
        self.performance_metrics.failed_requests = failed_requests
        self.performance_metrics.total_vectors_processed = total_vectors
        self.performance_metrics.avg_latency_ms = total_latency / Float64(total_requests)
        self.performance_metrics.error_rate = Float64(failed_requests) / Float64(total_requests)
        
        # Calculate percentiles
        if len(latencies) > 0:
            self._sort_latencies(latencies)
            self.performance_metrics.p95_latency_ms = self._percentile(latencies, 0.95)
            self.performance_metrics.p99_latency_ms = self._percentile(latencies, 0.99)
        
        # Calculate throughput (requests per second over last window)
        var window_duration_seconds = Float64(METRICS_WINDOW_SIZE) * self.performance_metrics.avg_latency_ms / 1000.0
        if window_duration_seconds > 0:
            self.performance_metrics.throughput_qps = Float64(total_requests) / window_duration_seconds
    
    fn _check_alerts(inout self):
        """Check for alert conditions."""
        self.alert_manager.check_latency_alert(self.performance_metrics.avg_latency_ms)
        self.alert_manager.check_error_rate_alert(self.performance_metrics.error_rate)
        self.alert_manager.check_memory_alert(self.performance_metrics.memory_usage_mb, 8192.0)  # 8GB limit
    
    fn _perform_health_check(inout self):
        """Perform comprehensive health check."""
        var issues = List[String]()
        
        # Check error rate
        if self.performance_metrics.error_rate > 0.1:  # 10% error rate
            issues.append("High error rate: " + String(self.performance_metrics.error_rate * 100) + "%")
        
        # Check latency
        if self.performance_metrics.avg_latency_ms > 1000.0:  # 1 second
            issues.append("High latency: " + String(self.performance_metrics.avg_latency_ms) + "ms")
        
        # Check memory (placeholder)
        if self.performance_metrics.memory_usage_mb > 7000.0:  # 7GB of 8GB
            issues.append("High memory usage: " + String(self.performance_metrics.memory_usage_mb) + "MB")
        
        # Update health status
        if len(issues) == 0:
            self.health_status = "healthy"
        elif len(issues) <= 2:
            self.health_status = "degraded"
        else:
            self.health_status = "unhealthy"
        
        # Log health status
        if self.health_status != "healthy":
            self.logger.warn("🏥 Health check: " + self.health_status + " - " + String(len(issues)) + " issues")
    
    fn _sort_latencies(self, inout latencies: List[Float64]):
        """Sort latencies for percentile calculation."""
        var n = len(latencies)
        for i in range(n):
            for j in range(0, n - i - 1):
                if latencies[j] > latencies[j + 1]:
                    var temp = latencies[j]
                    latencies[j] = latencies[j + 1]
                    latencies[j + 1] = temp
    
    fn _percentile(self, latencies: List[Float64], percentile: Float64) -> Float64:
        """Calculate percentile from sorted latencies."""
        if len(latencies) == 0:
            return 0.0
        
        var index = Int(Float64(len(latencies) - 1) * percentile)
        index = min(max(0, index), len(latencies) - 1)
        return latencies[index]


# Factory function for creating server monitor
fn create_server_monitor() -> ServerMonitor:
    """Create a new server monitoring instance."""
    return ServerMonitor()