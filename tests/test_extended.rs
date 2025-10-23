// Minimal test for Extended Query Protocol
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 port=5433 user=postgres",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("Connected!");

    // Try simple execute first
    client.execute("DROP TABLE IF EXISTS test_ext", &[]).await?;
    println!("Dropped table");

    client.execute("CREATE TABLE test_ext (id INT, name TEXT)", &[]).await?;
    println!("Created table");

    client.execute("INSERT INTO test_ext VALUES (1, 'Alice')", &[]).await?;
    println!("Inserted row");

    // Test 1: SELECT *
    println!("\n=== Test 1: SELECT * ===");
    let rows = client.query("SELECT * FROM test_ext", &[]).await?;
    println!("Query returned {} rows", rows.len());

    if !rows.is_empty() {
        println!("Row 0 has {} columns", rows[0].len());
        for i in 0..rows[0].len() {
            println!("  Column {}: {:?}", i, rows[0].columns().get(i));
        }

        match rows[0].try_get::<_, i32>(0) {
            Ok(val) => println!("✓ Column 0 (id): {}", val),
            Err(e) => println!("✗ ERROR column 0 (id): {}", e),
        }

        match rows[0].try_get::<_, String>(1) {
            Ok(val) => println!("✓ Column 1 (name): '{}'", val),
            Err(e) => println!("✗ ERROR column 1 (name): {}", e),
        }
    }

    // Test 2: SELECT name only (text column)
    println!("\n=== Test 2: SELECT name (text only) ===");
    let rows2 = client.query("SELECT name FROM test_ext", &[]).await?;
    println!("Query returned {} rows", rows2.len());

    if !rows2.is_empty() {
        println!("Row 0 has {} columns", rows2[0].len());
        match rows2[0].try_get::<_, String>(0) {
            Ok(val) => println!("✓ Name value: '{}'", val),
            Err(e) => println!("✗ ERROR accessing name: {}", e),
        }
    }

    // Test 3: SELECT id only (int column)
    println!("\n=== Test 3: SELECT id (int only) ===");
    let rows3 = client.query("SELECT id FROM test_ext", &[]).await?;
    println!("Query returned {} rows", rows3.len());

    if !rows3.is_empty() {
        println!("Row 0 has {} columns", rows3[0].len());
        match rows3[0].try_get::<_, i32>(0) {
            Ok(val) => println!("✓ ID value: {}", val),
            Err(e) => println!("✗ ERROR accessing id: {}", e),
        }
    }

    Ok(())
}
