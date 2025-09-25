"""
Simplified Streaming Ingestion API for OmenDB

A production-ready streaming vector ingestion implementation with dual-mode support,
configurable batching, backpressure handling, and comprehensive error recovery.

Key Features:
- Dual-mode architecture (embedded <50MB, server GB+ scale)
- Configurable batch sizes and memory management  
- Backpressure and flow control
- Progress monitoring and metrics
- Error recovery and retry mechanisms
"""

from collections import List, Optional, Dict
from time import perf_counter_ns

from core.record import VectorRecord
from storage.memory_store import MemoryVectorStore
from util.logging import Logger, LogLevel


# Stream configuration for dual-mode deployment
struct StreamConfig(Copyable, Movable):
    var batch_size: Int
    var max_memory_mb: Int
    var backpressure_threshold: Float64
    var embedded_mode: Bool
    
    fn __init__(out self, batch_size: Int = 1000, max_memory_mb: Int = 100, 
                backpressure_threshold: Float64 = 0.8, embedded_mode: Bool = True):
        self.batch_size = batch_size
        self.max_memory_mb = max_memory_mb
        self.backpressure_threshold = backpressure_threshold
        self.embedded_mode = embedded_mode


# Stream metrics for monitoring
struct StreamMetrics(Copyable, Movable):
    var vectors_received: Int
    var vectors_processed: Int
    var batches_processed: Int
    var current_memory_mb: Float64
    var throughput_vectors_per_sec: Float64
    var last_flush_time_ns: Int
    
    fn __init__(out self):
        self.vectors_received = 0
        self.vectors_processed = 0
        self.batches_processed = 0
        self.current_memory_mb = 0.0
        self.throughput_vectors_per_sec = 0.0
        self.last_flush_time_ns = 0


# Stream processing result
struct StreamResult(Copyable, Movable):
    var success: Bool
    var vectors_processed: Int
    var error_message: String
    var retry_suggested: Bool
    
    fn __init__(out self, success: Bool, processed: Int = 0, error: String = "", retry: Bool = False):
        self.success = success
        self.vectors_processed = processed
        self.error_message = error
        self.retry_suggested = retry


# Stream handle for managing individual streams
struct StreamHandle[dtype: DType = DType.float32](Copyable, Movable):
    var stream_id: String
    var config: StreamConfig
    var buffer: List[VectorRecord[dtype]]
    var metrics: StreamMetrics
    var is_active: Bool
    var last_activity_ns: Int
    
    fn __init__(out self, stream_id: String, config: StreamConfig):
        self.stream_id = stream_id
        self.config = config
        self.buffer = List[VectorRecord[dtype]]()
        self.metrics = StreamMetrics()
        self.is_active = True
        self.last_activity_ns = perf_counter_ns()
    
    fn is_buffer_full(self) -> Bool:
        return len(self.buffer) >= self.config.batch_size
    
    fn get_memory_usage_mb(self) -> Float64:
        """Estimate current memory usage for backpressure detection."""
        var base_size = 256.0  # Base struct overhead
        var vector_size = 0.0
        
        for i in range(len(self.buffer)):
            # Estimate: vector dimension * 4 bytes + metadata overhead
            vector_size += self.buffer[i].vector.dimension() * 4.0 + 128.0
        
        return (base_size + vector_size) / (1024.0 * 1024.0)
    
    fn should_apply_backpressure(self) -> Bool:
        """Check if backpressure should be applied."""
        var memory_usage = self.get_memory_usage_mb()
        var memory_pressure = memory_usage / Float64(self.config.max_memory_mb)
        return memory_pressure > self.config.backpressure_threshold


