//! Vector index management for HNSW+BQ indexes
//!
//! Implements PostgreSQL-compatible CREATE INDEX syntax for vector columns:
//! ```sql
//! CREATE INDEX idx_name ON table_name
//! USING hnsw_bq (column_name vector_l2_ops)
//! WITH (m = 48, ef_construction = 200, expansion = 150);
//! ```

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector index type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorIndexType {
    /// HNSW with Binary Quantization
    HnswBq,
}

impl VectorIndexType {
    /// Parse index type from SQL string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hnsw_bq" => Some(VectorIndexType::HnswBq),
            _ => None,
        }
    }

    /// Get SQL string for index type
    pub fn to_str(&self) -> &'static str {
        match self {
            VectorIndexType::HnswBq => "hnsw_bq",
        }
    }
}

/// Vector distance operator class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatorClass {
    /// L2 distance (Euclidean)
    VectorL2Ops,

    /// Cosine distance
    VectorCosineOps,

    /// Inner product (dot product)
    VectorIpOps,
}

impl OperatorClass {
    /// Parse operator class from SQL string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "vector_l2_ops" => Some(OperatorClass::VectorL2Ops),
            "vector_cosine_ops" => Some(OperatorClass::VectorCosineOps),
            "vector_ip_ops" => Some(OperatorClass::VectorIpOps),
            _ => None,
        }
    }

    /// Get SQL string for operator class
    pub fn to_str(&self) -> &'static str {
        match self {
            OperatorClass::VectorL2Ops => "vector_l2_ops",
            OperatorClass::VectorCosineOps => "vector_cosine_ops",
            OperatorClass::VectorIpOps => "vector_ip_ops",
        }
    }
}

/// HNSW+BQ index parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexParameters {
    /// Max connections per node (default: 48)
    pub m: usize,

    /// Build-time search depth (default: 200)
    pub ef_construction: usize,

    /// Query-time candidate expansion factor (default: 150)
    /// - 150x → 92.7% recall @ 5.6ms
    /// - 200x → 95.1% recall @ 6.9ms
    pub expansion: usize,
}

impl Default for IndexParameters {
    fn default() -> Self {
        Self {
            m: 48,
            ef_construction: 200,
            expansion: 150, // Production-ready default (92.7% recall)
        }
    }
}

impl IndexParameters {
    /// Parse parameters from SQL WITH clause
    ///
    /// # Arguments
    /// * `options` - Map of option name → value
    ///
    /// # Example
    /// ```
    /// let mut options = HashMap::new();
    /// options.insert("m".to_string(), "48".to_string());
    /// options.insert("ef_construction".to_string(), "200".to_string());
    /// let params = IndexParameters::from_options(&options)?;
    /// ```
    pub fn from_options(options: &HashMap<String, String>) -> Result<Self> {
        let mut params = Self::default();

        if let Some(m_str) = options.get("m") {
            params.m = m_str
                .parse()
                .map_err(|_| anyhow!("Invalid value for 'm': {}", m_str))?;
        }

        if let Some(ef_str) = options.get("ef_construction") {
            params.ef_construction = ef_str
                .parse()
                .map_err(|_| anyhow!("Invalid value for 'ef_construction': {}", ef_str))?;
        }

        if let Some(exp_str) = options.get("expansion") {
            params.expansion = exp_str
                .parse()
                .map_err(|_| anyhow!("Invalid value for 'expansion': {}", exp_str))?;
        }

        // Validate parameters
        if params.m == 0 {
            return Err(anyhow!("Parameter 'm' must be greater than 0"));
        }
        if params.ef_construction == 0 {
            return Err(anyhow!("Parameter 'ef_construction' must be greater than 0"));
        }
        if params.expansion == 0 {
            return Err(anyhow!("Parameter 'expansion' must be greater than 0"));
        }

        Ok(params)
    }
}

/// Vector index metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorIndexMetadata {
    /// Index name
    pub index_name: String,

    /// Table name
    pub table_name: String,

    /// Column name
    pub column_name: String,

    /// Index type (currently only HNSW_BQ)
    pub index_type: VectorIndexType,

    /// Operator class (L2, cosine, or inner product)
    pub operator_class: OperatorClass,

    /// Index parameters
    pub parameters: IndexParameters,
}

impl VectorIndexMetadata {
    /// Create new vector index metadata
    pub fn new(
        index_name: String,
        table_name: String,
        column_name: String,
        index_type: VectorIndexType,
        operator_class: OperatorClass,
        parameters: IndexParameters,
    ) -> Self {
        Self {
            index_name,
            table_name,
            column_name,
            index_type,
            operator_class,
            parameters,
        }
    }

