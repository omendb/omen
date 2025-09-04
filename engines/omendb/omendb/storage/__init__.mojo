"""Storage backends for OmenDB.

Provides different storage implementations for persistence.
"""

from .memory_mapped import MemoryMappedStorage, create_optimal_storage