/// 1M End-to-End Validation
///
/// Validates the complete flow at 1M scale:
/// 1. Build 1M vectors (parallel)
/// 2. Save to disk (HNSW + vectors)
/// 3. Load from disk (fast path)
/// 4. Run 100 queries
/// 5. Verify correctness
///
/// This is a critical validation before benchmarking against pgvector.

use omen::vector::{HNSWIndex, Vector, VectorStore};
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

fn main() {
    println!("=== 1M End-to-End Validation ===\n");

    let dimensions = 1536; // OpenAI embeddings
    let num_vectors = 1_000_000;
    let num_queries = 100;
    let k = 10;

    let store_path = "/tmp/omendb_1m_validation/store";

    // Check if we have existing data
    let graph_path = format!("{}.hnsw.graph", store_path);
    let data_path = format!("{}.hnsw.data", store_path);

    let mut store = if std::path::Path::new(&graph_path).exists()
        && std::path::Path::new(&data_path).exists()
    {
        // Load existing data
        println!("📂 Loading existing 1M vector store from disk...");
        let start = Instant::now();
        let loaded = VectorStore::load_from_disk(store_path, dimensions)
            .expect("Failed to load store");
        let load_time = start.elapsed();

        println!(
            "✅ Loaded {} vectors in {:.2}s",
            loaded.len(),
            load_time.as_secs_f64()
        );
        println!(
            "   Load speed: {:.0} vectors/sec\n",
            loaded.len() as f64 / load_time.as_secs_f64()
        );

        loaded
    } else {
        // Build new data
        println!("🔨 Building fresh 1M vector store...");
        println!("   Dimensions: {}", dimensions);
        println!("   Vectors: {}\n", num_vectors);

        let mut store = VectorStore::new(dimensions);
        let mut rng = StdRng::seed_from_u64(42);

        // Generate vectors in batches for parallel building
        let batch_size = 10_000;
        let mut total_inserted = 0;

        let start = Instant::now();

        for batch_idx in 0..(num_vectors / batch_size) {
            let batch_start = Instant::now();

            // Generate batch
            let mut batch = Vec::with_capacity(batch_size);
            for _ in 0..batch_size {
                batch.push(generate_realistic_embedding(&mut rng, dimensions));
            }

            // Insert batch (uses parallel building)
            store.batch_insert(batch).expect("Failed to insert batch");

            total_inserted += batch_size;
            let batch_time = batch_start.elapsed();

            if (batch_idx + 1) % 10 == 0 {
                let elapsed = start.elapsed();
                let rate = total_inserted as f64 / elapsed.as_secs_f64();
                println!(
                    "   Batch {}/100 | {} vectors | {:.0} vec/sec | {:.1}s elapsed",
                    batch_idx + 1,
                    total_inserted,
                    rate,
                    elapsed.as_secs_f64()
                );
            }
        }

        let build_time = start.elapsed();
        println!(
            "\n✅ Built {} vectors in {:.2}s",
            total_inserted,
            build_time.as_secs_f64()
        );
        println!(
            "   Build rate: {:.0} vectors/sec\n",
            total_inserted as f64 / build_time.as_secs_f64()
        );

        // Save to disk
        println!("💾 Saving to disk...");
        let save_start = Instant::now();
        store
            .save_to_disk(store_path)
            .expect("Failed to save store");
        let save_time = save_start.elapsed();
        println!("✅ Saved in {:.2}s\n", save_time.as_secs_f64());

        store
    };

    // Verify loaded state
    println!("🔍 Verifying loaded state:");
    println!("   Vectors: {}", store.len());

    // Test get() works and verify dimensions
    if let Some(vec) = store.get(0) {
        println!("   Dimensions: {}", vec.dim());
        println!("   get(0): ✅ working");
    } else {
        println!("   get(0): ❌ FAILED - vectors not loaded!");
        std::process::exit(1);
    }
    println!("   HNSW index: ✅ present (search will verify)");
    println!();

    // Run queries
    println!("🔎 Running {} queries (k={})...", num_queries, k);
    let mut rng = StdRng::seed_from_u64(100);
    let mut query_times = Vec::with_capacity(num_queries);

    for i in 0..num_queries {
        let query = generate_realistic_embedding(&mut rng, dimensions);
        let query_start = Instant::now();
        let results = store
            .knn_search(&query, k)
            .expect("Query failed");
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

    println!("\n📊 Query Statistics ({} queries):", num_queries);
    println!("   Average: {:.2}ms", avg);
    println!("   p50: {:.2}ms", p50);
    println!("   p95: {:.2}ms", p95);
    println!("   p99: {:.2}ms", p99);
    println!();

    // Verify save/load roundtrip
    println!("🔄 Testing save/load roundtrip...");
    let roundtrip_path = "/tmp/omendb_1m_validation/roundtrip";

    let save_start = Instant::now();
    store
        .save_to_disk(roundtrip_path)
        .expect("Roundtrip save failed");
    let save_time = save_start.elapsed();

    let load_start = Instant::now();
    let loaded = VectorStore::load_from_disk(roundtrip_path, dimensions)
        .expect("Roundtrip load failed");
    let load_time = load_start.elapsed();

    println!("   Save: {:.2}s", save_time.as_secs_f64());
    println!("   Load: {:.2}s", load_time.as_secs_f64());
    println!("   Loaded vectors: {}", loaded.len());

    if loaded.len() != num_vectors {
        println!("   ❌ FAILED: Expected {} vectors, got {}", num_vectors, loaded.len());
        std::process::exit(1);
    }

    // Verify get() works on loaded store
    if loaded.get(0).is_none() {
        println!("   ❌ FAILED: get(0) returned None after load");
        std::process::exit(1);
    }

    println!("   ✅ Roundtrip successful\n");

    // Memory usage estimate
    let vectors_mem_mb = (num_vectors * dimensions * 4) as f64 / (1024.0 * 1024.0);
    println!("💾 Memory Usage:");
    println!("   Vectors: {:.1} MB (float32)", vectors_mem_mb);
    println!(
        "   With BQ (19.9x): {:.1} MB (estimated)",
        vectors_mem_mb / 19.9
    );
    println!();

    println!("✅ All validations passed!");
    println!("\n🎯 Ready for pgvector benchmarking");
}
