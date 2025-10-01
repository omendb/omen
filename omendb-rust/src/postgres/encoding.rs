//! Arrow to PostgreSQL type conversion and encoding

use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use futures::{stream, Stream};
use pgwire::api::results::{DataRowEncoder, FieldFormat, FieldInfo, QueryResponse, Response};
use pgwire::api::Type as PgType;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use pgwire::messages::data::DataRow;
use std::sync::Arc;

/// Convert Arrow DataType to PostgreSQL Type
pub fn arrow_to_pg_type(data_type: &DataType) -> PgWireResult<PgType> {
    match data_type {
        DataType::Boolean => Ok(PgType::BOOL),
        DataType::Int8 | DataType::UInt8 => Ok(PgType::CHAR),
        DataType::Int16 | DataType::UInt16 => Ok(PgType::INT2),
        DataType::Int32 | DataType::UInt32 => Ok(PgType::INT4),
        DataType::Int64 | DataType::UInt64 => Ok(PgType::INT8),
        DataType::Float32 => Ok(PgType::FLOAT4),
        DataType::Float64 => Ok(PgType::FLOAT8),
        DataType::Utf8 | DataType::LargeUtf8 => Ok(PgType::VARCHAR),
        DataType::Binary | DataType::LargeBinary => Ok(PgType::BYTEA),
        DataType::Date32 | DataType::Date64 => Ok(PgType::DATE),
        DataType::Time32(_) | DataType::Time64(_) => Ok(PgType::TIME),
        DataType::Timestamp(_, _) => Ok(PgType::TIMESTAMP),
        DataType::Decimal128(_, _) | DataType::Decimal256(_, _) => Ok(PgType::NUMERIC),
        _ => Ok(PgType::TEXT), // Fallback to TEXT for complex types
    }
}

/// Convert Arrow Schema to PostgreSQL FieldInfo
pub fn arrow_schema_to_field_info(schema: &Schema) -> PgWireResult<Vec<FieldInfo>> {
    schema
        .fields()
        .iter()
        .enumerate()
        .map(|(idx, field)| {
            let pg_type = arrow_to_pg_type(field.data_type())?;
            Ok(FieldInfo::new(
                field.name().clone(),
                None,
                None,
                pg_type,
                FieldFormat::Text,
            ))
        })
        .collect()
}

/// Encode a single RecordBatch into a stream of DataRow
pub fn encode_record_batch(
    batch: RecordBatch,
    schema: Arc<Vec<FieldInfo>>,
) -> impl Stream<Item = PgWireResult<DataRow>> {
    let num_rows = batch.num_rows();
    let num_cols = batch.num_columns();

    let mut results = Vec::with_capacity(num_rows);

    for row_idx in 0..num_rows {
        let mut encoder = DataRowEncoder::new(schema.clone());

        for col_idx in 0..num_cols {
            let column = batch.column(col_idx);

            let encode_result = encode_array_value(column.as_ref(), row_idx, &mut encoder);
            if let Err(e) = encode_result {
                results.push(Err(e));
                return stream::iter(results);
            }
        }

        results.push(encoder.finish());
    }

    stream::iter(results)
}

/// Encode a single value from an Arrow array at a specific row index
fn encode_array_value(
    array: &dyn Array,
    row_idx: usize,
    encoder: &mut DataRowEncoder,
) -> PgWireResult<()> {
    if array.is_null(row_idx) {
        return encoder.encode_field(&None::<i8>);
    }

    match array.data_type() {
        DataType::Boolean => {
            let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Int8 => {
            let arr = array.as_any().downcast_ref::<Int8Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Int16 => {
            let arr = array.as_any().downcast_ref::<Int16Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::UInt8 => {
            let arr = array.as_any().downcast_ref::<UInt8Array>().unwrap();
            encoder.encode_field(&(arr.value(row_idx) as i8))
        }
        DataType::UInt16 => {
            let arr = array.as_any().downcast_ref::<UInt16Array>().unwrap();
            encoder.encode_field(&(arr.value(row_idx) as i16))
        }
        DataType::UInt32 => {
            let arr = array.as_any().downcast_ref::<UInt32Array>().unwrap();
            encoder.encode_field(&(arr.value(row_idx) as i32))
        }
        DataType::UInt64 => {
            let arr = array.as_any().downcast_ref::<UInt64Array>().unwrap();
            encoder.encode_field(&(arr.value(row_idx) as i64))
        }
        DataType::Float32 => {
            let arr = array.as_any().downcast_ref::<Float32Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Utf8 => {
            let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::LargeUtf8 => {
            let arr = array.as_any().downcast_ref::<LargeStringArray>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Binary => {
            let arr = array.as_any().downcast_ref::<BinaryArray>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::LargeBinary => {
            let arr = array.as_any().downcast_ref::<LargeBinaryArray>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Date32 => {
            let arr = array.as_any().downcast_ref::<Date32Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Date64 => {
            let arr = array.as_any().downcast_ref::<Date64Array>().unwrap();
            encoder.encode_field(&arr.value(row_idx))
        }
        DataType::Timestamp(_, _) => {
            let arr = array.as_any().downcast_ref::<TimestampNanosecondArray>();
            if let Some(arr) = arr {
                encoder.encode_field(&arr.value(row_idx))
            } else {
                let arr = array.as_any().downcast_ref::<TimestampMicrosecondArray>();
                if let Some(arr) = arr {
                    encoder.encode_field(&arr.value(row_idx))
                } else {
                    encoder.encode_field(&format!("{:?}", array))
                }
            }
        }
        _ => {
            // Fallback: convert to string representation
            encoder.encode_field(&format!("{:?}", array))
        }
    }
}

/// Create a QueryResponse from a RecordBatch
pub fn record_batch_to_query_response<'a>(batch: RecordBatch) -> PgWireResult<Response<'a>> {
    let field_info = arrow_schema_to_field_info(batch.schema().as_ref())?;
    let schema = Arc::new(field_info);
    let stream = encode_record_batch(batch, schema.clone());
    Ok(Response::Query(QueryResponse::new(schema, stream)))
}

/// Create multiple QueryResponses from multiple RecordBatches
pub fn record_batches_to_query_response<'a>(
    batches: Vec<RecordBatch>,
) -> PgWireResult<Vec<Response<'a>>> {
    if batches.is_empty() {
        return Ok(vec![Response::EmptyQuery]);
    }

    // Get schema from first batch
    let schema = batches[0].schema();
    let field_info = arrow_schema_to_field_info(schema.as_ref())?;
    let schema_arc = Arc::new(field_info);

    // Combine all batches into a single stream
    let mut all_rows = Vec::new();

    for batch in batches {
        let num_rows = batch.num_rows();
        let num_cols = batch.num_columns();

        for row_idx in 0..num_rows {
            let mut encoder = DataRowEncoder::new(schema_arc.clone());

            for col_idx in 0..num_cols {
                let column = batch.column(col_idx);
                encode_array_value(column.as_ref(), row_idx, &mut encoder)?;
            }

            all_rows.push(encoder.finish());
        }
    }

    let stream = stream::iter(all_rows);
    Ok(vec![Response::Query(QueryResponse::new(schema_arc, stream))])
}
