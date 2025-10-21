//! Arrow to PostgreSQL type conversion and encoding

use arrow::array::*;
use arrow::datatypes::{DataType, Schema};
use arrow::record_batch::RecordBatch;
use futures::{stream, Stream};
use pgwire::api::results::{DataRowEncoder, FieldFormat, FieldInfo, QueryResponse, Response};
use pgwire::api::Type as PgType;
use pgwire::error::PgWireResult;
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
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => Ok(PgType::VARCHAR),
        DataType::Binary | DataType::LargeBinary => Ok(PgType::BYTEA),
        DataType::Date32 | DataType::Date64 => Ok(PgType::DATE),
        DataType::Time32(_) | DataType::Time64(_) => Ok(PgType::TIME),
        DataType::Timestamp(_, _) => Ok(PgType::TIMESTAMP),
        DataType::Decimal128(_, _) | DataType::Decimal256(_, _) => Ok(PgType::NUMERIC),
        _ => Ok(PgType::TEXT), // Fallback to TEXT for complex types
    }
}

/// Convert Arrow Schema to PostgreSQL FieldInfo with specific format
pub fn arrow_schema_to_field_info_with_format(
    schema: &Schema,
    format: FieldFormat,
) -> PgWireResult<Vec<FieldInfo>> {
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
                format,
            ))
        })
        .collect()
}

/// Convert Arrow Schema to PostgreSQL FieldInfo (defaults to Text format)
pub fn arrow_schema_to_field_info(schema: &Schema) -> PgWireResult<Vec<FieldInfo>> {
    arrow_schema_to_field_info_with_format(schema, FieldFormat::Text)
}

/// Encode a single RecordBatch into a stream of DataRow
pub fn encode_record_batch(
    batch: RecordBatch,
    schema: Arc<Vec<FieldInfo>>,
) -> impl Stream<Item = PgWireResult<DataRow>> {
    let num_rows = batch.num_rows();
    let num_cols = batch.num_columns();

    tracing::info!("[ENCODING-SINGLE] Processing batch with {} rows, {} columns", num_rows, num_cols);

    let mut results = Vec::with_capacity(num_rows);

    for row_idx in 0..num_rows {
        let mut encoder = DataRowEncoder::new(schema.clone());

        for col_idx in 0..num_cols {
            let column = batch.column(col_idx);
            tracing::info!("[ENCODING-SINGLE] Encoding col {} of row {}, type: {:?}", col_idx, row_idx, column.data_type());

            let encode_result = encode_array_value(column.as_ref(), row_idx, &mut encoder);
            if let Err(ref e) = encode_result {
                tracing::error!("[ENCODING-SINGLE] ERROR encoding col {} of row {}: {:?}", col_idx, row_idx, e);
                results.push(Err(encode_result.unwrap_err()));
                return stream::iter(results);
            }
        }

        let row_result = encoder.finish();
        match row_result {
            Ok(row) => {
                tracing::info!("[ENCODING-SINGLE] Successfully finished row {}", row_idx);
                results.push(Ok(row));
            }
            Err(e) => {
                tracing::error!("[ENCODING-SINGLE] ERROR finishing row {}: {:?}", row_idx, e);
                results.push(Err(e));
                return stream::iter(results);
            }
        }
    }

    tracing::info!("[ENCODING-SINGLE] Total rows encoded: {}", results.len());

    stream::iter(results)
}

/// Encode a single value from an Arrow array at a specific row index
fn encode_array_value(
    array: &dyn Array,
    row_idx: usize,
    encoder: &mut DataRowEncoder,
) -> PgWireResult<()> {
    if array.is_null(row_idx) {
        tracing::info!("[ENCODING-SINGLE] Value at row {} is NULL", row_idx);
        return encoder.encode_field(&None::<i8>);
    }

    tracing::info!("[ENCODING-SINGLE] Encoding value at row {} with type {:?}", row_idx, array.data_type());

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
            let value = arr.value(row_idx);
            tracing::info!("[ENCODING-SINGLE] Int32 value: {}", value);
            encoder.encode_field(&value)
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
        DataType::Utf8View => {
            let arr = array.as_any().downcast_ref::<StringViewArray>().unwrap();
            let value = arr.value(row_idx);
            tracing::info!("[ENCODING-SINGLE] Utf8View value: '{}'", value);
            encoder.encode_field(&value)
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
    tracing::info!("[ENCODING-SINGLE] record_batch_to_query_response called");
    let field_info = arrow_schema_to_field_info(batch.schema().as_ref())?;
    tracing::info!("[ENCODING-SINGLE] Created {} field descriptions", field_info.len());
    for (i, field) in field_info.iter().enumerate() {
        tracing::info!("[ENCODING-SINGLE]   Field {}: {:?}", i, field);
    }
    let schema = Arc::new(field_info);
    let stream = encode_record_batch(batch, schema.clone());
    Ok(Response::Query(QueryResponse::new(schema, stream)))
}

/// Create multiple QueryResponses from multiple RecordBatches with specific format
pub fn record_batches_to_query_response_with_format<'a>(
    batches: Vec<RecordBatch>,
    format: FieldFormat,
) -> PgWireResult<Vec<Response<'a>>> {
    if batches.is_empty() {
        tracing::info!("[ENCODING] No batches to encode, returning EmptyQuery");
        return Ok(vec![Response::EmptyQuery]);
    }

    // Get schema from first batch
    let schema = batches[0].schema();
    let field_info = arrow_schema_to_field_info_with_format(schema.as_ref(), format)?;
    tracing::info!("[ENCODING] Created {} field descriptions for response (format: {:?})", field_info.len(), format);
    for (i, field) in field_info.iter().enumerate() {
        tracing::info!("[ENCODING]   Field {}: {:?}", i, field);
    }
    let schema_arc = Arc::new(field_info);

    // Combine all batches into a single stream
    let mut all_rows = Vec::new();

    for batch in batches {
        let num_rows = batch.num_rows();
        let num_cols = batch.num_columns();
        tracing::info!("[ENCODING] Processing batch with {} rows, {} columns", num_rows, num_cols);

        for row_idx in 0..num_rows {
            let mut encoder = DataRowEncoder::new(schema_arc.clone());

            for col_idx in 0..num_cols {
                let column = batch.column(col_idx);
                tracing::info!("[ENCODING] Encoding col {} of row {}, type: {:?}", col_idx, row_idx, column.data_type());

                let result = encode_array_value(column.as_ref(), row_idx, &mut encoder);
                if let Err(ref e) = result {
                    tracing::error!("[ENCODING] ERROR encoding col {} of row {}: {:?}", col_idx, row_idx, e);
                    return Err(result.unwrap_err());
                }
            }

            let row_result = encoder.finish();
            match row_result {
                Ok(row) => {
                    tracing::info!("[ENCODING] Successfully finished row {}", row_idx);
                    all_rows.push(Ok(row));
                }
                Err(e) => {
                    tracing::error!("[ENCODING] ERROR finishing row {}: {:?}", row_idx, e);
                    return Err(e);
                }
            }
        }
    }

    tracing::info!("[ENCODING] Total rows encoded: {}", all_rows.len());

    let stream = stream::iter(all_rows);
    Ok(vec![Response::Query(QueryResponse::new(
        schema_arc, stream,
    ))])
}

/// Create multiple QueryResponses from multiple RecordBatches (defaults to Text format)
pub fn record_batches_to_query_response<'a>(
    batches: Vec<RecordBatch>,
) -> PgWireResult<Vec<Response<'a>>> {
    record_batches_to_query_response_with_format(batches, FieldFormat::Text)
}
