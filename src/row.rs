//! Generic row abstraction for OmenDB
//! Represents a single row of data with any schema

use crate::value::Value;
use anyhow::{anyhow, Result};
use arrow::array::*;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use std::collections::HashMap;
use std::sync::Arc;

/// Generic row that can hold any schema
#[derive(Debug, Clone)]
pub struct Row {
    /// Values in schema order
    values: Vec<Value>,
}

impl Row {
    /// Create new row from values
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Create row from column name -> value map
    pub fn from_map(schema: &SchemaRef, map: HashMap<String, Value>) -> Result<Self> {
        let mut values = Vec::with_capacity(schema.fields().len());

        for field in schema.fields() {
            let value = map
                .get(field.name())
                .ok_or_else(|| anyhow!("Missing value for column {}", field.name()))?;

            if !value.matches_type(field.data_type()) {
                return Err(anyhow!(
                    "Type mismatch for column {}: expected {:?}, got {:?}",
                    field.name(),
                    field.data_type(),
                    value.arrow_type()
                ));
            }

            values.push(value.clone());
        }

        Ok(Self { values })
    }

    /// Get value by column index
    pub fn get(&self, index: usize) -> Result<&Value> {
        self.values
            .get(index)
            .ok_or_else(|| anyhow!("Column index {} out of bounds", index))
    }

    /// Get value by column name (requires schema context)
    pub fn get_by_name(&self, schema: &SchemaRef, column_name: &str) -> Result<&Value> {
        let index = schema.index_of(column_name)?;
        self.get(index)
    }

    /// Get all values
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Number of columns
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if row is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Validate row matches schema
    pub fn validate(&self, schema: &SchemaRef) -> Result<()> {
        if self.values.len() != schema.fields().len() {
            return Err(anyhow!(
                "Row has {} values but schema has {} fields",
                self.values.len(),
                schema.fields().len()
            ));
        }

        for (i, value) in self.values.iter().enumerate() {
            let field = schema.field(i);
            if !value.matches_type(field.data_type()) {
                return Err(anyhow!(
                    "Type mismatch for column {}: expected {:?}, got value {:?}",
                    field.name(),
                    field.data_type(),
                    value
                ));
            }
        }

        Ok(())
    }

    /// Convert row to Arrow RecordBatch (single row batch)
    pub fn to_record_batch(&self, schema: &SchemaRef) -> Result<RecordBatch> {
        self.validate(schema)?;

        let mut arrays: Vec<ArrayRef> = Vec::new();

        for (i, field) in schema.fields().iter().enumerate() {
            let value = &self.values[i];
            let array = value_to_array(value, field.data_type())?;
            arrays.push(array);
        }

        RecordBatch::try_new(schema.clone(), arrays)
            .map_err(|e| anyhow!("Failed to create RecordBatch: {}", e))
    }

    /// Extract row from RecordBatch at given index
    pub fn from_batch(batch: &RecordBatch, row_index: usize) -> Result<Self> {
        if row_index >= batch.num_rows() {
            return Err(anyhow!(
                "Row index {} out of bounds (batch has {} rows)",
                row_index,
                batch.num_rows()
            ));
        }

        let mut values = Vec::with_capacity(batch.num_columns());

        for col_idx in 0..batch.num_columns() {
            let array = batch.column(col_idx);
            let value = Value::from_array(array.as_ref(), row_index)?;
            values.push(value);
        }

        Ok(Self { values })
    }

