"""
Production-grade structured logging for OmenDB.

Provides structured logging with context, metrics collection, and
operational visibility for production deployments.
"""

import json
import logging
import time
import threading
import traceback
from datetime import datetime
from typing import Dict, Any, Optional, List, Union
from contextlib import contextmanager
from dataclasses import dataclass, asdict
from enum import Enum


class LogLevel(Enum):
    """Standardized log levels for OmenDB operations."""

    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"


class OperationType(Enum):
    """Database operation types for structured logging."""

    DB_CREATE = "db_create"
    DB_OPEN = "db_open"
    DB_CLOSE = "db_close"
    VECTOR_INSERT = "vector_insert"
    VECTOR_SEARCH = "vector_search"
    INDEX_CREATE = "index_create"
    INDEX_SAVE = "index_save"
    INDEX_LOAD = "index_load"
    ROARGRAPH_CONSTRUCT = "roargraph_construct"
    ROARGRAPH_SEARCH = "roargraph_search"
    MEMORY_PRESSURE = "memory_pressure"
    GARBAGE_COLLECTION = "garbage_collection"
    RESOURCE_CLEANUP = "resource_cleanup"


@dataclass
class LogContext:
    """Structured context for log entries."""

    operation: str
    database_path: Optional[str] = None
    vector_count: Optional[int] = None
    dimension: Optional[int] = None
    duration_ms: Optional[float] = None
    memory_usage_mb: Optional[float] = None
    error_type: Optional[str] = None
    error_code: Optional[str] = None
    user_context: Optional[Dict[str, Any]] = None


@dataclass
class PerformanceMetrics:
    """Performance metrics for operations."""

    operation_count: int = 0
    total_duration_ms: float = 0.0
    avg_duration_ms: float = 0.0
    min_duration_ms: float = float("inf")
    max_duration_ms: float = 0.0
    error_count: int = 0
    success_rate: float = 100.0


class OmenLogger:
    """Production-grade structured logger for OmenDB."""

    def __init__(
        self,
        name: str = "omendb",
        level: LogLevel = LogLevel.INFO,
        enable_metrics: bool = True,
        log_file: Optional[str] = None,
    ):
        """
        Initialize OmenDB logger.

        Args:
            name: Logger name
            level: Minimum log level
            enable_metrics: Enable performance metrics collection
            log_file: Optional log file path
        """
        self.name = name
        self.level = level
        self.enable_metrics = enable_metrics

        # Set up Python logger
        self._logger = logging.getLogger(name)
        self._logger.setLevel(getattr(logging, level.value))

        # Clear existing handlers
        self._logger.handlers.clear()

        # Console handler with structured format
        console_handler = logging.StreamHandler()
        console_handler.setFormatter(StructuredFormatter())
        self._logger.addHandler(console_handler)

        # File handler if specified
        if log_file:
            file_handler = logging.FileHandler(log_file)
            file_handler.setFormatter(StructuredFormatter())
            self._logger.addHandler(file_handler)

        # Metrics collection
        self._metrics: Dict[str, PerformanceMetrics] = {}
        self._metrics_lock = threading.Lock()

        # Operation context stack
        self._context_stack: List[Dict[str, Any]] = []
        self._context_lock = threading.Lock()

    def debug(self, message: str, context: Optional[LogContext] = None) -> None:
        """Log debug message."""
        self._log(LogLevel.DEBUG, message, context)

    def info(self, message: str, context: Optional[LogContext] = None) -> None:
        """Log info message."""
        self._log(LogLevel.INFO, message, context)

    def warning(self, message: str, context: Optional[LogContext] = None) -> None:
        """Log warning message."""
        self._log(LogLevel.WARNING, message, context)

    def error(
        self, message: str, context: Optional[LogContext] = None, exc_info: bool = False
    ) -> None:
        """Log error message."""
        self._log(LogLevel.ERROR, message, context, exc_info=exc_info)

    def critical(
        self, message: str, context: Optional[LogContext] = None, exc_info: bool = False
    ) -> None:
        """Log critical message."""
        self._log(LogLevel.CRITICAL, message, context, exc_info=exc_info)

    def _log(
        self,
        level: LogLevel,
        message: str,
        context: Optional[LogContext] = None,
        exc_info: bool = False,
    ) -> None:
        """Internal logging method."""
        if not self._should_log(level):
            return

        # Build log record
        log_data = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": level.value,
            "logger": self.name,
            "message": message,
            "thread_id": threading.get_ident(),
        }

        # Add context
        if context:
            log_data["context"] = asdict(context)

        # Add current operation context
        with self._context_lock:
            if self._context_stack:
                log_data["operation_context"] = self._context_stack[-1].copy()

        # Add exception info
        if exc_info:
            log_data["exception"] = traceback.format_exc()

        # Log to Python logger
        python_level = getattr(logging, level.value)
        self._logger.log(python_level, json.dumps(log_data, indent=None))

        # Update metrics
        if self.enable_metrics and context and context.operation:
            self._update_metrics(context)

    def _should_log(self, level: LogLevel) -> bool:
        """Check if message should be logged."""
        level_values = {
            LogLevel.DEBUG: 10,
            LogLevel.INFO: 20,
            LogLevel.WARNING: 30,
            LogLevel.ERROR: 40,
            LogLevel.CRITICAL: 50,
        }
        return level_values[level] >= level_values[self.level]

    def _update_metrics(self, context: LogContext) -> None:
        """Update performance metrics."""
        with self._metrics_lock:
            operation = context.operation

            if operation not in self._metrics:
                self._metrics[operation] = PerformanceMetrics()

            metrics = self._metrics[operation]
            metrics.operation_count += 1

            # Update duration metrics
            if context.duration_ms is not None:
                duration = context.duration_ms
                metrics.total_duration_ms += duration
                metrics.avg_duration_ms = (
                    metrics.total_duration_ms / metrics.operation_count
                )
                metrics.min_duration_ms = min(metrics.min_duration_ms, duration)
                metrics.max_duration_ms = max(metrics.max_duration_ms, duration)

            # Update error metrics
            if context.error_type:
                metrics.error_count += 1

            metrics.success_rate = (
                (metrics.operation_count - metrics.error_count)
                / metrics.operation_count
            ) * 100

    def get_metrics(
        self, operation: Optional[str] = None
    ) -> Union[Dict[str, PerformanceMetrics], PerformanceMetrics]:
        """Get performance metrics."""
        with self._metrics_lock:
            if operation:
                return self._metrics.get(operation, PerformanceMetrics())
            return self._metrics.copy()

    def reset_metrics(self, operation: Optional[str] = None) -> None:
        """Reset performance metrics."""
        with self._metrics_lock:
            if operation:
                self._metrics.pop(operation, None)
            else:
                self._metrics.clear()

    @contextmanager
    def operation_context(self, operation_type: OperationType, **kwargs):
        """Context manager for tracking operations."""
        context_data = {
            "operation": operation_type.value,
            "start_time": time.time(),
            **kwargs,
        }

        with self._context_lock:
            self._context_stack.append(context_data)

        start_time = time.time()
        error_occurred = False
        error_type = None

        try:
            yield context_data
        except Exception as e:
            error_occurred = True
            error_type = type(e).__name__
            raise
        finally:
            # Calculate duration
            duration_ms = (time.time() - start_time) * 1000

            # Create log context
            log_context = LogContext(
                operation=operation_type.value,
                duration_ms=duration_ms,
                error_type=error_type if error_occurred else None,
                **{
                    k: v
                    for k, v in kwargs.items()
                    if k in LogContext.__dataclass_fields__
                },
            )

            # Log operation completion
            if error_occurred:
                self.error(
                    f"Operation {operation_type.value} failed",
                    log_context,
                    exc_info=True,
                )
            else:
                self.info(f"Operation {operation_type.value} completed", log_context)

            # Remove from context stack
            with self._context_lock:
                if self._context_stack:
                    self._context_stack.pop()


