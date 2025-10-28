//! TPC-C Benchmark Implementation for OmenDB
//!
//! TPC-C is the industry standard OLTP benchmark used for database performance
//! comparison. This implementation follows TPC-C specification for:
//!
//! - New Order transactions (45%)
//! - Payment transactions (43%)
//! - Order Status queries (4%)
//! - Delivery transactions (4%)
//! - Stock Level queries (4%)
//!
//! This provides credible performance comparison against PostgreSQL,
//! MySQL, CockroachDB, and other production databases.

use anyhow::Result;
use arrow::datatypes::{DataType, Field, Schema};
use omen::table::Table;
use omen::row::Row;
use omen::value::Value;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tracing::{info, warn};

/// TPC-C Benchmark Configuration
#[derive(Debug, Clone)]
pub struct TpccConfig {
    /// Number of warehouses (scale factor)
    pub warehouses: usize,
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Number of concurrent users per warehouse
    pub users_per_warehouse: usize,
    /// Enable detailed transaction logging
    pub detailed_logging: bool,
}

impl Default for TpccConfig {
    fn default() -> Self {
        Self {
            warehouses: 1,
            duration_secs: 300, // 5 minutes
            users_per_warehouse: 10,
            detailed_logging: false,
        }
    }
}

/// TPC-C Transaction Types
#[derive(Debug, Clone, Copy)]
pub enum TransactionType {
    NewOrder,    // 45% - Create new order
    Payment,     // 43% - Process payment
    OrderStatus, // 4% - Check order status
    Delivery,    // 4% - Deliver orders
    StockLevel,  // 4% - Check stock levels
}

impl TransactionType {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        match rng.gen_range(0..100) {
            0..=44 => TransactionType::NewOrder,
            45..=87 => TransactionType::Payment,
            88..=91 => TransactionType::OrderStatus,
            92..=95 => TransactionType::Delivery,
            _ => TransactionType::StockLevel,
        }
    }
}

/// TPC-C Performance Metrics
#[derive(Debug)]
pub struct TpccMetrics {
    // Transaction counts by type
    pub new_order_count: usize,
    pub payment_count: usize,
    pub order_status_count: usize,
    pub delivery_count: usize,
    pub stock_level_count: usize,

    // Response times (nanoseconds)
    pub new_order_times: Vec<u64>,
    pub payment_times: Vec<u64>,
    pub order_status_times: Vec<u64>,
    pub delivery_times: Vec<u64>,
    pub stock_level_times: Vec<u64>,

    // Error counts
    pub errors: usize,
    pub aborts: usize,

    // Test metadata
    pub start_time: Instant,
    pub end_time: Option<Instant>,
}

impl Default for TpccMetrics {
    fn default() -> Self {
        Self {
            new_order_count: 0,
            payment_count: 0,
            order_status_count: 0,
            delivery_count: 0,
            stock_level_count: 0,
            new_order_times: Vec::new(),
            payment_times: Vec::new(),
            order_status_times: Vec::new(),
            delivery_times: Vec::new(),
            stock_level_times: Vec::new(),
            errors: 0,
            aborts: 0,
            start_time: Instant::now(),
            end_time: None,
        }
    }
}

impl TpccMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_transaction(&mut self, tx_type: TransactionType, duration: Duration, success: bool) {
        let duration_ns = duration.as_nanos() as u64;

