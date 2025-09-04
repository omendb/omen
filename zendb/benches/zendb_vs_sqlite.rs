use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;
use zendb::storage::{PageManager, BTree};
use zendb::transaction::TransactionManager;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Duration;

fn setup_sqlite(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_table (
            key BLOB PRIMARY KEY,
            value BLOB
        )",
        [],
    ).unwrap();
    
    // Optimize SQLite for fair comparison
    conn.execute("PRAGMA synchronous = NORMAL", []).unwrap();
    conn.execute("PRAGMA journal_mode = WAL", []).unwrap();
    conn.execute("PRAGMA cache_size = 10000", []).unwrap();
    conn.execute("PRAGMA page_size = 16384", []).unwrap(); // Match ZenDB page size
    
    conn
}

fn bench_sequential_inserts(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_inserts");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 1000, 10000].iter() {
        // ZenDB benchmark
        group.bench_with_input(BenchmarkId::new("ZenDB", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let db_path = temp_dir.path().join("zendb_bench.db");
                let pm = Arc::new(PageManager::open(&db_path).unwrap());
                let mut btree = BTree::create(pm.clone()).unwrap();
                
                for i in 0..size {
                    let key = format!("key{:08}", i);
                    let value = format!("value{}", i);
                    btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
                }
                
                pm.sync().unwrap();
            });
        });
        
        // SQLite benchmark
        group.bench_with_input(BenchmarkId::new("SQLite", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = TempDir::new().unwrap();
                let db_path = temp_dir.path().join("sqlite_bench.db");
                let conn = setup_sqlite(&db_path);
                
                let tx = conn.unchecked_transaction().unwrap();
                for i in 0..size {
                    let key = format!("key{:08}", i);
                    let value = format!("value{}", i);
                    tx.execute(
                        "INSERT INTO bench_table (key, value) VALUES (?1, ?2)",
                        [key.as_bytes(), value.as_bytes()],
                    ).unwrap();
                }
                tx.commit().unwrap();
            });
        });
    }
    
    group.finish();
}

fn bench_random_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_reads");
    group.measurement_time(Duration::from_secs(10));
    
    let size = 10000;
    
    // Setup ZenDB with data
    let temp_dir = TempDir::new().unwrap();
    let zendb_path = temp_dir.path().join("zendb_read.db");
    let pm = Arc::new(PageManager::open(&zendb_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }
    pm.sync().unwrap();
    
    // Setup SQLite with data
    let sqlite_path = temp_dir.path().join("sqlite_read.db");
    let conn = setup_sqlite(&sqlite_path);
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        tx.execute(
            "INSERT INTO bench_table (key, value) VALUES (?1, ?2)",
            [key.as_bytes(), value.as_bytes()],
        ).unwrap();
    }
    tx.commit().unwrap();
    
    // Create index for fair comparison
    conn.execute("CREATE INDEX idx_key ON bench_table(key)", []).unwrap();
    
    // Benchmark reads
    group.bench_function("ZenDB", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let i = rand::random::<usize>() % size;
                let key = format!("key{:08}", i);
                let _ = black_box(btree.search(key.as_bytes()));
            }
        });
    });
    
    group.bench_function("SQLite", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let i = rand::random::<usize>() % size;
                let key = format!("key{:08}", i);
                let mut stmt = conn.prepare_cached("SELECT value FROM bench_table WHERE key = ?1").unwrap();
                let _ = black_box(stmt.query_row([key.as_bytes()], |row| {
                    row.get::<_, Vec<u8>>(0)
                }));
            }
        });
    });
    
    group.finish();
}

fn bench_range_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("range_scan");
    group.measurement_time(Duration::from_secs(10));
    
    let size = 10000;
    
    // Setup ZenDB
    let temp_dir = TempDir::new().unwrap();
    let zendb_path = temp_dir.path().join("zendb_range.db");
    let pm = Arc::new(PageManager::open(&zendb_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }
    pm.sync().unwrap();
    
    // Setup SQLite
    let sqlite_path = temp_dir.path().join("sqlite_range.db");
    let conn = setup_sqlite(&sqlite_path);
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        tx.execute(
            "INSERT INTO bench_table (key, value) VALUES (?1, ?2)",
            [key.as_bytes(), value.as_bytes()],
        ).unwrap();
    }
    tx.commit().unwrap();
    conn.execute("CREATE INDEX idx_key ON bench_table(key)", []).unwrap();
    
    // Benchmark range scans
    group.bench_function("ZenDB", |b| {
        b.iter(|| {
            let start = format!("key{:08}", 1000);
            let end = format!("key{:08}", 2000);
            let results = btree.range_scan(start.as_bytes(), end.as_bytes()).unwrap();
            black_box(results);
        });
    });
    
    group.bench_function("SQLite", |b| {
        b.iter(|| {
            let start = format!("key{:08}", 1000);
            let end = format!("key{:08}", 2000);
            let mut stmt = conn.prepare_cached(
                "SELECT key, value FROM bench_table WHERE key >= ?1 AND key <= ?2 ORDER BY key"
            ).unwrap();
            let results: Vec<(Vec<u8>, Vec<u8>)> = stmt.query_map(
                [start.as_bytes(), end.as_bytes()],
                |row| Ok((row.get(0)?, row.get(1)?))
            ).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
            black_box(results);
        });
    });
    
    group.finish();
}

