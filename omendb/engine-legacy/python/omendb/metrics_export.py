"""
Lightweight metrics export for OmenDB embedded package.

Provides standard format metrics export without storage or complex observability.
Follows the principle of generating metrics data for external monitoring systems
rather than building a complete observability platform into the embedded package.
"""

from typing import Dict, Any, Optional
from dataclasses import dataclass
from enum import Enum
import json
import time


class MetricsFormat(Enum):
    """Supported metrics export formats."""

    PROMETHEUS = "prometheus"
    JSON = "json"
    STATSD = "statsd"


@dataclass
class DatabaseStats:
    """Simple statistics from the database engine."""

    query_count: int = 0
    insert_count: int = 0
    error_count: int = 0
    memory_allocated_bytes: int = 0
    uptime_seconds: float = 0.0
    last_query_duration_ms: float = 0.0
    average_query_duration_ms: float = 0.0

    # Additional computed metrics
    success_rate: float = 100.0
    queries_per_second: float = 0.0

    def __post_init__(self):
        """Calculate derived metrics."""
        total_operations = self.query_count + self.insert_count
        if total_operations > 0:
            self.success_rate = (
                (total_operations - self.error_count) / total_operations
            ) * 100

        if self.uptime_seconds > 0:
            self.queries_per_second = self.query_count / self.uptime_seconds


class MetricsExporter:
    """Lightweight metrics exporter for embedded OmenDB.

    Generates metrics in standard formats for external monitoring systems.
    Does not store metrics or provide complex observability features.
    """

    def __init__(self, database_id: str = "omendb"):
        """Initialize metrics exporter.

        Args:
            database_id: Identifier for this database instance
        """
        self.database_id = database_id
        self._start_time = time.time()

    def get_stats_from_engine(self) -> DatabaseStats:
        """Get current statistics from the native engine.

        Integrates with the Mojo metrics system to get real-time counters.
        """
        try:
            # Try to get metrics from native module if available
            from . import api

            if hasattr(api, "_native") and api._native is not None:
                if hasattr(api._native, "get_metrics_snapshot"):
                    snapshot = api._native.get_metrics_snapshot()

                    return DatabaseStats(
                        query_count=int(snapshot.get("query_count", 0)),
                        insert_count=int(snapshot.get("insert_count", 0)),
                        error_count=int(snapshot.get("error_count", 0)),
                        memory_allocated_bytes=int(
                            snapshot.get("memory_allocated_bytes", 0)
                        ),
                        uptime_seconds=float(
                            snapshot.get(
                                "uptime_seconds", time.time() - self._start_time
                            )
                        ),
                        last_query_duration_ms=float(
                            snapshot.get("last_query_duration_ms", 0.0)
                        ),
                        average_query_duration_ms=float(
                            snapshot.get("average_query_duration_ms", 0.0)
                        ),
                    )
        except Exception:
            # Fall back to placeholder data if native integration fails
            pass

        # Fallback: return basic stats with uptime
        uptime = time.time() - self._start_time
        return DatabaseStats(
            query_count=0,
            insert_count=0,
            error_count=0,
            memory_allocated_bytes=0,
            uptime_seconds=uptime,
            last_query_duration_ms=0.0,
            average_query_duration_ms=0.0,
        )

    def export_metrics(self, format: MetricsFormat = MetricsFormat.PROMETHEUS) -> str:
        """Export current metrics in specified format.

        Args:
            format: Output format for metrics

        Returns:
            Metrics data in requested format
        """
        stats = self.get_stats_from_engine()

        if format == MetricsFormat.PROMETHEUS:
            return self._format_prometheus(stats)
        elif format == MetricsFormat.JSON:
            return self._format_json(stats)
        elif format == MetricsFormat.STATSD:
            return self._format_statsd(stats)
        else:
            raise ValueError(f"Unsupported metrics format: {format}")

    def _format_prometheus(self, stats: DatabaseStats) -> str:
        """Format metrics for Prometheus consumption."""
        lines = []

        # Counter metrics
        lines.extend(
            [
                "# HELP omendb_queries_total Total number of queries executed",
                "# TYPE omendb_queries_total counter",
                f'omendb_queries_total{{db_id="{self.database_id}"}} {stats.query_count}',
                "",
                "# HELP omendb_inserts_total Total number of vectors inserted",
                "# TYPE omendb_inserts_total counter",
                f'omendb_inserts_total{{db_id="{self.database_id}"}} {stats.insert_count}',
                "",
                "# HELP omendb_errors_total Total number of errors",
                "# TYPE omendb_errors_total counter",
                f'omendb_errors_total{{db_id="{self.database_id}"}} {stats.error_count}',
                "",
            ]
        )

        # Gauge metrics
        lines.extend(
            [
                "# HELP omendb_memory_allocated_bytes Currently allocated memory in bytes",
                "# TYPE omendb_memory_allocated_bytes gauge",
                f'omendb_memory_allocated_bytes{{db_id="{self.database_id}"}} {stats.memory_allocated_bytes}',
                "",
                "# HELP omendb_query_duration_seconds Average query duration in seconds",
                "# TYPE omendb_query_duration_seconds gauge",
                f'omendb_query_duration_seconds{{db_id="{self.database_id}"}} {stats.average_query_duration_ms / 1000.0}',
                "",
                "# HELP omendb_success_rate_percentage Success rate percentage",
                "# TYPE omendb_success_rate_percentage gauge",
                f'omendb_success_rate_percentage{{db_id="{self.database_id}"}} {stats.success_rate}',
                "",
                "# HELP omendb_queries_per_second Query rate per second",
                "# TYPE omendb_queries_per_second gauge",
                f'omendb_queries_per_second{{db_id="{self.database_id}"}} {stats.queries_per_second}',
                "",
            ]
        )

        return "\n".join(lines)

    def _format_json(self, stats: DatabaseStats) -> str:
        """Format metrics as JSON."""
        metrics_data = {
            "database_id": self.database_id,
            "timestamp": time.time(),
            "metrics": {
                "counters": {
                    "queries_total": stats.query_count,
                    "inserts_total": stats.insert_count,
                    "errors_total": stats.error_count,
                },
                "gauges": {
                    "memory_allocated_bytes": stats.memory_allocated_bytes,
                    "uptime_seconds": stats.uptime_seconds,
                    "last_query_duration_ms": stats.last_query_duration_ms,
                    "average_query_duration_ms": stats.average_query_duration_ms,
                    "success_rate_percentage": stats.success_rate,
                    "queries_per_second": stats.queries_per_second,
                },
            },
        }

        return json.dumps(metrics_data, indent=2)

    def _format_statsd(self, stats: DatabaseStats) -> str:
        """Format metrics for StatsD/DataDog consumption."""
        lines = [
            f"omendb.queries_total:{stats.query_count}|c|#db_id:{self.database_id}",
            f"omendb.inserts_total:{stats.insert_count}|c|#db_id:{self.database_id}",
            f"omendb.errors_total:{stats.error_count}|c|#db_id:{self.database_id}",
            f"omendb.memory_allocated_bytes:{stats.memory_allocated_bytes}|g|#db_id:{self.database_id}",
            f"omendb.query_duration_ms:{stats.average_query_duration_ms}|g|#db_id:{self.database_id}",
            f"omendb.success_rate:{stats.success_rate}|g|#db_id:{self.database_id}",
            f"omendb.queries_per_second:{stats.queries_per_second}|g|#db_id:{self.database_id}",
        ]

        return "\n".join(lines)

    def get_health_status(self) -> Dict[str, Any]:
        """Get simple health status for health checks.

        Returns basic health information suitable for load balancer
        or orchestration system health checks.
        """
        stats = self.get_stats_from_engine()

        # Simple health logic
        is_healthy = (
            stats.error_count == 0 or stats.success_rate > 95.0
        ) and stats.memory_allocated_bytes < 1024 * 1024 * 1024  # 1GB limit

        return {
            "status": "healthy" if is_healthy else "unhealthy",
            "uptime_seconds": stats.uptime_seconds,
            "memory_mb": stats.memory_allocated_bytes / 1024 / 1024,
            "success_rate": stats.success_rate,
            "total_operations": stats.query_count + stats.insert_count,
            "last_query_ms": stats.last_query_duration_ms,
        }