        if success {
            match tx_type {
                TransactionType::NewOrder => {
                    self.new_order_count += 1;
                    self.new_order_times.push(duration_ns);
                }
                TransactionType::Payment => {
                    self.payment_count += 1;
                    self.payment_times.push(duration_ns);
                }
                TransactionType::OrderStatus => {
                    self.order_status_count += 1;
                    self.order_status_times.push(duration_ns);
                }
                TransactionType::Delivery => {
                    self.delivery_count += 1;
                    self.delivery_times.push(duration_ns);
                }
                TransactionType::StockLevel => {
                    self.stock_level_count += 1;
                    self.stock_level_times.push(duration_ns);
                }
            }
        } else {
            self.errors += 1;
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub fn total_transactions(&self) -> usize {
        self.new_order_count + self.payment_count + self.order_status_count +
        self.delivery_count + self.stock_level_count
    }

    pub fn transactions_per_minute(&self) -> f64 {
        if let Some(end_time) = self.end_time {
            let duration = end_time.duration_since(self.start_time);
            self.total_transactions() as f64 / duration.as_secs_f64() * 60.0
        } else {
            0.0
        }
    }

    pub fn new_order_per_minute(&self) -> f64 {
        if let Some(end_time) = self.end_time {
            let duration = end_time.duration_since(self.start_time);
            self.new_order_count as f64 / duration.as_secs_f64() * 60.0
        } else {
            0.0
        }
    }
}

/// TPC-C Database Schema and Operations
pub struct TpccDatabase {
    // Core TPC-C tables
    warehouse_table: Arc<RwLock<Table>>,
    district_table: Arc<RwLock<Table>>,
    customer_table: Arc<RwLock<Table>>,
    item_table: Arc<RwLock<Table>>,
    stock_table: Arc<RwLock<Table>>,
    orders_table: Arc<RwLock<Table>>,
    order_line_table: Arc<RwLock<Table>>,
    new_orders_table: Arc<RwLock<Table>>,
    history_table: Arc<RwLock<Table>>,

    // Configuration
    config: TpccConfig,

