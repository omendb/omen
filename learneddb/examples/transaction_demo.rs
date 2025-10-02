use learneddb::{IndexType, IsolationLevel, OmenDB};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

fn main() -> learneddb::Result<()> {
    println!("🚀 OmenDB Transaction Demo");
    println!("==========================\n");

    // Create database with learned indexes
    let db_path = "./txn_demo.db";
    let db = Arc::new(OmenDB::open_with_index(db_path, IndexType::Linear)?);

    // Initial data setup
    println!("📝 Setting up initial data...");
    let mut initial_data = Vec::new();
    for i in 0..1000 {
        initial_data.push((i * 2, format!("initial_value_{}", i).into_bytes()));
    }
    Arc::get_mut(&mut db.clone())
        .unwrap()
        .bulk_insert(initial_data)?;

    // Demo 1: Basic transaction commit
    println!("\n✅ Demo 1: Basic Transaction Commit");
    println!("------------------------------------");
    let txn1 = db.begin_transaction(IsolationLevel::ReadCommitted);
    println!("Transaction {} started", txn1);

    db.txn_put(txn1, 2000, b"txn_value_1".to_vec())?;
    db.txn_put(txn1, 2002, b"txn_value_2".to_vec())?;
    println!("Added 2 values in transaction");

    // Read within transaction (read-your-writes)
    match db.txn_get(txn1, 2000)? {
        Some(val) => println!(
            "Read within txn: key=2000, value={}",
            String::from_utf8_lossy(&val)
        ),
        None => println!("Key 2000 not found"),
    }

    db.commit(txn1)?;
    println!("Transaction {} committed", txn1);

    // Verify committed data
    match db.get(2000)? {
        Some(val) => println!(
            "After commit: key=2000, value={}",
            String::from_utf8_lossy(&val)
        ),
        None => println!("Key 2000 not found after commit"),
    }

    // Demo 2: Transaction rollback
    println!("\n❌ Demo 2: Transaction Rollback");
    println!("--------------------------------");
    let txn2 = db.begin_transaction(IsolationLevel::ReadCommitted);
    println!("Transaction {} started", txn2);

    db.txn_put(txn2, 3000, b"will_be_rolled_back".to_vec())?;
    println!("Added value that will be rolled back");

    db.rollback(txn2)?;
    println!("Transaction {} rolled back", txn2);

    // Verify rollback
    match db.get(3000)? {
        Some(_) => println!("ERROR: Rolled back data still exists!"),
        None => println!("✅ Rollback successful: key 3000 not found"),
    }

    // Demo 3: Isolation levels
    println!("\n🔒 Demo 3: Transaction Isolation");
    println!("---------------------------------");

    let db_clone = Arc::clone(&db);

    // Start transaction in thread 1
    let handle1 = thread::spawn(move || {
        let txn = db_clone.begin_transaction(IsolationLevel::ReadCommitted);
        println!("Thread 1: Transaction {} started", txn);

        // Write data
        db_clone
            .txn_put(txn, 4000, b"thread1_data".to_vec())
            .unwrap();
        println!("Thread 1: Wrote key=4000");

        // Sleep to simulate work
        thread::sleep(std::time::Duration::from_millis(100));

        // Commit
        db_clone.commit(txn).unwrap();
        println!("Thread 1: Transaction {} committed", txn);
    });

    // Start transaction in thread 2
    let db_clone2 = Arc::clone(&db);
    let handle2 = thread::spawn(move || {
        // Wait a bit to ensure thread 1 starts first
        thread::sleep(std::time::Duration::from_millis(50));

        let txn = db_clone2.begin_transaction(IsolationLevel::ReadCommitted);
        println!("Thread 2: Transaction {} started", txn);

        // Try to read uncommitted data
        match db_clone2.txn_get(txn, 4000).unwrap() {
            Some(_) => println!("Thread 2: ❌ Read uncommitted data (isolation violated)"),
            None => println!("Thread 2: ✅ Cannot read uncommitted data (isolation working)"),
        }

        // Wait for thread 1 to commit
        thread::sleep(std::time::Duration::from_millis(100));

        // Now should be able to read
        match db_clone2.txn_get(txn, 4000).unwrap() {
            Some(val) => println!(
                "Thread 2: ✅ Read committed data: {}",
                String::from_utf8_lossy(&val)
            ),
            None => println!("Thread 2: ❌ Cannot read committed data"),
        }

        db_clone2.commit(txn).unwrap();
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    // Demo 4: Performance with transactions
    println!("\n⚡ Demo 4: Transaction Performance");
    println!("----------------------------------");

    let num_txns = 100;
    let ops_per_txn = 10;

    let start = Instant::now();
    for i in 0..num_txns {
        let txn = db.begin_transaction(IsolationLevel::ReadCommitted);

        for j in 0..ops_per_txn {
            let key = 10000 + (i * ops_per_txn + j) as i64;
            db.txn_put(txn, key, format!("perf_test_{}", key).into_bytes())?;
        }

        db.commit(txn)?;
    }
    let elapsed = start.elapsed();

    let total_ops = num_txns * ops_per_txn;
    let throughput = total_ops as f64 / elapsed.as_secs_f64();

    println!("Transactions: {}", num_txns);
    println!("Operations per transaction: {}", ops_per_txn);
    println!("Total operations: {}", total_ops);
    println!("Time: {:?}", elapsed);
    println!("Throughput: {:.0} ops/sec", throughput);

    // Demo 5: MVCC with multiple versions
    println!("\n📚 Demo 5: Multi-Version Concurrency Control");
    println!("--------------------------------------------");

    // Update same key multiple times
    let key = 5000;
    for i in 0..3 {
        let txn = db.begin_transaction(IsolationLevel::ReadCommitted);
        db.txn_put(txn, key, format!("version_{}", i).into_bytes())?;
        db.commit(txn)?;
        println!("Committed version {} for key {}", i, key);
    }

    // Latest version should be visible
    match db.get(key)? {
        Some(val) => println!("Current value: {}", String::from_utf8_lossy(&val)),
        None => println!("Key not found"),
    }

    println!("\n✅ Transaction demo completed successfully!");
    println!("\nDatabase statistics:");
    println!("{}", db.stats());

    Ok(())
}
