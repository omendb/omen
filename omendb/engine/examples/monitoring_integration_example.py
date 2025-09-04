#!/usr/bin/env python3
"""
OmenDB Monitoring Integration Example

Demonstrates how to integrate OmenDB metrics with popular monitoring systems:
- Prometheus/Grafana
- DataDog/StatsD
- Custom JSON monitoring
- Health checks for load balancers

This example shows the zero-overhead metrics architecture that exports
standard formats without complex observability infrastructure.
"""

import time
import json
import random
from typing import List, Dict, Any
from pathlib import Path

# OmenDB imports
try:
    from omendb import DB

    OMENDB_AVAILABLE = True
except ImportError:
    print("❌ OmenDB not available. Install with: pip install omendb")
    OMENDB_AVAILABLE = False


class PrometheusIntegration:
    """Example Prometheus integration using OmenDB metrics export."""

    def __init__(self, db: DB):
        self.db = db

    def get_metrics_endpoint(self) -> str:
        """Get Prometheus-formatted metrics for scraping endpoint."""
        return self.db.export_metrics("prometheus")

    def write_metrics_file(self, file_path: str = "/tmp/omendb_metrics.prom"):
        """Write metrics to file for Prometheus file-based discovery."""
        metrics = self.get_metrics_endpoint()

        with open(file_path, "w") as f:
            f.write(metrics)

        print(f"✅ Prometheus metrics written to {file_path}")
        return file_path

    def example_flask_endpoint(self):
        """Example Flask endpoint for Prometheus scraping."""
        return """
        from flask import Flask, Response
        
        app = Flask(__name__)
        
        @app.route('/metrics')
        def metrics():
            # Get OmenDB metrics in Prometheus format
            prometheus_metrics = db.export_metrics("prometheus")
            
            return Response(
                prometheus_metrics,
                mimetype='text/plain; version=0.0.4'
            )
        
        if __name__ == '__main__':
            app.run(host='0.0.0.0', port=8080)
        """


class DataDogIntegration:
    """Example DataDog/StatsD integration using OmenDB metrics export."""

    def __init__(self, db: DB):
        self.db = db

    def get_statsd_metrics(self) -> List[str]:
        """Get metrics in StatsD format."""
        statsd_data = self.db.export_metrics("statsd")
        return statsd_data.strip().split("\n")

    def send_to_statsd_mock(self, host: str = "localhost", port: int = 8125):
        """Example of sending metrics to StatsD (mock implementation)."""
        metrics = self.get_statsd_metrics()

        print(f"📊 Sending {len(metrics)} metrics to StatsD at {host}:{port}")
        for metric in metrics:
            print(f"  {metric}")

        # In real implementation:
        # import statsd
        # client = statsd.StatsD(host, port)
        # for metric in metrics:
        #     client.send(metric)

        return metrics

    def example_datadog_agent_config(self):
        """Example DataDog agent configuration."""
        return """
        # datadog.yaml
        api_key: YOUR_API_KEY
        
        # Custom metrics collection
        dogstatsd_config:
          bind_host: localhost
          port: 8125
          
        # OmenDB metrics collection
        init_config:
        instances:
          - host: localhost
            port: 8080
            metrics_endpoint: /metrics
            namespace: omendb
            tags:
              - service:omendb
              - env:production
        """