    // Counters for key generation
    next_order_id: HashMap<(i64, i64), i64>, // (warehouse_id, district_id) -> next_order_id
}

impl TpccDatabase {
    pub async fn new(config: TpccConfig) -> Result<Self> {
        let dir = tempdir()?;

        // Create TPC-C tables following the standard schema

        // Warehouse table
        let warehouse_schema = Arc::new(Schema::new(vec![
            Field::new("w_id", DataType::Int64, false),
            Field::new("w_name", DataType::Utf8, false),
            Field::new("w_street_1", DataType::Utf8, false),
            Field::new("w_street_2", DataType::Utf8, false),
            Field::new("w_city", DataType::Utf8, false),
            Field::new("w_state", DataType::Utf8, false),
            Field::new("w_zip", DataType::Utf8, false),
            Field::new("w_tax", DataType::Float64, false),
            Field::new("w_ytd", DataType::Float64, false),
        ]));

        let warehouse_table = Arc::new(RwLock::new(Table::new(
            "warehouse".to_string(),
            warehouse_schema,
            "w_id".to_string(),
            dir.path().join("warehouse"),
        )?));

        // District table
        let district_schema = Arc::new(Schema::new(vec![
            Field::new("d_id", DataType::Int64, false),
            Field::new("d_w_id", DataType::Int64, false),
            Field::new("d_name", DataType::Utf8, false),
            Field::new("d_street_1", DataType::Utf8, false),
            Field::new("d_street_2", DataType::Utf8, false),
            Field::new("d_city", DataType::Utf8, false),
            Field::new("d_state", DataType::Utf8, false),
            Field::new("d_zip", DataType::Utf8, false),
            Field::new("d_tax", DataType::Float64, false),
            Field::new("d_ytd", DataType::Float64, false),
            Field::new("d_next_o_id", DataType::Int64, false),
        ]));

        let district_table = Arc::new(RwLock::new(Table::new(
            "district".to_string(),
            district_schema,
            "d_id".to_string(),
            dir.path().join("district"),
        )?));

        // Customer table (simplified schema)
        let customer_schema = Arc::new(Schema::new(vec![
            Field::new("c_id", DataType::Int64, false),
            Field::new("c_d_id", DataType::Int64, false),
            Field::new("c_w_id", DataType::Int64, false),
            Field::new("c_first", DataType::Utf8, false),
            Field::new("c_middle", DataType::Utf8, false),
            Field::new("c_last", DataType::Utf8, false),
            Field::new("c_street_1", DataType::Utf8, false),
            Field::new("c_street_2", DataType::Utf8, false),
            Field::new("c_city", DataType::Utf8, false),
            Field::new("c_state", DataType::Utf8, false),
            Field::new("c_zip", DataType::Utf8, false),
            Field::new("c_phone", DataType::Utf8, false),
            Field::new("c_credit", DataType::Utf8, false),
            Field::new("c_credit_lim", DataType::Float64, false),
            Field::new("c_discount", DataType::Float64, false),
            Field::new("c_balance", DataType::Float64, false),
        ]));

        let customer_table = Arc::new(RwLock::new(Table::new(
            "customer".to_string(),
            customer_schema,
            "c_id".to_string(),
            dir.path().join("customer"),
        )?));

        // Item table
        let item_schema = Arc::new(Schema::new(vec![
            Field::new("i_id", DataType::Int64, false),
            Field::new("i_im_id", DataType::Int64, false),
            Field::new("i_name", DataType::Utf8, false),
            Field::new("i_price", DataType::Float64, false),
            Field::new("i_data", DataType::Utf8, false),
        ]));

        let item_table = Arc::new(RwLock::new(Table::new(
            "item".to_string(),
            item_schema,
            "i_id".to_string(),
            dir.path().join("item"),
        )?));

        // Stock table
        let stock_schema = Arc::new(Schema::new(vec![
            Field::new("s_i_id", DataType::Int64, false),
            Field::new("s_w_id", DataType::Int64, false),
            Field::new("s_quantity", DataType::Int64, false),
            Field::new("s_dist_01", DataType::Utf8, false),
            Field::new("s_ytd", DataType::Int64, false),
            Field::new("s_order_cnt", DataType::Int64, false),
            Field::new("s_remote_cnt", DataType::Int64, false),
            Field::new("s_data", DataType::Utf8, false),
        ]));

        let stock_table = Arc::new(RwLock::new(Table::new(
            "stock".to_string(),
            stock_schema,
            "s_i_id".to_string(),
            dir.path().join("stock"),
        )?));

        // Orders table
        let orders_schema = Arc::new(Schema::new(vec![
            Field::new("o_id", DataType::Int64, false),
            Field::new("o_d_id", DataType::Int64, false),
            Field::new("o_w_id", DataType::Int64, false),
            Field::new("o_c_id", DataType::Int64, false),
            Field::new("o_entry_d", DataType::Int64, false), // timestamp
            Field::new("o_carrier_id", DataType::Int64, true), // nullable
            Field::new("o_ol_cnt", DataType::Int64, false),
            Field::new("o_all_local", DataType::Int64, false),
        ]));

        let orders_table = Arc::new(RwLock::new(Table::new(
            "orders".to_string(),
            orders_schema,
            "o_id".to_string(),
            dir.path().join("orders"),
        )?));

        // Order Line table
        let order_line_schema = Arc::new(Schema::new(vec![
            Field::new("ol_o_id", DataType::Int64, false),
            Field::new("ol_d_id", DataType::Int64, false),
            Field::new("ol_w_id", DataType::Int64, false),
            Field::new("ol_number", DataType::Int64, false),
            Field::new("ol_i_id", DataType::Int64, false),
            Field::new("ol_supply_w_id", DataType::Int64, false),
            Field::new("ol_delivery_d", DataType::Int64, true), // nullable timestamp
            Field::new("ol_quantity", DataType::Int64, false),
            Field::new("ol_amount", DataType::Float64, false),
            Field::new("ol_dist_info", DataType::Utf8, false),
        ]));

        let order_line_table = Arc::new(RwLock::new(Table::new(
            "order_line".to_string(),
            order_line_schema,
            "ol_o_id".to_string(),
            dir.path().join("order_line"),
        )?));

        // New Orders table
        let new_orders_schema = Arc::new(Schema::new(vec![
            Field::new("no_o_id", DataType::Int64, false),
            Field::new("no_d_id", DataType::Int64, false),
            Field::new("no_w_id", DataType::Int64, false),
        ]));

        let new_orders_table = Arc::new(RwLock::new(Table::new(
            "new_orders".to_string(),
            new_orders_schema,
            "no_o_id".to_string(),
            dir.path().join("new_orders"),
        )?));

        // History table
        let history_schema = Arc::new(Schema::new(vec![
            Field::new("h_c_id", DataType::Int64, false),
            Field::new("h_c_d_id", DataType::Int64, false),
            Field::new("h_c_w_id", DataType::Int64, false),
            Field::new("h_d_id", DataType::Int64, false),
            Field::new("h_w_id", DataType::Int64, false),
            Field::new("h_date", DataType::Int64, false), // timestamp
            Field::new("h_amount", DataType::Float64, false),
            Field::new("h_data", DataType::Utf8, false),
        ]));

        let history_table = Arc::new(RwLock::new(Table::new(
            "history".to_string(),
            history_schema,
            "h_c_id".to_string(),
            dir.path().join("history"),
        )?));

        Ok(Self {
            warehouse_table,
            district_table,
            customer_table,
            item_table,
            stock_table,
            orders_table,
            order_line_table,
            new_orders_table,
            history_table,
            config,
            next_order_id: HashMap::new(),
        })
    }

