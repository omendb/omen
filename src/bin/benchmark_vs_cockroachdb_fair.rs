//! Fair CockroachDB vs OmenDB Benchmark
//!
//! Compares server-to-server performance:
//! - Both accessed via PostgreSQL wire protocol
//! - Both have network overhead
//! - Both enforce their durability guarantees
//! - Same client library (tokio_postgres)

use anyhow::Result;
use std::time::Instant;
use tokio_postgres::NoTls;

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

async fn benchmark_cockroachdb(num_rows: usize) -> Result<f64> {
    println!("🔵 Testing CockroachDB...");

    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=26257 user=root dbname=defaultdb",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("CockroachDB connection error: {}", e);
        }
    });

    client
        .simple_query("DROP TABLE IF EXISTS bench_test")
        .await?;

    client
        .simple_query(
            "CREATE TABLE bench_test (
                id BIGINT PRIMARY KEY,
                value TEXT,
                amount FLOAT
            )",
        )
        .await?;

    println!("   Inserting {} rows...", format_number(num_rows));

    let start = Instant::now();

    for i in 0..num_rows {
        let id = i as i64;
        let value = format!("value_{}", i);
        let amount = i as f64 * 1.5;

        let query = format!(
            "INSERT INTO bench_test (id, value, amount) VALUES ({}, '{}', {})",
            id, value, amount
        );
        client.simple_query(&query).await?;

        if (i + 1) % 10000 == 0 {
            print!(
                "\r   Progress: {}/{} ({:.1}%)   ",
                format_number(i + 1),
                format_number(num_rows),
                (i + 1) as f64 / num_rows as f64 * 100.0
            );
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    println!("\r   Progress: {}/{} (100.0%)   ", format_number(num_rows), format_number(num_rows));

    client.simple_query("DROP TABLE bench_test").await?;

    Ok(elapsed)
}

async fn benchmark_omendb(num_rows: usize) -> Result<f64> {
    println!("\n🟢 Testing OmenDB...");

    let (client, connection) = tokio_postgres::connect(
        "host=localhost port=5433 user=postgres",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("OmenDB connection error: {}", e);
        }
    });

    client
        .simple_query("DROP TABLE IF EXISTS bench_test")
        .await?;

    client
        .simple_query(
            "CREATE TABLE bench_test (
                id BIGINT PRIMARY KEY,
                value TEXT,
                amount DOUBLE
            )",
        )
        .await?;

    println!("   Inserting {} rows...", format_number(num_rows));

    let start = Instant::now();

    for i in 0..num_rows {
        let id = i as i64;
        let value = format!("value_{}", i);
        let amount = i as f64 * 1.5;

        let query = format!(
            "INSERT INTO bench_test (id, value, amount) VALUES ({}, '{}', {})",
            id, value, amount
        );
        client.simple_query(&query).await?;

        if (i + 1) % 10000 == 0 {
            print!(
                "\r   Progress: {}/{} ({:.1}%)   ",
                format_number(i + 1),
                format_number(num_rows),
                (i + 1) as f64 / num_rows as f64 * 100.0
            );
        }
    }

    let elapsed = start.elapsed().as_secs_f64();

    println!("\r   Progress: {}/{} (100.0%)   ", format_number(num_rows), format_number(num_rows));

    client.simple_query("DROP TABLE bench_test").await?;

    Ok(elapsed)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let num_rows = if args.len() > 1 {
        args[1].parse().unwrap_or(10000)
    } else {
        10000
    };

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║    OmenDB vs CockroachDB: Fair Server Comparison           ║");
    println!("║    Both via PostgreSQL wire protocol (apples-to-apples)    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("📊 Benchmark Configuration:");
    println!("   Rows: {}", format_number(num_rows));
    println!("   Workload: Individual INSERT statements via network");
    println!("   CockroachDB: localhost:26257 (PostgreSQL protocol)");
    println!("   OmenDB: localhost:5433 (PostgreSQL protocol)");
    println!("   Client: tokio_postgres (same for both)");
    println!();

    let cockroach_time = benchmark_cockroachdb(num_rows).await?;
    let omendb_time = benchmark_omendb(num_rows).await?;

    let cockroach_throughput = num_rows as f64 / cockroach_time;
    let omendb_throughput = num_rows as f64 / omendb_time;
    let speedup = omendb_throughput / cockroach_throughput;

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    Benchmark Results                         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("📊 Write Performance (via PostgreSQL protocol):");
    println!("   CockroachDB:");
    println!("     Total time: {:.2}s", cockroach_time);
    println!("     Throughput: {} rows/sec", format_number(cockroach_throughput as usize));
    println!("     Avg latency: {:.2}ms per row", cockroach_time * 1000.0 / num_rows as f64);
    println!();
    println!("   OmenDB:");
    println!("     Total time: {:.2}s", omendb_time);
    println!("     Throughput: {} rows/sec", format_number(omendb_throughput as usize));
    println!("     Avg latency: {:.2}ms per row", omendb_time * 1000.0 / num_rows as f64);
    println!();
    println!("🚀 Speedup:");
    println!("   OmenDB is {:.2}x faster than CockroachDB", speedup);
    println!();

    if speedup >= 10.0 {
        println!("   ✅ VALIDATED: 10-50x faster claim confirmed");
    } else if speedup >= 5.0 {
        println!("   ⚠️  PARTIAL: {:.2}x faster (below 10x target)", speedup);
    } else if speedup >= 2.0 {
        println!("   ⚠️  MODEST: {:.2}x faster (below 5x target)", speedup);
    } else {
        println!("   ❌ NOT VALIDATED: Only {:.2}x faster", speedup);
    }

    println!();
    println!("📈 Analysis:");
    println!("   Fair comparison: Both systems via PostgreSQL wire protocol.");
    println!("   Both have network overhead, durability, and full DB features.");
    if speedup > 5.0 {
        println!("   Speedup likely from: Multi-level ALEX vs B-tree, less distributed overhead.");
    }
    println!();
    println!("✅ Benchmark Complete");
    println!();

    Ok(())
}
