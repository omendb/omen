//! Competitive benchmark: OmenDB vs CockroachDB
//!
//! Validates the claim: "10-50x faster single-node writes"
//!
//! Setup:
//! - CockroachDB: Docker single-node, in-memory store
//! - OmenDB: Multi-level ALEX with production durability
//!
//! Workload: Write-heavy OLTP (similar to TPC-C)

use anyhow::Result;
use omendb::table::Table;
use omendb::value::Value;
use omendb::row::Row;
use arrow::datatypes::{DataType, Field, Schema};
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║        OmenDB vs CockroachDB Write Performance              ║");
    println!("║        Validating: 10-50x faster single-node writes         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let num_rows = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    println!("📊 Benchmark Configuration:");
    println!("   Rows: {}", format_number(num_rows));
    println!("   Workload: Sequential inserts (write-heavy)");
    println!("   CockroachDB: v25.3.2 single-node, in-memory");
    println!("   OmenDB: Multi-level ALEX with durability\n");

    // Benchmark CockroachDB
    println!("🔵 Testing CockroachDB...");
    let cockroach_result = benchmark_cockroachdb(num_rows).await?;

    // Benchmark OmenDB
    println!("\n🟢 Testing OmenDB...");
    let omendb_result = benchmark_omendb(num_rows)?;

    // Results
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Benchmark Results                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("📊 Write Performance:");
    println!("   CockroachDB:");
    println!("     Total time: {:.2}s", cockroach_result.total_time);
    println!("     Throughput: {} rows/sec", format_number(cockroach_result.throughput as usize));
    println!("     Avg latency: {:.2}ms per row", cockroach_result.avg_latency_ms);

    println!("\n   OmenDB:");
    println!("     Total time: {:.2}s", omendb_result.total_time);
    println!("     Throughput: {} rows/sec", format_number(omendb_result.throughput as usize));
    println!("     Avg latency: {:.2}ms per row", omendb_result.avg_latency_ms);

    let speedup = omendb_result.throughput / cockroach_result.throughput;
    println!("\n🚀 Speedup:");
    println!("   OmenDB is {:.2}x faster than CockroachDB", speedup);

    if speedup >= 10.0 {
        println!("   ✅ VALIDATED: 10-50x faster claim confirmed");
    } else if speedup >= 5.0 {
        println!("   ⚠️  PARTIAL: {}x faster (below 10x target)", speedup);
    } else {
        println!("   ❌ NOT VALIDATED: Only {}x faster", speedup);
    }

    println!("\n📈 Analysis:");
    if speedup >= 10.0 {
        println!("   Multi-level ALEX provides significant advantage for write-heavy workloads.");
        println!("   CockroachDB's distributed architecture overhead visible even in single-node.");
    } else {
        println!("   Results indicate competitive but not dominant performance.");
        println!("   Further optimization or different workload may be needed.");
    }

    println!("\n✅ Benchmark Complete\n");

    Ok(())
}

#[derive(Debug)]
struct BenchmarkResult {
    total_time: f64,
    throughput: f64,
    avg_latency_ms: f64,
}

async fn benchmark_cockroachdb(num_rows: usize) -> Result<BenchmarkResult> {
    // Connect to CockroachDB
    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=26257 user=root dbname=defaultdb",
        NoTls,
    ).await?;

    // Spawn connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("CockroachDB connection error: {}", e);
        }
    });

    // Create table
    client.execute(
        "CREATE TABLE IF NOT EXISTS bench_test (
            id BIGINT PRIMARY KEY,
            value TEXT,
            amount FLOAT
        )",
        &[],
    ).await?;

    // Clear existing data
    client.execute("TRUNCATE TABLE bench_test", &[]).await?;

    println!("   Inserting {} rows...", format_number(num_rows));

    let start = Instant::now();

    // Insert rows
    for i in 0..num_rows {
        let id = i as i64;
        let value = format!("value_{}", i);
        let amount = i as f64 * 1.5;
        client.execute(
            "INSERT INTO bench_test (id, value, amount) VALUES ($1, $2, $3)",
            &[&id, &value, &amount],
        ).await?;

        if (i + 1) % 10000 == 0 {
            print!("\r   Progress: {}/{} ({:.1}%)",
                format_number(i + 1),
                format_number(num_rows),
                (i + 1) as f64 / num_rows as f64 * 100.0
            );
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }

    let duration = start.elapsed();
    println!("\r   Progress: {}/{} (100.0%)", format_number(num_rows), format_number(num_rows));

    // Cleanup
    client.execute("DROP TABLE bench_test", &[]).await?;

    let total_time = duration.as_secs_f64();
    let throughput = num_rows as f64 / total_time;
    let avg_latency_ms = (total_time / num_rows as f64) * 1000.0;

    Ok(BenchmarkResult {
        total_time,
        throughput,
        avg_latency_ms,
    })
}

fn benchmark_omendb(num_rows: usize) -> Result<BenchmarkResult> {
    let temp_dir = TempDir::new()?;

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Utf8, false),
        Field::new("amount", DataType::Float64, false),
    ]));

    let mut table = Table::new(
        "bench_test".to_string(),
        schema.clone(),
        "id".to_string(),
        temp_dir.path().to_path_buf(),
    )?;

    println!("   Inserting {} rows...", format_number(num_rows));

    let start = Instant::now();

    for i in 0..num_rows {
        let row = Row::new(vec![
            Value::Int64(i as i64),
            Value::Text(format!("value_{}", i)),
            Value::Float64(i as f64 * 1.5),
        ]);
        table.insert(row)?;

        if (i + 1) % 10000 == 0 {
            print!("\r   Progress: {}/{} ({:.1}%)",
                format_number(i + 1),
                format_number(num_rows),
                (i + 1) as f64 / num_rows as f64 * 100.0
            );
            use std::io::Write;
            std::io::stdout().flush().unwrap();
        }
    }

    let duration = start.elapsed();
    println!("\r   Progress: {}/{} (100.0%)", format_number(num_rows), format_number(num_rows));

    let total_time = duration.as_secs_f64();
    let throughput = num_rows as f64 / total_time;
    let avg_latency_ms = (total_time / num_rows as f64) * 1000.0;

    Ok(BenchmarkResult {
        total_time,
        throughput,
        avg_latency_ms,
    })
}

fn format_number(n: usize) -> String {
    n.to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}