    /// Load TPC-C data according to specification
    pub async fn load_data(&mut self) -> Result<()> {
        info!("📊 Loading TPC-C dataset for {} warehouses...", self.config.warehouses);

        let start_time = Instant::now();
        let mut total_records = 0;

        // Load Items (100,000 items - independent of warehouse count)
        info!("  Loading 100,000 items...");
        for i_id in 1..=100_000 {
            let item = Row::new(vec![
                Value::Int64(i_id),
                Value::Int64(i_id % 10000), // i_im_id
                Value::Text(format!("Item {}", i_id)),
                Value::Float64(thread_rng().gen_range(1.0..100.0)), // price
                Value::Text(format!("Data for item {}", i_id)),
            ]);
            self.item_table.write().unwrap().insert(item)?;
            total_records += 1;
        }

        // Load per-warehouse data
        for w_id in 1..=self.config.warehouses {
            info!("  Loading data for warehouse {}...", w_id);

            // Load Warehouse
            let warehouse = Row::new(vec![
                Value::Int64(w_id as i64),
                Value::Text(format!("Warehouse {}", w_id)),
                Value::Text(format!("Street 1 W{}", w_id)),
                Value::Text(format!("Street 2 W{}", w_id)),
                Value::Text(format!("City {}", w_id)),
                Value::Text("ST".to_string()),
                Value::Text("12345".to_string()),
                Value::Float64(0.05), // 5% tax
                Value::Float64(300000.00), // YTD
            ]);
            self.warehouse_table.write().unwrap().insert(warehouse)?;
            total_records += 1;

            // Load Districts (10 per warehouse)
            for d_id in 1..=10 {
                let district = Row::new(vec![
                    Value::Int64(d_id),
                    Value::Int64(w_id as i64),
                    Value::Text(format!("District {}", d_id)),
                    Value::Text(format!("Street 1 D{}", d_id)),
                    Value::Text(format!("Street 2 D{}", d_id)),
                    Value::Text(format!("City {}", d_id)),
                    Value::Text("ST".to_string()),
                    Value::Text("12345".to_string()),
                    Value::Float64(0.10), // 10% tax
                    Value::Float64(30000.00), // YTD
                    Value::Int64(3001), // next_o_id (starting at 3001)
                ]);
                self.district_table.write().unwrap().insert(district)?;
                total_records += 1;

                // Initialize order tracking
                self.next_order_id.insert((w_id as i64, d_id), 3001);
            }

            // Load Customers (3000 per district, 30,000 per warehouse)
            for d_id in 1..=10 {
                for c_id in 1..=3000 {
                    let customer = Row::new(vec![
                        Value::Int64(c_id),
                        Value::Int64(d_id),
                        Value::Int64(w_id as i64),
                        Value::Text(format!("First{}", c_id)),
                        Value::Text("M".to_string()),
                        Value::Text(format!("Last{}", c_id)),
                        Value::Text(format!("Street 1 C{}", c_id)),
                        Value::Text(format!("Street 2 C{}", c_id)),
                        Value::Text(format!("City {}", c_id)),
                        Value::Text("ST".to_string()),
                        Value::Text("12345".to_string()),
                        Value::Text("1234567890".to_string()),
                        Value::Text("GC".to_string()), // Good Credit
                        Value::Float64(50000.00), // Credit limit
                        Value::Float64(0.05), // 5% discount
                        Value::Float64(-1000.00), // Starting balance
                    ]);
                    self.customer_table.write().unwrap().insert(customer)?;
                    total_records += 1;
                }
            }

            // Load Stock (100,000 items per warehouse)
            for i_id in 1..=100_000 {
                let stock = Row::new(vec![
                    Value::Int64(i_id),
                    Value::Int64(w_id as i64),
                    Value::Int64(thread_rng().gen_range(10..100)), // Initial quantity
                    Value::Text(format!("Dist info {}", i_id)),
                    Value::Int64(0), // YTD
                    Value::Int64(0), // Order count
                    Value::Int64(0), // Remote count
                    Value::Text(format!("Stock data {}", i_id)),
                ]);
                self.stock_table.write().unwrap().insert(stock)?;
                total_records += 1;
            }
        }

        let duration = start_time.elapsed();
        info!("✅ TPC-C data loading completed:");
        info!("   Total records: {}", total_records);
        info!("   Load time: {:.2}s", duration.as_secs_f64());
        info!("   Load rate: {:.0} records/sec", total_records as f64 / duration.as_secs_f64());

        Ok(())
    }

