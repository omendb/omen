//! Table constraint management and validation

use anyhow::{anyhow, Result};
use datafusion::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Stores table constraint metadata
#[derive(Debug, Clone)]
pub struct TableConstraints {
    /// Primary key column names for each table
    primary_keys: HashMap<String, Vec<String>>,
}

impl Default for TableConstraints {
    fn default() -> Self {
        Self::new()
    }
}

impl TableConstraints {
    pub fn new() -> Self {
        Self {
            primary_keys: HashMap::new(),
        }
    }

    /// Register a primary key constraint for a table
    pub fn add_primary_key(&mut self, table_name: String, column_names: Vec<String>) {
        info!("Registering PRIMARY KEY constraint on table '{}': {:?}", table_name, column_names);
        self.primary_keys.insert(table_name, column_names);
    }

    /// Get primary key columns for a table
    pub fn get_primary_key(&self, table_name: &str) -> Option<&Vec<String>> {
        self.primary_keys.get(table_name)
    }

    /// Check if table has a primary key constraint
    pub fn has_primary_key(&self, table_name: &str) -> bool {
        self.primary_keys.contains_key(table_name)
    }

    /// Remove constraints for a table (e.g., when dropping table)
    pub fn remove_table(&mut self, table_name: &str) {
        self.primary_keys.remove(table_name);
    }
}

/// Constraint manager that validates operations
pub struct ConstraintManager {
    constraints: Arc<RwLock<TableConstraints>>,
    ctx: Arc<RwLock<SessionContext>>,
}

impl ConstraintManager {
    pub fn new(ctx: Arc<RwLock<SessionContext>>) -> Self {
        Self {
            constraints: Arc::new(RwLock::new(TableConstraints::new())),
            ctx,
        }
    }

    pub fn with_constraints(
        ctx: Arc<RwLock<SessionContext>>,
        constraints: Arc<RwLock<TableConstraints>>,
    ) -> Self {
        Self { constraints, ctx }
    }

    pub fn constraints(&self) -> Arc<RwLock<TableConstraints>> {
        self.constraints.clone()
    }

    /// Parse CREATE TABLE statement and extract PRIMARY KEY constraint
    pub async fn register_table_schema(&self, query: &str) -> Result<()> {
        let upper = query.trim().to_uppercase();

        if !upper.starts_with("CREATE TABLE") {
            return Ok(());
        }

        // Extract table name and check for PRIMARY KEY
        if let Some(table_name) = Self::extract_table_name(query) {
            if let Some(pk_columns) = Self::extract_primary_key(query) {
                let mut constraints = self.constraints.write().await;
                constraints.add_primary_key(table_name, pk_columns);
            }
        }

        Ok(())
    }

    /// Extract table name from CREATE TABLE statement
    fn extract_table_name(query: &str) -> Option<String> {
        let upper = query.trim().to_uppercase();
        let create_table = "CREATE TABLE ";

        if let Some(pos) = upper.find(create_table) {
            let after_create = &query[pos + create_table.len()..];

            // Handle "CREATE TABLE IF NOT EXISTS"
            let after_if_not_exists = if after_create.trim().to_uppercase().starts_with("IF NOT EXISTS") {
                after_create[13..].trim()
            } else {
                after_create.trim()
            };

            // Get table name (until space or opening paren)
            if let Some(end) = after_if_not_exists.find([' ', '(']) {
                return Some(after_if_not_exists[..end].trim().to_string());
            }
        }

        None
    }

    /// Extract PRIMARY KEY column(s) from CREATE TABLE statement
    fn extract_primary_key(query: &str) -> Option<Vec<String>> {
        let upper = query.trim().to_uppercase();

        // Look for table-level constraint FIRST (more specific): "PRIMARY KEY (column_name)"
        if let Some(pk_start) = upper.find("PRIMARY KEY (") {
            let after_pk = &query[pk_start + 13..]; // Skip "PRIMARY KEY ("
            if let Some(end_paren) = after_pk.find(')') {
                let columns_str = &after_pk[..end_paren];
                let columns: Vec<String> = columns_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                debug!("Extracted PRIMARY KEY columns: {:?}", columns);
                return Some(columns);
            }
        }

        // Look for inline PRIMARY KEY (fallback): "column_name INT PRIMARY KEY"
        if upper.contains(" PRIMARY KEY") {
            // Find column definition before PRIMARY KEY
            if let Some(pk_pos) = upper.find(" PRIMARY KEY") {
                let before_pk = &query[..pk_pos];

                // Find the column name (look backwards for last identifier before type)
                let parts: Vec<&str> = before_pk.split_whitespace().collect();
                if parts.len() >= 2 {
                    // Get column name (should be second-to-last token before PRIMARY KEY)
                    let col_name = parts[parts.len() - 2].trim_matches(|c| c == '(' || c == ',').to_string();
                    debug!("Extracted PRIMARY KEY column: {}", col_name);
                    return Some(vec![col_name]);
                }
            }
        }

        None
    }