    /// Validate index metadata
    pub fn validate(&self) -> Result<()> {
        if self.index_name.is_empty() {
            return Err(anyhow!("Index name cannot be empty"));
        }
        if self.table_name.is_empty() {
            return Err(anyhow!("Table name cannot be empty"));
        }
        if self.column_name.is_empty() {
            return Err(anyhow!("Column name cannot be empty"));
        }
        Ok(())
    }

    /// Get SQL representation of this index
    pub fn to_sql(&self) -> String {
        format!(
            "CREATE INDEX {} ON {} USING {} ({} {}) WITH (m = {}, ef_construction = {}, expansion = {})",
            self.index_name,
            self.table_name,
            self.index_type.to_str(),
            self.column_name,
            self.operator_class.to_str(),
            self.parameters.m,
            self.parameters.ef_construction,
            self.parameters.expansion
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_type_from_str() {
        assert_eq!(
            VectorIndexType::from_str("hnsw_bq"),
            Some(VectorIndexType::HnswBq)
        );
        assert_eq!(
            VectorIndexType::from_str("HNSW_BQ"),
            Some(VectorIndexType::HnswBq)
        );
        assert_eq!(VectorIndexType::from_str("invalid"), None);
    }

    #[test]
    fn test_operator_class_from_str() {
        assert_eq!(
            OperatorClass::from_str("vector_l2_ops"),
            Some(OperatorClass::VectorL2Ops)
        );
        assert_eq!(
            OperatorClass::from_str("vector_cosine_ops"),
            Some(OperatorClass::VectorCosineOps)
        );
        assert_eq!(
            OperatorClass::from_str("vector_ip_ops"),
            Some(OperatorClass::VectorIpOps)
        );
        assert_eq!(OperatorClass::from_str("invalid"), None);
    }

    #[test]
    fn test_index_parameters_default() {
        let params = IndexParameters::default();
        assert_eq!(params.m, 48);
        assert_eq!(params.ef_construction, 200);
        assert_eq!(params.expansion, 150);
    }

    #[test]
    fn test_index_parameters_from_options() {
        let mut options = HashMap::new();
        options.insert("m".to_string(), "64".to_string());
        options.insert("ef_construction".to_string(), "300".to_string());
        options.insert("expansion".to_string(), "200".to_string());

        let params = IndexParameters::from_options(&options).unwrap();
        assert_eq!(params.m, 64);
        assert_eq!(params.ef_construction, 300);
        assert_eq!(params.expansion, 200);
    }

    #[test]
    fn test_index_parameters_partial_options() {
        let mut options = HashMap::new();
        options.insert("m".to_string(), "32".to_string());

        let params = IndexParameters::from_options(&options).unwrap();
        assert_eq!(params.m, 32);
        assert_eq!(params.ef_construction, 200); // default
        assert_eq!(params.expansion, 150); // default
    }

    #[test]
    fn test_index_parameters_validation() {
        let mut options = HashMap::new();
        options.insert("m".to_string(), "0".to_string());

        let result = IndexParameters::from_options(&options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be greater"));
    }

    #[test]
    fn test_index_parameters_invalid_value() {
        let mut options = HashMap::new();
        options.insert("m".to_string(), "not_a_number".to_string());

        let result = IndexParameters::from_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_vector_index_metadata() {
        let metadata = VectorIndexMetadata::new(
            "idx_embeddings".to_string(),
            "documents".to_string(),
            "embedding".to_string(),
            VectorIndexType::HnswBq,
            OperatorClass::VectorL2Ops,
            IndexParameters::default(),
        );

        assert_eq!(metadata.index_name, "idx_embeddings");
        assert_eq!(metadata.table_name, "documents");
        assert_eq!(metadata.column_name, "embedding");
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn test_vector_index_metadata_validate() {
        let metadata = VectorIndexMetadata::new(
            "".to_string(), // empty name
            "documents".to_string(),
            "embedding".to_string(),
            VectorIndexType::HnswBq,
            OperatorClass::VectorL2Ops,
            IndexParameters::default(),
        );

        assert!(metadata.validate().is_err());
    }

    #[test]
    fn test_vector_index_metadata_to_sql() {
        let metadata = VectorIndexMetadata::new(
            "idx_embeddings".to_string(),
            "documents".to_string(),
            "embedding".to_string(),
            VectorIndexType::HnswBq,
            OperatorClass::VectorL2Ops,
            IndexParameters::default(),
        );

        let sql = metadata.to_sql();
        assert!(sql.contains("CREATE INDEX idx_embeddings"));
        assert!(sql.contains("ON documents"));
        assert!(sql.contains("USING hnsw_bq"));
        assert!(sql.contains("embedding vector_l2_ops"));
        assert!(sql.contains("m = 48"));
        assert!(sql.contains("ef_construction = 200"));
        assert!(sql.contains("expansion = 150"));
    }
}