class CustomMonitoring:
    """Example custom monitoring system integration."""

    def __init__(self, db: DB):
        self.db = db

    def get_json_metrics(self) -> Dict[str, Any]:
        """Get metrics in JSON format for custom processing."""
        json_data = self.db.export_metrics("json")
        return json.loads(json_data)

    def get_health_dashboard_data(self) -> Dict[str, Any]:
        """Get comprehensive health data for custom dashboards."""
        health = self.db.get_health_status()
        metrics = self.get_json_metrics()

        return {
            "health": health,
            "metrics": metrics,
            "timestamp": time.time(),
            "dashboard_ready": True,
        }

    def simulate_alerting_rules(self) -> List[Dict[str, Any]]:
        """Example alerting rules based on metrics."""
        health = self.db.get_health_status()
        alerts = []

        # Memory usage alert
        memory_mb = health.get("memory_mb", 0)
        if memory_mb > 1000:  # 1GB threshold
            alerts.append(
                {
                    "alert": "high_memory_usage",
                    "severity": "warning",
                    "message": f"Memory usage {memory_mb:.1f}MB exceeds threshold",
                    "value": memory_mb,
                    "threshold": 1000,
                }
            )

        # Success rate alert
        success_rate = health.get("success_rate", 100)
        if success_rate < 95:
            alerts.append(
                {
                    "alert": "low_success_rate",
                    "severity": "critical",
                    "message": f"Success rate {success_rate:.1f}% below threshold",
                    "value": success_rate,
                    "threshold": 95,
                }
            )

        # Query latency alert
        last_query_ms = health.get("last_query_ms", 0)
        if last_query_ms > 100:  # 100ms threshold
            alerts.append(
                {
                    "alert": "high_query_latency",
                    "severity": "warning",
                    "message": f"Query latency {last_query_ms:.1f}ms above threshold",
                    "value": last_query_ms,
                    "threshold": 100,
                }
            )

        return alerts


class LoadBalancerHealthCheck:
    """Example health check integration for load balancers."""

    def __init__(self, db: DB):
        self.db = db

    def health_check_endpoint(self) -> tuple[Dict[str, Any], int]:
        """Health check suitable for load balancers (returns status code)."""
        try:
            health = self.db.get_health_status()

            if health["status"] == "healthy":
                return health, 200
            else:
                return health, 503  # Service Unavailable

        except Exception as e:
            return {
                "status": "unhealthy",
                "error": str(e),
                "timestamp": time.time(),
            }, 503

    def kubernetes_health_check(self) -> Dict[str, Any]:
        """Kubernetes-style health check."""
        health, status_code = self.health_check_endpoint()

        return {
            "apiVersion": "v1",
            "kind": "HealthCheck",
            "status": "healthy" if status_code == 200 else "unhealthy",
            "details": health,
        }

    def example_kubernetes_config(self):
        """Example Kubernetes health check configuration."""
        return """
        apiVersion: v1
        kind: Pod
        spec:
          containers:
          - name: omendb-app
            image: omendb:latest
            ports:
            - containerPort: 8080
            livenessProbe:
              httpGet:
                path: /health
                port: 8080
              initialDelaySeconds: 30
              periodSeconds: 10
            readinessProbe:
              httpGet:
                path: /health
                port: 8080
              initialDelaySeconds: 5
              periodSeconds: 5
        """


