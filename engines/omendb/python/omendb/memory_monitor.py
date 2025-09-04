"""
Memory monitoring and management for OmenDB production environments.

This module provides memory usage tracking, leak detection, and resource
management to ensure stable operation under memory pressure.
"""

import gc
import threading
import time
import weakref
from typing import Dict, List, Optional, Callable
from dataclasses import dataclass
from collections import defaultdict

from .exceptions import (
    OutOfMemoryError,
    MemoryLeakError,
    ResourceExhaustedError,
    create_context_error,
)


@dataclass
class MemoryStats:
    """Memory usage statistics."""

    total_allocated: int
    peak_usage: int
    current_usage: int
    active_databases: int
    vector_count: int
    last_gc_time: float
    gc_collections: int


class MemoryMonitor:
    """Memory usage monitor for production environments."""

    def __init__(
        self,
        max_memory_mb: int = 1024,
        warning_threshold: float = 0.8,
        monitor_interval: float = 5.0,
    ):
        """
        Initialize memory monitor.

        Args:
            max_memory_mb: Maximum memory usage in MB
            warning_threshold: Warning threshold (0.0-1.0)
            monitor_interval: Monitoring interval in seconds
        """
        self.max_memory_bytes = max_memory_mb * 1024 * 1024
        self.warning_threshold = warning_threshold
        self.monitor_interval = monitor_interval

        self._start_time = time.time()
        self._peak_usage = 0
        self._last_gc_time = 0
        self._gc_collections = 0
        self._active_databases = weakref.WeakSet()
        self._memory_history: List[int] = []
        self._warning_callbacks: List[Callable[[MemoryStats], None]] = []

        self._monitor_thread: Optional[threading.Thread] = None
        self._stop_monitoring = threading.Event()
        self._lock = threading.Lock()

    def register_database(self, db) -> None:
        """Register a database instance for monitoring."""
        with self._lock:
            self._active_databases.add(db)

    def unregister_database(self, db) -> None:
        """Unregister a database instance."""
        with self._lock:
            self._active_databases.discard(db)

    def add_warning_callback(self, callback: Callable[[MemoryStats], None]) -> None:
        """Add callback for memory warnings."""
        self._warning_callbacks.append(callback)

    def get_current_usage(self) -> int:
        """Get current memory usage in bytes."""
        try:
            import psutil
            import os

            process = psutil.Process(os.getpid())
            return process.memory_info().rss
        except ImportError:
            # Fallback to gc stats if psutil not available
            return sum(gc.get_count()) * 1000  # Rough estimate

    def get_stats(self) -> MemoryStats:
        """Get current memory statistics."""
        current_usage = self.get_current_usage()

        with self._lock:
            if current_usage > self._peak_usage:
                self._peak_usage = current_usage

            # Store history (keep last 100 readings)
            self._memory_history.append(current_usage)
            if len(self._memory_history) > 100:
                self._memory_history.pop(0)

            total_allocated = sum(self._memory_history)
            active_dbs = len(self._active_databases)

            # Count vectors across all databases
            vector_count = 0
            for db in self._active_databases:
                try:
                    if hasattr(db, "_get_vector_count"):
                        vector_count += db._get_vector_count()
                except:
                    pass

        return MemoryStats(
            total_allocated=total_allocated,
            peak_usage=self._peak_usage,
            current_usage=current_usage,
            active_databases=active_dbs,
            vector_count=vector_count,
            last_gc_time=self._last_gc_time,
            gc_collections=self._gc_collections,
        )

    def check_memory_pressure(self) -> bool:
        """Check if system is under memory pressure."""
        current_usage = self.get_current_usage()
        usage_ratio = current_usage / self.max_memory_bytes

        if usage_ratio > 1.0:
            raise OutOfMemoryError(
                f"Memory usage {current_usage} bytes exceeds limit {self.max_memory_bytes} bytes",
                context={
                    "current_usage": current_usage,
                    "limit": self.max_memory_bytes,
                },
            )

        if usage_ratio > self.warning_threshold:
            stats = self.get_stats()
            for callback in self._warning_callbacks:
                try:
                    callback(stats)
                except Exception:
                    pass  # Don't let callback errors affect monitoring
            return True

        return False

    def force_garbage_collection(self) -> int:
        """Force garbage collection and return freed bytes."""
        before_usage = self.get_current_usage()

        # Run garbage collection
        collected = gc.collect()
        self._gc_collections += collected
        self._last_gc_time = time.time()

        after_usage = self.get_current_usage()
        freed_bytes = max(0, before_usage - after_usage)

        return freed_bytes

    def detect_memory_leaks(self, threshold_mb: int = 100) -> bool:
        """Detect potential memory leaks."""
        if len(self._memory_history) < 10:
            return False

        # Check for consistent growth pattern
        recent_samples = self._memory_history[-10:]
        growth_trend = all(
            recent_samples[i] <= recent_samples[i + 1]
            for i in range(len(recent_samples) - 1)
        )

        if growth_trend:
            total_growth = recent_samples[-1] - recent_samples[0]
            if total_growth > threshold_mb * 1024 * 1024:
                return True

        return False

    def start_monitoring(self) -> None:
        """Start background memory monitoring."""
        if self._monitor_thread is not None:
            return

        self._stop_monitoring.clear()
        self._monitor_thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self._monitor_thread.start()

    def stop_monitoring(self) -> None:
        """Stop background memory monitoring."""
        if self._monitor_thread is None:
            return

        self._stop_monitoring.set()
        self._monitor_thread.join(timeout=1.0)
        self._monitor_thread = None

    def _monitor_loop(self) -> None:
        """Background monitoring loop."""
        while not self._stop_monitoring.wait(self.monitor_interval):
            try:
                # Check memory pressure
                self.check_memory_pressure()

                # Detect memory leaks
                if self.detect_memory_leaks():
                    stats = self.get_stats()
                    # Use structured logging instead of print
                    from .logging import get_logger, OperationType

                    logger = get_logger()
                    logger.warning(
                        "Memory leak detected",
                        operation=OperationType.MEMORY_PRESSURE.value,
                        memory_usage_mb=stats.current_usage / 1024 / 1024,
                    )

                # Automatic garbage collection under pressure
                current_usage = self.get_current_usage()
                if current_usage > self.max_memory_bytes * 0.9:
                    freed = self.force_garbage_collection()
                    if freed > 0:
                        from .logging import get_logger, OperationType

                        logger = get_logger()
                        logger.info(
                            "Automatic garbage collection completed",
                            operation=OperationType.GARBAGE_COLLECTION.value,
                            memory_usage_mb=current_usage / 1024 / 1024,
                            freed_bytes=freed,
                        )

            except Exception as e:
                # Use structured logging for errors
                from .logging import get_logger

                logger = get_logger()
                logger.error(
                    "Memory monitoring error",
                    operation="memory_monitor_error",
                    error_type=type(e).__name__,
                    exc_info=True,
                )


