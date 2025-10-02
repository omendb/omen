//! Multi-Version Concurrency Control (MVCC) support
//! Adds version tracking to enable UPDATE/DELETE with transaction isolation

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;

/// MVCC metadata column names (hidden from users)
pub const MVCC_VERSION_COL: &str = "__mvcc_version";
pub const MVCC_TXN_ID_COL: &str = "__mvcc_txn_id";
pub const MVCC_DELETED_COL: &str = "__mvcc_deleted";

/// Adds MVCC metadata columns to a user schema
/// Returns new schema with hidden MVCC columns appended
pub fn add_mvcc_columns(user_schema: SchemaRef) -> SchemaRef {
    let mut fields: Vec<Field> = user_schema
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();

    // Add MVCC metadata columns
    fields.push(Field::new(MVCC_VERSION_COL, DataType::UInt64, false));
    fields.push(Field::new(MVCC_TXN_ID_COL, DataType::UInt64, false));
    fields.push(Field::new(MVCC_DELETED_COL, DataType::Boolean, false));

    Arc::new(Schema::new(fields))
}

/// Extract user schema from MVCC-enhanced schema (removes hidden columns)
pub fn extract_user_schema(mvcc_schema: SchemaRef) -> SchemaRef {
    let fields: Vec<Field> = mvcc_schema
        .fields()
        .iter()
        .filter(|f| {
            let name = f.name();
            name != MVCC_VERSION_COL && name != MVCC_TXN_ID_COL && name != MVCC_DELETED_COL
        })
        .map(|f| f.as_ref().clone())
        .collect();

    Arc::new(Schema::new(fields))
}

/// Check if schema has MVCC columns
pub fn has_mvcc_columns(schema: &SchemaRef) -> bool {
    schema.index_of(MVCC_VERSION_COL).is_ok()
        && schema.index_of(MVCC_TXN_ID_COL).is_ok()
        && schema.index_of(MVCC_DELETED_COL).is_ok()
}

/// Get MVCC column indices in schema
#[derive(Debug, Clone)]
pub struct MvccIndices {
    pub version: usize,
    pub txn_id: usize,
    pub deleted: usize,
}

impl MvccIndices {
    pub fn from_schema(schema: &SchemaRef) -> Option<Self> {
        let version = schema.index_of(MVCC_VERSION_COL).ok()?;
        let txn_id = schema.index_of(MVCC_TXN_ID_COL).ok()?;
        let deleted = schema.index_of(MVCC_DELETED_COL).ok()?;

        Some(Self {
            version,
            txn_id,
            deleted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_mvcc_columns() {
        let user_schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let mvcc_schema = add_mvcc_columns(user_schema.clone());

        assert_eq!(mvcc_schema.fields().len(), 5); // 2 user + 3 MVCC
        assert!(has_mvcc_columns(&mvcc_schema));

        // Verify MVCC columns exist
        assert!(mvcc_schema.index_of(MVCC_VERSION_COL).is_ok());
        assert!(mvcc_schema.index_of(MVCC_TXN_ID_COL).is_ok());
        assert!(mvcc_schema.index_of(MVCC_DELETED_COL).is_ok());
    }

    #[test]
    fn test_extract_user_schema() {
        let user_schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let mvcc_schema = add_mvcc_columns(user_schema.clone());
        let extracted = extract_user_schema(mvcc_schema);

        assert_eq!(extracted.fields().len(), 2);
        assert_eq!(extracted.field(0).name(), "id");
        assert_eq!(extracted.field(1).name(), "value");
    }

    #[test]
    fn test_mvcc_indices() {
        let user_schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));

        let mvcc_schema = add_mvcc_columns(user_schema);
        let indices = MvccIndices::from_schema(&mvcc_schema).unwrap();

        assert_eq!(indices.version, 1);
        assert_eq!(indices.txn_id, 2);
        assert_eq!(indices.deleted, 3);
    }
}