    /// Convert multiple rows to RecordBatch
    pub fn rows_to_batch(rows: &[Row], schema: &SchemaRef) -> Result<RecordBatch> {
        if rows.is_empty() {
            // Create empty arrays for each column
            let empty_arrays: Vec<ArrayRef> = schema
                .fields()
                .iter()
                .map(|field| {
                    let array = create_array_builder(field.data_type(), 0)
                        .expect("Failed to create builder")
                        .finish();
                    array as ArrayRef
                })
                .collect();

            return RecordBatch::try_new(schema.clone(), empty_arrays)
                .map_err(|e| anyhow!("Failed to create empty batch: {}", e));
        }

        // Validate all rows
        for row in rows {
            row.validate(schema)?;
        }

        let num_rows = rows.len();
        let mut builders: Vec<Box<dyn ArrayBuilder>> = Vec::new();

        // Create builders for each column
        for field in schema.fields() {
            let builder = create_array_builder(field.data_type(), num_rows)?;
            builders.push(builder);
        }

        // Append each row to builders
        for row in rows {
            for (col_idx, value) in row.values.iter().enumerate() {
                append_value_to_builder(&mut builders[col_idx], value)?;
            }
        }

        // Finish builders and create arrays
        let arrays: Vec<ArrayRef> = builders
            .iter_mut()
            .map(|builder| builder.finish())
            .collect();

        RecordBatch::try_new(schema.clone(), arrays)
            .map_err(|e| anyhow!("Failed to create RecordBatch: {}", e))
    }

    /// Convert single row to RecordBatch
    pub fn to_batch(&self, schema: &SchemaRef) -> Result<RecordBatch> {
        Self::rows_to_batch(&[self.clone()], schema)
    }
}

/// Convert Value to single-element Arrow array
fn value_to_array(value: &Value, data_type: &arrow::datatypes::DataType) -> Result<ArrayRef> {
    use arrow::datatypes::DataType;

    match (value, data_type) {
        (Value::Int64(v), DataType::Int64) => Ok(Arc::new(Int64Array::from(vec![*v]))),
        (Value::UInt64(v), DataType::UInt64) => Ok(Arc::new(UInt64Array::from(vec![*v]))),
        (Value::Float64(v), DataType::Float64) => Ok(Arc::new(Float64Array::from(vec![*v]))),
        (Value::Text(v), DataType::Utf8) => Ok(Arc::new(StringArray::from(vec![v.as_str()]))),
        (Value::Timestamp(v), DataType::Timestamp(_, _)) => {
            Ok(Arc::new(TimestampMicrosecondArray::from(vec![*v])))
        }
        (Value::Boolean(v), DataType::Boolean) => Ok(Arc::new(BooleanArray::from(vec![*v]))),
        (Value::Null, _) => {
            // Create single-element null array of appropriate type
            match data_type {
                DataType::Int64 => Ok(Arc::new(Int64Array::from(vec![None as Option<i64>]))),
                DataType::UInt64 => Ok(Arc::new(UInt64Array::from(vec![None as Option<u64>]))),
                DataType::Float64 => Ok(Arc::new(Float64Array::from(vec![None as Option<f64>]))),
                DataType::Utf8 => Ok(Arc::new(StringArray::from(vec![None as Option<&str>]))),
                DataType::Timestamp(_, _) => Ok(Arc::new(TimestampMicrosecondArray::from(vec![
                    None as Option<i64>,
                ]))),
                DataType::Boolean => Ok(Arc::new(BooleanArray::from(vec![None as Option<bool>]))),
                _ => Err(anyhow!("Unsupported data type for NULL: {:?}", data_type)),
            }
        }
        _ => Err(anyhow!(
            "Type mismatch: value {:?} doesn't match type {:?}",
            value,
            data_type
        )),
    }
}

/// Create appropriate ArrayBuilder for data type
fn create_array_builder(
    data_type: &arrow::datatypes::DataType,
    capacity: usize,
) -> Result<Box<dyn ArrayBuilder>> {
    use arrow::datatypes::DataType;

    match data_type {
        DataType::Int64 => Ok(Box::new(Int64Builder::with_capacity(capacity))),
        DataType::UInt64 => Ok(Box::new(UInt64Builder::with_capacity(capacity))),
        DataType::Float64 => Ok(Box::new(Float64Builder::with_capacity(capacity))),
        DataType::Utf8 => Ok(Box::new(StringBuilder::with_capacity(capacity, 1024))),
        DataType::Timestamp(_, _) => Ok(Box::new(TimestampMicrosecondBuilder::with_capacity(
            capacity,
        ))),
        DataType::Boolean => Ok(Box::new(BooleanBuilder::with_capacity(capacity))),
        _ => Err(anyhow!(
            "Unsupported data type for builder: {:?}",
            data_type
        )),
    }
}

