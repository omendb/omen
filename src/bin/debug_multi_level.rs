//! Debug multi-level ALEX to find the issue

use anyhow::Result;
use omendb::alex::MultiLevelAlexTree;

fn main() -> Result<()> {
    println!("Debug test with small dataset\n");

    // Very small test
    let data = vec![
        (10, vec![1]),
        (20, vec![2]),
        (30, vec![3]),
        (40, vec![4]),
        (50, vec![5]),
    ];

    println!("Building tree with: {:?}", data.iter().map(|(k, _)| k).collect::<Vec<_>>());
    let tree = MultiLevelAlexTree::bulk_build(data)?;

    println!("\nTree stats:");
    println!("  Total keys: {}", tree.len());
    println!("  Height: {}", tree.height());
    println!("  Leaves: {}", tree.num_leaves());

    // Test each key
    println!("\nTesting queries:");
    for key in vec![10, 20, 30, 40, 50, 60] {
        match tree.get(key)? {
            Some(value) => println!("  Key {}: Found value {:?}", key, value),
            None => println!("  Key {}: NOT FOUND", key),
        }
    }

    // Test with sequential keys
    println!("\n\nTest 2: Sequential keys");
    let mut data2 = Vec::new();
    for i in 0..20 {
        data2.push((i, vec![i as u8]));
    }

    let tree2 = MultiLevelAlexTree::bulk_build(data2)?;
    println!("Built tree with {} keys", tree2.len());

    let mut found = 0;
    let mut missing = Vec::new();
    for i in 0..20 {
        if tree2.get(i)?.is_some() {
            found += 1;
        } else {
            missing.push(i);
        }
    }

    println!("Found {}/20 keys", found);
    if !missing.is_empty() {
        println!("Missing keys: {:?}", missing);
    }

    // Test 3: Force multiple leaves (>64 keys)
    println!("\n\nTest 3: Multiple leaves (200 keys)");
    let mut data3 = Vec::new();
    for i in 0..200 {
        data3.push((i * 10, vec![i as u8]));
    }

    let tree3 = MultiLevelAlexTree::bulk_build(data3)?;
    println!("Built tree with {} keys, {} leaves, height {}",
             tree3.len(), tree3.num_leaves(), tree3.height());

    let mut found = 0;
    let mut missing = Vec::new();
    for i in 0..200 {
        let key = i * 10;
        if tree3.get(key)?.is_some() {
            found += 1;
        } else {
            missing.push(key);
        }
    }

    println!("Found {}/200 keys", found);
    if !missing.is_empty() {
        println!("Missing keys (showing first 10): {:?}", &missing[..missing.len().min(10)]);
    }

    Ok(())
}