#!/bin/bash

# Quick start script for standalone learned database

echo "🚀 Starting OmenDB Standalone Database Project"

# Create new Rust project
echo "Creating learneddb project..."
cargo new learneddb --lib

cd learneddb

# Add dependencies
echo "Adding dependencies..."
cargo add rocksdb serde bincode tokio --features tokio/full
cargo add --dev criterion

# Copy learned index code
echo "Copying learned index implementations..."
cp ../src/linear.rs src/
cp ../src/rmi.rs src/
cp ../src/error.rs src/

# Create basic structure
cat > src/lib.rs << 'EOF'
pub mod linear;
pub mod rmi;
pub mod error;
pub mod storage;
pub mod db;

pub use linear::LinearIndex;
pub use rmi::RMIIndex;
pub use error::{Error, Result};
pub use db::LearnedDB;
EOF

# Create storage module
cat > src/storage.rs << 'EOF'
use rocksdb::{DB, Options};
use crate::Result;

pub struct Storage {
    db: DB,
}

impl Storage {
    pub fn open(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path)
            .map_err(|e| crate::Error::Storage(e.to_string()))?;

        Ok(Storage { db })
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)
            .map_err(|e| crate::Error::Storage(e.to_string()))
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.db.get(key)
            .map_err(|e| crate::Error::Storage(e.to_string()))
    }
}
EOF

# Create main DB module
cat > src/db.rs << 'EOF'
use crate::{storage::Storage, LinearIndex, Result};
use std::collections::HashMap;

pub struct LearnedDB {
    storage: Storage,
    indexes: HashMap<String, LinearIndex<Vec<u8>>>,
}

impl LearnedDB {
    pub fn open(path: &str) -> Result<Self> {
        Ok(LearnedDB {
            storage: Storage::open(path)?,
            indexes: HashMap::new(),
        })
    }

    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<()> {
        // For now, just use storage directly
        // TODO: Update learned indexes
        self.storage.put(&key.to_le_bytes(), &value)
    }

    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        // TODO: Use learned index for faster lookup
        self.storage.get(&key.to_le_bytes())
    }
}
EOF

# Update error module
cat >> src/error.rs << 'EOF'

#[derive(Debug)]
pub enum Error {
    Storage(String),
    InvalidData(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Storage(msg) => write!(f, "Storage error: {}", msg),
            Error::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
EOF

# Create example
cat > examples/basic.rs << 'EOF'
use learneddb::LearnedDB;

fn main() -> learneddb::Result<()> {
    // Open database
    let mut db = LearnedDB::open("./test.db")?;

    // Insert some data
    db.insert(1, b"value1".to_vec())?;
    db.insert(2, b"value2".to_vec())?;
    db.insert(10, b"value10".to_vec())?;

    // Query data
    if let Some(value) = db.get(2)? {
        println!("Found: {}", String::from_utf8_lossy(&value));
    }

    Ok(())
}
EOF

# Create benchmark
cat > benches/lookup.rs << 'EOF'
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use learneddb::LearnedDB;

fn benchmark_lookups(c: &mut Criterion) {
    let mut db = LearnedDB::open("./bench.db").unwrap();

    // Insert test data
    for i in 0..10000 {
        db.insert(i, format!("value{}", i).into_bytes()).unwrap();
    }

    c.bench_function("learned_db_get", |b| {
        b.iter(|| {
            db.get(black_box(5000))
        });
    });
}

criterion_group!(benches, benchmark_lookups);
criterion_main!(benches);
EOF

echo "✅ Standalone database project created!"
echo ""
echo "Next steps:"
echo "  cd learneddb"
echo "  cargo build"
echo "  cargo run --example basic"
echo "  cargo bench"
echo ""
echo "The learned index implementations still need to be integrated with storage."
echo "This is your starting point for the standalone database."