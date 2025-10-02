# OmenDB Architecture Refactor - Correct Design

**Date**: September 29, 2025
**Goal**: Build proper ordered data database, not time-series-only hack

---

## Current Problems

### 1. No Table Abstraction
```rust
// Current: Database IS a single table
pub struct OmenDB {
    index: RecursiveModelIndex,
    storage: ArrowStorage,  // Hardcoded schema
    name: String,
}
```
**Problem**: Can't have multiple tables with different schemas

### 2. Hardcoded Schema
```rust
// storage.rs
let schema = Schema::new(vec![
    Field::new("timestamp", ...),  // FIXED
    Field::new("value", ...),      // FIXED
    Field::new("series_id", ...),  // FIXED
]);
```
**Problem**: Can't CREATE TABLE with custom columns

### 3. Fixed API
```rust
pub fn insert(&mut self, timestamp: i64, value: f64, series_id: i64)
```
**Problem**: Can't insert into tables with different schemas

### 4. Index Coupled to Timestamps
```rust
pub fn add_key(timestamp: i64)  // Assumes timestamp
```
**Problem**: Can't index user_id, order_id, or other ordered columns

---

## Correct Architecture

### Core Abstractions

```rust
//=============================================================================
// Database - Top level
//=============================================================================

pub struct OmenDB {
    /// Catalog of all tables
    catalog: Catalog,

    /// Configuration
    config: DatabaseConfig,

    /// Data directory
    data_dir: PathBuf,
}

impl OmenDB {
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        // Initialize database
    }

    pub fn create_table(&mut self, name: &str, schema: SchemaRef,
                        primary_key: &str) -> Result<()> {
        // CREATE TABLE ...
    }

    pub fn get_table(&self, name: &str) -> Result<&Table> {
        // Get table reference
    }

    pub fn execute_query(&self, sql: &str) -> Result<QueryResult> {
        // Execute SQL query
    }
}

//=============================================================================
// Catalog - Manages all tables
//=============================================================================

pub struct Catalog {
    /// All tables by name
    tables: HashMap<String, Table>,

    /// Metadata persistence
    metadata_file: PathBuf,
}

impl Catalog {
    pub fn create_table(&mut self, name: String, schema: SchemaRef,
                        primary_key: String, data_dir: PathBuf) -> Result<()> {
        // Validate primary key is ordered type
        validate_primary_key(&schema, &primary_key)?;

        // Create table
        let table = Table::new(name.clone(), schema, primary_key, data_dir)?;
        self.tables.insert(name, table);

        // Persist catalog metadata
        self.save_metadata()?;

        Ok(())
    }

    pub fn get_table(&self, name: &str) -> Result<&Table> {
        self.tables.get(name)
            .ok_or_else(|| anyhow!("Table {} not found", name))
    }

    pub fn drop_table(&mut self, name: &str) -> Result<()> {
        // DROP TABLE ...
    }

    fn save_metadata(&self) -> Result<()> {
        // Persist catalog to disk
    }

    fn load_metadata(&mut self) -> Result<()> {
        // Load catalog from disk on startup
    }
}

//=============================================================================
// Table - Core abstraction
//=============================================================================

pub struct Table {
    /// Table name
    name: String,

    /// Schema (user-defined columns)
    schema: SchemaRef,

    /// Primary key column name
    primary_key: String,

    /// Primary key data type
    primary_key_type: DataType,

    /// Storage layer
    storage: TableStorage,

    /// Learned index on primary key
    index: TableIndex,

    /// Statistics
    row_count: usize,
}

impl Table {
    pub fn new(name: String, schema: SchemaRef,
               primary_key: String, data_dir: PathBuf) -> Result<Self> {
        // Validate primary key exists in schema
        let pk_field = schema.field_with_name(&primary_key)?;
        let pk_type = pk_field.data_type().clone();

        // Validate primary key is orderable
        if !is_orderable_type(&pk_type) {
            return Err(anyhow!("Primary key must be orderable type"));
        }

        // Create storage
        let storage = TableStorage::new(
            schema.clone(),
            primary_key.clone(),
            data_dir.join(&name)
        )?;

        // Create index
        let index = TableIndex::new(primary_key.clone(), pk_type.clone());

        Ok(Self {
            name,
            schema,
            primary_key,
            primary_key_type: pk_type,
            storage,
            index,
            row_count: 0,
        })
    }

    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Validate row matches schema
        row.validate(&self.schema)?;

        // Extract primary key value
        let pk_value = row.get(&self.primary_key)?;

        // Insert into storage
        self.storage.insert(row)?;

        // Update index
        self.index.add_key(pk_value, self.row_count)?;

        self.row_count += 1;

        Ok(())
    }

    pub fn get(&self, pk_value: &Value) -> Result<Option<Row>> {
        // Use index to find position
        if let Some(position) = self.index.search(pk_value)? {
            self.storage.get_at_position(position)
        } else {
            Ok(None)
        }
    }

    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<Row>> {
        // Use index to find range
        let positions = self.index.range_search(start, end)?;

        // Fetch rows from storage
        self.storage.get_range(positions)
    }

    pub fn scan(&self) -> Result<TableScan> {
        // Full table scan
        self.storage.scan()
    }
}

//=============================================================================
// TableStorage - Generic row storage
//=============================================================================

pub struct TableStorage {
    /// Schema
    schema: SchemaRef,

    /// Primary key column
    primary_key: String,

    /// In-memory batches (hot data)
    hot_batches: Vec<RecordBatch>,

    /// On-disk files (cold data)
    cold_files: Vec<PathBuf>,

    /// Write-ahead log
    wal: WalManager,

    /// Data directory
    data_dir: PathBuf,
}

impl TableStorage {
    pub fn new(schema: SchemaRef, primary_key: String,
               data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir)?;

        // Initialize WAL
        let wal_dir = data_dir.join("wal");
        let wal = WalManager::new(wal_dir)?;
        wal.open()?;

        let mut storage = Self {
            schema,
            primary_key,
            hot_batches: Vec::new(),
            cold_files: Vec::new(),
            wal,
            data_dir,
        };

        // Recover from WAL
        storage.recover_from_wal()?;

        Ok(storage)
    }

    pub fn insert(&mut self, row: Row) -> Result<()> {
        // Write to WAL first (durability)
        self.wal.write_insert(&row)?;

        // Convert Row to RecordBatch
        let batch = row.to_record_batch(&self.schema)?;

        // Add to hot data
        self.hot_batches.push(batch);

        // Flush if needed
        if self.should_flush() {
            self.flush_to_disk()?;
        }

        Ok(())
    }

    pub fn get_at_position(&self, position: usize) -> Result<Option<Row>> {
        // Find row by position across batches
        let mut current_pos = 0;

        for batch in &self.hot_batches {
            if position < current_pos + batch.num_rows() {
                let row_in_batch = position - current_pos;
                return Ok(Some(Row::from_batch(batch, row_in_batch)?));
            }
            current_pos += batch.num_rows();
        }

        // Check cold files if not in hot data
        self.get_from_cold_storage(position)
    }

    pub fn get_range(&self, positions: Vec<usize>) -> Result<Vec<Row>> {
        // Fetch multiple rows efficiently
        let mut rows = Vec::new();
        for pos in positions {
            if let Some(row) = self.get_at_position(pos)? {
                rows.push(row);
            }
        }
        Ok(rows)
    }

    pub fn scan(&self) -> Result<TableScan> {
        // Return iterator over all rows
        TableScan::new(self)
    }

    fn flush_to_disk(&mut self) -> Result<()> {
        // Write hot batches to Parquet files
        // Merge batches, sort by primary key
        // Compact files periodically
        Ok(())
    }

    fn recover_from_wal(&mut self) -> Result<()> {
        // Replay WAL on startup
        Ok(())
    }
}

//=============================================================================
// TableIndex - Learned index per table
//=============================================================================

pub struct TableIndex {
    /// Column being indexed
    column: String,

    /// Data type of column
    data_type: DataType,

    /// Index implementation
    index_type: IndexType,
}

pub enum IndexType {
    /// Learned index (for ordered data)
    Learned(Box<dyn LearnedIndex>),

    /// B-tree fallback (if data not orderable)
    BTree(BTreeMap<Value, Vec<usize>>),
}

/// Generic learned index trait
pub trait LearnedIndex: Send + Sync {
    fn train(&mut self, keys: Vec<Value>, positions: Vec<usize>) -> Result<()>;
    fn search(&self, key: &Value) -> Result<Option<usize>>;
    fn range_search(&self, start: &Value, end: &Value) -> Result<Vec<usize>>;
    fn add_key(&mut self, key: Value, position: usize) -> Result<()>;
}

impl TableIndex {
    pub fn new(column: String, data_type: DataType) -> Self {
        // Create appropriate index type
        let index_type = if is_orderable_type(&data_type) {
            // Use learned index
            IndexType::Learned(Box::new(GenericLearnedIndex::new(data_type.clone())))
        } else {
            // Fall back to B-tree
            IndexType::BTree(BTreeMap::new())
        };

        Self {
            column,
            data_type,
            index_type,
        }
    }

    pub fn search(&self, key: &Value) -> Result<Option<usize>> {
        match &self.index_type {
            IndexType::Learned(index) => index.search(key),
            IndexType::BTree(btree) => Ok(btree.get(key).and_then(|v| v.first()).copied()),
        }
    }

    pub fn range_search(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        match &self.index_type {
            IndexType::Learned(index) => index.range_search(start, end),
            IndexType::BTree(btree) => {
                // Range query on B-tree
                Ok(btree.range(start..=end)
                    .flat_map(|(_, positions)| positions.iter().copied())
                    .collect())
            }
        }
    }

    pub fn add_key(&mut self, key: Value, position: usize) -> Result<()> {
        match &mut self.index_type {
            IndexType::Learned(index) => index.add_key(key, position),
            IndexType::BTree(btree) => {
                btree.entry(key).or_insert_with(Vec::new).push(position);
                Ok(())
            }
        }
    }
}

//=============================================================================
// GenericLearnedIndex - Learned index over any orderable type
//=============================================================================

pub struct GenericLearnedIndex {
    /// Data type being indexed
    data_type: DataType,

    /// Underlying RMI (works on i64)
    rmi: RecursiveModelIndex,

    /// Map Value -> i64 for indexing
    value_map: Vec<Value>,

    /// Sorted keys
    keys: Vec<i64>,
}

impl GenericLearnedIndex {
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            rmi: RecursiveModelIndex::new(1_000_000),
            value_map: Vec::new(),
            keys: Vec::new(),
        }
    }

    fn value_to_i64(&self, value: &Value) -> Result<i64> {
        // Convert any orderable type to i64 for indexing
        match value {
            Value::Int64(v) => Ok(*v),
            Value::Timestamp(v) => Ok(*v),
            Value::Float64(v) => Ok(v.to_bits() as i64),
            // Add more conversions as needed
            _ => Err(anyhow!("Type not orderable")),
        }
    }
}

impl LearnedIndex for GenericLearnedIndex {
    fn train(&mut self, keys: Vec<Value>, positions: Vec<usize>) -> Result<()> {
        // Convert values to i64
        let i64_keys: Vec<i64> = keys.iter()
            .map(|v| self.value_to_i64(v))
            .collect::<Result<Vec<_>>>()?;

        // Train RMI on i64 keys
        for &key in &i64_keys {
            self.rmi.add_key(key);
        }

        self.keys = i64_keys;
        self.value_map = keys;

        Ok(())
    }

    fn search(&self, key: &Value) -> Result<Option<usize>> {
        let i64_key = self.value_to_i64(key)?;
        Ok(self.rmi.search(i64_key))
    }

    fn range_search(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        let start_i64 = self.value_to_i64(start)?;
        let end_i64 = self.value_to_i64(end)?;

        Ok(self.rmi.range_search(start_i64, end_i64))
    }

    fn add_key(&mut self, key: Value, position: usize) -> Result<()> {
        let i64_key = self.value_to_i64(&key)?;
        self.rmi.add_key(i64_key);
        self.keys.push(i64_key);
        self.value_map.push(key);
        Ok(())
    }
}

//=============================================================================
// Row - Generic row abstraction
//=============================================================================

#[derive(Debug, Clone)]
pub struct Row {
    /// Values in schema order
    values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub fn get(&self, column_name: &str) -> Result<&Value> {
        // Get value by column name (need schema context)
        // In practice, we'd pass schema or use column index
        todo!("Implement with schema context")
    }

    pub fn validate(&self, schema: &SchemaRef) -> Result<()> {
        // Check row matches schema
        if self.values.len() != schema.fields().len() {
            return Err(anyhow!("Row length doesn't match schema"));
        }

        // Type checking
        for (i, value) in self.values.iter().enumerate() {
            let field = schema.field(i);
            if !value.matches_type(field.data_type()) {
                return Err(anyhow!("Type mismatch for column {}", field.name()));
            }
        }

        Ok(())
    }

    pub fn to_record_batch(&self, schema: &SchemaRef) -> Result<RecordBatch> {
        // Convert Row to Arrow RecordBatch
        // Build arrays for each column
        todo!("Convert to Arrow format")
    }

    pub fn from_batch(batch: &RecordBatch, row_index: usize) -> Result<Self> {
        // Extract row from RecordBatch
        let mut values = Vec::new();

        for col_idx in 0..batch.num_columns() {
            let array = batch.column(col_idx);
            let value = Value::from_array(array, row_index)?;
            values.push(value);
        }

        Ok(Self { values })
    }
}

//=============================================================================
// Value - Generic value type
//=============================================================================

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Int64(i64),
    Float64(f64),
    Text(String),
    Timestamp(i64),
    Boolean(bool),
    Null,
}

impl Value {
    pub fn matches_type(&self, data_type: &DataType) -> bool {
        match (self, data_type) {
            (Value::Int64(_), DataType::Int64) => true,
            (Value::Float64(_), DataType::Float64) => true,
            (Value::Text(_), DataType::Utf8) => true,
            (Value::Timestamp(_), DataType::Timestamp(_, _)) => true,
            (Value::Boolean(_), DataType::Boolean) => true,
            (Value::Null, _) => true,
            _ => false,
        }
    }

    pub fn from_array(array: &ArrayRef, index: usize) -> Result<Self> {
        // Extract value from Arrow array
        use arrow::array::*;

        if array.is_null(index) {
            return Ok(Value::Null);
        }

        match array.data_type() {
            DataType::Int64 => {
                let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
                Ok(Value::Int64(arr.value(index)))
            }
            DataType::Float64 => {
                let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
                Ok(Value::Float64(arr.value(index)))
            }
            DataType::Utf8 => {
                let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
                Ok(Value::Text(arr.value(index).to_string()))
            }
            DataType::Timestamp(_, _) => {
                let arr = array.as_any().downcast_ref::<TimestampMicrosecondArray>().unwrap();
                Ok(Value::Timestamp(arr.value(index)))
            }
            DataType::Boolean => {
                let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
                Ok(Value::Boolean(arr.value(index)))
            }
            _ => Err(anyhow!("Unsupported data type")),
        }
    }
}

//=============================================================================
// Helper functions
//=============================================================================

fn is_orderable_type(data_type: &DataType) -> bool {
    matches!(data_type,
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
        DataType::Float32 | DataType::Float64 |
        DataType::Timestamp(_, _) | DataType::Date32 | DataType::Date64
    )
}

fn validate_primary_key(schema: &SchemaRef, pk_name: &str) -> Result<()> {
    // Check primary key exists
    let field = schema.field_with_name(pk_name)?;

    // Check it's orderable
    if !is_orderable_type(field.data_type()) {
        return Err(anyhow!("Primary key {} must be orderable type, got {:?}",
                          pk_name, field.data_type()));
    }

    Ok(())
}
```

