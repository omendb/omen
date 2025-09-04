"""
Diagnostic capabilities for OmenDB production support.

Provides comprehensive diagnostic information, performance analysis,
and troubleshooting tools for production deployments.
"""

import time
import os
import sys
import json
import platform
import traceback
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, asdict
from datetime import datetime, timedelta

from .metrics import get_metrics_collector
from .logging import get_logger
from .health import get_health_monitor


@dataclass
class SystemInfo:
    """System environment information."""

    platform: str
    python_version: str
    omendb_version: str
    architecture: str
    cpu_count: int
    memory_total_gb: float
    disk_total_gb: float
    hostname: str
    process_id: int
    working_directory: str
    environment_vars: Dict[str, str]


@dataclass
class DatabaseInfo:
    """Database-specific diagnostic information."""

    path: str
    size_mb: float
    vector_count: int
    dimension: int
    index_type: str
    created_time: Optional[float]
    last_modified: Optional[float]
    file_permissions: str
    corruption_check: bool


@dataclass
class PerformanceProfile:
    """Performance profiling data."""

    operation: str
    sample_count: int
    avg_duration_ms: float
    min_duration_ms: float
    max_duration_ms: float
    p50_duration_ms: float
    p95_duration_ms: float
    p99_duration_ms: float
    throughput_ops_per_sec: float
    error_rate: float


@dataclass
class DiagnosticReport:
    """Comprehensive diagnostic report."""

    timestamp: float
    report_id: str
    system_info: SystemInfo
    health_status: Dict[str, Any]
    database_info: List[DatabaseInfo]
    performance_profiles: List[PerformanceProfile]
    metrics_summary: Dict[str, Any]
    recent_errors: List[Dict[str, Any]]
    configuration: Dict[str, Any]
    recommendations: List[str]


