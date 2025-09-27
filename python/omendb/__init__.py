"""
OmenDB - 10x faster learned database with ML-powered indexing

A high-performance database that uses machine learning to predict data locations,
achieving 5-10x speedup over traditional B-tree indexes for sequential workloads.
"""

from omendb._omendb import (
    OmenDB,
    __version__,
    INDEX_NONE,
    INDEX_LINEAR,
    INDEX_RMI,
    ISO_READ_UNCOMMITTED,
    ISO_READ_COMMITTED,
    ISO_REPEATABLE_READ,
    ISO_SERIALIZABLE,
)
import numpy as np
from typing import Optional, List, Tuple, Union


class DB:
    """
    High-level Python interface for OmenDB

    Example:
        >>> db = DB("./mydb.omen", index_type="linear")
        >>> db.put(1, b"hello")
        >>> db.get(1)
        b'hello'
        >>> db.bulk_insert([(i, f"value_{i}".encode()) for i in range(1000)])
        >>> results = db.range(10, 100)
    """

    def __init__(self, path: str, index_type: str = "linear"):
        """
        Create or open an OmenDB database

        Args:
            path: Path to database directory
            index_type: Type of learned index ("none", "linear", "rmi")
        """
        self._db = OmenDB(path, index_type)
        self.path = path
        self.index_type = index_type

    def put(self, key: int, value: Union[bytes, str]) -> None:
        """Store a key-value pair"""
        if isinstance(value, str):
            value = value.encode('utf-8')
        self._db.put(key, value)

    def get(self, key: int) -> Optional[bytes]:
        """Retrieve a value by key"""
        return self._db.get(key)

    def delete(self, key: int) -> None:
        """Delete a key-value pair"""
        self._db.delete(key)

    def range(self, start: int, end: int) -> List[Tuple[int, bytes]]:
        """Query a range of keys (inclusive)"""
        return self._db.range(start, end)

    def bulk_insert(self, data: List[Tuple[int, Union[bytes, str]]]) -> None:
        """
        Insert many key-value pairs efficiently

        Args:
            data: List of (key, value) tuples
        """
        # Convert strings to bytes
        converted = []
        for key, value in data:
            if isinstance(value, str):
                value = value.encode('utf-8')
            converted.append((key, value))

        self._db.bulk_insert(converted)

    def bulk_insert_numpy(self, keys: np.ndarray, values: List[bytes]) -> None:
        """
        Insert from NumPy array (optimized for ML workloads)

        Args:
            keys: NumPy array of integer keys
            values: List of byte values
        """
        data = [(int(k), v) for k, v in zip(keys, values)]
        self._db.bulk_insert(data)

    def benchmark(self, num_queries: int = 10000) -> str:
        """Run a performance benchmark"""
        return self._db.benchmark(num_queries)

    def stats(self) -> str:
        """Get database statistics"""
        return self._db.stats()

    # Transaction support
    def begin_transaction(self, isolation: str = "read_committed") -> int:
        """
        Begin a new transaction

        Args:
            isolation: Isolation level (read_uncommitted, read_committed,
                      repeatable_read, serializable)

        Returns:
            Transaction ID
        """
        return self._db.begin_transaction(isolation)

    def commit(self, txn_id: int) -> None:
        """Commit a transaction"""
        self._db.commit(txn_id)

    def rollback(self, txn_id: int) -> None:
        """Rollback a transaction"""
        self._db.rollback(txn_id)

    def txn_get(self, txn_id: int, key: int) -> Optional[bytes]:
        """Get within a transaction"""
        return self._db.txn_get(txn_id, key)

    def txn_put(self, txn_id: int, key: int, value: Union[bytes, str]) -> None:
        """Put within a transaction"""
        if isinstance(value, str):
            value = value.encode('utf-8')
        self._db.txn_put(txn_id, key, value)

    def __repr__(self) -> str:
        return f"<OmenDB path='{self.path}' index='{self.index_type}'>"


# Convenience functions
def open(path: str, index_type: str = "linear") -> DB:
    """Open an OmenDB database"""
    return DB(path, index_type)


__all__ = [
    "DB",
    "OmenDB",
    "open",
    "__version__",
    "INDEX_NONE",
    "INDEX_LINEAR",
    "INDEX_RMI",
    "ISO_READ_UNCOMMITTED",
    "ISO_READ_COMMITTED",
    "ISO_REPEATABLE_READ",
    "ISO_SERIALIZABLE",
]