# Simple module-level interface for easy integration
_default_exporter: Optional[MetricsExporter] = None


def init_metrics_export(database_id: str = "omendb") -> MetricsExporter:
    """Initialize metrics export for this database instance."""
    global _default_exporter
    _default_exporter = MetricsExporter(database_id)
    return _default_exporter


def export_metrics(format: MetricsFormat = MetricsFormat.PROMETHEUS) -> str:
    """Export metrics using default exporter."""
    if _default_exporter is None:
        raise RuntimeError(
            "Metrics export not initialized. Call init_metrics_export() first."
        )

    return _default_exporter.export_metrics(format)


def get_health_status() -> Dict[str, Any]:
    """Get health status using default exporter."""
    if _default_exporter is None:
        raise RuntimeError(
            "Metrics export not initialized. Call init_metrics_export() first."
        )

    return _default_exporter.get_health_status()


# Example usage patterns
def example_prometheus_integration():
    """Example: Exposing metrics for Prometheus scraping."""
    # In a Flask/FastAPI app:
    # @app.route('/metrics')
    # def metrics():
    #     return export_metrics(MetricsFormat.PROMETHEUS), 200, {'Content-Type': 'text/plain'}
    pass


def example_custom_monitoring():
    """Example: Custom monitoring system integration."""
    # For custom monitoring systems:
    # metrics_json = export_metrics(MetricsFormat.JSON)
    # data = json.loads(metrics_json)
    # custom_monitoring_system.send_metrics(data)
    pass


def example_datadog_integration():
    """Example: DataDog integration via StatsD."""
    # For DataDog/StatsD integration:
    # import statsd
    # statsd_client = statsd.StatsD(host='localhost', port=8125)
    # metrics_lines = export_metrics(MetricsFormat.STATSD).split('\n')
    # for line in metrics_lines:
    #     statsd_client.send(line)
    pass