fn bench_transaction_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_throughput");
    group.measurement_time(Duration::from_secs(10));
    
    // ZenDB with MVCC
    group.bench_function("ZenDB_MVCC", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                let tm = TransactionManager::new();
                
                // Run 100 transactions
                for i in 0..100 {
                    let tx = tm.begin().await.unwrap();
                    let key = format!("key{}", i);
                    let value = format!("value{}", i);
                    tm.put(key.into_bytes(), value.into_bytes(), tx.id).await.unwrap();
                    tm.commit(tx).await.unwrap();
                }
            });
        });
    });
    
    // SQLite transactions
    group.bench_function("SQLite", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let db_path = temp_dir.path().join("sqlite_tx.db");
            let conn = setup_sqlite(&db_path);
            
            // Run 100 transactions
            for i in 0..100 {
                let tx = conn.unchecked_transaction().unwrap();
                let key = format!("key{}", i);
                let value = format!("value{}", i);
                tx.execute(
                    "INSERT INTO bench_table (key, value) VALUES (?1, ?2)",
                    [key.as_bytes(), value.as_bytes()],
                ).unwrap();
                tx.commit().unwrap();
            }
        });
    });
    
    group.finish();
}

fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_reads");
    group.measurement_time(Duration::from_secs(10));
    
    let size = 10000;
    
    // Setup ZenDB
    let temp_dir = TempDir::new().unwrap();
    let zendb_path = temp_dir.path().join("zendb_concurrent.db");
    let pm = Arc::new(PageManager::open(&zendb_path).unwrap());
    let mut btree = BTree::create(pm.clone()).unwrap();
    
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        btree.insert(key.as_bytes(), value.as_bytes()).unwrap();
    }
    pm.sync().unwrap();
    
    let btree = Arc::new(btree);
    
    // Setup SQLite (with shared cache for concurrency)
    let sqlite_path = temp_dir.path().join("sqlite_concurrent.db");
    let conn = Connection::open(&sqlite_path).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bench_table (key BLOB PRIMARY KEY, value BLOB)",
        [],
    ).unwrap();
    
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..size {
        let key = format!("key{:08}", i);
        let value = format!("value{}", i);
        tx.execute(
            "INSERT INTO bench_table (key, value) VALUES (?1, ?2)",
            [key.as_bytes(), value.as_bytes()],
        ).unwrap();
    }
    tx.commit().unwrap();
    
    // Benchmark concurrent reads
    group.bench_function("ZenDB", |b| {
        b.iter(|| {
            // Simulate 4 concurrent readers
            std::thread::scope(|s| {
                for _ in 0..4 {
                    let btree = btree.clone();
                    s.spawn(move || {
                        for _ in 0..25 {
                            let i = rand::random::<usize>() % size;
                            let key = format!("key{:08}", i);
                            let _ = black_box(btree.search(key.as_bytes()));
                        }
                    });
                }
            });
        });
    });
    
    group.bench_function("SQLite", |b| {
        b.iter(|| {
            // SQLite with multiple connections for concurrency
            std::thread::scope(|s| {
                for _ in 0..4 {
                    let path = sqlite_path.clone();
                    s.spawn(move || {
                        let conn = Connection::open(&path).unwrap();
                        for _ in 0..25 {
                            let i = rand::random::<usize>() % size;
                            let key = format!("key{:08}", i);
                            let mut stmt = conn.prepare_cached(
                                "SELECT value FROM bench_table WHERE key = ?1"
                            ).unwrap();
                            let _ = black_box(stmt.query_row([key.as_bytes()], |row| {
                                row.get::<_, Vec<u8>>(0)
                            }));
                        }
                    });
                }
            });
        });
    });
    
    group.finish();
}

// Add rand for random number generation
use rand;

criterion_group!(
    benches,
    bench_sequential_inserts,
    bench_random_reads,
    bench_range_scan,
    bench_transaction_throughput,
    bench_concurrent_reads
);

criterion_main!(benches);