//! SQL execution engine for OmenDB
//! Parses and executes SQL statements using the multi-table catalog

use crate::catalog::Catalog;
use crate::row::Row;
use crate::value::Value;
use anyhow::{Result, anyhow};
use arrow::datatypes::{DataType, Field, Schema};
use sqlparser::ast::{
    ColumnDef, DataType as SqlDataType, Expr, ObjectName, Query, Select,
    SelectItem, SetExpr, Statement, TableFactor, TableWithJoins, Values,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::sync::Arc;

/// SQL execution engine
pub struct SqlEngine {
    catalog: Catalog,
}

impl SqlEngine {
    /// Create new SQL engine with catalog
    pub fn new(catalog: Catalog) -> Self {
        Self { catalog }
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, sql: &str) -> Result<ExecutionResult> {
        let dialect = GenericDialect {};
        let statements = Parser::parse_sql(&dialect, sql)?;

        if statements.is_empty() {
            return Err(anyhow!("No SQL statement found"));
        }

        if statements.len() > 1 {
            return Err(anyhow!("Multiple statements not supported"));
        }

        match &statements[0] {
            Statement::CreateTable(stmt) => {
                self.execute_create_table(&stmt.name, &stmt.columns)
            }
            Statement::Insert(stmt) => {
                self.execute_insert(&stmt.table_name, &stmt.source)
            }
            Statement::Query(query) => self.execute_query(query),
            _ => Err(anyhow!("Unsupported SQL statement")),
        }
    }

    /// Execute CREATE TABLE statement
    fn execute_create_table(&mut self, name: &ObjectName, columns: &[ColumnDef]) -> Result<ExecutionResult> {
        let table_name = Self::extract_table_name(name)?;

        // Extract columns
        let mut fields = Vec::new();
        let mut primary_key = None;

        for column in columns {
            let field_name = column.name.value.clone();
            let data_type = Self::sql_type_to_arrow(&column.data_type)?;
            let nullable = !column.options.iter().any(|opt| {
                matches!(opt.option, sqlparser::ast::ColumnOption::NotNull)
            });

            // Check if this is the primary key
            if column.options.iter().any(|opt| {
                matches!(&opt.option, sqlparser::ast::ColumnOption::Unique { .. })
            }) {
                primary_key = Some(field_name.clone());
            }

            fields.push(Field::new(field_name, data_type, nullable));
        }

        // Default to first column as primary key if not specified
        let primary_key = primary_key.unwrap_or_else(|| {
            fields[0].name().clone()
        });

        let schema = Arc::new(Schema::new(fields));
        self.catalog.create_table(table_name.clone(), schema, primary_key)?;

        Ok(ExecutionResult::Created {
            message: format!("Table '{}' created", table_name),
        })
    }

    /// Execute INSERT statement
    fn execute_insert(&mut self, table_name: &ObjectName, source: &Option<Box<Query>>) -> Result<ExecutionResult> {
        let table_name = Self::extract_table_name(table_name)?;
        let table = self.catalog.get_table_mut(&table_name)?;
        let schema = table.schema().clone();

        // Extract values
        let source = source.as_ref().ok_or_else(|| anyhow!("No source for INSERT"))?;

        let rows_inserted = match source.body.as_ref() {
            SetExpr::Values(Values { rows, .. }) => {
                let mut count = 0;
                for row_values in rows {
                    let mut values = Vec::new();

                    for (i, expr) in row_values.iter().enumerate() {
                        if i >= schema.fields().len() {
                            return Err(anyhow!("Too many values for INSERT"));
                        }

                        let value = Self::expr_to_value(expr, schema.field(i).data_type())?;
                        values.push(value);
                    }

                    let row = Row::new(values);
                    table.insert(row)?;
                    count += 1;
                }
                count
            }
            _ => return Err(anyhow!("Only VALUES clause supported for INSERT")),
        };

        Ok(ExecutionResult::Inserted { rows: rows_inserted })
    }

    /// Execute SELECT query
    fn execute_query(&self, query: &Query) -> Result<ExecutionResult> {
        match query.body.as_ref() {
            SetExpr::Select(select) => self.execute_select(select),
            _ => Err(anyhow!("Only SELECT queries supported")),
        }
    }

    /// Execute SELECT statement
    fn execute_select(&self, select: &Select) -> Result<ExecutionResult> {
        // Extract table name
        if select.from.len() != 1 {
            return Err(anyhow!("Only single table SELECT supported"));
        }

        let table_name = match &select.from[0].relation {
            TableFactor::Table { name, .. } => Self::extract_table_name(name)?,
            _ => return Err(anyhow!("Only table SELECT supported")),
        };

        let table = self.catalog.get_table(&table_name)?;

        // Get all rows (for now, no WHERE clause support)
        let rows = table.scan_all()?;

        // Extract column names to return
        let column_names: Vec<String> = match &select.projection[0] {
            SelectItem::Wildcard(_) => {
                table.schema().fields().iter()
                    .map(|f| f.name().clone())
                    .collect()
            }
            _ => {
                select.projection.iter()
                    .filter_map(|item| {
                        if let SelectItem::UnnamedExpr(Expr::Identifier(ident)) = item {
                            Some(ident.value.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        };

        Ok(ExecutionResult::Selected {
            columns: column_names,
            rows: rows.len(),
            data: rows,
        })
    }

    /// Convert SQL data type to Arrow data type
    fn sql_type_to_arrow(sql_type: &SqlDataType) -> Result<DataType> {
        match sql_type {
            SqlDataType::BigInt(_) | SqlDataType::Int8(_) => Ok(DataType::Int64),
            SqlDataType::Double | SqlDataType::Float8 => Ok(DataType::Float64),
            SqlDataType::Varchar(_) | SqlDataType::Text | SqlDataType::String(_) => Ok(DataType::Utf8),
            SqlDataType::Timestamp(_, _) => Ok(DataType::Timestamp(
                arrow::datatypes::TimeUnit::Microsecond,
                None,
            )),
            SqlDataType::Boolean => Ok(DataType::Boolean),
            _ => Err(anyhow!("Unsupported SQL data type: {:?}", sql_type)),
        }
    }

    /// Convert SQL expression to Value
    fn expr_to_value(expr: &Expr, expected_type: &DataType) -> Result<Value> {
        match expr {
            Expr::Value(sqlparser::ast::Value::Number(n, _)) => {
                match expected_type {
                    DataType::Int64 => Ok(Value::Int64(n.parse()?)),
                    DataType::Float64 => Ok(Value::Float64(n.parse()?)),
                    DataType::Timestamp(_, _) => Ok(Value::Timestamp(n.parse()?)),
                    _ => Err(anyhow!("Cannot convert number to {:?}", expected_type)),
                }
            }
            Expr::Value(sqlparser::ast::Value::SingleQuotedString(s)) => {
                match expected_type {
                    DataType::Utf8 => Ok(Value::Text(s.clone())),
                    _ => Err(anyhow!("Cannot convert string to {:?}", expected_type)),
                }
            }
            Expr::Value(sqlparser::ast::Value::Boolean(b)) => {
                match expected_type {
                    DataType::Boolean => Ok(Value::Boolean(*b)),
                    _ => Err(anyhow!("Cannot convert boolean to {:?}", expected_type)),
                }
            }
            Expr::Value(sqlparser::ast::Value::Null) => Ok(Value::Null),
            _ => Err(anyhow!("Unsupported expression: {:?}", expr)),
        }
    }

    /// Extract table name from ObjectName
    fn extract_table_name(name: &ObjectName) -> Result<String> {
        if name.0.is_empty() {
            return Err(anyhow!("Empty table name"));
        }
        Ok(name.0[name.0.len() - 1].value.clone())
    }

    /// Get reference to catalog
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Get mutable reference to catalog
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }
}

/// Result of SQL execution
#[derive(Debug)]
pub enum ExecutionResult {
    /// Table created
    Created { message: String },

    /// Rows inserted
    Inserted { rows: usize },

    /// Rows selected
    Selected {
        columns: Vec<String>,
        rows: usize,
        data: Vec<Row>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_table() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        let sql = "CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))";
        let result = engine.execute(sql).unwrap();

        match result {
            ExecutionResult::Created { message } => {
                assert!(message.contains("users"));
            }
            _ => panic!("Expected Created result"),
        }

        assert!(engine.catalog().table_exists("users"));
    }

    #[test]
    fn test_insert_and_select() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        // Create table
        engine.execute("CREATE TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(255))").unwrap();

        // Insert data
        let result = engine.execute("INSERT INTO users VALUES (1, 'Alice'), (2, 'Bob')").unwrap();
        match result {
            ExecutionResult::Inserted { rows } => assert_eq!(rows, 2),
            _ => panic!("Expected Inserted result"),
        }

        // Select data
        let result = engine.execute("SELECT * FROM users").unwrap();
        match result {
            ExecutionResult::Selected { columns, rows, .. } => {
                assert_eq!(columns, vec!["id", "name"]);
                assert_eq!(rows, 2);
            }
            _ => panic!("Expected Selected result"),
        }
    }

    #[test]
    fn test_multiple_data_types() {
        let temp_dir = TempDir::new().unwrap();
        let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();
        let mut engine = SqlEngine::new(catalog);

        let sql = "CREATE TABLE metrics (
            timestamp BIGINT PRIMARY KEY,
            value DOUBLE,
            label VARCHAR(100),
            active BOOLEAN
        )";
        engine.execute(sql).unwrap();

        let sql = "INSERT INTO metrics VALUES (1000, 1.5, 'test', true)";
        engine.execute(sql).unwrap();

        let result = engine.execute("SELECT * FROM metrics").unwrap();
        match result {
            ExecutionResult::Selected { rows, data, .. } => {
                assert_eq!(rows, 1);
                assert_eq!(data[0].get(0).unwrap(), &Value::Int64(1000));
                assert_eq!(data[0].get(1).unwrap(), &Value::Float64(1.5));
                assert_eq!(data[0].get(2).unwrap(), &Value::Text("test".to_string()));
                assert_eq!(data[0].get(3).unwrap(), &Value::Boolean(true));
            }
            _ => panic!("Expected Selected result"),
        }
    }
}