class ResourceManager:
    """Resource management for database operations."""

    def __init__(self, memory_monitor: Optional[MemoryMonitor] = None):
        """Initialize resource manager."""
        self.memory_monitor = memory_monitor or MemoryMonitor()
        self._operation_locks: Dict[str, threading.Lock] = defaultdict(threading.Lock)
        self._active_operations: Dict[str, int] = defaultdict(int)
        self._max_concurrent_ops = 10

    def acquire_operation_lock(self, operation_type: str, database_path: str) -> bool:
        """Acquire lock for database operation."""
        lock_key = f"{operation_type}:{database_path}"

        # Check if too many operations are active
        with self._operation_locks[lock_key]:
            if self._active_operations[lock_key] >= self._max_concurrent_ops:
                raise ResourceExhaustedError(
                    f"Too many concurrent {operation_type} operations",
                    context={
                        "operation": operation_type,
                        "active": self._active_operations[lock_key],
                    },
                )

            # Check memory pressure before expensive operations
            if operation_type in ["insert", "query"] and self.memory_monitor:
                try:
                    self.memory_monitor.check_memory_pressure()
                except OutOfMemoryError:
                    # Try garbage collection before failing
                    freed = self.memory_monitor.force_garbage_collection()
                    if freed < 1024 * 1024:  # Less than 1MB freed
                        raise

                    # Retry memory check after GC
                    self.memory_monitor.check_memory_pressure()

            self._active_operations[lock_key] += 1
            return True

    def release_operation_lock(self, operation_type: str, database_path: str) -> None:
        """Release lock for database operation."""
        lock_key = f"{operation_type}:{database_path}"

        with self._operation_locks[lock_key]:
            self._active_operations[lock_key] = max(
                0, self._active_operations[lock_key] - 1
            )

    def cleanup_resources(self, database_path: str) -> None:
        """Clean up resources for a database."""
        # Remove locks for this database
        keys_to_remove = [
            key
            for key in self._operation_locks.keys()
            if key.endswith(f":{database_path}")
        ]

        for key in keys_to_remove:
            del self._operation_locks[key]
            del self._active_operations[key]

        # Force garbage collection
        if self.memory_monitor:
            self.memory_monitor.force_garbage_collection()


# Global resource manager instance
_global_resource_manager: Optional[ResourceManager] = None


def get_resource_manager() -> ResourceManager:
    """Get global resource manager instance."""
    global _global_resource_manager

    if _global_resource_manager is None:
        _global_resource_manager = ResourceManager()
        _global_resource_manager.memory_monitor.start_monitoring()

    return _global_resource_manager


def cleanup_global_resources() -> None:
    """Clean up global resources on shutdown."""
    global _global_resource_manager

    if _global_resource_manager is not None:
        _global_resource_manager.memory_monitor.stop_monitoring()
        _global_resource_manager = None


# Context manager for operation resource management
class OperationContext:
    """Context manager for database operations with resource management."""

    def __init__(self, operation_type: str, database_path: str):
        """Initialize operation context."""
        self.operation_type = operation_type
        self.database_path = database_path
        self.resource_manager = get_resource_manager()
        self.acquired = False

    def __enter__(self):
        """Enter context and acquire resources."""
        self.acquired = self.resource_manager.acquire_operation_lock(
            self.operation_type, self.database_path
        )
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context and release resources."""
        if self.acquired:
            self.resource_manager.release_operation_lock(
                self.operation_type, self.database_path
            )


# Register cleanup on module exit
import atexit

atexit.register(cleanup_global_resources)
