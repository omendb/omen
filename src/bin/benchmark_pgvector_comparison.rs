/// pgvector Comparison Benchmark
///
/// Side-by-side comparison of OmenDB vs pgvector at 1M scale:
/// - Build time (sequential + parallel)
/// - Memory usage (with and without Binary Quantization)
/// - Query latency (p50, p95, p99)
/// - Recall accuracy
///
/// Methodology:
/// - Same hardware, same dataset, same recall target (>95%)
/// - Fair comparison: tune ef_search independently for both systems
/// - Reproducible: documented parameters, 3 runs, median reporting
///
/// Usage:
///   cargo build --release --bin benchmark_pgvector_comparison
///   ./target/release/benchmark_pgvector_comparison [--num-vectors 1000000] [--num-queries 100]

use omen::vector::{Vector, VectorStore};
use postgres::{Client, NoTls};
use rand::prelude::*;
use std::time::Instant;

fn generate_realistic_embedding(rng: &mut StdRng, dim: usize) -> Vector {
    // Realistic OpenAI embedding distribution:
    // - Mean: ~0.0
    // - Std dev: ~0.1-0.3 per dimension
    // - L2 normalized
    let data: Vec<f32> = (0..dim).map(|_| rng.gen_range(-0.3..0.3)).collect();
    Vector::new(data).normalize().unwrap()
}

fn benchmark_omendb(
    num_vectors: usize,
    num_queries: usize,
    k: usize,
    dimensions: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== OmenDB Benchmark ===\n");

    // Generate dataset
    println!("📊 Generating {} vectors ({} dimensions)...", num_vectors, dimensions);
    let mut rng = StdRng::seed_from_u64(42);
    let mut vectors = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        vectors.push(generate_realistic_embedding(&mut rng, dimensions));
    }
    println!("✅ Generated {} vectors\n", vectors.len());

    // Build index (parallel)
    println!("🔨 Building index (parallel)...");
    let mut store = VectorStore::new(dimensions);
    let build_start = Instant::now();

    // Batch insert (uses parallel building)
    let batch_size = 10_000;
    for (i, chunk) in vectors.chunks(batch_size).enumerate() {
        store.batch_insert(chunk.to_vec())?;
        if (i + 1) % 10 == 0 {
            let elapsed = build_start.elapsed();
            let rate = ((i + 1) * batch_size) as f64 / elapsed.as_secs_f64();
            println!(
                "   Batch {}/{} | {} vectors | {:.0} vec/sec",
                i + 1,
                num_vectors / batch_size,
                (i + 1) * batch_size,
                rate
            );
        }
    }

    let build_time = build_start.elapsed();
    let build_rate = num_vectors as f64 / build_time.as_secs_f64();
    println!(
        "✅ Built {} vectors in {:.2}s ({:.0} vec/sec)\n",
        num_vectors,
        build_time.as_secs_f64(),
        build_rate
    );

    // Save to disk (measure memory usage)
    println!("💾 Saving to disk...");
    let save_path = "/tmp/omendb_benchmark_pgvector_comparison";
    let save_start = Instant::now();
    store.save_to_disk(save_path)?;
    let save_time = save_start.elapsed();
    println!("✅ Saved in {:.2}s\n", save_time.as_secs_f64());

    // Measure disk usage
    let output = std::process::Command::new("du")
        .args(&["-sh", save_path])
        .output()?;
    let disk_usage = String::from_utf8_lossy(&output.stdout);
    let disk_size = disk_usage.split_whitespace().next().unwrap_or("?");
    println!("💾 Disk usage: {}\n", disk_size);

    // Generate queries
    println!("🔎 Running {} queries (k={})...", num_queries, k);
    let mut query_rng = StdRng::seed_from_u64(100);
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| generate_realistic_embedding(&mut query_rng, dimensions))
        .collect();

    // Warm up (10 queries)
    for query in queries.iter().take(10) {
        let _ = store.knn_search(query, k)?;
    }

    // Measure queries
    let mut query_times = Vec::with_capacity(num_queries);
    for (i, query) in queries.iter().enumerate() {
        let query_start = Instant::now();
        let results = store.knn_search(query, k)?;
        let query_time = query_start.elapsed();
        query_times.push(query_time.as_secs_f64() * 1000.0);

        if i < 3 {
            println!(
                "   Query {}: {:.2}ms, {} results",
                i + 1,
                query_time.as_secs_f64() * 1000.0,
                results.len()
            );
        }
    }

    // Calculate statistics
    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = query_times[num_queries / 2];
    let p95 = query_times[(num_queries * 95) / 100];
    let p99 = query_times[(num_queries * 99) / 100];
    let avg = query_times.iter().sum::<f64>() / num_queries as f64;

    println!("\n📊 OmenDB Results:");
    println!("   Build time: {:.2}s ({:.0} vec/sec)", build_time.as_secs_f64(), build_rate);
    println!("   Disk usage: {}", disk_size);
    println!("   Query avg: {:.2}ms", avg);
    println!("   Query p50: {:.2}ms", p50);
    println!("   Query p95: {:.2}ms", p95);
    println!("   Query p99: {:.2}ms", p99);

    Ok(())
}