class DiagnosticsCollector:
    """Comprehensive diagnostics collection for OmenDB."""

    def __init__(self):
        """Initialize diagnostics collector."""
        self.logger = get_logger()
        self.metrics = get_metrics_collector()
        self.health_monitor = get_health_monitor()

        # Registered databases for diagnostics
        self._databases: List[Any] = []

        # Recent error tracking
        self._recent_errors: List[Dict[str, Any]] = []
        self._max_recent_errors = 100

    def register_database(self, database) -> None:
        """Register database for diagnostics collection."""
        if database not in self._databases:
            self._databases.append(database)

    def unregister_database(self, database) -> None:
        """Unregister database from diagnostics."""
        if database in self._databases:
            self._databases.remove(database)

    def record_error(
        self, error: Exception, context: Optional[Dict[str, Any]] = None
    ) -> None:
        """Record error for diagnostics."""
        error_record = {
            "timestamp": time.time(),
            "error_type": type(error).__name__,
            "error_message": str(error),
            "traceback": traceback.format_exc(),
            "context": context or {},
        }

        self._recent_errors.append(error_record)

        # Keep only recent errors
        if len(self._recent_errors) > self._max_recent_errors:
            self._recent_errors = self._recent_errors[-self._max_recent_errors :]

    def get_system_info(self) -> SystemInfo:
        """Collect system environment information."""
        try:
            import psutil

            # Memory info
            memory = psutil.virtual_memory()
            memory_total_gb = memory.total / 1024 / 1024 / 1024

            # Disk info
            disk = psutil.disk_usage("/")
            disk_total_gb = disk.total / 1024 / 1024 / 1024

        except ImportError:
            memory_total_gb = 0.0
            disk_total_gb = 0.0

        # Environment variables (filtered for security)
        safe_env_vars = {}
        for key, value in os.environ.items():
            # Only include safe environment variables
            if not any(
                secret in key.lower()
                for secret in ["password", "secret", "key", "token"]
            ):
                safe_env_vars[key] = value

        return SystemInfo(
            platform=platform.platform(),
            python_version=sys.version,
            omendb_version=getattr(
                sys.modules.get("omendb", {}), "__version__", "unknown"
            ),
            architecture=platform.architecture()[0],
            cpu_count=os.cpu_count() or 0,
            memory_total_gb=memory_total_gb,
            disk_total_gb=disk_total_gb,
            hostname=platform.node(),
            process_id=os.getpid(),
            working_directory=os.getcwd(),
            environment_vars=safe_env_vars,
        )

    def get_database_info(self, database) -> DatabaseInfo:
        """Collect database-specific diagnostic information."""
        try:
            # Get database path
            db_path = getattr(database, "path", "unknown")

            # File information
            size_mb = 0.0
            created_time = None
            last_modified = None
            file_permissions = "unknown"

            if os.path.exists(db_path):
                stat_info = os.stat(db_path)
                size_mb = stat_info.st_size / 1024 / 1024
                created_time = stat_info.st_ctime
                last_modified = stat_info.st_mtime
                file_permissions = oct(stat_info.st_mode)[-3:]

            # Database metrics
            vector_count = 0
            dimension = 0
            index_type = "unknown"

            if hasattr(database, "size"):
                try:
                    vector_count = database.size()
                except:
                    pass

            if hasattr(database, "dimension"):
                try:
                    dimension = database.dimension
                except:
                    pass

            if hasattr(database, "index_type"):
                try:
                    index_type = database.index_type
                except:
                    pass

            # Basic corruption check
            corruption_check = True
            try:
                if hasattr(database, "_verify_integrity"):
                    corruption_check = database._verify_integrity()
            except:
                corruption_check = False

            return DatabaseInfo(
                path=db_path,
                size_mb=size_mb,
                vector_count=vector_count,
                dimension=dimension,
                index_type=index_type,
                created_time=created_time,
                last_modified=last_modified,
                file_permissions=file_permissions,
                corruption_check=corruption_check,
            )

        except Exception as e:
            self.logger.error(f"Failed to collect database info: {e}")
            return DatabaseInfo(
                path="error",
                size_mb=0.0,
                vector_count=0,
                dimension=0,
                index_type="error",
                created_time=None,
                last_modified=None,
                file_permissions="error",
                corruption_check=False,
            )

    def get_performance_profiles(self) -> List[PerformanceProfile]:
        """Generate performance profiles from metrics."""
        profiles = []

        try:
            all_metrics = self.metrics.get_all_metrics()

            # Process histogram metrics for performance profiles
            for metric_name, metric_data in all_metrics.items():
                if (
                    metric_data.get("type") == "histogram"
                    and "operation_duration_ms" in metric_name
                ):
                    # Extract operation name from metric
                    operation = "unknown"
                    if "search" in metric_name:
                        operation = "search"
                    elif "insert" in metric_name:
                        operation = "insert"
                    elif "construction" in metric_name:
                        operation = "construction"
                    elif "roargraph" in metric_name:
                        if "search" in metric_name:
                            operation = "roargraph_search"
                        else:
                            operation = "roargraph_construction"

                    # Calculate throughput
                    avg_duration_s = metric_data.get("average", 0) / 1000
                    throughput = 1.0 / avg_duration_s if avg_duration_s > 0 else 0.0

                    profile = PerformanceProfile(
                        operation=operation,
                        sample_count=metric_data.get("count", 0),
                        avg_duration_ms=metric_data.get("average", 0),
                        min_duration_ms=0,  # Not available in current histogram
                        max_duration_ms=0,  # Not available in current histogram
                        p50_duration_ms=metric_data.get("p50", 0),
                        p95_duration_ms=metric_data.get("p95", 0),
                        p99_duration_ms=metric_data.get("p99", 0),
                        throughput_ops_per_sec=throughput,
                        error_rate=0.0,  # Would need error tracking per operation
                    )
                    profiles.append(profile)

        except Exception as e:
            self.logger.error(f"Failed to generate performance profiles: {e}")

        return profiles

    def get_recommendations(
        self,
        system_info: SystemInfo,
        database_info: List[DatabaseInfo],
        performance_profiles: List[PerformanceProfile],
        health_status: Dict[str, Any],
    ) -> List[str]:
        """Generate diagnostic recommendations."""
        recommendations = []

        try:
            # System resource recommendations
            if system_info.memory_total_gb < 8:
                recommendations.append(
                    "Consider increasing system memory (current: {:.1f}GB, recommended: 8GB+)".format(
                        system_info.memory_total_gb
                    )
                )

            if system_info.cpu_count < 4:
                recommendations.append(
                    "Consider using a system with more CPU cores for better performance (current: {}, recommended: 4+)".format(
                        system_info.cpu_count
                    )
                )

            # Database recommendations
            for db_info in database_info:
                if db_info.size_mb > 1000:  # > 1GB
                    recommendations.append(
                        f"Large database file detected ({db_info.size_mb:.1f}MB): consider implementing data archival or partitioning"
                    )

                if not db_info.corruption_check:
                    recommendations.append(
                        f"Database integrity check failed for {db_info.path}: consider rebuilding the database"
                    )

                if db_info.vector_count > 100000:
                    recommendations.append(
                        f"Large vector count ({db_info.vector_count:,}): ensure adequate memory and consider RoarGraph indexing"
                    )

            # Performance recommendations
            for profile in performance_profiles:
                if profile.operation == "search" and profile.avg_duration_ms > 50:
                    recommendations.append(
                        f"Search performance is slow (avg: {profile.avg_duration_ms:.1f}ms): consider optimizing index or reducing dataset size"
                    )

                if (
                    profile.operation == "insert"
                    and profile.throughput_ops_per_sec < 1000
                ):
                    recommendations.append(
                        f"Insert throughput is low ({profile.throughput_ops_per_sec:.0f} ops/sec): consider batch operations or index tuning"
                    )

            # Health status recommendations
            overall_status = health_status.get("overall_status", "unknown")
            if overall_status == "critical":
                recommendations.append(
                    "Critical health issues detected: review health check details and address immediately"
                )
            elif overall_status == "warning":
                recommendations.append(
                    "Health warnings detected: monitor system closely and consider preventive action"
                )

            # Default recommendation if no issues found
            if not recommendations:
                recommendations.append(
                    "System appears healthy: continue monitoring for optimal performance"
                )

        except Exception as e:
            self.logger.error(f"Failed to generate recommendations: {e}")
            recommendations.append(
                "Error generating recommendations: manual review recommended"
            )

        return recommendations

    def generate_diagnostic_report(self) -> DiagnosticReport:
        """Generate comprehensive diagnostic report."""
        report_id = f"omendb-diag-{int(time.time())}"

        self.logger.info(f"Generating diagnostic report {report_id}")

        try:
            # Collect system information
            system_info = self.get_system_info()

            # Collect health status
            health_status = self.health_monitor.get_health_status()

            # Collect database information
            database_info = []
            for db in self._databases:
                db_info = self.get_database_info(db)
                database_info.append(db_info)

            # Generate performance profiles
            performance_profiles = self.get_performance_profiles()

            # Get metrics summary
            metrics_summary = self.metrics.get_metrics_summary()

            # Get recent errors
            recent_errors = self._recent_errors.copy()

            # Configuration (placeholder for now)
            configuration = {
                "log_level": self.logger.level.value,
                "metrics_enabled": self.metrics is not None,
                "health_monitoring_enabled": self.health_monitor is not None,
                "database_count": len(self._databases),
            }

            # Generate recommendations
            recommendations = self.get_recommendations(
                system_info, database_info, performance_profiles, health_status
            )

            report = DiagnosticReport(
                timestamp=time.time(),
                report_id=report_id,
                system_info=system_info,
                health_status=health_status,
                database_info=database_info,
                performance_profiles=performance_profiles,
                metrics_summary=metrics_summary,
                recent_errors=recent_errors,
                configuration=configuration,
                recommendations=recommendations,
            )

            self.logger.info(f"Diagnostic report {report_id} generated successfully")
            return report

        except Exception as e:
            self.logger.error(
                f"Failed to generate diagnostic report: {e}", exc_info=True
            )

            # Return minimal report on error
            return DiagnosticReport(
                timestamp=time.time(),
                report_id=report_id,
                system_info=SystemInfo(
                    platform="error",
                    python_version="error",
                    omendb_version="error",
                    architecture="error",
                    cpu_count=0,
                    memory_total_gb=0.0,
                    disk_total_gb=0.0,
                    hostname="error",
                    process_id=0,
                    working_directory="error",
                    environment_vars={},
                ),
                health_status={
                    "overall_status": "critical",
                    "message": "Diagnostic collection failed",
                },
                database_info=[],
                performance_profiles=[],
                metrics_summary={},
                recent_errors=[{"error": str(e), "timestamp": time.time()}],
                configuration={},
                recommendations=[
                    "Diagnostic collection failed: manual investigation required"
                ],
            )

    def export_diagnostic_report(
        self, report: DiagnosticReport, file_path: Optional[str] = None
    ) -> str:
        """Export diagnostic report to file."""
        if file_path is None:
            timestamp = datetime.fromtimestamp(report.timestamp).strftime(
                "%Y%m%d_%H%M%S"
            )
            file_path = f"omendb_diagnostic_report_{timestamp}.json"

        try:
            # Convert dataclasses to dict for JSON serialization
            report_data = asdict(report)

            with open(file_path, "w") as f:
                json.dump(report_data, f, indent=2, default=str)

            self.logger.info(f"Diagnostic report exported to {file_path}")
            return file_path

        except Exception as e:
            self.logger.error(f"Failed to export diagnostic report: {e}")
            raise


# Global diagnostics collector
_global_diagnostics: Optional[DiagnosticsCollector] = None


def get_diagnostics_collector() -> DiagnosticsCollector:
    """Get global diagnostics collector."""
    global _global_diagnostics

    if _global_diagnostics is None:
        _global_diagnostics = DiagnosticsCollector()

    return _global_diagnostics


# Convenience functions
def generate_diagnostic_report() -> DiagnosticReport:
    """Generate diagnostic report."""
    collector = get_diagnostics_collector()
    return collector.generate_diagnostic_report()


def export_diagnostic_report(file_path: Optional[str] = None) -> str:
    """Generate and export diagnostic report."""
    collector = get_diagnostics_collector()
    report = collector.generate_diagnostic_report()
    return collector.export_diagnostic_report(report, file_path)