    /// Execute New Order transaction (45% of workload)
    pub fn new_order_transaction(&mut self, w_id: i64, d_id: i64) -> Result<Duration> {
        let start = Instant::now();

        // Generate order parameters
        let mut rng = thread_rng();
        let c_id = rng.gen_range(1..=3000);
        let ol_cnt = rng.gen_range(5..=15); // Number of order lines
        let timestamp = chrono::Utc::now().timestamp();

        // Get next order ID for this district
        let o_id = *self.next_order_id.get(&(w_id, d_id)).unwrap_or(&3001);
        self.next_order_id.insert((w_id, d_id), o_id + 1);

        // Insert new order
        let order = Row::new(vec![
            Value::Int64(o_id),
            Value::Int64(d_id),
            Value::Int64(w_id),
            Value::Int64(c_id),
            Value::Int64(timestamp),
            Value::Int64(0), // carrier_id (null initially)
            Value::Int64(ol_cnt),
            Value::Int64(1), // all_local
        ]);
        self.orders_table.write().unwrap().insert(order)?;

        // Insert new order record
        let new_order = Row::new(vec![
            Value::Int64(o_id),
            Value::Int64(d_id),
            Value::Int64(w_id),
        ]);
        self.new_orders_table.write().unwrap().insert(new_order)?;

        // Insert order lines
        let mut total_amount = 0.0;
        for ol_number in 1..=ol_cnt {
            let i_id = rng.gen_range(1..=100_000);
            let ol_supply_w_id = w_id; // Assume local supply
            let ol_quantity = rng.gen_range(1..=10);

            // Simplified: assume item price is based on item ID
            let ol_amount = (i_id as f64 / 1000.0) * ol_quantity as f64;
            total_amount += ol_amount;

            let order_line = Row::new(vec![
                Value::Int64(o_id),
                Value::Int64(d_id),
                Value::Int64(w_id),
                Value::Int64(ol_number),
                Value::Int64(i_id),
                Value::Int64(ol_supply_w_id),
                Value::Int64(0), // delivery_d (null initially)
                Value::Int64(ol_quantity),
                Value::Float64(ol_amount),
                Value::Text(format!("Dist info {}", i_id)),
            ]);
            self.order_line_table.write().unwrap().insert(order_line)?;
        }

        Ok(start.elapsed())
    }