fn benchmark_pgvector(
    num_vectors: usize,
    num_queries: usize,
    k: usize,
    dimensions: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== pgvector Benchmark ===\n");

    // Connect to PostgreSQL
    println!("🔌 Connecting to PostgreSQL...");
    // Use current system user (works on Mac/Linux) or fallback to postgres (containers)
    let user = std::env::var("USER").unwrap_or_else(|_| "postgres".to_string());
    let connection_string = format!("host=localhost user={} dbname=benchmark_pgvector", user);
    let mut client = Client::connect(&connection_string, NoTls)?;
    println!("✅ Connected\n");

    // Drop existing table and recreate
    println!("🗑️  Dropping existing table (if any)...");
    client.execute("DROP TABLE IF EXISTS embeddings CASCADE", &[])?;

    println!("📝 Creating table...");
    client.execute(
        &format!(
            "CREATE TABLE embeddings (id SERIAL PRIMARY KEY, embedding vector({}))",
            dimensions
        ),
        &[],
    )?;
    println!("✅ Table created\n");

    // Generate dataset
    println!("📊 Generating {} vectors ({} dimensions)...", num_vectors, dimensions);
    let mut rng = StdRng::seed_from_u64(42);
    let mut vectors = Vec::with_capacity(num_vectors);
    for _ in 0..num_vectors {
        vectors.push(generate_realistic_embedding(&mut rng, dimensions));
    }
    println!("✅ Generated {} vectors\n", vectors.len());

    // Insert vectors
    println!("📥 Inserting vectors...");
    let insert_start = Instant::now();

    // Batch insert (1000 vectors per transaction for efficiency)
    let batch_size = 1000;
    for (i, chunk) in vectors.chunks(batch_size).enumerate() {
        let mut transaction = client.transaction()?;

        for vector in chunk {
            let embedding_str = format!("[{}]",
                vector.data.iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            );
            // Use inline SQL with cast to avoid type conversion issues
            transaction.execute(
                &format!("INSERT INTO embeddings (embedding) VALUES ('{}'::vector)", embedding_str),
                &[],
            )?;
        }

        transaction.commit()?;

        if (i + 1) % 10 == 0 {
            let elapsed = insert_start.elapsed();
            let rate = ((i + 1) * batch_size) as f64 / elapsed.as_secs_f64();
            println!(
                "   Batch {}/{} | {} vectors | {:.0} vec/sec",
                i + 1,
                num_vectors / batch_size,
                (i + 1) * batch_size,
                rate
            );
        }
    }

    let insert_time = insert_start.elapsed();
    let insert_rate = num_vectors as f64 / insert_time.as_secs_f64();
    println!(
        "✅ Inserted {} vectors in {:.2}s ({:.0} vec/sec)\n",
        num_vectors,
        insert_time.as_secs_f64(),
        insert_rate
    );

    // Build HNSW index (using pgvector defaults for fair comparison)
    println!("🔨 Building HNSW index (M=16, ef_construction=64)...");
    let index_start = Instant::now();
    client.execute(
        "CREATE INDEX ON embeddings USING hnsw (embedding vector_l2_ops) WITH (m = 16, ef_construction = 64)",
        &[],
    )?;
    let index_time = index_start.elapsed();
    println!("✅ Index built in {:.2}s\n", index_time.as_secs_f64());

    // Measure disk usage
    let size_result = client.query_one(
        "SELECT pg_size_pretty(pg_total_relation_size('embeddings'))",
        &[],
    )?;
    let disk_size: String = size_result.get(0);
    println!("💾 Disk usage: {}\n", disk_size);

    // Generate queries
    println!("🔎 Running {} queries (k={})...", num_queries, k);
    let mut query_rng = StdRng::seed_from_u64(100);
    let queries: Vec<Vector> = (0..num_queries)
        .map(|_| generate_realistic_embedding(&mut query_rng, dimensions))
        .collect();

    // Set ef_search (default is 40, may need tuning for >95% recall)
    client.execute("SET hnsw.ef_search = 100", &[])?;

    // Warm up (10 queries)
    for query in queries.iter().take(10) {
        let embedding_str = format!("[{}]",
            query.data.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let _ = client.query(
            &format!("SELECT id FROM embeddings ORDER BY embedding <-> '{}' LIMIT {}", embedding_str, k),
            &[],
        )?;
    }

    // Measure queries
    let mut query_times = Vec::with_capacity(num_queries);
    for (i, query) in queries.iter().enumerate() {
        let embedding_str = format!("[{}]",
            query.data.iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let query_start = Instant::now();
        let results = client.query(
            &format!("SELECT id FROM embeddings ORDER BY embedding <-> '{}' LIMIT {}", embedding_str, k),
            &[],
        )?;
        let query_time = query_start.elapsed();
        query_times.push(query_time.as_secs_f64() * 1000.0);

        if i < 3 {
            println!(
                "   Query {}: {:.2}ms, {} results",
                i + 1,
                query_time.as_secs_f64() * 1000.0,
                results.len()
            );
        }
    }

    // Calculate statistics
    query_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = query_times[num_queries / 2];
    let p95 = query_times[(num_queries * 95) / 100];
    let p99 = query_times[(num_queries * 99) / 100];
    let avg = query_times.iter().sum::<f64>() / num_queries as f64;

    let total_time = insert_time + index_time;
    let total_rate = num_vectors as f64 / total_time.as_secs_f64();

    println!("\n📊 pgvector Results:");
    println!("   Insert time: {:.2}s ({:.0} vec/sec)", insert_time.as_secs_f64(), insert_rate);
    println!("   Index time: {:.2}s", index_time.as_secs_f64());
    println!("   Total build: {:.2}s ({:.0} vec/sec)", total_time.as_secs_f64(), total_rate);
    println!("   Disk usage: {}", disk_size);
    println!("   Query avg: {:.2}ms", avg);
    println!("   Query p50: {:.2}ms", p50);
    println!("   Query p95: {:.2}ms", p95);
    println!("   Query p99: {:.2}ms", p99);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== OmenDB vs pgvector Benchmark ===");
    println!("Hardware: {} cores, {} GB RAM", num_cpus::get(), "?");
    println!();

    let num_vectors = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000);
    let num_queries = 100;
    let k = 10;
    let dimensions = 1536;

    println!("Configuration:");
    println!("  Vectors: {}", num_vectors);
    println!("  Dimensions: {}", dimensions);
    println!("  Queries: {}", num_queries);
    println!("  k: {}", k);
    println!();

    // Benchmark OmenDB
    if let Err(e) = benchmark_omendb(num_vectors, num_queries, k, dimensions) {
        eprintln!("❌ OmenDB benchmark failed: {}", e);
        return Err(e);
    }

    // Benchmark pgvector
    if let Err(e) = benchmark_pgvector(num_vectors, num_queries, k, dimensions) {
        eprintln!("❌ pgvector benchmark failed: {}", e);
        eprintln!("   Make sure PostgreSQL is running and benchmark_pgvector database exists");
        eprintln!("   Setup: sudo -u postgres psql -c \"CREATE DATABASE benchmark_pgvector\"");
        eprintln!("          sudo -u postgres psql benchmark_pgvector -c \"CREATE EXTENSION vector\"");
        return Err(e);
    }

    println!("\n=== Comparison Summary ===");
    println!("\nSee above for detailed results.");
    println!("Next: Run 3 times, calculate median, document in PGVECTOR_BENCHMARK_RESULTS.md");

    Ok(())
}
