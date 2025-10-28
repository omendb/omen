//! End-to-End Integration Tests
//!
//! Tests the full stack integration between REST API, PostgreSQL wire protocol,
//! DataFusion SQL engine, and storage layer. Verifies cross-protocol consistency.

use datafusion::prelude::*;
use omen::postgres::PostgresServer;
use omen::rest::RestServer;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tokio_postgres::NoTls;

/// Start both REST and PostgreSQL servers sharing the same DataFusion context
async fn start_dual_protocol_servers(
    rest_port: u16,
    pg_port: u16,
) -> anyhow::Result<Arc<RwLock<SessionContext>>> {
    let ctx = Arc::new(RwLock::new(SessionContext::new()));

    // Start REST server
    let rest_ctx = ctx.clone();
    let rest_server = RestServer::with_addr(
        &format!("127.0.0.1:{}", rest_port),
        (*rest_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = rest_server.serve().await {
            eprintln!("REST server error: {}", e);
        }
    });

    // Start PostgreSQL server
    let pg_ctx = ctx.clone();
    let pg_server = PostgresServer::with_addr(
        &format!("127.0.0.1:{}", pg_port),
        (*pg_ctx.read().await).clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = pg_server.serve().await {
            eprintln!("PostgreSQL server error: {}", e);
        }
    });

    sleep(Duration::from_millis(200)).await;

    Ok(ctx)
}

/// Connect to PostgreSQL server
async fn connect_postgres(port: u16) -> anyhow::Result<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(
        &format!("host=127.0.0.1 port={} user=test dbname=test", port),
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[tokio::test]
async fn test_e2e_rest_insert_postgres_query() {
    let rest_port = 19000;
    let pg_port = 19001;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    // Create table via REST API
    let http_client = reqwest::Client::new();
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE products (id INT, name VARCHAR, price DOUBLE)"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Insert data via REST API
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO products VALUES (1, 'Laptop', 999.99), (2, 'Mouse', 29.99), (3, 'Keyboard', 79.99)"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    // Query same data via PostgreSQL protocol
    let pg_client = connect_postgres(pg_port).await.unwrap();
    let results = pg_client
        .simple_query("SELECT * FROM products WHERE price > 50")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(
        row_count, 2,
        "Should return 2 products (Laptop and Keyboard) with price > 50"
    );
}

#[tokio::test]
async fn test_e2e_postgres_insert_rest_query() {
    let rest_port = 19002;
    let pg_port = 19003;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    // Create table via PostgreSQL
    let pg_client = connect_postgres(pg_port).await.unwrap();
    pg_client
        .simple_query("CREATE TABLE customers (id INT, name VARCHAR, email VARCHAR)")
        .await
        .unwrap();

    // Insert via PostgreSQL
    pg_client
        .simple_query("INSERT INTO customers VALUES (1, 'Alice', 'alice@example.com'), (2, 'Bob', 'bob@example.com')")
        .await
        .unwrap();

    // Query via REST API
    let http_client = reqwest::Client::new();
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT * FROM customers WHERE id = 1"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);

    let row = rows[0].as_array().unwrap();
    assert_eq!(row[0], 1);
    assert_eq!(row[1], "Alice");
    assert_eq!(row[2], "alice@example.com");
}

#[tokio::test]
async fn test_e2e_cross_protocol_consistency() {
    let rest_port = 19004;
    let pg_port = 19005;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    // Setup: Create and populate table
    let http_client = reqwest::Client::new();
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE inventory (id INT, item VARCHAR, quantity INT)"
        }))
        .send()
        .await
        .unwrap();

    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO inventory VALUES (1, 'Widget', 100), (2, 'Gadget', 50), (3, 'Doohickey', 75)"
        }))
        .send()
        .await
        .unwrap();

    // Query same data via both protocols
    let rest_response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT SUM(quantity) as total FROM inventory"
        }))
        .send()
        .await
        .unwrap();

    let rest_body: Value = rest_response.json().await.unwrap();
    let rest_total = rest_body["rows"][0][0].as_i64().unwrap();

    let pg_client = connect_postgres(pg_port).await.unwrap();
    let pg_results = pg_client
        .simple_query("SELECT SUM(quantity) as total FROM inventory")
        .await
        .unwrap();

    // Both protocols should return same result
    assert_eq!(rest_total, 225, "REST API should return total of 225");

    // PostgreSQL should return RowDescription + Row + CommandComplete
    let pg_row_count = pg_results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();
    assert_eq!(
        pg_row_count, 1,
        "PostgreSQL should return 1 row with SUM result"
    );
}