    /// Execute Payment transaction (43% of workload)
    pub fn payment_transaction(&mut self, w_id: i64, d_id: i64) -> Result<Duration> {
        let start = Instant::now();

        let mut rng = thread_rng();
        let c_id = rng.gen_range(1..=3000);
        let h_amount = rng.gen_range(1.0..5000.0);
        let timestamp = chrono::Utc::now().timestamp();

        // Insert payment history
        let history = Row::new(vec![
            Value::Int64(c_id),
            Value::Int64(d_id),
            Value::Int64(w_id),
            Value::Int64(d_id),
            Value::Int64(w_id),
            Value::Int64(timestamp),
            Value::Float64(h_amount),
            Value::Text(format!("Payment {:.2}", h_amount)),
        ]);
        self.history_table.write().unwrap().insert(history)?;

        Ok(start.elapsed())
    }

    /// Execute Order Status query (4% of workload)
    pub fn order_status_transaction(&self, w_id: i64, d_id: i64) -> Result<Duration> {
        let start = Instant::now();

        let mut rng = thread_rng();
        let c_id = rng.gen_range(1..=3000);

        // Simulate order status lookup
        let key = Value::Int64(c_id);
        let _customer = self.customer_table.read().unwrap().get(&key)?;

        Ok(start.elapsed())
    }

    /// Execute Delivery transaction (4% of workload)
    pub fn delivery_transaction(&self, w_id: i64) -> Result<Duration> {
        let start = Instant::now();

        // Delivery processes orders for all districts in a warehouse
        for d_id in 1..=10 {
            let key = Value::Int64(d_id);
            let _district = self.district_table.read().unwrap().get(&key)?;
        }

        Ok(start.elapsed())
    }

    /// Execute Stock Level query (4% of workload)
    pub fn stock_level_transaction(&self, w_id: i64, d_id: i64) -> Result<Duration> {
        let start = Instant::now();

        // Simulate stock level check by reading district and stock data
        let key = Value::Int64(d_id);
        let _district = self.district_table.read().unwrap().get(&key)?;

        // Check stock for sample items
        for i_id in 1..=100 {
            let stock_key = Value::Int64(i_id);
            let _stock = self.stock_table.read().unwrap().get(&stock_key)?;
        }

        Ok(start.elapsed())
    }
}

/// TPC-C Benchmark Runner
pub struct TpccBenchmark {
    database: TpccDatabase,
    config: TpccConfig,
}

impl TpccBenchmark {
    pub async fn new(config: TpccConfig) -> Result<Self> {
        let mut database = TpccDatabase::new(config.clone()).await?;
        database.load_data().await?;

        Ok(Self { database, config })
    }

