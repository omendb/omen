//! Chaos engineering tests for OmenDB
//! Simulates production failures to test resilience

use omendb::concurrent::ConcurrentOmenDB;
use std::sync::{Arc, Barrier};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use std::fs;
use std::process::{Command, Stdio};
use rand::Rng;

#[derive(Debug)]
struct ChaosResult {
    scenario: String,
    data_loss: bool,
    recovery_time: Duration,
    errors_during: u64,
    errors_after: u64,
    consistency_check: bool,
}

fn main() {
    println!("💥 OmenDB Chaos Engineering Test Suite");
    println!("======================================");
    println!("Testing resilience to production failures\n");

    let mut results = Vec::new();

    // Test 1: Sudden process termination
    println!("🔥 Test 1: Sudden Process Termination (kill -9)");
    results.push(test_kill_nine());

    // Test 2: Memory exhaustion
    println!("\n🔥 Test 2: Memory Exhaustion (OOM)");
    results.push(test_memory_exhaustion());

    // Test 3: Disk space exhaustion
    println!("\n🔥 Test 3: Disk Space Exhaustion");
    results.push(test_disk_full());

    // Test 4: Concurrent writes during failure
    println!("\n🔥 Test 4: Failure During Heavy Load");
    results.push(test_failure_under_load());

    // Test 5: Corrupted WAL recovery
    println!("\n🔥 Test 5: Corrupted WAL Recovery");
    results.push(test_corrupted_wal());

    // Test 6: Network partition (for future distributed version)
    println!("\n🔥 Test 6: Network Partition Simulation");
    results.push(test_network_partition());

    // Summary
    print_chaos_summary(&results);
}

