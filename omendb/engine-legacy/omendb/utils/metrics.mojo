"""
Zero-overhead metrics collection for OmenDB engine.

Implements efficient metrics collection patterns based on research of 
high-performance databases like Redis, PostgreSQL, and Qdrant.

Design principles:
- Atomic counters for basic operations (zero overhead)
- On-demand calculation for complex metrics (like Redis INFO)
- Configurable via build flags
- No memory allocations in hot paths
"""

from time import perf_counter_ns
from memory import UnsafePointer
from collections import Dict

# Wrapper for now() function to get current time in seconds
fn now() -> Float64:
    return Float64(perf_counter_ns()) / 1_000_000_000.0

# Note: Using regular variables instead of atomics for now
# In production, these should be wrapped with proper synchronization


struct MetricsSnapshot(Copyable, Movable):
    """Snapshot of current database metrics."""
    
    var query_count: UInt64
    var insert_count: UInt64
    var error_count: UInt64
    var memory_allocated_bytes: UInt64
    var uptime_seconds: Float64
    var last_query_duration_ms: Float64
    var average_query_duration_ms: Float64
    
    fn __init__(out self, query_count: UInt64, insert_count: UInt64, error_count: UInt64,
                memory_allocated_bytes: UInt64, uptime_seconds: Float64,
                last_query_duration_ms: Float64, average_query_duration_ms: Float64):
        self.query_count = query_count
        self.insert_count = insert_count
        self.error_count = error_count
        self.memory_allocated_bytes = memory_allocated_bytes
        self.uptime_seconds = uptime_seconds
        self.last_query_duration_ms = last_query_duration_ms
        self.average_query_duration_ms = average_query_duration_ms


struct DatabaseMetrics(Movable):
    """Zero-overhead metrics collection for database engine.
    
    Uses atomic counters for hot path operations and on-demand
    calculation for complex metrics to minimize performance impact.
    """
    
    # Note: Using regular variables instead of atomics for now
    # In production, these should be protected with proper synchronization
    var _query_count: UInt64
    var _insert_count: UInt64 
    var _error_count: UInt64
    var _success_count: UInt64
    
    # Timing metrics - minimal overhead
    var _total_query_time_ms: UInt64
    var _last_query_time_ms: UInt64
    var _start_time: Float64
    
    # Memory tracking (updated by allocator hooks)
    var _memory_allocated: UInt64
    var _peak_memory: UInt64
    
    # Build-time configuration
    alias METRICS_ENABLED = True  # Can be disabled via build flags
    
    fn __init__(out self):
        """Initialize metrics system."""
        self._query_count = 0
        self._insert_count = 0
        self._error_count = 0
        self._success_count = 0
        self._total_query_time_ms = 0
        self._last_query_time_ms = 0
        self._start_time = now()
        self._memory_allocated = 0
        self._peak_memory = 0
    
    @always_inline
    fn record_query(mut self, duration_ms: Float64):
        """Record query execution (hot path - must be zero overhead)."""
        @parameter
        if Self.METRICS_ENABLED:
            self._query_count += 1
            self._success_count += 1
            
            # Convert to integer for storage
            var duration_int = UInt64(duration_ms * 1000)  # Store as microseconds
            self._last_query_time_ms = duration_int
            self._total_query_time_ms += duration_int
    
    @always_inline
    fn record_insert(mut self):
        """Record vector insertion (hot path)."""
        @parameter
        if Self.METRICS_ENABLED:
            self._insert_count += 1
            self._success_count += 1
    
    @always_inline
    fn record_error(mut self):
        """Record error occurrence (hot path)."""
        @parameter
        if Self.METRICS_ENABLED:
            self._error_count += 1
    
    fn update_memory_usage(mut self, allocated_bytes: UInt64):
        """Update memory usage (called by allocator hooks)."""
        @parameter
        if Self.METRICS_ENABLED:
            self._memory_allocated = allocated_bytes
            if allocated_bytes > self._peak_memory:
                self._peak_memory = allocated_bytes
    
    fn get_snapshot(self) -> MetricsSnapshot:
        """Get current metrics snapshot (on-demand calculation).
        
        This is where the expensive calculations happen, only when
        metrics are actually requested (like Redis INFO command).
        """
        @parameter
        if not Self.METRICS_ENABLED:
            return MetricsSnapshot(0, 0, 0, 0, 0.0, 0.0, 0.0)
        
        var query_count = self._query_count
        var insert_count = self._insert_count
        var error_count = self._error_count
        var total_query_time = self._total_query_time_ms
        var last_query_time = self._last_query_time_ms
        
        # Calculate derived metrics on-demand
        var uptime = now() - self._start_time
        var avg_query_time = Float64(total_query_time) / Float64(max(query_count, 1)) / 1000.0
        var last_query_ms = Float64(last_query_time) / 1000.0
        
        return MetricsSnapshot(
            query_count,
            insert_count, 
            error_count,
            self._memory_allocated,
            uptime,
            last_query_ms,
            avg_query_time
        )
    
    fn export_prometheus_format(self) -> String:
        """Export metrics in Prometheus format for standard monitoring.
        
        This enables integration with existing monitoring infrastructure
        without requiring OmenDB-specific tooling.
        """
        var snapshot = self.get_snapshot()
        
        var result = String()
        result += "# HELP omendb_queries_total Total number of queries executed\n"
        result += "# TYPE omendb_queries_total counter\n"
        result += "omendb_queries_total " + String(snapshot.query_count) + "\n"
        
        result += "# HELP omendb_inserts_total Total number of vectors inserted\n"
        result += "# TYPE omendb_inserts_total counter\n"
        result += "omendb_inserts_total " + String(snapshot.insert_count) + "\n"
        
        result += "# HELP omendb_errors_total Total number of errors\n"
        result += "# TYPE omendb_errors_total counter\n"
        result += "omendb_errors_total " + String(snapshot.error_count) + "\n"
        
        result += "# HELP omendb_memory_allocated_bytes Currently allocated memory\n"
        result += "# TYPE omendb_memory_allocated_bytes gauge\n"
        result += "omendb_memory_allocated_bytes " + String(snapshot.memory_allocated_bytes) + "\n"
        
        result += "# HELP omendb_query_duration_seconds Average query duration\n"
        result += "# TYPE omendb_query_duration_seconds gauge\n"
        result += "omendb_query_duration_seconds " + String(snapshot.average_query_duration_ms / 1000.0) + "\n"
        
        result += "# HELP omendb_uptime_seconds Database uptime\n"
        result += "# TYPE omendb_uptime_seconds gauge\n"
        result += "omendb_uptime_seconds " + String(snapshot.uptime_seconds) + "\n"
        
        return result
    
    fn export_json_format(self) -> String:
        """Export metrics in JSON format for custom consumers."""
        var snapshot = self.get_snapshot()
        
        var result = String()
        result += "{\n"
        result += '  "query_count": ' + String(snapshot.query_count) + ",\n"
        result += '  "insert_count": ' + String(snapshot.insert_count) + ",\n"
        result += '  "error_count": ' + String(snapshot.error_count) + ",\n"
        result += '  "memory_allocated_bytes": ' + String(snapshot.memory_allocated_bytes) + ",\n"
        result += '  "uptime_seconds": ' + String(snapshot.uptime_seconds) + ",\n"
        result += '  "last_query_duration_ms": ' + String(snapshot.last_query_duration_ms) + ",\n"
        result += '  "average_query_duration_ms": ' + String(snapshot.average_query_duration_ms) + "\n"
        result += "}"
        
        return result
    
    fn export_statsd_format(self) -> String:
        """Export metrics in StatsD format for DataDog/StatsD consumers."""
        var snapshot = self.get_snapshot()
        
        var result = String()
        result += "omendb.queries_total:" + String(snapshot.query_count) + "|c\n"
        result += "omendb.inserts_total:" + String(snapshot.insert_count) + "|c\n"
        result += "omendb.errors_total:" + String(snapshot.error_count) + "|c\n"
        result += "omendb.memory_allocated_bytes:" + String(snapshot.memory_allocated_bytes) + "|g\n"
        result += "omendb.query_duration_ms:" + String(snapshot.average_query_duration_ms) + "|g\n"
        result += "omendb.uptime_seconds:" + String(snapshot.uptime_seconds) + "|g\n"
        
        return result
    
    fn reset_counters(mut self):
        """Reset all metrics counters (useful for testing)."""
        @parameter
        if Self.METRICS_ENABLED:
            self._query_count = 0
            self._insert_count = 0
            self._error_count = 0
            self._success_count = 0
            self._total_query_time_ms = 0
            self._last_query_time_ms = 0
            self._start_time = now()


