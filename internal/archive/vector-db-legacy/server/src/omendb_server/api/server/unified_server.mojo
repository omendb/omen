"""
Unified OmenDB Server with Server Adapter Integration
=====================================================

This module provides the enhanced server implementation that integrates
the unified core engine via the server adapter, supporting both REST and gRPC
interfaces with the same 80% shared core as embedded mode.
"""

from collections import List, Dict, Optional
from time import perf_counter_ns

from core.vector import Vector, VectorID
from core.record import VectorRecord, SearchResult
from core.distance import DistanceMetric
from adapters.server_adapter import ServerAdapter
from adapters.deployment_interface import DeploymentConfig, DeploymentStats
from monitoring.server_monitoring import ServerMonitor, create_server_monitor
from util.logging import Logger, LogLevel


@value
struct UnifiedOmenDBServer[dtype: DType = DType.float32]:
    """
    Enhanced OmenDB server using unified core via server adapter.
    
    This server provides the same vector engine as embedded mode (80% shared)
    while adding server-specific features like distributed processing,
    monitoring, and enterprise capabilities.
    
    Key features:
    - Same core algorithms as embedded (RoarGraph, learned indices, Matryoshka)
    - Server optimizations (batching, concurrency, monitoring)
    - REST and gRPC API support
    - Production logging and metrics
    - Seamless embedded→server migration
    """
    
    var adapter: ServerAdapter
    var logger: Logger
    var config: DeploymentConfig
    var startup_time: Int
    var request_count: Int
    var monitor: ServerMonitor
    
    fn __init__(inout self, dimension: Int, storage_path: String = "server_storage", distance_metric: DistanceMetric = DistanceMetric.COSINE):
        """
        Initialize unified server with server adapter.
        
        Args:
            dimension: Vector dimension
            storage_path: Path for server storage
            distance_metric: Distance metric for similarity search
        """
        self.adapter = ServerAdapter(dimension, distance_metric)
        self.logger = Logger("UnifiedOmenDBServer", LogLevel.INFO)
        self.config = DeploymentConfig(0, storage_path, dimension)
        self.startup_time = perf_counter_ns()
        self.request_count = 0
        self.monitor = create_server_monitor()
        
        # Initialize server adapter
        try:
            self.adapter.initialize(self.config)
            self.logger.info("🚀 Unified OmenDB Server initialized")
            self.logger.info("   Mode: Server (unified core)")
            self.logger.info("   Dimension: " + String(dimension))
            self.logger.info("   Storage: " + storage_path)
        except e:
            self.logger.error("Failed to initialize server: " + String(e))
    
    fn insert_vector(inout self, id: String, vector: Vector[dtype], metadata: Dict[String, String] = Dict[String, String]()) raises -> (Bool, String):
        """
        Insert a vector using unified core engine.
        
        Args:
            id: Unique identifier for the vector
            vector: The vector to insert
            metadata: Optional metadata
            
        Returns:
            Tuple of (success, message)
        """
        self._increment_request_count()
        self.logger.debug("Server insert: " + id)
        
        # Start monitoring
        var request_id = self.monitor.start_request("insert", 1)
        
        try:
            var vector_id = VectorID(id)
            self.adapter.insert(vector, vector_id)
            
            # Complete monitoring - success
            self.monitor.complete_request(request_id, True)
            return (True, "Vector inserted successfully")
            
        except e:
            # Complete monitoring - failure
            self.monitor.complete_request(request_id, False, String(e))
            self.logger.error("Insert failed: " + String(e))
            return (False, "Insert failed: " + String(e))
    
    fn insert_batch(inout self, records: List[VectorRecord]) raises -> (Int, Int, String):
        """
        Batch insert vectors using server optimizations.
        
        Args:
            records: List of vector records to insert
            
        Returns:
            Tuple of (successful_count, failed_count, message)
        """
        self._increment_request_count()
        self.logger.info("Server batch insert: " + String(len(records)) + " vectors")
        
        # Start monitoring
        var request_id = self.monitor.start_request("batch_insert", len(records))
        
        try:
            self.adapter.insert_batch(records)
            
            # Complete monitoring - success
            self.monitor.complete_request(request_id, True, "", len(records))
            return (len(records), 0, "All vectors inserted successfully")
            
        except e:
            # Complete monitoring - failure
            self.monitor.complete_request(request_id, False, String(e))
            self.logger.error("Batch insert failed: " + String(e))
            return (0, len(records), "Batch insert failed: " + String(e))
    
    fn search_vectors(self, query: Vector[dtype], k: Int = 10) raises -> (Bool, String, List[SearchResult]):
        """
        Search vectors using unified core engine.
        
        Args:
            query: Query vector
            k: Number of results to return
            
        Returns:
            Tuple of (success, message, results)
        """
        var request_count = self._increment_request_count()
        self.logger.debug("Server search #" + String(request_count) + ", k=" + String(k))
        
        # Start monitoring
        var request_id = self.monitor.start_request("search", 1)
        
        try:
            var results = self.adapter.search(query, k)
            
            # Complete monitoring - success
            self.monitor.complete_request(request_id, True, "", len(results))
            self.logger.debug("Search completed, found " + String(len(results)) + " results")
            return (True, "Search completed successfully", results)
            
        except e:
            # Complete monitoring - failure
            self.monitor.complete_request(request_id, False, String(e))
            self.logger.error("Search failed: " + String(e))
            return (False, "Search failed: " + String(e), List[SearchResult]())
    
    fn get_stats(self) -> DeploymentStats:
        """Get comprehensive server statistics."""
        var stats = self.adapter.get_stats()
        
        # Add server-specific metrics
        var uptime = perf_counter_ns() - self.startup_time
        stats.uptime_seconds = uptime / 1000000000  # Convert to seconds
        stats.total_requests = self.request_count
        
        return stats
    
    fn persist(self) raises:
        """Persist server state to storage."""
        self.logger.info("Persisting server state...")
        self.adapter.persist()
    
    fn load(inout self, path: String) raises:
        """Load server state from storage."""
        self.logger.info("Loading server state from: " + path)
        self.adapter.load(path)
    
    fn optimize(inout self) raises:
        """Optimize server performance."""
        self.logger.info("Optimizing server performance...")
        self.adapter.optimize()
    
    fn shutdown(inout self) raises:
        """Gracefully shutdown the server."""
        self.logger.info("Shutting down unified server...")
        self.adapter.shutdown()
        self.logger.info("✅ Server shutdown complete")
    
    fn memory_footprint(self) -> Int:
        """Get total server memory usage."""
        return self.adapter.memory_footprint()
    
    fn get_performance_summary(self) -> String:
        """Get human-readable performance summary."""
        var stats = self.get_stats()
        var uptime_hours = Float64(stats.uptime_seconds) / 3600.0
        var qps = 0.0
        
        if stats.uptime_seconds > 0:
            qps = Float64(stats.total_requests) / Float64(stats.uptime_seconds)
        
        var summary = "🚀 Unified OmenDB Server Performance\\n"
        summary += "=====================================\\n"
        summary += "Vectors: " + String(stats.vector_count) + "\\n"
        summary += "Uptime: " + String(uptime_hours) + " hours\\n"
        summary += "Requests: " + String(stats.total_requests) + "\\n"
        summary += "QPS: " + String(qps) + "\\n"
        summary += "Memory: " + String(stats.memory_usage / 1024 / 1024) + " MB\\n"
        summary += "Search Latency: " + String(stats.search_latency / 1000) + " μs\\n"
        summary += "Cluster Health: " + stats.cluster_health + "\\n"
        
        return summary
    
    fn get_health_status(inout self) -> String:
        """Get current server health status."""
        return self.monitor.get_health_status()
    
    fn get_monitoring_summary(self) -> String:
        """Get comprehensive monitoring summary."""
        return self.monitor.get_monitoring_summary()
    
    fn enable_monitoring(inout self):
        """Enable request monitoring."""
        self.monitor.enable_monitoring()
        self.logger.info("📊 Request monitoring enabled")
    
    fn disable_monitoring(inout self):
        """Disable request monitoring."""
        self.monitor.disable_monitoring()
        self.logger.info("📊 Request monitoring disabled")
    
    # Private helper methods
    fn _increment_request_count(inout self) -> Int:
        """Increment and return request count for metrics."""
        self.request_count += 1
        return self.request_count


# Factory function for creating servers
fn create_unified_server[dtype: DType = DType.float32](dimension: Int, storage_path: String = "server_storage") -> UnifiedOmenDBServer[dtype]:
    """
    Factory function to create a unified OmenDB server.
    
    Args:
        dimension: Vector dimension
        storage_path: Storage path for server data
        
    Returns:
        Configured UnifiedOmenDBServer instance
    """
    return UnifiedOmenDBServer[dtype](dimension, storage_path)