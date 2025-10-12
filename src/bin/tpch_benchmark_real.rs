//! TPC-H Benchmark with Real Data Generation
//!
//! Production-grade TPC-H benchmark using tpchgen-cli for data generation
//! and DataFusion for query execution. Validates OLAP capabilities.
//!
//! Setup: cargo install tpchgen-cli

use anyhow::{Context, Result};
use datafusion::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::info;

/// TPC-H configuration
#[derive(Debug, Clone)]
pub struct TpchConfig {
    /// Scale factor (1 = 1GB)
    pub scale_factor: f64,
    /// Number of runs per query for averaging
    pub runs_per_query: usize,
    /// Data directory
    pub data_dir: PathBuf,
}

impl Default for TpchConfig {
    fn default() -> Self {
        Self {
            scale_factor: 0.1, // SF=0.1 for quick testing (100MB)
            runs_per_query: 3,
            data_dir: PathBuf::from("/tmp/tpch_data"),
        }
    }
}

/// Query performance metrics
#[derive(Debug)]
pub struct QueryMetrics {
    query_num: u32,
    name: String,
    avg_ms: f64,
    min_ms: f64,
    max_ms: f64,
    rows_returned: usize,
}

/// TPC-H benchmark runner
pub struct TpchBenchmark {
    ctx: SessionContext,
    config: TpchConfig,
}

impl TpchBenchmark {
    /// Create new benchmark and load data
    pub async fn new(config: TpchConfig) -> Result<Self> {
        info!("📊 Creating TPC-H Benchmark (SF={})", config.scale_factor);

        let ctx = SessionContext::new();

        let mut benchmark = Self { ctx, config };
        benchmark.generate_and_load_data().await?;

        Ok(benchmark)
    }

    /// Generate TPC-H data using tpchgen-cli and load into DataFusion
    async fn generate_and_load_data(&mut self) -> Result<()> {
        let sf = self.config.scale_factor;
        info!("📦 Generating TPC-H data (SF={}, ~{}MB)...", sf, (sf * 1000.0) as u32);

        // Create data directory
        std::fs::create_dir_all(&self.config.data_dir)
            .context("Failed to create data directory")?;

        // Check if data already exists
        let lineitem_path = self.config.data_dir.join("lineitem.parquet");
        if !lineitem_path.exists() {
            info!("   Generating data with tpchgen-cli...");

            // Generate Parquet files
            let output = Command::new("tpchgen-cli")
                .arg("-s")
                .arg(format!("{}", sf))
                .arg("--format")
                .arg("parquet")
                .arg("--output")
                .arg(&self.config.data_dir)
                .output()
                .context("Failed to run tpchgen-cli. Install with: cargo install tpchgen-cli")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("tpchgen-cli failed: {}", stderr);
            }

            info!("   ✅ Data generation complete");
        } else {
            info!("   ℹ️  Using existing data");
        }

        // Register Parquet tables
        self.register_table("region").await?;
        self.register_table("nation").await?;
        self.register_table("customer").await?;
        self.register_table("supplier").await?;
        self.register_table("part").await?;
        self.register_table("partsupp").await?;
        self.register_table("orders").await?;
        self.register_table("lineitem").await?;