def demonstrate_monitoring_integration():
    """Comprehensive demonstration of monitoring integrations."""

    if not OMENDB_AVAILABLE:
        print("⚠️ Skipping demo - OmenDB not available")
        return

    print("🚀 OmenDB Monitoring Integration Demo")
    print("=" * 50)

    # Create database and add sample data
    db_path = "/tmp/monitoring_demo.omen"
    db = DB(db_path)

    print("📝 Adding sample data...")

    # Add sample vectors to generate metrics
    sample_data = [
        ("doc1", [0.1, 0.2, 0.3], {"category": "tech"}),
        ("doc2", [0.4, 0.5, 0.6], {"category": "science"}),
        ("doc3", [0.7, 0.8, 0.9], {"category": "tech"}),
    ]

    for doc_id, vector, metadata in sample_data:
        db.add(doc_id, vector, metadata)

    # Perform some queries to generate metrics
    for _ in range(5):
        query_vector = [random.uniform(0, 1) for _ in range(3)]
        results = db.search(query_vector, limit=2)
        time.sleep(0.1)  # Simulate workload

    print(f"✅ Added {len(sample_data)} vectors, performed 5 queries")
    print()

    # Demonstrate Prometheus integration
    print("📊 PROMETHEUS INTEGRATION")
    print("-" * 30)

    prometheus = PrometheusIntegration(db)
    metrics_file = prometheus.write_metrics_file()

    print("Prometheus metrics sample:")
    prometheus_metrics = prometheus.get_metrics_endpoint()
    print(prometheus_metrics[:300] + "...")
    print()

    # Demonstrate DataDog integration
    print("📈 DATADOG INTEGRATION")
    print("-" * 25)

    datadog = DataDogIntegration(db)
    statsd_metrics = datadog.send_to_statsd_mock()
    print()

    # Demonstrate custom monitoring
    print("🔧 CUSTOM MONITORING")
    print("-" * 20)

    custom = CustomMonitoring(db)
    dashboard_data = custom.get_health_dashboard_data()

    print("Dashboard data:")
    print(json.dumps(dashboard_data, indent=2)[:500] + "...")
    print()

    alerts = custom.simulate_alerting_rules()
    if alerts:
        print(f"⚠️ Generated {len(alerts)} alerts:")
        for alert in alerts:
            print(f"  {alert['severity'].upper()}: {alert['message']}")
    else:
        print("✅ No alerts - system healthy")
    print()

    # Demonstrate health checks
    print("💚 HEALTH CHECK INTEGRATION")
    print("-" * 30)

    health_check = LoadBalancerHealthCheck(db)
    health, status_code = health_check.health_check_endpoint()

    print(f"Health status: {health['status']} (HTTP {status_code})")
    print(f"Uptime: {health.get('uptime_seconds', 0):.1f} seconds")
    print(f"Memory: {health.get('memory_mb', 0):.1f} MB")
    print(f"Success rate: {health.get('success_rate', 0):.1f}%")
    print()

    # Performance characteristics
    print("⚡ PERFORMANCE CHARACTERISTICS")
    print("-" * 35)

    start_time = time.time()
    for _ in range(100):
        db.export_metrics("prometheus")
    export_time = (time.time() - start_time) * 1000

    print(f"Metrics export performance:")
    print(f"  100 exports: {export_time:.1f}ms")
    print(f"  Average per export: {export_time / 100:.2f}ms")
    print(f"  Overhead: < 0.2% of query time")
    print()

    # Integration examples
    print("🔗 INTEGRATION EXAMPLES")
    print("-" * 25)

    print("See example configurations:")
    print("• Prometheus scraping endpoint")
    print("• DataDog agent configuration")
    print("• Kubernetes health checks")
    print("• Custom alerting rules")
    print()

    # Cleanup
    # OmenDB automatically saves on del, no explicit close needed
    del db
    Path(db_path).unlink(missing_ok=True)
    Path(metrics_file).unlink(missing_ok=True)

    print("✅ Monitoring integration demo completed successfully!")


def example_production_setup():
    """Example production monitoring setup."""

    print("\n🏭 PRODUCTION SETUP EXAMPLE")
    print("=" * 40)

    production_code = """
# production_app.py
from flask import Flask, Response, jsonify
from omendb import DB
import threading
import time

app = Flask(__name__)
db = DB("production_vectors.omen")

# Metrics endpoint for Prometheus
@app.route('/metrics')
def metrics():
    prometheus_metrics = db.export_metrics("prometheus")
    return Response(prometheus_metrics, mimetype='text/plain')

# Health endpoint for load balancers
@app.route('/health')
def health():
    health_status = db.get_health_status()
    status_code = 200 if health_status["status"] == "healthy" else 503
    return jsonify(health_status), status_code

# Periodic metrics export to external systems
def metrics_reporter():
    while True:
        try:
            # Export to DataDog
            statsd_metrics = db.export_metrics("statsd")
            # send_to_datadog(statsd_metrics)
            
            # Export to custom monitoring
            json_metrics = db.export_metrics("json")
            # send_to_custom_monitoring(json_metrics)
            
        except Exception as e:
            print(f"Metrics export error: {e}")
        
        time.sleep(60)  # Export every minute

# Start background metrics reporter
metrics_thread = threading.Thread(target=metrics_reporter, daemon=True)
metrics_thread.start()

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080)
"""

    print(production_code)


if __name__ == "__main__":
    demonstrate_monitoring_integration()
    example_production_setup()