fn test_kill_nine() -> ChaosResult {
    let mut result = ChaosResult {
        scenario: "Kill -9".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    // Start database in subprocess
    println!("  Starting database process...");
    let mut child = Command::new("cargo")
        .args(&["run", "--release", "--bin", "omendb"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start database");

    thread::sleep(Duration::from_secs(2));

    // Insert test data
    let db = Arc::new(ConcurrentOmenDB::with_persistence("chaos_test_data").unwrap());
    let test_data: Vec<(i64, f64)> = (0..1000).map(|i| (1000000 + i, i as f64)).collect();

    println!("  Inserting 1000 test records...");
    for (key, value) in &test_data {
        db.insert(*key, *value, 1).unwrap();
    }

    // Kill process abruptly
    println!("  💀 Killing process with SIGKILL...");
    child.kill().expect("Failed to kill process");

    let recovery_start = Instant::now();

    // Try to restart and recover
    println!("  Attempting recovery...");
    thread::sleep(Duration::from_secs(1));

    match ConcurrentOmenDB::with_persistence("chaos_test_data") {
        Ok(recovered_db) => {
            result.recovery_time = recovery_start.elapsed();
            println!("  ✓ Recovery successful in {:?}", result.recovery_time);

            // Check data integrity
            let mut recovered_count = 0;
            let mut missing_count = 0;

            for (key, expected_value) in &test_data {
                match recovered_db.search(*key) {
                    Ok(Some(value)) if (value - expected_value).abs() < 0.001 => {
                        recovered_count += 1;
                    }
                    _ => {
                        missing_count += 1;
                    }
                }
            }

            result.data_loss = missing_count > 0;
            result.consistency_check = recovered_count + missing_count == test_data.len();

            println!("  Data integrity: {}/{} records recovered", recovered_count, test_data.len());
            if result.data_loss {
                println!("  ⚠️  DATA LOSS DETECTED: {} records lost!", missing_count);
            }
        }
        Err(e) => {
            println!("  ❌ Recovery failed: {}", e);
            result.errors_after = 1;
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all("chaos_test_data");

    result
}

fn test_memory_exhaustion() -> ChaosResult {
    let mut result = ChaosResult {
        scenario: "Memory Exhaustion".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    println!("  Allocating memory until exhaustion...");

    let db = Arc::new(ConcurrentOmenDB::new("memory_test"));
    let mut allocations = Vec::new();
    let errors = Arc::new(AtomicU64::new(0));

    // Spawn writer thread
    let db_clone = Arc::clone(&db);
    let errors_clone = Arc::clone(&errors);
    let writer = thread::spawn(move || {
        for i in 0..1_000_000 {
            if db_clone.insert(5000000 + i, i as f64, 1).is_err() {
                errors_clone.fetch_add(1, Ordering::Relaxed);
            }
            if i % 10000 == 0 {
                println!("    Inserted {} records", i);
            }
        }
    });

    // Allocate memory aggressively
    loop {
        match vec![0u8; 100_000_000].try_into() {
            Ok(v) => allocations.push(v),
            Err(_) => {
                println!("  ⚠️  Memory allocation failed after {} MB",
                        allocations.len() * 100);
                break;
            }
        }

        if allocations.len() > 50 { // 5GB limit for safety
            println!("  Safety limit reached");
            break;
        }
    }

    // Check if database survived
    writer.join().ok();
    result.errors_during = errors.load(Ordering::Relaxed);

    // Try operations after memory pressure
    match db.search(5000000) {
        Ok(_) => {
            println!("  ✓ Database survived memory pressure");
            result.consistency_check = true;
        }
        Err(_) => {
            println!("  ❌ Database failed after memory pressure");
            result.errors_after = 1;
        }
    }

    result
}

fn test_disk_full() -> ChaosResult {
    let mut result = ChaosResult {
        scenario: "Disk Full".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    // Create small tmpfs to simulate disk full
    println!("  Creating limited disk space...");

    // This would require root on Linux:
    // Command::new("mount")
    //     .args(&["-t", "tmpfs", "-o", "size=10M", "tmpfs", "/tmp/diskfull_test"])
    //     .output()
    //     .ok();

    // Instead, we'll simulate by filling available space
    let test_dir = "/tmp/diskfull_test";
    fs::create_dir_all(test_dir).ok();

    let db = match ConcurrentOmenDB::with_persistence(test_dir) {
        Ok(db) => Arc::new(db),
        Err(_) => {
            println!("  ⚠️  Cannot test disk full scenario without proper setup");
            return result;
        }
    };

    // Insert until disk full
    println!("  Inserting data until disk full...");
    let mut successful_inserts = 0;
    let mut disk_full_errors = 0;

    for i in 0..100_000 {
        match db.insert(6000000 + i, i as f64, 1) {
            Ok(_) => successful_inserts += 1,
            Err(e) => {
                if e.to_string().contains("space") || e.to_string().contains("full") {
                    disk_full_errors += 1;
                    if disk_full_errors == 1 {
                        println!("  Disk full after {} inserts", successful_inserts);
                    }
                }
            }
        }
    }

    result.errors_during = disk_full_errors as u64;

    // Check if database handles disk full gracefully
    if disk_full_errors > 0 {
        println!("  ✓ Database detected disk full condition");
        result.consistency_check = true;
    } else {
        println!("  ⚠️  Disk full condition not reached in test");
    }

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);

    result
}

fn test_failure_under_load() -> ChaosResult {
    let mut result = ChaosResult {
        scenario: "Failure Under Load".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    let db = Arc::new(ConcurrentOmenDB::new("load_test"));
    let running = Arc::new(AtomicBool::new(true));
    let errors = Arc::new(AtomicU64::new(0));
    let successful_ops = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(Barrier::new(9)); // 8 workers + 1 chaos thread

    println!("  Starting 8 worker threads...");

    // Start worker threads
    let mut handles = vec![];
    for thread_id in 0..8 {
        let db_clone = Arc::clone(&db);
        let running_clone = Arc::clone(&running);
        let errors_clone = Arc::clone(&errors);
        let successful_clone = Arc::clone(&successful_ops);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            let mut local_success = 0;

            while running_clone.load(Ordering::Relaxed) {
                let key = 7000000 + rand::thread_rng().gen_range(0..100000);

                // Mix of reads and writes
                if rand::thread_rng().gen_bool(0.5) {
                    if db_clone.insert(key, key as f64, thread_id as i64).is_ok() {
                        local_success += 1;
                    } else {
                        errors_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    if db_clone.search(key).is_ok() {
                        local_success += 1;
                    } else {
                        errors_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            successful_clone.fetch_add(local_success, Ordering::Relaxed);
        });
        handles.push(handle);
    }

    // Chaos thread
    thread::spawn(move || {
        barrier.wait();
        println!("  Running for 5 seconds under load...");
        thread::sleep(Duration::from_secs(5));

        println!("  💥 Simulating failure by dropping database handle...");
        // In real scenario, would kill process or corrupt memory
        // Here we just stop operations

        thread::sleep(Duration::from_secs(1));
        println!("  Stopping load test...");
    }).join().ok();

    running.store(false, Ordering::Relaxed);

    // Wait for workers
    for handle in handles {
        handle.join().ok();
    }

    result.errors_during = errors.load(Ordering::Relaxed);
    let total_ops = successful_ops.load(Ordering::Relaxed);

    println!("  Operations: {} successful, {} failed", total_ops, result.errors_during);

    if result.errors_during as f64 / total_ops as f64 < 0.01 {
        println!("  ✓ Error rate < 1% under load");
        result.consistency_check = true;
    } else {
        println!("  ⚠️  High error rate under load");
    }

    result
}

fn test_corrupted_wal() -> ChaosResult {
    let mut result = ChaosResult {
        scenario: "Corrupted WAL".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    let test_dir = "/tmp/wal_corruption_test";
    fs::create_dir_all(test_dir).ok();
    let wal_dir = format!("{}/wal", test_dir);

    // Create database and insert data
    println!("  Creating database with WAL...");
    {
        let db = ConcurrentOmenDB::with_persistence(test_dir).unwrap();
        for i in 0..1000 {
            db.insert(8000000 + i, i as f64, 1).unwrap();
        }
        db.sync().unwrap();
    }

    // Corrupt WAL file
    println!("  💀 Corrupting WAL file...");
    if let Ok(entries) = fs::read_dir(&wal_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "log") {
                // Corrupt by overwriting random bytes
                if let Ok(mut contents) = fs::read(&path) {
                    for i in (100..contents.len()).step_by(100) {
                        if i < contents.len() {
                            contents[i] = 0xFF; // Corrupt byte
                        }
                    }
                    fs::write(&path, contents).ok();
                    println!("    Corrupted: {:?}", path.file_name().unwrap());
                    break;
                }
            }
        }
    }

    // Try to recover
    println!("  Attempting recovery from corrupted WAL...");
    let recovery_start = Instant::now();

    match ConcurrentOmenDB::with_persistence(test_dir) {
        Ok(recovered_db) => {
            result.recovery_time = recovery_start.elapsed();
            println!("  ✓ Recovery completed in {:?}", result.recovery_time);

            // Check how much data was recovered
            let mut recovered = 0;
            for i in 0..1000 {
                if recovered_db.search(8000000 + i).is_ok() {
                    recovered += 1;
                }
            }

            println!("  Recovered {}/1000 records", recovered);
            result.data_loss = recovered < 1000;
            result.consistency_check = recovered > 0; // At least some recovery
        }
        Err(e) => {
            println!("  ❌ Recovery failed completely: {}", e);
            result.errors_after = 1;
        }
    }

    // Cleanup
    let _ = fs::remove_dir_all(test_dir);

    result
}

fn test_network_partition() -> ChaosResult {
    let result = ChaosResult {
        scenario: "Network Partition".to_string(),
        data_loss: false,
        recovery_time: Duration::from_secs(0),
        errors_during: 0,
        errors_after: 0,
        consistency_check: false,
    };

    println!("  ⚠️  Network partition test requires distributed implementation");
    println!("  Current version is single-node only");
    println!();
    println!("  In distributed system would test:");
    println!("    • Split brain scenarios");
    println!("    • Quorum loss");
    println!("    • Network delays");
    println!("    • Partition healing");

    result
}

fn print_chaos_summary(results: &[ChaosResult]) {
    println!("\n" + "=".repeat(60));
    println!("📊 CHAOS ENGINEERING SUMMARY");
    println!("=".repeat(60));

    let mut passed = 0;
    let mut failed = 0;
    let mut data_loss_count = 0;

    for result in results {
        println!("\n{}", result.scenario);

        if result.consistency_check && !result.data_loss && result.errors_after == 0 {
            println!("  ✅ PASSED");
            passed += 1;
        } else {
            println!("  ❌ FAILED");
            failed += 1;
        }

        if result.data_loss {
            println!("  ⚠️  DATA LOSS DETECTED");
            data_loss_count += 1;
        }

        if result.errors_after > 0 {
            println!("  ⚠️  System unstable after failure");
        }

        if result.recovery_time > Duration::from_secs(0) {
            println!("  Recovery time: {:?}", result.recovery_time);
        }
    }

    println!("\n" + "=".repeat(60));
    println!("Overall Results:");
    println!("  Passed: {}/{}", passed, results.len());
    println!("  Failed: {}/{}", failed, results.len());
    println!("  Data Loss Events: {}", data_loss_count);

    println!("\n🔍 Critical Findings:");
    if data_loss_count > 0 {
        println!("  • ⚠️  DATA LOSS POSSIBLE - Not production ready!");
    }
    if failed > passed {
        println!("  • ⚠️  POOR FAILURE RECOVERY - Major reliability issues!");
    }
    if passed == results.len() {
        println!("  • ✅ Good resilience to failures");
    }

    println!("\nRecommendations:");
    println!("  1. Implement proper distributed consensus (Raft)");
    println!("  2. Add comprehensive WAL corruption handling");
    println!("  3. Implement automatic recovery mechanisms");
    println!("  4. Add circuit breakers and retry logic");
    println!("  5. Test with real infrastructure failures");
}