    /// Validate INSERT statement doesn't violate PRIMARY KEY constraint
    pub async fn validate_insert(&self, query: &str) -> Result<()> {
        let upper = query.trim().to_uppercase();

        if !upper.starts_with("INSERT") {
            return Ok(());
        }

        // Extract table name from INSERT statement
        let table_name = Self::extract_insert_table_name(query)?;

        // Check if table has PRIMARY KEY constraint
        let constraints = self.constraints.read().await;
        if let Some(pk_columns) = constraints.get_primary_key(&table_name) {
            debug!("Validating PRIMARY KEY constraint for table '{}' on columns {:?}", table_name, pk_columns);

            // For now, we'll validate using DataFusion query
            // Extract values being inserted
            if let Some(values) = Self::extract_insert_values(query, pk_columns.len()) {
                // Check if key already exists
                self.check_duplicate_key(&table_name, pk_columns, &values).await?;
            }
        }

        Ok(())
    }

    /// Extract table name from INSERT statement
    fn extract_insert_table_name(query: &str) -> Result<String> {
        let upper = query.trim().to_uppercase();
        let insert_into = "INSERT INTO ";

        if let Some(pos) = upper.find(insert_into) {
            let after_insert = &query[pos + insert_into.len()..];

            // Get table name (until space or opening paren)
            if let Some(end) = after_insert.find([' ', '(']) {
                return Ok(after_insert[..end].trim().to_string());
            }
        }

        Err(anyhow!("Could not extract table name from INSERT statement"))
    }

    /// Extract values from INSERT VALUES statement
    fn extract_insert_values(query: &str, num_pk_cols: usize) -> Option<Vec<String>> {
        let upper = query.trim().to_uppercase();

        if let Some(values_pos) = upper.find("VALUES") {
            let after_values = &query[values_pos + 6..].trim();

            // Find opening and closing parentheses
            if let Some(open_paren) = after_values.find('(') {
                if let Some(close_paren) = after_values.find(')') {
                    let values_str = &after_values[open_paren + 1..close_paren];
                    let values: Vec<String> = values_str
                        .split(',')
                        .take(num_pk_cols)  // Only take as many values as there are PK columns
                        .map(|s| s.trim().trim_matches('\'').to_string())
                        .collect();

                    if values.len() == num_pk_cols {
                        return Some(values);
                    }
                }
            }
        }

        None
    }

    /// Check if primary key value already exists in table
    async fn check_duplicate_key(
        &self,
        table_name: &str,
        pk_columns: &[String],
        values: &[String],
    ) -> Result<()> {
        let ctx = self.ctx.read().await;

        // Build WHERE clause for primary key check
        let mut where_parts = Vec::new();
        for (col, val) in pk_columns.iter().zip(values.iter()) {
            where_parts.push(format!("{} = {}", col, val));
        }
        let where_clause = where_parts.join(" AND ");

        let check_query = format!("SELECT COUNT(*) FROM {} WHERE {}", table_name, where_clause);
        debug!("Checking for duplicate key: {}", check_query);

        // Execute check query
        let df = match ctx.sql(&check_query).await {
            Ok(df) => df,
            Err(e) => {
                // If table doesn't exist yet, there can't be duplicates
                let err_msg = e.to_string();
                if err_msg.contains("doesn't exist") || err_msg.contains("not found") {
                    debug!("Table '{}' doesn't exist yet, skipping duplicate check", table_name);
                    return Ok(());
                }
                return Err(e.into());
            }
        };
        let batches = df.collect().await?;

        // Check if any rows exist
        if !batches.is_empty() && batches[0].num_rows() > 0 {
            // Get count value
            let count_array = batches[0].column(0);
            if let Some(count_arr) = count_array.as_any().downcast_ref::<arrow::array::Int64Array>() {
                let count = count_arr.value(0);
                if count > 0 {
                    return Err(anyhow!(
                        "duplicate key value violates unique constraint: Key ({})=({}) already exists",
                        pk_columns.join(", "),
                        values.join(", ")
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_name() {
        assert_eq!(
            ConstraintManager::extract_table_name("CREATE TABLE users (id INT)"),
            Some("users".to_string())
        );
        assert_eq!(
            ConstraintManager::extract_table_name("CREATE TABLE IF NOT EXISTS users (id INT)"),
            Some("users".to_string())
        );
    }

    #[test]
    fn test_extract_primary_key_inline() {
        let sql = "CREATE TABLE users (id INT PRIMARY KEY, name TEXT)";
        assert_eq!(
            ConstraintManager::extract_primary_key(sql),
            Some(vec!["id".to_string()])
        );
    }

    #[test]
    fn test_extract_primary_key_table_level() {
        let sql = "CREATE TABLE users (id INT, name TEXT, PRIMARY KEY (id))";
        assert_eq!(
            ConstraintManager::extract_primary_key(sql),
            Some(vec!["id".to_string()])
        );
    }

    #[test]
    fn test_extract_insert_table_name() {
        assert_eq!(
            ConstraintManager::extract_insert_table_name("INSERT INTO users VALUES (1, 'Alice')").unwrap(),
            "users"
        );
    }

    #[test]
    fn test_extract_insert_values() {
        let values = ConstraintManager::extract_insert_values("INSERT INTO users VALUES (1, 'Alice')", 1);
        assert_eq!(values, Some(vec!["1".to_string()]));
    }
}
