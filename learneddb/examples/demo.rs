use learneddb::LearnedDB;
use std::time::Instant;

fn main() -> learneddb::Result<()> {
    println!("OmenDB Standalone Database Demo\n");

    // Open database
    let mut db = LearnedDB::open("./demo.db")?;

    // Insert test data
    println!("Inserting 10,000 records...");
    let mut data = Vec::new();
    for i in 0..10_000 {
        data.push((i, format!("value_{}", i).into_bytes()));
    }

    let start = Instant::now();
    db.bulk_insert(data)?;
    println!("Bulk insert took: {:?}\n", start.elapsed());

    // Test lookups
    println!("Testing lookups...");
    let start = Instant::now();
    for i in (0..1000).step_by(10) {
        if let Some(value) = db.get(i)? {
            if i < 50 {  // Only print first few
                println!("  Key {}: {}", i, String::from_utf8_lossy(&value));
            }
        }
    }
    println!("100 lookups took: {:?}\n", start.elapsed());

    // Print stats
    println!("{}", db.stats());

    Ok(())
}
