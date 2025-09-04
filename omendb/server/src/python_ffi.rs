//! Python FFI bridge for communicating with Mojo vector engine
//! 
//! Uses PyO3 to call the existing Python exports from the Mojo native module.
//! This approach is simpler and more reliable than C FFI.

use crate::types::Vector;
use crate::{Error, Result};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use tokio::task;
use tracing::{debug, info, instrument};

/// Internal search result for Python FFI (simplified metadata)
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub distance: f32,
    pub metadata: HashMap<String, String>,
}

/// Python-based Mojo engine that communicates via PyO3
#[derive(Debug)]
pub struct PythonMojoEngine {
    /// Vector dimension
    dimension: i32,
    /// Whether engine is initialized
    initialized: bool,
}

impl PythonMojoEngine {
    /// Check if engine is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    /// Create a new engine instance
    #[instrument(level = "debug")]
    pub fn new(dimension: i32) -> Result<Self> {
        info!("Creating Python-based Mojo engine with dimension {}", dimension);
        
        // Initialize Python environment
        task::block_in_place(|| {
            Python::with_gil(|py| {
                // Try to import the native module to ensure it's available
                py.import("omendb.native")
                    .map_err(|e| Error::Python(format!("Failed to import omendb.native: {}", e)))?;
                
                Ok::<(), Error>(())
            })
        })?;
        
        Ok(PythonMojoEngine {
            dimension,
            initialized: true,
        })
    }
    
    /// Initialize the VectorStore
    #[instrument(level = "debug", skip(self))]
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        let dimension = self.dimension;
        
        task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Call initialize function with dimension
                let result = omendb.call_method1("initialize", (dimension,))?;
                let success: bool = result.extract()?;
                
