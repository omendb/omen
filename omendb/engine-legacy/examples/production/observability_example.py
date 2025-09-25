#!/usr/bin/env python3
"""
Production Observability Example
===============================

Demonstrates observability patterns with OmenDB:
- Basic logging and monitoring
- Performance metrics collection
- Health checks
- Error tracking
- Simple diagnostic reporting

Note: This example shows how to add observability to OmenDB using
standard Python tools since OmenDB focuses on core vector database
functionality.
"""

import time
import json
import logging
import psutil
import os
import sys
import numpy as np
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Tuple

import omendb


def setup_observability():
    """Configure observability for production deployment."""
    print("🔧 Configuring observability for production...")

    # Configure basic logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        handlers=[
            logging.FileHandler("omendb_production.log"),
            logging.StreamHandler(),
        ],
    )
    logger = logging.getLogger("omendb")

    # Create a simple health monitor
    class SimpleHealthMonitor:
        def __init__(self):
            self.status = "healthy"
            self.checks = {}
            self.callbacks = []

        def add_callback(self, callback):
            self.callbacks.append(callback)

        def update_health(self, component: str, status: str, message: str = ""):
            self.checks[component] = {
                "status": status,
                "message": message,
                "timestamp": datetime.now().isoformat(),
            }

            # Update overall status
            if any(check["status"] == "critical" for check in self.checks.values()):
                self.status = "critical"
            elif any(check["status"] == "warning" for check in self.checks.values()):
                self.status = "warning"
            else:
                self.status = "healthy"

            # Call callbacks
            for callback in self.callbacks:
                callback(self.checks)

        def get_status(self):
            return {
                "status": self.status,
                "checks": self.checks,
                "timestamp": datetime.now().isoformat(),
            }

    health_monitor = SimpleHealthMonitor()

    # Add custom health monitoring callback
    def health_callback(health_results):
        """Custom callback for health status changes."""
        critical_checks = [
            name
            for name, check in health_results.items()
            if check.get("status") == "critical"
        ]

        if critical_checks:
            logger.critical(f"ALERT: Critical health issues in {critical_checks}")

        warning_checks = [
            name
            for name, check in health_results.items()
            if check.get("status") == "warning"
        ]

        if warning_checks:
            logger.warning(f"Warning: Health issues in {warning_checks}")

    health_monitor.add_callback(health_callback)

    print("✅ Observability configured successfully")
    return logger, health_monitor


def demonstrate_metrics_collection():
    """Demonstrate metrics collection patterns."""
    print("\n1️⃣ Metrics Collection Demonstration")
    print("-" * 40)

    logger = logging.getLogger("omendb")

    # Create metrics collector
    class MetricsCollector:
        def __init__(self):
            self.reset()

        def reset(self):
            self.metrics = {
                "operations": {"inserts": 0, "queries": 0, "deletes": 0, "errors": 0},
                "performance": {
                    "total_insert_time": 0,
                    "total_query_time": 0,
                    "max_query_time": 0,
                },
                "resource": {"memory_mb": 0, "cpu_percent": 0},
                "start_time": time.time(),
            }

        def record_operation(
            self, op_type: str, duration: float = 0, success: bool = True
        ):
            if success:
                self.metrics["operations"][op_type] += 1
                if op_type == "inserts":
                    self.metrics["performance"]["total_insert_time"] += duration
                elif op_type == "queries":
                    self.metrics["performance"]["total_query_time"] += duration
                    self.metrics["performance"]["max_query_time"] = max(
                        self.metrics["performance"]["max_query_time"], duration
                    )
            else:
                self.metrics["operations"]["errors"] += 1

        def update_resources(self):
            process = psutil.Process()
            self.metrics["resource"]["memory_mb"] = (
                process.memory_info().rss / 1024 / 1024
            )
            self.metrics["resource"]["cpu_percent"] = process.cpu_percent(interval=0.1)

        def get_summary(self):
            elapsed = time.time() - self.metrics["start_time"]
            total_ops = (
                sum(self.metrics["operations"].values())
                - self.metrics["operations"]["errors"]
            )

            return {
                "operations": self.metrics["operations"],
                "performance": {
                    **self.metrics["performance"],
                    "avg_insert_time": self.metrics["performance"]["total_insert_time"]
                    / max(1, self.metrics["operations"]["inserts"]),
                    "avg_query_time": self.metrics["performance"]["total_query_time"]
                    / max(1, self.metrics["operations"]["queries"]),
                    "ops_per_second": total_ops / elapsed if elapsed > 0 else 0,
                },
                "resource": self.metrics["resource"],
                "elapsed_seconds": elapsed,
            }

    # Demonstrate with actual database operations
    db = omendb.DB("metrics_demo.omen")
    metrics = MetricsCollector()

    # Insert test data with metrics
    dimension = 128
    print("📊 Inserting test vectors...")
    for i in range(50):
        start = time.time()
        try:
            vector = np.random.randn(dimension).tolist()
            success = db.add(f"vec_{i}", vector, {"index": str(i)})
            duration = time.time() - start
            metrics.record_operation("inserts", duration, success)
        except Exception as e:
            logger.error(f"Insert error: {e}")
            metrics.record_operation("inserts", 0, False)

    # Query with metrics
    print("🔍 Running test queries...")
    for i in range(10):
        start = time.time()
        try:
            query_vec = np.random.randn(dimension).tolist()
            results = db.search(query_vec, limit=5)
            duration = time.time() - start
            metrics.record_operation("queries", duration, True)
        except Exception as e:
            logger.error(f"Query error: {e}")
            metrics.record_operation("queries", 0, False)

    # Update resource metrics
    metrics.update_resources()

    # Display metrics summary
    summary = metrics.get_summary()
    print("\n📈 Metrics Summary:")
    print(json.dumps(summary, indent=2))

    logger.info(f"Metrics collection completed: {summary}")

    return db