class StructuredFormatter(logging.Formatter):
    """Custom formatter for structured JSON logging."""

    def format(self, record):
        """Format log record as JSON."""
        try:
            # Parse JSON from message if it's already structured
            if hasattr(record, "msg") and isinstance(record.msg, str):
                if record.msg.startswith("{"):
                    return record.msg

            # Fallback to standard formatting
            return super().format(record)
        except:
            return super().format(record)


# Global logger instance
_global_logger: Optional[OmenLogger] = None
_logger_lock = threading.Lock()


def get_logger(name: str = "omendb") -> OmenLogger:
    """Get global OmenDB logger instance."""
    global _global_logger

    with _logger_lock:
        if _global_logger is None:
            _global_logger = OmenLogger(name=name)
        return _global_logger


def configure_logging(
    level: LogLevel = LogLevel.INFO,
    log_file: Optional[str] = None,
    enable_metrics: bool = True,
) -> OmenLogger:
    """Configure global logging for OmenDB."""
    global _global_logger

    with _logger_lock:
        _global_logger = OmenLogger(
            level=level, log_file=log_file, enable_metrics=enable_metrics
        )
        return _global_logger


def log_operation(operation_type: OperationType):
    """Decorator for automatic operation logging."""

    def decorator(func):
        def wrapper(*args, **kwargs):
            logger = get_logger()

            with logger.operation_context(operation_type):
                return func(*args, **kwargs)

        return wrapper

    return decorator


# Convenience functions
def debug(message: str, **kwargs) -> None:
    """Log debug message."""
    context = LogContext(**kwargs) if kwargs else None
    get_logger().debug(message, context)


def info(message: str, **kwargs) -> None:
    """Log info message."""
    context = LogContext(**kwargs) if kwargs else None
    get_logger().info(message, context)


def warning(message: str, **kwargs) -> None:
    """Log warning message."""
    context = LogContext(**kwargs) if kwargs else None
    get_logger().warning(message, context)


def error(message: str, exc_info: bool = False, **kwargs) -> None:
    """Log error message."""
    context = LogContext(**kwargs) if kwargs else None
    get_logger().error(message, context, exc_info=exc_info)


def critical(message: str, exc_info: bool = False, **kwargs) -> None:
    """Log critical message."""
    context = LogContext(**kwargs) if kwargs else None
    get_logger().critical(message, context, exc_info=exc_info)
