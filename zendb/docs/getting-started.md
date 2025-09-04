# Getting Started with ZenDB

Welcome to ZenDB - the database that grows with you. This guide will help you get started with ZenDB in just a few minutes.

## 🧘 Philosophy

ZenDB follows zen principles:
- **Start simple**: Begin with embedded mode, no configuration needed
- **Grow naturally**: Scale to distributed when you need it
- **Find balance**: Perfect harmony between features and simplicity

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/nijaru/zendb.git
cd zendb

# Build with Rust
cargo build --release

# Run tests to verify
cargo test
```

### Using Cargo

```bash
# Add to your Cargo.toml
[dependencies]
zendb = "0.1.0"
```

## 🚀 Quick Start

### Embedded Mode (Development)

```rust
use zendb::{Config, ZenDB};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create embedded database with default config
    let db = ZenDB::new()?;
    
    // Execute SQL queries
    let results = db.query("SELECT * FROM users").await?;
    
    Ok(())
}
```

### Distributed Mode (Production)

```rust
use zendb::{Config, ZenDB};

let config = Config {
    distributed: true,
    bind_address: Some("0.0.0.0:5432".to_string()),
    cluster_peers: vec![
        "node1.example.com:5432".to_string(),
        "node2.example.com:5432".to_string(),
    ],
    ..Default::default()
};

let db = ZenDB::with_config(config)?;
```

## 🎯 Key Features

### Time-Travel Queries

Debug production issues by querying past states:

```sql
-- See data as it was yesterday
SELECT * FROM orders AS OF TIMESTAMP '2025-01-01 10:00:00';

-- Track changes over time
SELECT * FROM users FOR SYSTEM_TIME BETWEEN 
  '2025-01-01' AND '2025-01-02' 
WHERE id = 123;
```

### Real-Time Subscriptions

Get live updates without polling:

```rust
let subscription = db.subscribe(
    "SELECT COUNT(*) FROM active_users WHERE last_seen > NOW() - INTERVAL '5 min'"
).await?;

subscription.on_change(|count| {
    println!("Active users: {}", count);
});
```

### Vector Search

Native support for AI/ML workloads:

```sql
SELECT product_name, 
       vector_distance(embedding, $1) as similarity
FROM products
WHERE price < 1000
  AND vector_distance(embedding, $1) < 0.8
ORDER BY similarity LIMIT 10;
```

## 🔧 Configuration

### Basic Configuration

```rust
use zendb::Config;

let config = Config {
    // Storage
    data_path: Some("myapp.zendb".to_string()),
    page_size: 16384,  // 16KB pages
    buffer_pool_mb: 128,
    
    // WAL
    enable_wal: true,
    
    // Distributed (optional)
    distributed: false,
    bind_address: None,
    cluster_peers: vec![],
};
```

### Environment Variables

```bash
# Set data directory
export ZENDB_DATA_PATH=/var/lib/zendb

# Enable distributed mode
export ZENDB_DISTRIBUTED=true
export ZENDB_BIND_ADDRESS=0.0.0.0:5432

# Set buffer pool size
export ZENDB_BUFFER_POOL_MB=256
```

## 🌐 Language Bindings

### Python

```python
import zendb

# Connect to database
db = zendb.connect("zendb://localhost:5432/mydb")

# Execute queries
users = db.query("SELECT * FROM users WHERE active = true")

# Real-time subscriptions
@db.subscribe("SELECT COUNT(*) FROM orders WHERE status = 'pending'")
def on_pending_orders(count):
    print(f"Pending orders: {count}")
```

### Node.js

```javascript
const { ZenDB } = require('zendb');

const db = new ZenDB({
  embedded: true,
  dataPath: './myapp.zendb'
});

// Async queries
const users = await db.query('SELECT * FROM users');

// Subscriptions
db.subscribe('SELECT * FROM messages', (messages) => {
  console.log('New messages:', messages);
});
```

## 🔍 Monitoring

### Built-in Metrics

```rust
// Get database metrics
let metrics = db.metrics();
println!("Queries/sec: {}", metrics.queries_per_second);
println!("Cache hit ratio: {:.2}%", metrics.cache_hit_ratio * 100.0);
println!("Active connections: {}", metrics.active_connections);
```

### Prometheus Integration

ZenDB exposes metrics at `/metrics` endpoint in Prometheus format.

## 🚦 Next Steps

1. **Run Examples**: Check out the `examples/` directory
2. **Read Architecture**: Understand the [architecture](architecture.md)
3. **Join Community**: Discord and GitHub Discussions
4. **Contribute**: See [CONTRIBUTING.md](../CONTRIBUTING.md)

## 🆘 Getting Help

- **Documentation**: [docs.zendb.io](https://docs.zendb.io)
- **GitHub Issues**: [github.com/nijaru/zendb/issues](https://github.com/nijaru/zendb/issues)
- **Discord**: [discord.gg/zendb](https://discord.gg/zendb)

---

**Find zen in your data's natural flow** 🧘