def demonstrate_health_monitoring(db):
    """Demonstrate health monitoring patterns."""
    print("\n2️⃣ Health Monitoring Demonstration")
    print("-" * 40)

    logger = logging.getLogger("omendb")
    _, health_monitor = setup_observability()

    # Monitor system resources
    process = psutil.Process()
    memory_percent = psutil.virtual_memory().percent
    cpu_percent = psutil.cpu_percent(interval=1)

    # Update health status based on resources
    if memory_percent > 90:
        health_monitor.update_health(
            "memory", "critical", f"Memory usage: {memory_percent}%"
        )
    elif memory_percent > 70:
        health_monitor.update_health(
            "memory", "warning", f"Memory usage: {memory_percent}%"
        )
    else:
        health_monitor.update_health(
            "memory", "healthy", f"Memory usage: {memory_percent}%"
        )

    if cpu_percent > 90:
        health_monitor.update_health("cpu", "critical", f"CPU usage: {cpu_percent}%")
    elif cpu_percent > 70:
        health_monitor.update_health("cpu", "warning", f"CPU usage: {cpu_percent}%")
    else:
        health_monitor.update_health("cpu", "healthy", f"CPU usage: {cpu_percent}%")

    # Check database health
    try:
        stats = db.info()
        vector_count = stats.get("vector_count", 0)

        if vector_count == 0:
            health_monitor.update_health(
                "database", "warning", "No vectors in database"
            )
        else:
            health_monitor.update_health(
                "database", "healthy", f"{vector_count} vectors indexed"
            )

        # Test database responsiveness
        start = time.time()
        test_vec = np.random.randn(128).tolist()
        results = db.search(test_vec, limit=1)
        response_time = (time.time() - start) * 1000

        if response_time > 100:
            health_monitor.update_health(
                "performance", "warning", f"Slow response: {response_time:.1f}ms"
            )
        else:
            health_monitor.update_health(
                "performance", "healthy", f"Response time: {response_time:.1f}ms"
            )

    except Exception as e:
        health_monitor.update_health(
            "database", "critical", f"Database error: {str(e)}"
        )
        logger.error(f"Health check failed: {e}")

    # Display health status
    status = health_monitor.get_status()
    print("\n🏥 Health Status:")
    print(f"Overall: {status['status'].upper()}")
    for component, check in status["checks"].items():
        emoji = (
            "✅"
            if check["status"] == "healthy"
            else "⚠️"
            if check["status"] == "warning"
            else "❌"
        )
        print(f"{emoji} {component}: {check['status']} - {check['message']}")

    return status


def demonstrate_error_tracking():
    """Demonstrate error tracking and handling."""
    print("\n3️⃣ Error Tracking Demonstration")
    print("-" * 40)

    logger = logging.getLogger("omendb")

    # Create error tracker
    class ErrorTracker:
        def __init__(self):
            self.errors = []

        def track_error(
            self, error_type: str, message: str, context: Dict[str, Any] = None
        ):
            error_entry = {
                "timestamp": datetime.now().isoformat(),
                "type": error_type,
                "message": message,
                "context": context or {},
                "stack_trace": None,
            }

            # Try to get stack trace
            import traceback

            error_entry["stack_trace"] = traceback.format_exc()

            self.errors.append(error_entry)
            logger.error(f"{error_type}: {message}", extra={"context": context})

        def get_error_summary(self):
            error_types = {}
            for error in self.errors:
                error_types[error["type"]] = error_types.get(error["type"], 0) + 1

            return {
                "total_errors": len(self.errors),
                "error_types": error_types,
                "recent_errors": self.errors[-5:] if self.errors else [],
            }

    error_tracker = ErrorTracker()

    # Simulate various error scenarios
    db = omendb.DB("error_tracking_demo.omen")

    # Test dimension mismatch
    print("🧪 Testing error scenarios...")

    # Add initial vector
    try:
        db.add("vec_0", np.random.randn(128).tolist(), {"id": "0"})
    except Exception as e:
        error_tracker.track_error("initialization", str(e))

    # Try wrong dimension
    try:
        wrong_dim_vec = np.random.randn(256).tolist()
        db.add("vec_wrong", wrong_dim_vec, {"id": "wrong"})
    except Exception as e:
        error_tracker.track_error(
            "dimension_mismatch", str(e), {"expected": 128, "actual": 256}
        )

    # Try invalid vector
    try:
        db.add("vec_invalid", "not a vector", {"id": "invalid"})
    except Exception as e:
        error_tracker.track_error("invalid_vector", str(e), {"vector_type": "string"})

    # Display error summary
    summary = error_tracker.get_error_summary()
    print("\n❌ Error Summary:")
    print(f"Total errors: {summary['total_errors']}")
    print("Error types:")
    for error_type, count in summary["error_types"].items():
        print(f"  - {error_type}: {count}")

    if summary["recent_errors"]:
        print("\nRecent errors:")
        for error in summary["recent_errors"]:
            print(
                f"  [{error['timestamp']}] {error['type']}: {error['message'][:50]}..."
            )