struct OperationTimer:
    """RAII timer for measuring operation duration.
    
    Automatically records timing when destroyed, ensuring
    accurate measurements even if exceptions occur.
    """
    
    var _start_time: Float64
    var _metrics: UnsafePointer[DatabaseMetrics]
    var _operation_type: String
    
    fn __init__(out self, metrics: UnsafePointer[DatabaseMetrics], operation: String):
        """Start timing an operation."""
        self._start_time = now()
        self._metrics = metrics
        self._operation_type = operation
    
    fn __del__(owned self):
        """Record operation duration when timer is destroyed."""
        var duration_ms = (now() - self._start_time) * 1000.0
        
        if self._operation_type == "query":
            self._metrics[].record_query(duration_ms)
        # Add other operation types as needed


# Module-level metrics storage using static pointer pattern
var __global_metrics_ptr: UnsafePointer[DatabaseMetrics] = UnsafePointer[DatabaseMetrics]()
var __metrics_initialized: Bool = False

@always_inline
fn get_global_metrics() -> UnsafePointer[DatabaseMetrics]:
    """Get pointer to global metrics instance with zero overhead."""
    if not __metrics_initialized:
        __global_metrics_ptr = UnsafePointer[DatabaseMetrics].alloc(1)
        __global_metrics_ptr.init_pointee_move(DatabaseMetrics())
        __metrics_initialized = True
    return __global_metrics_ptr


fn init_metrics():
    """Initialize global metrics system."""
    # Already initialized at declaration
    pass


# Convenience functions for hot path operations
@always_inline
fn record_query_timing(duration_ms: Float64):
    """Record query timing (hot path)."""
    get_global_metrics()[].record_query(duration_ms)


@always_inline  
fn record_insert():
    """Record vector insertion (hot path)."""
    get_global_metrics()[].record_insert()


@always_inline
fn record_error():
    """Record error occurrence (hot path)."""
    get_global_metrics()[].record_error()


fn export_metrics_prometheus() -> String:
    """Export current metrics in Prometheus format."""
    return get_global_metrics()[].export_prometheus_format()


fn export_metrics_json() -> String:
    """Export current metrics in JSON format.""" 
    return get_global_metrics()[].export_json_format()


fn export_metrics_statsd() -> String:
    """Export current metrics in StatsD format."""
    return get_global_metrics()[].export_statsd_format()