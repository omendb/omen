//! Python bindings for OmenDB using PyO3

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::{IndexType, IsolationLevel, OmenDB as RustDB};

/// Python wrapper for OmenDB
#[pyclass(name = "OmenDB")]
pub struct PyOmenDB {
    db: Arc<Mutex<RustDB>>,
}

#[pymethods]
impl PyOmenDB {
    /// Create a new OmenDB instance
    #[new]
    #[pyo3(signature = (path, index_type=None))]
    fn new(path: &str, index_type: Option<&str>) -> PyResult<Self> {
        let index_type = match index_type {
            Some("linear") => IndexType::Linear,
            Some("rmi") => IndexType::RMI,
            Some("none") | None => IndexType::None,
            _ => {
                return Err(PyValueError::new_err(
                    "Invalid index type. Use 'linear', 'rmi', or 'none'",
                ))
            }
        };

        let db = RustDB::open_with_index(path, index_type)
            .map_err(|e| PyValueError::new_err(format!("Failed to open database: {}", e)))?;

        Ok(PyOmenDB {
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// Insert a key-value pair
    fn put(&self, key: i64, value: &[u8]) -> PyResult<()> {
        self.db
            .lock()
            .unwrap()
            .put(key, value)
            .map_err(|e| PyValueError::new_err(format!("Put failed: {}", e)))
    }

    /// Get a value by key
    fn get(&self, key: i64) -> PyResult<Option<Vec<u8>>> {
        self.db
            .lock()
            .unwrap()
            .get(key)
            .map_err(|e| PyValueError::new_err(format!("Get failed: {}", e)))
    }

    /// Delete a key
    fn delete(&self, key: i64) -> PyResult<()> {
        self.db
            .lock()
            .unwrap()
            .delete(key)
            .map_err(|e| PyValueError::new_err(format!("Delete failed: {}", e)))
    }

    /// Range query
    fn range(&self, start: i64, end: i64) -> PyResult<Vec<(i64, Vec<u8>)>> {
        self.db
            .lock()
            .unwrap()
            .range(start, end)
            .map_err(|e| PyValueError::new_err(format!("Range query failed: {}", e)))
    }

    /// Bulk insert data
    fn bulk_insert(&self, py: Python, data: Vec<(i64, Vec<u8>)>) -> PyResult<()> {
        py.allow_threads(|| {
            self.db
                .lock()
                .unwrap()
                .bulk_insert(data)
                .map_err(|e| PyValueError::new_err(format!("Bulk insert failed: {}", e)))
        })
    }

    /// Begin a transaction
    #[pyo3(signature = (isolation_level=None))]
    fn begin_transaction(&self, isolation_level: Option<&str>) -> PyResult<u64> {
        let isolation = match isolation_level {
            Some("read_uncommitted") => IsolationLevel::ReadUncommitted,
            Some("read_committed") | None => IsolationLevel::ReadCommitted,
            Some("repeatable_read") => IsolationLevel::RepeatableRead,
            Some("serializable") => IsolationLevel::Serializable,
            _ => return Err(PyValueError::new_err("Invalid isolation level")),
        };

        Ok(self.db.lock().unwrap().begin_transaction(isolation))
    }

    /// Commit a transaction
    fn commit(&self, txn_id: u64) -> PyResult<()> {
        self.db
            .lock()
            .unwrap()
            .commit(txn_id)
            .map_err(|e| PyValueError::new_err(format!("Commit failed: {}", e)))
    }

    /// Rollback a transaction
    fn rollback(&self, txn_id: u64) -> PyResult<()> {
        self.db
            .lock()
            .unwrap()
            .rollback(txn_id)
            .map_err(|e| PyValueError::new_err(format!("Rollback failed: {}", e)))
    }

    /// Get within a transaction
    fn txn_get(&self, txn_id: u64, key: i64) -> PyResult<Option<Vec<u8>>> {
        self.db
            .lock()
            .unwrap()
            .txn_get(txn_id, key)
            .map_err(|e| PyValueError::new_err(format!("Transaction get failed: {}", e)))
    }

    /// Put within a transaction
    fn txn_put(&self, txn_id: u64, key: i64, value: Vec<u8>) -> PyResult<()> {
        self.db
            .lock()
            .unwrap()
            .txn_put(txn_id, key, value)
            .map_err(|e| PyValueError::new_err(format!("Transaction put failed: {}", e)))
    }

    /// Run a benchmark
    fn benchmark(&self, py: Python, num_queries: usize) -> PyResult<String> {
        py.allow_threads(|| {
            self.db
                .lock()
                .unwrap()
                .benchmark(num_queries)
                .map_err(|e| PyValueError::new_err(format!("Benchmark failed: {}", e)))
        })
    }

    /// Get database statistics
    fn stats(&self) -> PyResult<String> {
        Ok(self.db.lock().unwrap().stats())
    }
}

/// Python module initialization
#[pymodule]
fn _omendb(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyOmenDB>()?;

    // Add version info
    m.add("__version__", "0.1.0")?;

    // Add index type constants
    m.add("INDEX_NONE", "none")?;
    m.add("INDEX_LINEAR", "linear")?;
    m.add("INDEX_RMI", "rmi")?;

    // Add isolation level constants
    m.add("ISO_READ_UNCOMMITTED", "read_uncommitted")?;
    m.add("ISO_READ_COMMITTED", "read_committed")?;
    m.add("ISO_REPEATABLE_READ", "repeatable_read")?;
    m.add("ISO_SERIALIZABLE", "serializable")?;

    Ok(())
}