                if !success {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "Failed to initialize VectorStore"
                    ));
                }
                
                Ok::<(), PyErr>(())
            })
        })
        .await??;
        
        self.initialized = true;
        debug!("Engine initialized successfully");
        Ok(())
    }
    
    /// Add a vector to the store
    #[instrument(level = "debug", skip(self, vector))]
    pub async fn add_vector(&self, id: &str, vector: &[f32]) -> Result<()> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let id_str = id.to_string();
        let id_clone = id_str.clone();
        let vector = vector.to_vec();
        
        task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Convert vector to Python list
                let py_vector = PyList::new(py, &vector)?;
                
                // Create empty metadata dict
                let py_metadata = PyDict::new(py);
                
                // Call add_vector function
                let result = omendb.call_method1("add_vector", (id_str, py_vector, py_metadata))?;
                
                // Check if successful
                let success: bool = result.extract()?;
                if !success {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to add vector"));
                }
                
                Ok::<(), PyErr>(())
            })
        })
        .await??;
        
        debug!("Vector {} added successfully", id_clone);
        Ok(())
    }
    
    /// Search for similar vectors
    #[instrument(level = "debug", skip(self, query_vector))]
    pub async fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let query_vector = query_vector.to_vec();
        let k = k as i32;
        
        let results = task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Convert query vector to Python list
                let py_query = PyList::new(py, &query_vector)?;
                
                // Call search_vectors function
                let result = omendb.call_method1("search_vectors", (py_query, k))?;
                
                // Extract results - should be a list of tuples/lists
                let py_results = result.downcast::<PyList>()?;
                let mut search_results = Vec::new();
                
                for item in py_results.iter() {
                    // Each result should be a tuple/list: (id, distance, metadata)
                    let result_tuple = item.downcast::<PyList>()?;
                    
                    if result_tuple.len() >= 2 {
                        let id: String = result_tuple.get_item(0)?.extract()?;
                        let distance: f32 = result_tuple.get_item(1)?.extract()?;
                        
                        // Try to extract metadata if available
                        let metadata = if result_tuple.len() > 2 {
                            match result_tuple.get_item(2)?.extract::<HashMap<String, String>>() {
                                Ok(meta) => meta,
                                Err(_) => HashMap::new(),
                            }
                        } else {
                            HashMap::new()
                        };
                        
                        search_results.push(SearchResult {
                            id,
                            distance,
                            metadata,
                        });
                    }
                }
                
                Ok::<Vec<SearchResult>, PyErr>(search_results)
            })
        })
        .await??;
        
        debug!("Found {} search results", results.len());
        Ok(results)
    }
    
    /// Add vector with full metadata
    #[instrument(level = "debug", skip(self, vector))]
    pub async fn add_vector_with_metadata(&self, vector: &Vector) -> Result<()> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let id = vector.id.clone();
        let id_clone = id.clone();
        let vector_data = vector.data.clone();
        let metadata = vector.metadata.clone();
        
        task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Convert vector to Python list
                let py_vector = PyList::new(py, &vector_data)?;
                
                // Convert metadata to Python dict
                let py_metadata = PyDict::new(py);
                for (key, value) in &metadata {
                    py_metadata.set_item(key, value.to_string())?;
                }
                
                // Call add_vector function
                let result = omendb.call_method1("add_vector", (id, py_vector, py_metadata))?;
                
                // Check if successful
                let success: bool = result.extract()?;
                if !success {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to add vector"));
                }
                
                Ok::<(), PyErr>(())
            })
        })
        .await??;
        
        debug!("Vector {} added with metadata", id_clone);
        Ok(())
    }
    
    /// Get vector by ID
    #[instrument(level = "debug", skip(self))]
    pub async fn get_vector(&self, id: &str) -> Result<Option<Vector>> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let id = id.to_string();
        
        let result = task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Call get_vector function
                let result = omendb.call_method1("get_vector", (id.clone(),))?;
                
                // Check if None was returned
                if result.is_none() {
                    return Ok::<Option<Vector>, PyErr>(None);
                }
                
                // Extract the vector data
                let py_results = result.downcast::<PyList>()?;
                
                if py_results.len() >= 2 {
                    let vector_data: Vec<f32> = py_results.get_item(0)?.extract()?;
                    
                    // Try to extract metadata
                    let metadata = if py_results.len() > 1 {
                        let metadata_item = py_results.get_item(1)?;
                        let py_dict = metadata_item.downcast::<PyDict>()?;
                        let mut meta_map = HashMap::new();
                        
                        for (key, value) in py_dict.iter() {
                            if let (Ok(k), Ok(v)) = (key.extract::<String>(), value.extract::<String>()) {
                                meta_map.insert(k, serde_json::Value::String(v));
                            }
                        }
                        meta_map
                    } else {
                        HashMap::new()
                    };
                    
                    Ok(Some(Vector {
                        id,
                        data: vector_data,
                        metadata,
                    }))
                } else {
                    Ok(None)
                }
            })
        })
        .await??;
        
        Ok(result)
    }
    
    /// Delete vector by ID
    #[instrument(level = "debug", skip(self))]
    pub async fn delete_vector(&self, id: &str) -> Result<bool> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let id = id.to_string();
        
        let result = task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Call delete_vector function
                let result = omendb.call_method1("delete_vector", (id,))?;
                let success: bool = result.extract()?;
                
                Ok::<bool, PyErr>(success)
            })
        })
        .await??;
        
        Ok(result)
    }
    
    /// Get engine statistics
    #[instrument(level = "debug", skip(self))]
    pub async fn get_stats(&self) -> Result<HashMap<String, serde_json::Value>> {
        if !self.initialized {
            return Err(Error::EngineNotInitialized);
        }
        
        let stats = task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let omendb = py.import("omendb.native")?;
                
                // Call get_stats function
                let result = omendb.call_method0("get_stats")?;
                let py_dict = result.downcast::<PyDict>()?;
                
                let mut stats_map = HashMap::new();
                
                for (key, value) in py_dict.iter() {
                    let key_str: String = key.extract()?;
                    
                    // Try different value types
                    if let Ok(int_val) = value.extract::<i64>() {
                        stats_map.insert(key_str, serde_json::Value::Number(int_val.into()));
                    } else if let Ok(float_val) = value.extract::<f64>() {
                        stats_map.insert(key_str, serde_json::json!(float_val));
                    } else if let Ok(str_val) = value.extract::<String>() {
                        stats_map.insert(key_str, serde_json::Value::String(str_val));
                    } else if let Ok(bool_val) = value.extract::<bool>() {
                        stats_map.insert(key_str, serde_json::Value::Bool(bool_val));
                    }
                }
                
                Ok::<HashMap<String, serde_json::Value>, PyErr>(stats_map)
            })
        })
        .await??;
        
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_creation() {
        // This test requires the Mojo Python module to be available
        // let engine = PythonMojoEngine::new(128);
        // assert!(engine.is_ok());
    }
}