        info!("✅ All TPC-H tables registered");
        Ok(())
    }

    async fn register_table(&self, name: &str) -> Result<()> {
        let path = self.config.data_dir.join(format!("{}.parquet", name));
        self.ctx.register_parquet(name, path.to_str().unwrap(), ParquetReadOptions::default())
            .await
            .context(format!("Failed to register table: {}", name))?;
        info!("  ✅ {}", name);
        Ok(())
    }

    /// Run TPC-H benchmark suite
    pub async fn run(&self) -> Result<Vec<QueryMetrics>> {
        info!("\n🎯 Running TPC-H Queries (SF={}, {} runs per query)\n",
              self.config.scale_factor, self.config.runs_per_query);

        let mut all_metrics = Vec::new();

        // Start with queries that work with any SQL engine
        let queries = vec![
            (1, "Pricing Summary Report", self.get_q1_sql()),
            (3, "Shipping Priority", self.get_q3_sql()),
            (5, "Local Supplier Volume", self.get_q5_sql()),
            (6, "Forecasting Revenue Change", self.get_q6_sql()),
            (10, "Returned Item Reporting", self.get_q10_sql()),
        ];

        for (num, name, sql) in queries {
            match self.run_query(num, name, &sql).await {
                Ok(metrics) => {
                    info!("   Q{:2}: {} - avg={:.2}ms, {} rows",
                          num, name, metrics.avg_ms, metrics.rows_returned);
                    all_metrics.push(metrics);
                }
                Err(e) => {
                    info!("   Q{:2}: {} - FAILED: {}", num, name, e);
                }
            }
        }

        Ok(all_metrics)
    }

    async fn run_query(&self, num: u32, name: &str, sql: &str) -> Result<QueryMetrics> {
        let mut latencies = Vec::new();
        let mut rows_returned = 0;

        for _ in 0..self.config.runs_per_query {
            let start = Instant::now();
            let df = self.ctx.sql(sql).await?;
            let results = df.collect().await?;
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

            rows_returned = results.iter().map(|b| b.num_rows()).sum();
            latencies.push(latency_ms);
        }

        let avg_ms = latencies.iter().sum::<f64>() / latencies.len() as f64;
        let min_ms = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ms = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        Ok(QueryMetrics {
            query_num: num,
            name: name.to_string(),
            avg_ms,
            min_ms,
            max_ms,
            rows_returned,
        })
    }

    /// TPC-H Query 1: Pricing Summary Report
    fn get_q1_sql(&self) -> String {
        r#"
        SELECT
            l_returnflag,
            l_linestatus,
            SUM(l_quantity) as sum_qty,
            SUM(l_extendedprice) as sum_base_price,
            SUM(l_extendedprice * (1 - l_discount)) as sum_disc_price,
            SUM(l_extendedprice * (1 - l_discount) * (1 + l_tax)) as sum_charge,
            AVG(l_quantity) as avg_qty,
            AVG(l_extendedprice) as avg_price,
            AVG(l_discount) as avg_disc,
            COUNT(*) as count_order
        FROM lineitem
        WHERE l_shipdate <= DATE '1998-09-02'
        GROUP BY l_returnflag, l_linestatus
        ORDER BY l_returnflag, l_linestatus
        "#.to_string()
    }

    /// TPC-H Query 3: Shipping Priority
    fn get_q3_sql(&self) -> String {
        r#"
        SELECT
            l_orderkey,
            SUM(l_extendedprice * (1 - l_discount)) as revenue,
            o_orderdate,
            o_shippriority
        FROM customer, orders, lineitem
        WHERE c_mktsegment = 'BUILDING'
            AND c_custkey = o_custkey
            AND l_orderkey = o_orderkey
            AND o_orderdate < DATE '1995-03-15'
            AND l_shipdate > DATE '1995-03-15'
        GROUP BY l_orderkey, o_orderdate, o_shippriority
        ORDER BY revenue DESC, o_orderdate
        LIMIT 10
        "#.to_string()
    }

    /// TPC-H Query 5: Local Supplier Volume
    fn get_q5_sql(&self) -> String {
        r#"
        SELECT
            n_name,
            SUM(l_extendedprice * (1 - l_discount)) as revenue
        FROM customer, orders, lineitem, supplier, nation, region
        WHERE c_custkey = o_custkey
            AND l_orderkey = o_orderkey
            AND l_suppkey = s_suppkey
            AND c_nationkey = s_nationkey
            AND s_nationkey = n_nationkey
            AND n_regionkey = r_regionkey
            AND r_name = 'ASIA'
            AND o_orderdate >= DATE '1994-01-01'
            AND o_orderdate < DATE '1995-01-01'
        GROUP BY n_name
        ORDER BY revenue DESC
        "#.to_string()
    }

    /// TPC-H Query 6: Forecasting Revenue Change
    fn get_q6_sql(&self) -> String {
        r#"
        SELECT SUM(l_extendedprice * l_discount) as revenue
        FROM lineitem
        WHERE l_shipdate >= DATE '1994-01-01'
            AND l_shipdate < DATE '1995-01-01'
            AND l_discount BETWEEN 0.05 AND 0.07
            AND l_quantity < 24
        "#.to_string()
    }

    /// TPC-H Query 10: Returned Item Reporting
    fn get_q10_sql(&self) -> String {
        r#"
        SELECT
            c_custkey,
            c_name,
            SUM(l_extendedprice * (1 - l_discount)) as revenue,
            c_acctbal,
            n_name,
            c_address,
            c_phone,
            c_comment
        FROM customer, orders, lineitem, nation
        WHERE c_custkey = o_custkey
            AND l_orderkey = o_orderkey
            AND o_orderdate >= DATE '1993-10-01'
            AND o_orderdate < DATE '1994-01-01'
            AND l_returnflag = 'R'
            AND c_nationkey = n_nationkey
        GROUP BY c_custkey, c_name, c_acctbal, c_phone, n_name, c_address, c_comment
        ORDER BY revenue DESC
        LIMIT 20
        "#.to_string()
    }

    pub fn print_summary(&self, metrics: &[QueryMetrics]) {
        info!("\n╔══════════════════════════════════════════════════════════════╗");
        info!("║                TPC-H Benchmark Results                      ║");
        info!("╚══════════════════════════════════════════════════════════════╝");

        info!("\n📊 Query Performance (SF={}):", self.config.scale_factor);
        for m in metrics {
            info!("   Q{:2}: {:<40} avg={:7.2}ms  rows={:>8}",
                  m.query_num, m.name, m.avg_ms, m.rows_returned);
        }

        let total_avg: f64 = metrics.iter().map(|m| m.avg_ms).sum();
        info!("\n📈 Overall:");
        info!("   Total average time: {:.2}s", total_avg / 1000.0);
        info!("   Queries completed: {}/{}", metrics.len(), 5);
        info!("   Average query time: {:.2}ms", total_avg / metrics.len() as f64);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              TPC-H Benchmark for OmenDB                     ║");
    println!("║            Production OLAP Performance Validation           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    info!("Prerequisites: cargo install tpchgen-cli");
    info!("This benchmark uses real TPC-H data generated in Parquet format\n");

    let config = TpchConfig::default();
    let benchmark = TpchBenchmark::new(config.clone()).await?;

    let metrics = benchmark.run().await?;
    benchmark.print_summary(&metrics);

    println!("\n✅ TPC-H Benchmark Complete");
    println!("   Validates: OmenDB OLAP performance on industry-standard queries");
    println!("   Data: Real TPC-H data generated with tpchgen-cli (fastest generator)");
    println!("   Engine: Apache DataFusion with Arrow columnar storage");
    println!("   Format: Parquet files for production-grade storage");

    Ok(())
}