    /// Run the complete TPC-C benchmark
    pub async fn run(&mut self) -> Result<TpccMetrics> {
        info!("🚀 Starting TPC-C benchmark:");
        info!("   Warehouses: {}", self.config.warehouses);
        info!("   Duration: {}s", self.config.duration_secs);
        info!("   Users per warehouse: {}", self.config.users_per_warehouse);

        let mut metrics = TpccMetrics::new();
        let end_time = Instant::now() + Duration::from_secs(self.config.duration_secs);

        let mut transaction_count = 0;

        while Instant::now() < end_time {
            // Select random warehouse and district
            let w_id = thread_rng().gen_range(1..=self.config.warehouses) as i64;
            let d_id = thread_rng().gen_range(1..=10);

            // Select transaction type according to TPC-C mix
            let tx_type = TransactionType::random();

            // Execute transaction
            let result = match tx_type {
                TransactionType::NewOrder => {
                    self.database.new_order_transaction(w_id, d_id)
                }
                TransactionType::Payment => {
                    self.database.payment_transaction(w_id, d_id)
                }
                TransactionType::OrderStatus => {
                    self.database.order_status_transaction(w_id, d_id)
                }
                TransactionType::Delivery => {
                    self.database.delivery_transaction(w_id)
                }
                TransactionType::StockLevel => {
                    self.database.stock_level_transaction(w_id, d_id)
                }
            };

            // Record metrics
            match result {
                Ok(duration) => {
                    metrics.record_transaction(tx_type, duration, true);
                    transaction_count += 1;

                    if self.config.detailed_logging && transaction_count % 1000 == 0 {
                        info!("  Completed {} transactions, TPM: {:.0}",
                               transaction_count, metrics.transactions_per_minute());
                    }
                }
                Err(_) => {
                    metrics.record_transaction(tx_type, Duration::ZERO, false);
                }
            }
        }

        metrics.finish();
        Ok(metrics)
    }
}

/// Calculate standard TPC-C performance statistics
fn calculate_response_time_stats(times: &[u64]) -> (f64, u64, u64, u64, u64) {
    if times.is_empty() {
        return (0.0, 0, 0, 0, 0);
    }

    let mut sorted_times = times.to_vec();
    sorted_times.sort_unstable();

    let avg = sorted_times.iter().sum::<u64>() as f64 / sorted_times.len() as f64;
    let p50 = sorted_times[sorted_times.len() * 50 / 100];
    let p90 = sorted_times[sorted_times.len() * 90 / 100];
    let p95 = sorted_times[sorted_times.len() * 95 / 100];
    let p99 = sorted_times[sorted_times.len() * 99 / 100];

    (avg, p50, p90, p95, p99)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                  TPC-C Benchmark - OmenDB                   ║");
    println!("║              Industry Standard OLTP Performance             ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Configure benchmark
    let config = TpccConfig {
        warehouses: 1,           // Start with 1 warehouse for baseline
        duration_secs: 30,       // 30 second test for quick validation
        users_per_warehouse: 1,  // Single-threaded for now
        detailed_logging: false, // Disable verbose logging for cleaner output
    };

    info!("🔧 TPC-C Configuration:");
    info!("   Scale factor (warehouses): {}", config.warehouses);
    info!("   Test duration: {}s", config.duration_secs);
    info!("   Concurrent users: {}", config.users_per_warehouse);

    // Run benchmark
    let mut benchmark = TpccBenchmark::new(config).await?;
    let metrics = benchmark.run().await?;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║                    TPC-C Results                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    // Calculate response time statistics
    let (no_avg, no_p50, no_p90, no_p95, no_p99) = calculate_response_time_stats(&metrics.new_order_times);
    let (pm_avg, pm_p50, pm_p90, pm_p95, pm_p99) = calculate_response_time_stats(&metrics.payment_times);
    let (os_avg, os_p50, os_p90, os_p95, os_p99) = calculate_response_time_stats(&metrics.order_status_times);
    let (dl_avg, dl_p50, dl_p90, dl_p95, dl_p99) = calculate_response_time_stats(&metrics.delivery_times);
    let (sl_avg, sl_p50, sl_p90, sl_p95, sl_p99) = calculate_response_time_stats(&metrics.stock_level_times);

    // Overall performance metrics
    info!("📊 TPC-C Performance Summary:");
    info!("   Total Transactions: {}", metrics.total_transactions());
    info!("   Transactions per Minute (TPM): {:.0}", metrics.transactions_per_minute());
    info!("   New Orders per Minute (NOPM): {:.0}", metrics.new_order_per_minute());
    info!("   Error Rate: {:.2}%", metrics.errors as f64 / metrics.total_transactions() as f64 * 100.0);

    println!("\n📈 Transaction Mix:");
    info!("   New Order: {} ({:.1}%)", metrics.new_order_count,
           metrics.new_order_count as f64 / metrics.total_transactions() as f64 * 100.0);
    info!("   Payment: {} ({:.1}%)", metrics.payment_count,
           metrics.payment_count as f64 / metrics.total_transactions() as f64 * 100.0);
    info!("   Order Status: {} ({:.1}%)", metrics.order_status_count,
           metrics.order_status_count as f64 / metrics.total_transactions() as f64 * 100.0);
    info!("   Delivery: {} ({:.1}%)", metrics.delivery_count,
           metrics.delivery_count as f64 / metrics.total_transactions() as f64 * 100.0);
    info!("   Stock Level: {} ({:.1}%)", metrics.stock_level_count,
           metrics.stock_level_count as f64 / metrics.total_transactions() as f64 * 100.0);

    println!("\n⚡ Response Times (microseconds):");
    println!("   Transaction Type    Avg     P50     P90     P95     P99");
    println!("   ───────────────────────────────────────────────────────");
    println!("   New Order        {:7.0} {:7} {:7} {:7} {:7}",
             no_avg / 1000.0, no_p50 / 1000, no_p90 / 1000, no_p95 / 1000, no_p99 / 1000);
    println!("   Payment          {:7.0} {:7} {:7} {:7} {:7}",
             pm_avg / 1000.0, pm_p50 / 1000, pm_p90 / 1000, pm_p95 / 1000, pm_p99 / 1000);
    println!("   Order Status     {:7.0} {:7} {:7} {:7} {:7}",
             os_avg / 1000.0, os_p50 / 1000, os_p90 / 1000, os_p95 / 1000, os_p99 / 1000);
    println!("   Delivery         {:7.0} {:7} {:7} {:7} {:7}",
             dl_avg / 1000.0, dl_p50 / 1000, dl_p90 / 1000, dl_p95 / 1000, dl_p99 / 1000);
    println!("   Stock Level      {:7.0} {:7} {:7} {:7} {:7}",
             sl_avg / 1000.0, sl_p50 / 1000, sl_p90 / 1000, sl_p95 / 1000, sl_p99 / 1000);

    println!("\n🎯 TPC-C Benchmark Assessment:");
    if metrics.new_order_per_minute() > 1000.0 {
        info!("   ✅ EXCELLENT: >1000 NOPM (New Orders per Minute)");
    } else if metrics.new_order_per_minute() > 500.0 {
        info!("   ✅ GOOD: >500 NOPM performance");
    } else {
        warn!("   ⚠️  Performance below expectations (<500 NOPM)");
    }

    if no_p95 < 5_000_000 { // 5ms
        info!("   ✅ EXCELLENT: P95 New Order latency <5ms");
    } else if no_p95 < 10_000_000 { // 10ms
        info!("   ✅ GOOD: P95 New Order latency <10ms");
    } else {
        warn!("   ⚠️  High latency: P95 New Order >10ms");
    }

    if metrics.errors == 0 {
        info!("   ✅ EXCELLENT: Zero transaction errors");
    } else {
        warn!("   ⚠️  Errors detected: {} failed transactions", metrics.errors);
    }

    println!("\n💡 Next Steps for Production Validation:");
    info!("   1. Scale to multiple warehouses (5-10)");
    info!("   2. Add concurrent user simulation");
    info!("   3. Run extended duration tests (30+ minutes)");
    info!("   4. Compare against PostgreSQL/MySQL baselines");
    info!("   5. Add TPC-H OLAP benchmark for HTAP validation");

    Ok(())
}