---

## SQL Support

### Supported SQL (Phase 1)

```sql
-- Table management
CREATE TABLE metrics (
    timestamp BIGINT PRIMARY KEY,
    value DOUBLE,
    series_id BIGINT,
    tags TEXT
);

CREATE TABLE users (
    user_id BIGINT PRIMARY KEY,
    email TEXT,
    created_at BIGINT
);

DROP TABLE metrics;

-- Data operations
INSERT INTO metrics (timestamp, value, series_id)
VALUES (1234567890, 42.5, 100);

SELECT * FROM metrics WHERE timestamp > 1000 AND timestamp < 2000;

SELECT value, series_id FROM metrics WHERE timestamp = 1234567890;

SELECT * FROM users WHERE user_id > 1000 ORDER BY user_id;
```

### Not Supported (Phase 1)

```sql
-- No joins (yet)
SELECT * FROM users JOIN orders ON users.user_id = orders.user_id;

-- No aggregations (yet)
SELECT COUNT(*), AVG(value) FROM metrics;

-- No updates (insert-only)
UPDATE metrics SET value = 100 WHERE timestamp = 1234;

-- No transactions (yet)
BEGIN;
INSERT ...;
COMMIT;
```

**Keep it simple. Add later based on usage.**

---

## Migration Strategy

