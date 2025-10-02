//! Unit tests for PostgreSQL wire protocol modules

#[cfg(test)]
mod encoding_tests {
    use crate::postgres::encoding::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use pgwire::api::Type;
    use std::sync::Arc;

    #[test]
    fn test_arrow_to_pg_type_int64() {
        let pg_type = arrow_to_pg_type(&DataType::Int64).unwrap();
        assert_eq!(pg_type, Type::INT8);
    }

    #[test]
    fn test_arrow_to_pg_type_int32() {
        let pg_type = arrow_to_pg_type(&DataType::Int32).unwrap();
        assert_eq!(pg_type, Type::INT4);
    }

    #[test]
    fn test_arrow_to_pg_type_int16() {
        let pg_type = arrow_to_pg_type(&DataType::Int16).unwrap();
        assert_eq!(pg_type, Type::INT2);
    }

    #[test]
    fn test_arrow_to_pg_type_float64() {
        let pg_type = arrow_to_pg_type(&DataType::Float64).unwrap();
        assert_eq!(pg_type, Type::FLOAT8);
    }

    #[test]
    fn test_arrow_to_pg_type_float32() {
        let pg_type = arrow_to_pg_type(&DataType::Float32).unwrap();
        assert_eq!(pg_type, Type::FLOAT4);
    }

    #[test]
    fn test_arrow_to_pg_type_utf8() {
        let pg_type = arrow_to_pg_type(&DataType::Utf8).unwrap();
        assert_eq!(pg_type, Type::VARCHAR);
    }

    #[test]
    fn test_arrow_to_pg_type_large_utf8() {
        let pg_type = arrow_to_pg_type(&DataType::LargeUtf8).unwrap();
        assert_eq!(pg_type, Type::VARCHAR);
    }

    #[test]
    fn test_arrow_to_pg_type_boolean() {
        let pg_type = arrow_to_pg_type(&DataType::Boolean).unwrap();
        assert_eq!(pg_type, Type::BOOL);
    }

    #[test]
    fn test_arrow_to_pg_type_timestamp() {
        let pg_type = arrow_to_pg_type(&DataType::Timestamp(
            arrow::datatypes::TimeUnit::Microsecond,
            None,
        ))
        .unwrap();
        assert_eq!(pg_type, Type::TIMESTAMP);
    }

    #[test]
    fn test_arrow_to_pg_type_date32() {
        let pg_type = arrow_to_pg_type(&DataType::Date32).unwrap();
        assert_eq!(pg_type, Type::DATE);
    }

    #[test]
    fn test_arrow_to_pg_type_binary() {
        let pg_type = arrow_to_pg_type(&DataType::Binary).unwrap();
        assert_eq!(pg_type, Type::BYTEA);
    }

    #[test]
    fn test_arrow_to_pg_type_decimal() {
        let pg_type = arrow_to_pg_type(&DataType::Decimal128(10, 2)).unwrap();
        assert_eq!(pg_type, Type::NUMERIC);
    }

    #[test]
    fn test_arrow_to_pg_type_unsupported_falls_back_to_text() {
        let pg_type =
            arrow_to_pg_type(&DataType::Duration(arrow::datatypes::TimeUnit::Second)).unwrap();
        assert_eq!(pg_type, Type::TEXT);
    }

    #[test]
    fn test_arrow_schema_to_field_info() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("age", DataType::Int32, true),
        ]);

        let field_info = arrow_schema_to_field_info(&Arc::new(schema)).unwrap();

        assert_eq!(field_info.len(), 3);
        assert_eq!(field_info[0].name(), "id");
        assert_eq!(field_info[0].datatype(), &Type::INT8);

        assert_eq!(field_info[1].name(), "name");
        assert_eq!(field_info[1].datatype(), &Type::VARCHAR);

        assert_eq!(field_info[2].name(), "age");
        assert_eq!(field_info[2].datatype(), &Type::INT4);
    }
}

#[cfg(test)]
mod handlers_tests {
    use crate::postgres::handlers::OmenDbQueryHandler;
    use datafusion::prelude::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_handler_creation() {
        let ctx = SessionContext::new();
        let _handler = OmenDbQueryHandler::new(Arc::new(RwLock::new(ctx)));
        // Handler created successfully
    }

    #[tokio::test]
    async fn test_handler_factory_creation() {
        use crate::postgres::handlers::OmenDbHandlerFactory;
        use pgwire::api::PgWireHandlerFactory;

        let ctx = SessionContext::new();
        let factory = OmenDbHandlerFactory::new(Arc::new(RwLock::new(ctx)));

        // Verify factory can create handlers
        let _simple_handler = factory.simple_query_handler();
        let _startup_handler = factory.startup_handler();
        let _copy_handler = factory.copy_handler();
        let _extended_handler = factory.extended_query_handler();
    }
}
