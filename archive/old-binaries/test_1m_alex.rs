use omendb::alex::AlexTree;
use rand::{Rng, SeedableRng};
use std::time::Instant;

fn main() {
    println!("Testing ALEX scaling...\n");

    for scale in &[100_000, 1_000_000] {
        println!("=== {} keys ===", scale);

        // Random keys (worst case)
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let entries: Vec<(i64, Vec<u8>)> = (0..*scale)
            .map(|_| (rng.gen::<i64>(), vec![1, 2, 3]))
            .collect();

        let start = Instant::now();
        let mut alex = AlexTree::new();
        alex.insert_batch(entries).unwrap();
        let elapsed = start.elapsed();

        println!("  Time: {:?}", elapsed);
        println!("  Per-key: {} ns", elapsed.as_nanos() / *scale as u128);
        println!("  Leaves: {}", alex.num_leaves());
        println!();
    }
}
