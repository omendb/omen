//! Generic value type system for OmenDB
//! Supports any SQL data type with proper ordering semantics

use anyhow::{anyhow, Result};
use arrow::array::*;
use arrow::datatypes::DataType;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Generic value type that can represent any SQL value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// 64-bit signed integer
    Int64(i64),

    /// 64-bit unsigned integer (for MVCC version tracking)
    UInt64(u64),

    /// 64-bit floating point
    Float64(f64),

    /// Text/string value
    Text(String),

    /// Timestamp (microseconds since epoch)
    Timestamp(i64),

    /// Boolean value
    Boolean(bool),

    /// NULL value
    Null,
}

impl Value {
    /// Check if value matches the expected Arrow data type
    pub fn matches_type(&self, data_type: &DataType) -> bool {
        match (self, data_type) {
            (Value::Int64(_), DataType::Int64) => true,
            (Value::UInt64(_), DataType::UInt64) => true,
            (Value::Float64(_), DataType::Float64) => true,
            (Value::Text(_), DataType::Utf8) => true,
            (Value::Timestamp(_), DataType::Timestamp(_, _)) => true,
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Null, _) => true, // NULL matches any type
            _ => false,
        }
    }

    /// Extract value from Arrow array at given index
    pub fn from_array(array: &dyn Array, index: usize) -> Result<Self> {
        if array.is_null(index) {
            return Ok(Value::Null);
        }

        match array.data_type() {
            DataType::Int64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Int64Array"))?;
                Ok(Value::Int64(arr.value(index)))
            }
            DataType::UInt64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<UInt64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to UInt64Array"))?;
                Ok(Value::UInt64(arr.value(index)))
            }
            DataType::Float64 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .ok_or_else(|| anyhow!("Failed to downcast to Float64Array"))?;
                Ok(Value::Float64(arr.value(index)))
            }
            DataType::Utf8 => {
                let arr = array
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to StringArray"))?;
                Ok(Value::Text(arr.value(index).to_string()))
            }
            DataType::Timestamp(_, _) => {
                let arr = array
                    .as_any()
                    .downcast_ref::<TimestampMicrosecondArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to TimestampArray"))?;
                Ok(Value::Timestamp(arr.value(index)))
            }
            DataType::Boolean => {
                let arr = array
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .ok_or_else(|| anyhow!("Failed to downcast to BooleanArray"))?;
                Ok(Value::Boolean(arr.value(index)))
            }
            _ => Err(anyhow!("Unsupported data type: {:?}", array.data_type())),
        }
    }

    /// Convert value to i64 for learned index operations
    /// Works for orderable types only
    pub fn to_i64(&self) -> Result<i64> {
        match self {
            Value::Int64(v) => Ok(*v),
            Value::UInt64(v) => Ok(*v as i64),
            Value::Timestamp(v) => Ok(*v),
            Value::Float64(v) => {
                // Use bit representation for ordering
                Ok(v.to_bits() as i64)
            }
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            Value::Text(_) => Err(anyhow!("Cannot convert Text to i64")),
            Value::Null => Err(anyhow!("Cannot convert NULL to i64")),
        }
    }

    /// Check if this value type is orderable (can be used as primary key)
    pub fn is_orderable(&self) -> bool {
        matches!(
            self,
            Value::Int64(_)
                | Value::UInt64(_)
                | Value::Timestamp(_)
                | Value::Float64(_)
                | Value::Boolean(_)
        )
    }

    /// Get the Arrow data type for this value
    pub fn arrow_type(&self) -> DataType {
        match self {
            Value::Int64(_) => DataType::Int64,
            Value::UInt64(_) => DataType::UInt64,
            Value::Float64(_) => DataType::Float64,
            Value::Text(_) => DataType::Utf8,
            Value::Timestamp(_) => {
                DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None)
            }
            Value::Boolean(_) => DataType::Boolean,
            Value::Null => DataType::Null,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int64(a), Value::Int64(b)) => a.partial_cmp(b),
            (Value::UInt64(a), Value::UInt64(b)) => a.partial_cmp(b),
            (Value::Float64(a), Value::Float64(b)) => a.partial_cmp(b),
            (Value::Text(a), Value::Text(b)) => a.partial_cmp(b),
            (Value::Timestamp(a), Value::Timestamp(b)) => a.partial_cmp(b),
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Null, _) => Some(Ordering::Less),
            (_, Value::Null) => Some(Ordering::Greater),
            _ => None, // Different types not comparable
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int64(v) => write!(f, "{}", v),
            Value::UInt64(v) => write!(f, "{}", v),
            Value::Float64(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "'{}'", v),
            Value::Timestamp(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Null => write!(f, "NULL"),
        }
    }
}

/// Helper function to check if Arrow data type is orderable
pub fn is_orderable_type(data_type: &DataType) -> bool {
    matches!(
        data_type,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
            | DataType::Timestamp(_, _)
            | DataType::Date32
            | DataType::Date64
            | DataType::Boolean
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_matches_type() {
        assert!(Value::Int64(42).matches_type(&DataType::Int64));
        assert!(Value::Text("hello".into()).matches_type(&DataType::Utf8));
        assert!(!Value::Int64(42).matches_type(&DataType::Float64));
        assert!(Value::Null.matches_type(&DataType::Int64)); // NULL matches any type
    }

    #[test]
    fn test_value_to_i64() {
        assert_eq!(Value::Int64(42).to_i64().unwrap(), 42);
        assert_eq!(Value::Timestamp(1000).to_i64().unwrap(), 1000);
        assert_eq!(Value::Boolean(true).to_i64().unwrap(), 1);
        assert!(Value::Text("hello".into()).to_i64().is_err());
    }

    #[test]
    fn test_value_ordering() {
        assert!(Value::Int64(1) < Value::Int64(2));
        assert!(Value::Float64(1.5) < Value::Float64(2.5));
        assert!(Value::Timestamp(100) < Value::Timestamp(200));
        assert!(Value::Null < Value::Int64(1));
    }

    #[test]
    fn test_is_orderable() {
        assert!(Value::Int64(42).is_orderable());
        assert!(Value::Timestamp(1000).is_orderable());
        assert!(!Value::Text("hello".into()).is_orderable());
        assert!(!Value::Null.is_orderable());
    }

    #[test]
    fn test_is_orderable_type() {
        assert!(is_orderable_type(&DataType::Int64));
        assert!(is_orderable_type(&DataType::Timestamp(
            arrow::datatypes::TimeUnit::Microsecond,
            None
        )));
        assert!(!is_orderable_type(&DataType::Utf8));
    }
}
