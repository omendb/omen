//! Validate mmap read performance assumptions
//!
//! Before committing 10-12 weeks to custom storage, we need to validate
//! that mmap reads are actually 100-200ns as projected.
//!
//! This benchmark tests:
//! 1. Sequential mmap reads (best case)
//! 2. Random mmap reads (realistic case)
//! 3. Different value sizes (8B, 64B, 256B, 1KB)
//!
//! Run with: cargo run --release --bin benchmark_mmap_validation

use memmap2::Mmap;
use rand::{Rng, SeedableRng};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::Instant;
use tempfile::tempdir;

fn main() {
    println!("=== MMAP PERFORMANCE VALIDATION ===");
    println!("Testing if mmap reads are actually 100-200ns\n");

    // Test different value sizes
    for &value_size in &[8, 64, 256, 1024] {
        println!("\n=== VALUE SIZE: {} bytes ===", value_size);

        println!("\n1. Sequential Reads (Best Case)");
        benchmark_sequential_reads(100_000, value_size);

        println!("\n2. Random Reads (Realistic Case)");
        benchmark_random_reads(100_000, value_size);
    }

    println!("\n=== CONCLUSION ===");
    println!("Check if random reads are 100-200ns for realistic validation.");
}

fn benchmark_sequential_reads(n: usize, value_size: usize) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("data.bin");

    // Create test file with sequential data
    println!("  Creating test file with {} entries...", n);
    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .unwrap();

        // Write entries: [offset:8 bytes][value:N bytes]
        for i in 0..n {
            let offset = (i * (8 + value_size)) as u64;
            file.write_all(&offset.to_le_bytes()).unwrap();

            let value = vec![i as u8; value_size];
            file.write_all(&value).unwrap();
        }
        file.flush().unwrap();
    }

    // Memory-map the file
    let file = File::open(&file_path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };

    // Benchmark sequential reads
    println!("  Reading {} entries sequentially...", n);
    let start = Instant::now();
    let mut sum: u64 = 0;

    for i in 0..n {
        let entry_offset = i * (8 + value_size);

        // Read offset (8 bytes)
        let offset_bytes = &mmap[entry_offset..entry_offset + 8];
        let offset = u64::from_le_bytes(offset_bytes.try_into().unwrap());
        sum = sum.wrapping_add(offset);

        // Read value (N bytes) - simulate access
        let _value = &mmap[entry_offset + 8..entry_offset + 8 + value_size];
    }

    let elapsed = start.elapsed();
    let ns_per_read = elapsed.as_nanos() / n as u128;

    println!("  Time: {:?}", elapsed);
    println!("  Per-read: {} ns", ns_per_read);
    println!("  Throughput: {:.2}M reads/sec", n as f64 / elapsed.as_secs_f64() / 1_000_000.0);
    println!("  Sum (prevent optimization): {}", sum);

    if ns_per_read <= 200 {
        println!("  ✅ Within 100-200ns target");
    } else {
        println!("  ❌ Above 200ns target (actual: {}ns)", ns_per_read);
    }
}

fn benchmark_random_reads(n: usize, value_size: usize) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("data.bin");

    // Create test file with random data
    println!("  Creating test file with {} entries...", n);
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let mut offsets = Vec::with_capacity(n);

    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .unwrap();

        // Write entries at random positions
        for i in 0..n {
            let offset = (i * (8 + value_size)) as u64;
            offsets.push(offset);

            file.write_all(&offset.to_le_bytes()).unwrap();

            let value = vec![rng.gen::<u8>(); value_size];
            file.write_all(&value).unwrap();
        }
        file.flush().unwrap();
    }

    // Memory-map the file
    let file = File::open(&file_path).unwrap();
    let mmap = unsafe { Mmap::map(&file).unwrap() };

    // Generate random read order
    let num_reads = 10_000;
    let read_indices: Vec<usize> = (0..num_reads)
        .map(|_| rng.gen_range(0..n))
        .collect();

    // Benchmark random reads
    println!("  Reading {} random entries...", num_reads);
    let start = Instant::now();
    let mut sum: u64 = 0;

    for &idx in &read_indices {
        let entry_offset = idx * (8 + value_size);

        // Read offset (8 bytes)
        let offset_bytes = &mmap[entry_offset..entry_offset + 8];
        let offset = u64::from_le_bytes(offset_bytes.try_into().unwrap());
        sum = sum.wrapping_add(offset);

        // Read value (N bytes) - simulate access
        let _value = &mmap[entry_offset + 8..entry_offset + 8 + value_size];
    }

    let elapsed = start.elapsed();
    let ns_per_read = elapsed.as_nanos() / num_reads as u128;

    println!("  Time: {:?}", elapsed);
    println!("  Per-read: {} ns", ns_per_read);
    println!("  Throughput: {:.2}M reads/sec", num_reads as f64 / elapsed.as_secs_f64() / 1_000_000.0);
    println!("  Sum (prevent optimization): {}", sum);

    if ns_per_read <= 200 {
        println!("  ✅ Within 100-200ns target");
    } else if ns_per_read <= 500 {
        println!("  ⚠️  Above 200ns but acceptable (actual: {}ns)", ns_per_read);
    } else {
        println!("  ❌ Above 500ns - mmap may not help (actual: {}ns)", ns_per_read);
    }
}
