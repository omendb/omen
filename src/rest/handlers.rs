//! REST API request handlers

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use datafusion::prelude::SessionContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

/// Metrics response
#[derive(Serialize)]
pub struct MetricsResponse {
    uptime_seconds: u64,
    queries_executed: u64,
}

/// Query request
#[derive(Deserialize)]
pub struct QueryRequest {
    sql: String,
}

/// Query response
#[derive(Serialize)]
pub struct QueryResponse {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    rows_affected: usize,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Metrics endpoint
pub async fn metrics() -> Json<MetricsResponse> {
    Json(MetricsResponse {
        uptime_seconds: 0,   // TODO: Track actual uptime
        queries_executed: 0, // TODO: Track query count
    })
}

/// Query execution endpoint
pub async fn query(
    State(ctx): State<Arc<RwLock<SessionContext>>>,
    Json(request): Json<QueryRequest>,
) -> Response {
    info!("Executing query: {}", request.sql);

    let ctx = ctx.read().await;

    // Execute query
    let df = match ctx.sql(&request.sql).await {
        Ok(df) => df,
        Err(e) => {
            error!("SQL error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("SQL error: {}", e),
                }),
            )
                .into_response();
        }
    };

    // Collect results
    let batches = match df.collect().await {
        Ok(batches) => batches,
        Err(e) => {
            error!("Execution error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Execution error: {}", e),
                }),
            )
                .into_response();
        }
    };

    // Convert to JSON
    if batches.is_empty() {
        return Json(QueryResponse {
            columns: vec![],
            rows: vec![],
            rows_affected: 0,
        })
        .into_response();
    }

    let schema = batches[0].schema();
    let columns: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();

    let mut all_rows = Vec::new();
    let mut total_rows = 0;

    for batch in batches {
        total_rows += batch.num_rows();

        for row_idx in 0..batch.num_rows() {
            let mut row = Vec::new();

            for col_idx in 0..batch.num_columns() {
                let column = batch.column(col_idx);
                let value = arrow_array_to_json(column.as_ref(), row_idx);
                row.push(value);
            }

            all_rows.push(row);
        }
    }

    Json(QueryResponse {
        columns,
        rows: all_rows,
        rows_affected: total_rows,
    })
    .into_response()
}

/// Convert Arrow array value to JSON
fn arrow_array_to_json(array: &dyn arrow::array::Array, idx: usize) -> serde_json::Value {
    use arrow::array::*;
    use arrow::datatypes::DataType;

    if array.is_null(idx) {
        return serde_json::Value::Null;
    }

    match array.data_type() {
        DataType::Boolean => {
            let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
            serde_json::Value::Bool(arr.value(idx))
        }
        DataType::Int8 => {
            let arr = array.as_any().downcast_ref::<Int8Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::Int16 => {
            let arr = array.as_any().downcast_ref::<Int16Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::UInt8 => {
            let arr = array.as_any().downcast_ref::<UInt8Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::UInt16 => {
            let arr = array.as_any().downcast_ref::<UInt16Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::UInt32 => {
            let arr = array.as_any().downcast_ref::<UInt32Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::UInt64 => {
            let arr = array.as_any().downcast_ref::<UInt64Array>().unwrap();
            serde_json::Value::Number(arr.value(idx).into())
        }
        DataType::Float32 => {
            let arr = array.as_any().downcast_ref::<Float32Array>().unwrap();
            serde_json::json!(arr.value(idx))
        }
        DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            serde_json::json!(arr.value(idx))
        }
        DataType::Utf8 => {
            let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
            serde_json::Value::String(arr.value(idx).to_string())
        }
        DataType::LargeUtf8 => {
            let arr = array.as_any().downcast_ref::<LargeStringArray>().unwrap();
            serde_json::Value::String(arr.value(idx).to_string())
        }
        _ => serde_json::Value::String(format!("{:?}", array)),
    }
}
