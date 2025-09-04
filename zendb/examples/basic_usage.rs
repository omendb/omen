//! Basic usage example for ZenDB
//!
//! Run with: cargo run --example basic_usage

use zendb::{Config, ZenDB};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧘 ZenDB Basic Usage Example\n");
    
    // Create configuration for embedded mode
    let config = Config {
        data_path: Some("example.zendb".to_string()),
        distributed: false,
        enable_wal: true,
        page_size: 16384,
        buffer_pool_mb: 64,
        ..Default::default()
    };
    
    println!("📚 Creating embedded database...");
    let db = ZenDB::with_config(config)?;
    println!("✅ Database created successfully!\n");
    
    // TODO: Once SQL execution is implemented, demonstrate:
    // 1. Creating tables
    // 2. Inserting data
    // 3. Querying data
    // 4. Time-travel queries
    // 5. Real-time subscriptions
    
    println!("🎯 Example queries (to be implemented):\n");
    
    println!("-- Create a table");
    println!("CREATE TABLE users (");
    println!("    id SERIAL PRIMARY KEY,");
    println!("    name TEXT NOT NULL,");
    println!("    email TEXT UNIQUE,");
    println!("    created_at TIMESTAMP DEFAULT NOW()");
    println!(");");
    println!();
    
    println!("-- Insert data");
    println!("INSERT INTO users (name, email)");
    println!("VALUES ('Alice', 'alice@example.com');");
    println!();
    
    println!("-- Query data");
    println!("SELECT * FROM users WHERE email LIKE '%@example.com';");
    println!();
    
    println!("-- Time-travel query");
    println!("SELECT * FROM users AS OF TIMESTAMP '2025-01-01 10:00:00';");
    println!();
    
    println!("-- Vector similarity search");
    println!("SELECT name, vector_distance(embedding, $1) as similarity");
    println!("FROM products");
    println!("WHERE vector_distance(embedding, $1) < 0.8");
    println!("ORDER BY similarity LIMIT 10;");
    println!();
    
    println!("🧘 Find zen in your data's natural flow");
    
    Ok(())
}