def generate_comprehensive_diagnostic():
    """Generate a comprehensive diagnostic report."""
    print("\n4️⃣ Diagnostic Report Generation")
    print("-" * 40)

    logger = logging.getLogger("omendb")

    # Collect system information
    report = {
        "timestamp": datetime.now().isoformat(),
        "system": {
            "platform": sys.platform,
            "python_version": sys.version,
            "cpu_count": psutil.cpu_count(),
            "memory_total_mb": psutil.virtual_memory().total / 1024 / 1024,
            "memory_available_mb": psutil.virtual_memory().available / 1024 / 1024,
            "disk_usage_percent": psutil.disk_usage("/").percent,
        },
        "process": {
            "pid": os.getpid(),
            "memory_mb": psutil.Process().memory_info().rss / 1024 / 1024,
            "cpu_percent": psutil.Process().cpu_percent(interval=0.1),
            "num_threads": psutil.Process().num_threads(),
        },
        "omendb": {
            "version": getattr(omendb, "__version__", "unknown"),
            "databases": [],
        },
    }

    # Check OmenDB databases
    for db_file in Path(".").glob("*.omen"):
        try:
            db = omendb.DB(str(db_file))
            stats = db.info()
            report["omendb"]["databases"].append(
                {
                    "path": str(db_file),
                    "size_mb": db_file.stat().st_size / 1024 / 1024,
                    "stats": stats,
                }
            )
        except Exception as e:
            report["omendb"]["databases"].append(
                {"path": str(db_file), "error": str(e)}
            )

    # Save report
    report_file = f"omendb_diagnostic_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
    with open(report_file, "w") as f:
        json.dump(report, f, indent=2)

    print(f"\n📄 Diagnostic Report Generated:")
    print(f"File: {report_file}")
    print(
        f"System: {report['system']['platform']} with {report['system']['cpu_count']} CPUs"
    )
    print(
        f"Memory: {report['system']['memory_available_mb']:.0f}/{report['system']['memory_total_mb']:.0f} MB available"
    )
    print(f"OmenDB databases found: {len(report['omendb']['databases'])}")

    logger.info(f"Diagnostic report generated: {report_file}")

    return report, report_file


def demonstrate_production_deployment():
    """Demonstrate complete production deployment with observability."""
    print("\n🚀 Production Deployment Demonstration")
    print("=" * 50)

    # Step 1: Setup observability
    logger, health_monitor = setup_observability()

    # Step 2: Demonstrate metrics collection
    db = demonstrate_metrics_collection()

    # Step 3: Health monitoring
    health_status = demonstrate_health_monitoring(db)

    # Step 4: Error tracking
    demonstrate_error_tracking()

    # Step 5: Diagnostic reporting
    report, report_file = generate_comprehensive_diagnostic()

    # Step 6: Production summary
    print("\n" + "=" * 50)
    print("🎯 PRODUCTION OBSERVABILITY SUMMARY")
    print("=" * 50)

    print("✅ Observability Features Demonstrated:")
    print("  • Structured logging with context")
    print("  • Performance metrics collection")
    print("  • Health monitoring with thresholds")
    print("  • Error tracking and analysis")
    print("  • Diagnostic report generation")

    print("\n📊 Key Metrics:")
    print(f"  • Health Status: {health_status['status']}")
    print(f"  • Active Databases: {len(report['omendb']['databases'])}")
    print(f"  • Diagnostic Report: {report_file}")

    print("\n🔗 Integration Points:")
    print("  • Logs: omendb_production.log")
    print("  • Metrics: Export to Prometheus/DataDog")
    print("  • Health: HTTP endpoint for k8s probes")
    print("  • Alerts: Integrate with PagerDuty/Slack")

    return {"health_status": health_status, "report_file": report_file}


def main():
    """Main demonstration function."""
    try:
        results = demonstrate_production_deployment()
        print("\n✅ Production observability demo completed successfully!")

    except Exception as e:
        print(f"\n❌ Demonstration failed: {e}")
        import traceback

        traceback.print_exc()

    finally:
        # Cleanup
        for db_file in Path(".").glob("*_demo.omen"):
            db_file.unlink(missing_ok=True)

        log_files = ["omendb_production.log"]
        for log_file in log_files:
            if Path(log_file).exists():
                print(f"\n📝 Log file available: {log_file}")


if __name__ == "__main__":
    main()
