"""Auto-batching implementation for OmenDB.

Automatically batches individual add() calls for better performance.
Expected improvement: 5-10x for individual operations.
"""

import time
import threading
from typing import List, Tuple, Optional, Dict, Any
from collections import deque


class AutoBatcher:
    """Automatically batches add operations within a time window.

    When rapid add() calls are detected, they're collected and
    sent as a batch for much better performance.
    """

    def __init__(
        self,
        flush_callback,
        max_batch_size: int = 1000,
        max_wait_ms: float = 10.0,
        auto_batch_threshold: int = 10,
    ):
        """Initialize auto-batcher.

        Args:
            flush_callback: Function to call with batched items
            max_batch_size: Maximum items before auto-flush (default: 1000)
            max_wait_ms: Maximum milliseconds to wait before flush (default: 10ms)
            auto_batch_threshold: Min items to trigger batching mode (default: 10)
        """
        self.flush_callback = flush_callback
        self.max_batch_size = max_batch_size
        self.max_wait_ms = max_wait_ms / 1000.0  # Convert to seconds
        self.auto_batch_threshold = auto_batch_threshold

        self.pending_batch = deque()
        self.batch_deadline = None
        self.batch_lock = threading.Lock()
        self.flush_timer = None
        self.is_batching = False

        # Statistics
        self.total_batched = 0
        self.total_individual = 0
        self.batch_count = 0

    def add(
        self, id: str, vector: List[float], metadata: Optional[Dict[str, str]] = None
    ) -> bool:
        """Add an item, potentially batching it.

        Returns immediately with assumed success for better UX.
        Actual add happens asynchronously when batch flushes.
        """
        with self.batch_lock:
            # Add to pending batch
            self.pending_batch.append((id, vector, metadata))

            # Check if we should enter batching mode
            if (
                not self.is_batching
                and len(self.pending_batch) >= self.auto_batch_threshold
            ):
                self.is_batching = True

            # If in batching mode, manage the batch
            if self.is_batching:
                # Flush if batch is full
                if len(self.pending_batch) >= self.max_batch_size:
                    self._flush_batch_locked()
                # Set deadline for time-based flush
                elif not self.flush_timer:
                    self._schedule_flush()
            else:
                # Not batching yet - if we have enough items, flush immediately
                if len(self.pending_batch) == 1:
                    # Single item, flush immediately
                    self._flush_batch_locked()

        return True  # Optimistic return

    def _schedule_flush(self):
        """Schedule a flush after max_wait_ms."""
        if self.flush_timer:
            self.flush_timer.cancel()

        self.flush_timer = threading.Timer(self.max_wait_ms, self._flush_on_timer)
        self.flush_timer.start()

    def _flush_on_timer(self):
        """Flush when timer expires."""
        with self.batch_lock:
            if self.pending_batch:
                self._flush_batch_locked()

    def _flush_batch_locked(self):
        """Flush pending batch (must hold lock)."""
        if not self.pending_batch:
            return

        # Cancel any pending timer
        if self.flush_timer:
            self.flush_timer.cancel()
            self.flush_timer = None

        # Extract batch
        batch = list(self.pending_batch)
        self.pending_batch.clear()

        # Update statistics
        batch_size = len(batch)
        if batch_size > 1:
            self.total_batched += batch_size
            self.batch_count += 1
        else:
            self.total_individual += 1

        # Exit batching mode if batch is small
        if batch_size < self.auto_batch_threshold:
            self.is_batching = False

        # Call flush callback (releases lock during callback)
        self.batch_lock.release()
        try:
            self.flush_callback(batch)
        finally:
            self.batch_lock.acquire()

    def flush(self):
        """Force flush any pending items."""
        with self.batch_lock:
            if self.pending_batch:
                self._flush_batch_locked()

    def get_stats(self) -> Dict[str, Any]:
        """Get batching statistics."""
        with self.batch_lock:
            if self.batch_count > 0:
                avg_batch_size = self.total_batched / self.batch_count
            else:
                avg_batch_size = 0

            return {
                "total_batched": self.total_batched,
                "total_individual": self.total_individual,
                "batch_count": self.batch_count,
                "avg_batch_size": avg_batch_size,
                "pending": len(self.pending_batch),
                "is_batching": self.is_batching,
                "efficiency": self.total_batched
                / max(1, self.total_batched + self.total_individual),
            }

    def __del__(self):
        """Ensure pending items are flushed on cleanup."""
        if hasattr(self, "pending_batch") and self.pending_batch:
            self.flush()
        if hasattr(self, "flush_timer") and self.flush_timer:
            self.flush_timer.cancel()