### Phase 1: New Architecture (Week 1-2)
- Build new abstractions (Catalog, Table, TableStorage, TableIndex)
- Leave old code in place
- New code in `src/v2/` directory

### Phase 2: Integration (Week 3)
- PostgreSQL protocol uses new architecture
- Tests use new architecture
- Old `OmenDB` struct becomes deprecated

### Phase 3: Cleanup (Week 4)
- Remove old code
- Merge v2 into main
- Update all tests

**Don't try to refactor in place - build new alongside old.**

---

## Testing Strategy

```rust
#[test]
fn test_multiple_tables() {
    let db = OmenDB::new("/tmp/test_db").unwrap();

    // Create time-series table
    let schema1 = Schema::new(vec![
        Field::new("timestamp", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]);
    db.create_table("metrics", Arc::new(schema1), "timestamp").unwrap();

    // Create users table
    let schema2 = Schema::new(vec![
        Field::new("user_id", DataType::Int64, false),
        Field::new("email", DataType::Utf8, false),
    ]);
    db.create_table("users", Arc::new(schema2), "user_id").unwrap();

    // Insert into both
    let metrics_table = db.get_table("metrics").unwrap();
    metrics_table.insert(Row::new(vec![
        Value::Int64(1000),
        Value::Float64(42.5),
    ])).unwrap();

    let users_table = db.get_table("users").unwrap();
    users_table.insert(Row::new(vec![
        Value::Int64(1),
        Value::Text("user@example.com".to_string()),
    ])).unwrap();

    // Query both
    let metric = metrics_table.get(&Value::Int64(1000)).unwrap();
    let user = users_table.get(&Value::Int64(1)).unwrap();

    assert!(metric.is_some());
    assert!(user.is_some());
}
```