/// Append value to array builder
fn append_value_to_builder(builder: &mut Box<dyn ArrayBuilder>, value: &Value) -> Result<()> {
    match value {
        Value::Int64(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<Int64Builder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(*v);
        }
        Value::UInt64(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<UInt64Builder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(*v);
        }
        Value::Float64(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<Float64Builder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(*v);
        }
        Value::Text(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<StringBuilder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(v);
        }
        Value::Timestamp(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<TimestampMicrosecondBuilder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(*v);
        }
        Value::Boolean(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<BooleanBuilder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            b.append_value(*v);
        }
        Value::Vector(v) => {
            let b = builder
                .as_any_mut()
                .downcast_mut::<BinaryBuilder>()
                .ok_or_else(|| anyhow!("Builder type mismatch"))?;
            let bytes = v.to_postgres_binary();
            b.append_value(&bytes);
        }
        Value::Null => {
            // Determine builder type and append null appropriately
            if let Some(b) = builder.as_any_mut().downcast_mut::<Int64Builder>() {
                b.append_null();
            } else if let Some(b) = builder.as_any_mut().downcast_mut::<UInt64Builder>() {
                b.append_null();
            } else if let Some(b) = builder.as_any_mut().downcast_mut::<Float64Builder>() {
                b.append_null();
            } else if let Some(b) = builder.as_any_mut().downcast_mut::<StringBuilder>() {
                b.append_null();
            } else if let Some(b) = builder
                .as_any_mut()
                .downcast_mut::<TimestampMicrosecondBuilder>()
            {
                b.append_null();
            } else if let Some(b) = builder.as_any_mut().downcast_mut::<BooleanBuilder>() {
                b.append_null();
            } else if let Some(b) = builder.as_any_mut().downcast_mut::<BinaryBuilder>() {
                b.append_null();
            } else {
                return Err(anyhow!("Unsupported builder type for NULL"));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    #[test]
    fn test_row_creation() {
        let row = Row::new(vec![Value::Int64(42), Value::Text("hello".into())]);

        assert_eq!(row.len(), 2);
        assert_eq!(row.get(0).unwrap(), &Value::Int64(42));
        assert_eq!(row.get(1).unwrap(), &Value::Text("hello".into()));
    }

    #[test]
    fn test_row_validation() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let valid_row = Row::new(vec![Value::Int64(1), Value::Text("Alice".into())]);
        assert!(valid_row.validate(&schema).is_ok());

        let invalid_row = Row::new(vec![Value::Text("wrong type".into()), Value::Int64(123)]);
        assert!(invalid_row.validate(&schema).is_err());
    }

    #[test]
    fn test_row_to_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let row = Row::new(vec![Value::Int64(42), Value::Float64(3.14)]);

        let batch = row.to_record_batch(&schema).unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_row_from_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        )
        .unwrap();

        let row = Row::from_batch(&batch, 1).unwrap();
        assert_eq!(row.get(0).unwrap(), &Value::Int64(2));
        assert_eq!(row.get(1).unwrap(), &Value::Text("b".into()));
    }

    #[test]
    fn test_rows_to_batch() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let rows = vec![
            Row::new(vec![Value::Int64(1), Value::Float64(1.1)]),
            Row::new(vec![Value::Int64(2), Value::Float64(2.2)]),
            Row::new(vec![Value::Int64(3), Value::Float64(3.3)]),
        ];

        let batch = Row::rows_to_batch(&rows, &schema).unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 2);
    }
}