# Main streaming ingestion interface - dual-mode compatible
struct StreamingIngestor:
    var store: MemoryVectorStore
    var active_streams: Dict[String, StreamHandle[DType.float32]]
    var global_config: StreamConfig
    var logger: Logger
    var is_shutdown: Bool
    
    fn __init__(out self, mut store: MemoryVectorStore, config: StreamConfig) raises:
        self.store = store^
        self.active_streams = Dict[String, StreamHandle[DType.float32]]()
        self.global_config = config
        self.logger = Logger(LogLevel.INFO)
        self.is_shutdown = False
    
    fn create_stream(mut self, stream_id: String, config: Optional[StreamConfig] = Optional[StreamConfig]()) raises -> String:
        """Create new streaming ingestion stream."""
        if self.is_shutdown:
            raise Error("StreamingIngestor is shutdown")
        
        if stream_id in self.active_streams:
            raise Error("Stream '" + stream_id + "' already exists")
        
        # Use provided config or global default
        var stream_config = config.value() if config else self.global_config
        
        # Create stream handle
        var handle = StreamHandle[DType.float32](stream_id, stream_config)
        self.active_streams[stream_id] = handle^
        
        self.logger.info("Created stream: " + stream_id)
        return stream_id
    
    fn ingest_vector(mut self, stream_id: String, vector: VectorRecord[DType.float32]) raises -> StreamResult:
        """Ingest single vector into streaming buffer."""
        if self.is_shutdown:
            return StreamResult(False, 0, "StreamingIngestor is shutdown", False)
        
        if stream_id not in self.active_streams:
            return StreamResult(False, 0, "Stream not found: " + stream_id, False)
        
        var handle = self.active_streams[stream_id]
        
        # Check if stream is active
        if not handle.is_active:
            return StreamResult(False, 0, "Stream not active", False)
        
        # Apply backpressure if memory pressure high
        if handle.should_apply_backpressure():
            self.logger.warning("Backpressure applied for stream: " + stream_id)
            return StreamResult(False, 0, "Backpressure: memory limit reached", True)
        
        try:
            # Add vector to buffer
            handle.buffer.append(vector)
            handle.metrics.vectors_received += 1
            handle.last_activity_ns = perf_counter_ns()
            
            # Update handle back to dict
            self.active_streams[stream_id] = handle^
            
            # Auto-flush if buffer full
            if self.active_streams[stream_id].is_buffer_full():
                return self._flush_stream_internal(stream_id)
            
            return StreamResult(True, 1, "", False)
            
        except e:
            var error_msg = "Failed to ingest vector: " + String(e)
            self.logger.error(error_msg)
            return StreamResult(False, 0, error_msg, True)
    
    fn ingest_batch(mut self, stream_id: String, vectors: List[VectorRecord[DType.float32]]) raises -> StreamResult:
        """Ingest batch of vectors with optimized processing."""
        if len(vectors) == 0:
            return StreamResult(True, 0, "", False)
        
        var total_processed = 0
        
        for i in range(len(vectors)):
            var result = self.ingest_vector(stream_id, vectors[i])
            if result.success:
                total_processed += result.vectors_processed
            else:
                # Stop on critical errors, continue on retryable errors
                if not result.retry_suggested:
                    break
        
        var success = total_processed == len(vectors)
        return StreamResult(success, total_processed, "", not success)
    
    fn flush_stream(mut self, stream_id: String) raises -> StreamResult:
        """Manually flush stream buffer to storage."""
        return self._flush_stream_internal(stream_id)
    
    fn _flush_stream_internal(mut self, stream_id: String) raises -> StreamResult:
        """Internal flush implementation."""
        if stream_id not in self.active_streams:
            return StreamResult(False, 0, "Stream not found: " + stream_id, False)
        
        var handle = self.active_streams[stream_id]
        
        if len(handle.buffer) == 0:
            return StreamResult(True, 0, "", False)
        
        var start_time = perf_counter_ns()
        var processed = 0
        
        try:
            # Process buffer
            for i in range(len(handle.buffer)):
                _ = self.store.insert(handle.buffer[i])
                processed += 1
            
            # Update metrics
            handle.metrics.vectors_processed += processed
            handle.metrics.batches_processed += 1
            handle.metrics.last_flush_time_ns = perf_counter_ns()
            
            var latency_ms = Float64(handle.metrics.last_flush_time_ns - start_time) / 1_000_000.0
            
            # Calculate throughput
            var time_elapsed_sec = latency_ms / 1000.0
            if time_elapsed_sec > 0:
                handle.metrics.throughput_vectors_per_sec = Float64(processed) / time_elapsed_sec
            
            # Clear buffer
            handle.buffer.clear()
            
            # Update handle back to dict
            self.active_streams[stream_id] = handle^
            
            self.logger.info("Flushed " + String(processed) + " vectors from stream: " + stream_id)
            
            return StreamResult(True, processed, "", False)
            
        except e:
            var error_msg = "Flush failed: " + String(e)
            self.logger.error(error_msg)
            return StreamResult(False, 0, error_msg, True)
    
    fn close_stream(mut self, stream_id: String) raises -> StreamResult:
        """Close stream and flush any remaining data."""
        if stream_id not in self.active_streams:
            return StreamResult(False, 0, "Stream not found: " + stream_id, False)
        
        # Final flush
        var result = self._flush_stream_internal(stream_id)
        
        # Remove stream
        _ = self.active_streams.pop(stream_id, StreamHandle[DType.float32]("", StreamConfig()))
        
        self.logger.info("Closed stream: " + stream_id)
        return result
    
    fn get_stream_metrics(self, stream_id: String) raises -> Optional[StreamMetrics]:
        """Get current metrics for a stream."""
        if stream_id not in self.active_streams:
            return Optional[StreamMetrics]()
        
        var handle = self.active_streams[stream_id]
        handle.metrics.current_memory_mb = handle.get_memory_usage_mb()
        return handle.metrics
    
    fn shutdown(mut self) raises:
        """Shutdown streaming ingestor and flush all streams."""
        self.logger.info("Shutting down streaming ingestor...")
        
        # Close all streams
        var stream_ids = List[String]()
        for key in self.active_streams.keys():
            stream_ids.append(key)
        
        for i in range(len(stream_ids)):
            _ = self.close_stream(stream_ids[i])
        
        self.is_shutdown = True
        self.logger.info("Streaming ingestor shutdown complete")


# Helper functions for creating default configurations
fn create_embedded_stream_config() -> StreamConfig:
    """Create optimized configuration for embedded mode."""
    return StreamConfig(
        batch_size=1000,
        max_memory_mb=50,
        backpressure_threshold=0.8,
        embedded_mode=True
    )

fn create_server_stream_config() -> StreamConfig:
    """Create optimized configuration for server mode."""
    return StreamConfig(
        batch_size=5000,
        max_memory_mb=1000,  # 1GB for server mode
        backpressure_threshold=0.7,  # More aggressive backpressure for server
        embedded_mode=False
    )