//! Quick test of multi-level ALEX at 100K scale
//!
//! A simpler test to verify the implementation works before
//! running large-scale benchmarks.

use anyhow::Result;
use omendb::alex::MultiLevelAlexTree;

fn main() -> Result<()> {
    println!("Testing multi-level ALEX with 100K keys...\n");

    // Generate sorted test data
    let size = 100_000;
    let mut data = Vec::with_capacity(size);

    for i in 0..size {
        data.push((i as i64 * 7, vec![i as u8; 4])); // Sparse keys
    }

    // Build tree
    println!("Building tree...");
    let tree = MultiLevelAlexTree::bulk_build(data.clone())?;

    println!("Tree built:");
    println!("  Keys: {}", tree.len());
    println!("  Height: {}", tree.height());
    println!("  Leaves: {}", tree.num_leaves());

    // Test queries
    println!("\nTesting queries...");
    let test_keys = vec![0, 7000, 140000, 350000, 693000];

    for key in test_keys {
        match tree.get(key)? {
            Some(value) => {
                let expected_idx = key / 7;
                if value[0] == (expected_idx % 256) as u8 {
                    println!("  ✅ Key {} found with correct value", key);
                } else {
                    println!("  ❌ Key {} found but value incorrect!", key);
                }
            }
            None => {
                if key < 700_000 && key % 7 == 0 {
                    println!("  ❌ Key {} should exist but not found!", key);
                } else {
                    println!("  ✅ Key {} correctly not found", key);
                }
            }
        }
    }

    // Test inserts
    println!("\nTesting inserts...");
    let mut tree = MultiLevelAlexTree::bulk_build(vec![(100, vec![1]), (200, vec![2])])?;

    tree.insert(150, vec![15])?;
    tree.insert(50, vec![5])?;
    tree.insert(250, vec![25])?;

    println!("After inserts, tree has {} keys", tree.len());

    // Verify inserts
    assert_eq!(tree.get(50)?, Some(vec![5]));
    assert_eq!(tree.get(100)?, Some(vec![1]));
    assert_eq!(tree.get(150)?, Some(vec![15]));
    assert_eq!(tree.get(200)?, Some(vec![2]));
    assert_eq!(tree.get(250)?, Some(vec![25]));

    println!("  ✅ All inserted keys found correctly");

    println!("\n✅ Multi-level ALEX basic tests pass!");

    Ok(())
}