---

## Timeline

### Week 1: Core Abstractions
- Day 1-2: Value, Row types
- Day 3-4: Catalog, Table
- Day 5-7: TableStorage refactor

### Week 2: Indexes
- Day 1-3: TableIndex, GenericLearnedIndex
- Day 4-5: Integration testing
- Day 6-7: WAL integration

### Week 3: PostgreSQL Protocol
- Day 1-3: Protocol implementation
- Day 4-5: CREATE TABLE, INSERT, SELECT
- Day 6-7: Integration with new architecture

### Week 4: Testing & Polish
- Day 1-3: Comprehensive tests
- Day 4-5: Docker, documentation
- Day 6-7: Performance validation

### Week 5: Launch Prep
- Day 1-2: README, examples
- Day 3-4: Benchmarks
- Day 5-7: Launch materials

**Total: 5 weeks to launch**

---

## Success Criteria

**Must work:**
```sql
-- Multiple tables
CREATE TABLE metrics (...);
CREATE TABLE users (...);
CREATE TABLE orders (...);

-- Different primary keys
timestamp BIGINT PRIMARY KEY  -- time-series
user_id BIGINT PRIMARY KEY    -- sequential IDs
order_id BIGINT PRIMARY KEY   -- sequential IDs

-- Basic operations
INSERT INTO ...
SELECT * FROM ... WHERE pk > X AND pk < Y

-- 5-10x faster than B-tree
Benchmark shows learned index advantage
```

**This is a real database, not a time-series hack.**

---

*This is the correct architecture. Build it right, not fast.*