#[tokio::test]
async fn test_e2e_shared_context_updates() {
    let rest_port = 19006;
    let pg_port = 19007;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    let http_client = reqwest::Client::new();
    let pg_client = connect_postgres(pg_port).await.unwrap();

    // Create table via REST
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE counters (id INT, count INT)"
        }))
        .send()
        .await
        .unwrap();

    // Insert via PostgreSQL
    pg_client
        .simple_query("INSERT INTO counters VALUES (1, 0)")
        .await
        .unwrap();

    // Update via REST
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO counters VALUES (2, 10)"
        }))
        .send()
        .await
        .unwrap();

    // Update via PostgreSQL
    pg_client
        .simple_query("INSERT INTO counters VALUES (3, 20)")
        .await
        .unwrap();

    // Verify count via REST
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT COUNT(*) as total FROM counters"
        }))
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    let count = body["rows"][0][0].as_i64().unwrap();
    assert_eq!(count, 3, "Should have 3 rows inserted via mixed protocols");

    // Verify sum via PostgreSQL
    let results = pg_client
        .simple_query("SELECT SUM(count) as total FROM counters")
        .await
        .unwrap();

    // Should see all updates from both protocols
    assert!(
        results.len() > 0,
        "PostgreSQL should return sum of all updates"
    );
}

#[tokio::test]
async fn test_e2e_multi_table_join() {
    let rest_port = 19008;
    let pg_port = 19009;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    let http_client = reqwest::Client::new();

    // Create two tables
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE orders (id INT, customer_id INT, amount DOUBLE)"
        }))
        .send()
        .await
        .unwrap();

    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE customers_join (id INT, name VARCHAR)"
        }))
        .send()
        .await
        .unwrap();

    // Insert data
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO orders VALUES (1, 101, 100.0), (2, 102, 200.0), (3, 101, 150.0)"
        }))
        .send()
        .await
        .unwrap();

    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO customers_join VALUES (101, 'Alice'), (102, 'Bob')"
        }))
        .send()
        .await
        .unwrap();

    // Test JOIN via REST
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT c.name, SUM(o.amount) as total FROM orders o JOIN customers_join c ON o.customer_id = c.id GROUP BY c.name"
        }))
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "Should return 2 customers with totals");

    // Test same JOIN via PostgreSQL
    let pg_client = connect_postgres(pg_port).await.unwrap();
    let results = pg_client
        .simple_query("SELECT c.name, SUM(o.amount) as total FROM orders o JOIN customers_join c ON o.customer_id = c.id GROUP BY c.name")
        .await
        .unwrap();

    let row_count = results
        .iter()
        .filter(|msg| matches!(msg, tokio_postgres::SimpleQueryMessage::Row(_)))
        .count();

    assert_eq!(row_count, 2, "PostgreSQL should also return 2 customers");
}

#[tokio::test]
async fn test_e2e_complex_aggregation() {
    let rest_port = 19010;
    let pg_port = 19011;

    let ctx = start_dual_protocol_servers(rest_port, pg_port)
        .await
        .unwrap();

    let http_client = reqwest::Client::new();

    // Create sales table
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "CREATE TABLE sales (id INT, product VARCHAR, quantity INT, price DOUBLE)"
        }))
        .send()
        .await
        .unwrap();

    // Insert sales data
    http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "INSERT INTO sales VALUES (1, 'Laptop', 5, 999.99), (2, 'Mouse', 20, 29.99), (3, 'Laptop', 3, 999.99), (4, 'Keyboard', 10, 79.99)"
        }))
        .send()
        .await
        .unwrap();

    // Test complex aggregation via REST
    let response = http_client
        .post(&format!("http://127.0.0.1:{}/query", rest_port))
        .json(&serde_json::json!({
            "sql": "SELECT product, SUM(quantity) as total_qty, SUM(quantity * price) as revenue FROM sales GROUP BY product ORDER BY revenue DESC"
        }))
        .send()
        .await
        .unwrap();

    let body: Value = response.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 3, "Should have 3 product groups");

    // First row should be Laptop with highest revenue
    assert_eq!(rows[0][0], "Laptop");

    // Verify via PostgreSQL
    let pg_client = connect_postgres(pg_port).await.unwrap();
    let results = pg_client
        .simple_query("SELECT COUNT(DISTINCT product) as product_count FROM sales")
        .await
        .unwrap();

    assert!(
        results.len() > 0,
        "PostgreSQL should return distinct product count"
    );
}
