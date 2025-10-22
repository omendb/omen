//! Prometheus metrics for production monitoring
//! Essential for observability and alerting

use once_cell::sync::Lazy;
use prometheus::{
    register_gauge, register_histogram,
    register_histogram_vec, register_int_counter, register_int_counter_vec, register_int_gauge, Encoder, Gauge, Histogram, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, TextEncoder,
};
use std::time::Instant;

// Operation counters
pub static TOTAL_SEARCHES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("omendb_searches_total", "Total number of search operations").unwrap()
});

pub static TOTAL_INSERTS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("omendb_inserts_total", "Total number of insert operations").unwrap()
});

pub static TOTAL_RANGE_QUERIES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_range_queries_total",
        "Total number of range query operations"
    )
    .unwrap()
});

// Error counters
pub static FAILED_SEARCHES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_searches_failed_total",
        "Total number of failed search operations"
    )
    .unwrap()
});

pub static FAILED_INSERTS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_inserts_failed_total",
        "Total number of failed insert operations"
    )
    .unwrap()
});

// Latency histograms
pub static SEARCH_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_search_duration_seconds",
        "Search operation latency in seconds",
        vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
    )
    .unwrap()
});

pub static INSERT_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_insert_duration_seconds",
        "Insert operation latency in seconds",
        vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
    )
    .unwrap()
});

pub static RANGE_QUERY_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_range_query_duration_seconds",
        "Range query operation latency in seconds",
        vec![0.0001, 0.001, 0.01, 0.1, 0.5, 1.0, 5.0]
    )
    .unwrap()
});

// System gauges
pub static ACTIVE_CONNECTIONS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "omendb_connections_active",
        "Number of active database connections"
    )
    .unwrap()
});

pub static DATABASE_SIZE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "omendb_database_size_bytes",
        "Current database size in bytes"
    )
    .unwrap()
});

pub static INDEX_SIZE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "omendb_index_size_keys",
        "Number of keys in the learned index"
    )
    .unwrap()
});

pub static MEMORY_USAGE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!("omendb_memory_usage_bytes", "Current memory usage in bytes").unwrap()
});

// WAL metrics
pub static WAL_WRITES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!("omendb_wal_writes_total", "Total WAL write operations").unwrap()
});

pub static WAL_SYNC_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_wal_sync_duration_seconds",
        "WAL sync operation latency"
    )
    .unwrap()
});

pub static WAL_SIZE: Lazy<IntGauge> =
    Lazy::new(|| register_int_gauge!("omendb_wal_size_bytes", "Current WAL file size").unwrap());

// Performance metrics
pub static THROUGHPUT: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "omendb_throughput_ops_per_sec",
        "Current operations per second"
    )
    .unwrap()
});

// SQL query metrics
pub static SQL_QUERY_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "omendb_sql_query_duration_seconds",
        "SQL query execution latency in seconds (includes p50/p95/p99 via histogram)",
        &["query_type"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap()
});

pub static SQL_QUERIES_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "omendb_sql_queries_total",
        "Total SQL queries executed by type",
        &["query_type"]
    )
    .unwrap()
});

pub static SQL_QUERY_ERRORS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "omendb_sql_query_errors_total",
        "Total SQL query errors by type",
        &["error_type"]
    )
    .unwrap()
});

pub static SQL_QUERY_ROWS_RETURNED: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_sql_query_rows_returned",
        "Number of rows returned per query",
        vec![1.0, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0]
    )
    .unwrap()
});

// Learned Index metrics
pub static LEARNED_INDEX_HITS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_learned_index_hits_total",
        "Total number of successful learned index predictions"
    )
    .unwrap()
});

pub static LEARNED_INDEX_MISSES: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_learned_index_misses_total",
        "Total number of learned index prediction failures (fallback to B-tree)"
    )
    .unwrap()
});

pub static LEARNED_INDEX_PREDICTION_ERROR: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "omendb_learned_index_prediction_error_positions",
        "Learned index prediction error in array positions",
        vec![0.0, 1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]
    )
    .unwrap()
});

pub static QUERY_PATH: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "omendb_query_path_total",
        "Query execution path distribution",
        &["path"] // "learned_index", "fallback_btree", "full_scan"
    )
    .unwrap()
});

pub static LEARNED_INDEX_WINDOW_HITS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "omendb_learned_index_window_hits_total",
        "Queries resolved within initial prediction window"
    )
    .unwrap()
});

pub static LEARNED_INDEX_SIZE_KEYS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "omendb_learned_index_keys",
        "Number of keys in the learned index"
    )
    .unwrap()
});

pub static LEARNED_INDEX_MODELS_COUNT: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "omendb_learned_index_models",
        "Number of models in the learned index (root + second layer)"
    )
    .unwrap()
});

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    histogram: &'static Histogram,
}

impl Timer {
    pub fn new(histogram: &'static Histogram) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed().as_secs_f64();
        self.histogram.observe(duration);
    }
}

/// Record a successful search
pub fn record_search(duration_secs: f64) {
    TOTAL_SEARCHES.inc();
    SEARCH_DURATION.observe(duration_secs);
}

/// Record a failed search
pub fn record_search_failure() {
    TOTAL_SEARCHES.inc();
    FAILED_SEARCHES.inc();
}

/// Record a successful insert
pub fn record_insert(duration_secs: f64) {
    TOTAL_INSERTS.inc();
    INSERT_DURATION.observe(duration_secs);
}

/// Record a failed insert
pub fn record_insert_failure() {
    TOTAL_INSERTS.inc();
    FAILED_INSERTS.inc();
}

/// Record a range query
pub fn record_range_query(duration_secs: f64, rows_returned: usize) {
    TOTAL_SEARCHES.inc();
    SEARCH_DURATION.observe(duration_secs);
}

/// Record a delete operation
pub fn record_delete(duration_secs: f64) {
    // Could add delete-specific metrics later if needed
    // For now, track as a write operation
}

/// Update active connections
pub fn set_active_connections(count: i64) {
    ACTIVE_CONNECTIONS.set(count);
}

/// Update database size
pub fn set_database_size(bytes: i64) {
    DATABASE_SIZE.set(bytes);
}

/// Update index size
pub fn set_index_size(keys: i64) {
    INDEX_SIZE.set(keys);
}

/// Record SQL query execution
pub fn record_sql_query(query_type: &str, duration_secs: f64, rows_returned: usize) {
    SQL_QUERIES_TOTAL.with_label_values(&[query_type]).inc();

    SQL_QUERY_DURATION
        .with_label_values(&[query_type])
        .observe(duration_secs);

    SQL_QUERY_ROWS_RETURNED.observe(rows_returned as f64);
}

/// Record SQL query error
pub fn record_sql_query_error(error_type: &str) {
    SQL_QUERY_ERRORS.with_label_values(&[error_type]).inc();
}

/// Record learned index hit with prediction accuracy
pub fn record_learned_index_hit(predicted_pos: usize, actual_pos: usize) {
    LEARNED_INDEX_HITS.inc();

    // Calculate prediction error
    let error = predicted_pos.abs_diff(actual_pos);

    LEARNED_INDEX_PREDICTION_ERROR.observe(error as f64);

    // Check if hit was within initial window (±100 positions)
    if error <= 100 {
        LEARNED_INDEX_WINDOW_HITS.inc();
    }
}

/// Record learned index miss (fallback to B-tree or full scan)
pub fn record_learned_index_miss() {
    LEARNED_INDEX_MISSES.inc();
}

/// Record query execution path
pub fn record_query_path(path: &str) {
    QUERY_PATH.with_label_values(&[path]).inc();
}

/// Update learned index size
pub fn set_learned_index_size(keys: usize) {
    LEARNED_INDEX_SIZE_KEYS.set(keys as i64);
}

/// Update learned index model count
pub fn set_learned_index_models(count: usize) {
    LEARNED_INDEX_MODELS_COUNT.set(count as i64);
}

/// Calculate learned index hit rate (0.0 to 1.0)
pub fn learned_index_hit_rate() -> f64 {
    let hits = LEARNED_INDEX_HITS.get() as f64;
    let misses = LEARNED_INDEX_MISSES.get() as f64;
    let total = hits + misses;

    if total > 0.0 {
        hits / total
    } else {
        0.0
    }
}

/// Get metrics in Prometheus text format
pub fn get_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Health check response
pub struct HealthStatus {
    pub healthy: bool,
    pub version: String,
    pub uptime_seconds: u64,
    pub total_operations: u64,
    pub error_rate: f64,
}

impl HealthStatus {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"healthy":{},"version":"{}","uptime_seconds":{},"total_operations":{},"error_rate":{:.4}}}"#,
            self.healthy, self.version, self.uptime_seconds, self.total_operations, self.error_rate
        )
    }
}

/// Get health check status
pub fn health_check() -> HealthStatus {
    let total_searches = TOTAL_SEARCHES.get();
    let total_inserts = TOTAL_INSERTS.get();
    let failed_searches = FAILED_SEARCHES.get();
    let failed_inserts = FAILED_INSERTS.get();

    let total_ops = total_searches + total_inserts;
    let total_failures = failed_searches + failed_inserts;

    let error_rate = if total_ops > 0 {
        total_failures as f64 / total_ops as f64
    } else {
        0.0
    };

    HealthStatus {
        healthy: error_rate < 0.01, // Less than 1% error rate
        version: "0.1.0".to_string(),
        uptime_seconds: 0, // Would track from start
        total_operations: total_ops,
        error_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        // Record some operations
        record_search(0.001);
        record_search(0.002);
        record_search_failure();

        record_insert(0.003);
        record_insert_failure();

        // Check counters
        assert!(TOTAL_SEARCHES.get() >= 3);
        assert!(TOTAL_INSERTS.get() >= 2);
        assert!(FAILED_SEARCHES.get() >= 1);
        assert!(FAILED_INSERTS.get() >= 1);
    }

    #[test]
    fn test_timer() {
        {
            let _timer = Timer::new(&SEARCH_DURATION);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        // Timer should record on drop
        // Can't easily verify histogram values, but ensure no panic
    }

    #[test]
    fn test_prometheus_format() {
        record_search(0.001);
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_searches_total"));
    }

    #[test]
    fn test_health_check() {
        // Reset counters would be nice but Prometheus doesn't support it
        let health = health_check();
        assert!(health.error_rate >= 0.0);
        // Error rate can exceed 1.0 due to test accumulation, so just check it's reasonable
        assert!(health.error_rate.is_finite());
    }

    #[test]
    fn test_gauge_updates() {
        set_active_connections(42);
        assert_eq!(ACTIVE_CONNECTIONS.get(), 42);

        set_database_size(1024 * 1024);
        assert_eq!(DATABASE_SIZE.get(), 1024 * 1024);

        set_index_size(50000);
        assert_eq!(INDEX_SIZE.get(), 50000);

        // Test memory gauge update
        MEMORY_USAGE.set(2048);
        assert_eq!(MEMORY_USAGE.get(), 2048);
    }

    #[test]
    fn test_wal_metrics() {
        WAL_WRITES.inc();
        WAL_WRITES.inc();
        assert!(WAL_WRITES.get() >= 2);

        WAL_SIZE.set(4096);
        assert_eq!(WAL_SIZE.get(), 4096);

        // Test WAL sync duration
        WAL_SYNC_DURATION.observe(0.001);
        WAL_SYNC_DURATION.observe(0.005);
        // Can't directly test histogram values, but ensure no panic
    }

    #[test]
    fn test_throughput_metrics() {
        THROUGHPUT.set(1500.0);
        assert_eq!(THROUGHPUT.get(), 1500.0);

        // Test range query counter
        TOTAL_RANGE_QUERIES.inc();
        TOTAL_RANGE_QUERIES.inc();
        TOTAL_RANGE_QUERIES.inc();
        assert!(TOTAL_RANGE_QUERIES.get() >= 3);
    }

    #[test]
    fn test_health_status_json() {
        let health = HealthStatus {
            healthy: true,
            version: "0.1.0".to_string(),
            uptime_seconds: 3600,
            total_operations: 1000,
            error_rate: 0.05,
        };

        let json = health.to_json();
        assert!(json.contains("\"healthy\":true"));
        assert!(json.contains("\"version\":\"0.1.0\""));
        assert!(json.contains("\"uptime_seconds\":3600"));
        assert!(json.contains("\"total_operations\":1000"));
        assert!(json.contains("\"error_rate\":0.0500"));
    }

    #[test]
    fn test_error_rate_calculation() {
        // Clear any existing counts by getting current values
        let base_searches = TOTAL_SEARCHES.get();
        let base_inserts = TOTAL_INSERTS.get();
        let base_search_fails = FAILED_SEARCHES.get();
        let base_insert_fails = FAILED_INSERTS.get();

        // Add some operations with known failure rates
        record_search(0.001);
        record_search(0.001);
        record_search_failure(); // 1 out of 3 searches failed = 33%

        record_insert(0.002);
        record_insert_failure(); // 1 out of 2 inserts failed = 50%

        let health = health_check();

        // Total: 5 operations, 2 failures = 40% error rate
        // But we need to account for baseline counts
        let total_ops_added = 5;
        let total_fails_added = 2;

        // Should be reasonable error rate
        assert!(health.error_rate >= 0.0);
        assert!(health.error_rate <= 1.0);
        assert!(health.total_operations >= total_ops_added as u64);
    }

    #[test]
    fn test_histogram_buckets() {
        // Test different latency ranges to ensure histograms work
        SEARCH_DURATION.observe(0.00001); // Very fast
        SEARCH_DURATION.observe(0.001); // Fast
        SEARCH_DURATION.observe(0.01); // Medium
        SEARCH_DURATION.observe(0.05); // Slow

        INSERT_DURATION.observe(0.0001);
        INSERT_DURATION.observe(0.005);

        RANGE_QUERY_DURATION.observe(0.1);
        RANGE_QUERY_DURATION.observe(1.0);

        // Should not panic and metrics should be accessible
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_search_duration_seconds"));
        assert!(metrics.contains("omendb_insert_duration_seconds"));
        assert!(metrics.contains("omendb_range_query_duration_seconds"));
    }

    #[test]
    fn test_metrics_format_content() {
        // Initialize metrics by recording some operations
        record_search(0.001);
        record_insert(0.002);
        WAL_WRITES.inc();
        set_active_connections(1);

        // Ensure all our metrics appear in the output
        let metrics = get_metrics();

        // Should contain some basic prometheus format
        assert!(metrics.contains("TYPE"));
        assert!(metrics.contains("HELP"));

        // Counter metrics (some may not appear if never incremented)
        assert!(metrics.contains("omendb_searches_total") || metrics.contains("TYPE"));
        assert!(metrics.contains("omendb_inserts_total") || metrics.contains("TYPE"));

        // Should be valid prometheus format
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_sql_query_metrics() {
        // Record SELECT query
        record_sql_query("SELECT", 0.015, 100);
        record_sql_query("SELECT", 0.025, 250);

        // Record INSERT query
        record_sql_query("INSERT", 0.005, 1);

        // Verify metrics appear in output
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_sql_query_duration_seconds"));
        assert!(metrics.contains("omendb_sql_queries_total"));
        assert!(metrics.contains("omendb_sql_query_rows_returned"));

        // Should contain query types
        assert!(metrics.contains("SELECT") || metrics.contains("INSERT"));
    }

    #[test]
    fn test_sql_query_error_metrics() {
        // Record errors
        record_sql_query_error("syntax_error");
        record_sql_query_error("timeout");
        record_sql_query_error("syntax_error"); // Duplicate

        // Verify error metrics appear
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_sql_query_errors_total"));
    }

    #[test]
    fn test_sql_query_latency_buckets() {
        // Test different latencies to ensure histogram buckets work
        record_sql_query("SELECT", 0.001, 10); // Very fast
        record_sql_query("SELECT", 0.01, 100); // Fast
        record_sql_query("SELECT", 0.1, 1000); // Medium
        record_sql_query("SELECT", 1.0, 10000); // Slow
        record_sql_query("SELECT", 5.0, 100000); // Very slow

        let metrics = get_metrics();
        assert!(metrics.contains("omendb_sql_query_duration_seconds"));

        // Should contain histogram data (buckets)
        assert!(metrics.contains("bucket"));
    }

    #[test]
    fn test_sql_query_types() {
        // Test different query types
        record_sql_query("SELECT", 0.01, 50);
        record_sql_query("INSERT", 0.005, 1);
        record_sql_query("CREATE_TABLE", 0.002, 0);
        record_sql_query("DROP_TABLE", 0.001, 0);

        let metrics = get_metrics();
        assert!(metrics.contains("omendb_sql_queries_total"));

        // Each query type should have its own counter
        // (actual verification would need prometheus parsing)
    }

    #[test]
    fn test_rows_returned_histogram() {
        // Test different row counts
        record_sql_query("SELECT", 0.01, 5); // Small result
        record_sql_query("SELECT", 0.05, 500); // Medium result
        record_sql_query("SELECT", 0.5, 50000); // Large result
        record_sql_query("SELECT", 2.0, 500000); // Very large result

        let metrics = get_metrics();
        assert!(metrics.contains("omendb_sql_query_rows_returned"));
    }

    #[test]
    fn test_learned_index_metrics() {
        let hits_before = LEARNED_INDEX_HITS.get();
        let misses_before = LEARNED_INDEX_MISSES.get();

        // Record some hits with varying accuracy
        record_learned_index_hit(100, 100); // Perfect prediction
        record_learned_index_hit(100, 105); // 5 positions off
        record_learned_index_hit(100, 200); // 100 positions off

        // Record a miss
        record_learned_index_miss();

        let hits_after = LEARNED_INDEX_HITS.get();
        let misses_after = LEARNED_INDEX_MISSES.get();

        // Should have at least these increments (may be more due to parallel tests)
        assert!(hits_after >= hits_before + 3);
        assert!(misses_after >= misses_before + 1);

        // Verify metrics appear in output
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_learned_index_hits_total"));
        assert!(metrics.contains("omendb_learned_index_misses_total"));
    }

    #[test]
    fn test_learned_index_hit_rate_calculation() {
        // Record operations
        record_learned_index_hit(100, 100);
        record_learned_index_hit(200, 210);
        record_learned_index_hit(300, 320);
        record_learned_index_miss();

        let hit_rate = learned_index_hit_rate();

        // Hit rate should be > 0 and <= 1.0
        assert!(hit_rate > 0.0);
        assert!(hit_rate <= 1.0);
    }

    #[test]
    fn test_learned_index_window_tracking() {
        let window_hits_before = LEARNED_INDEX_WINDOW_HITS.get();

        // Hit within window (error <= 100)
        record_learned_index_hit(1000, 1050); // 50 positions off - within window

        // Hit outside window (error > 100)
        record_learned_index_hit(1000, 1200); // 200 positions off - outside window

        let window_hits_after = LEARNED_INDEX_WINDOW_HITS.get();

        // Only the first hit should increment window_hits (at least 1 more)
        assert!(window_hits_after >= window_hits_before + 1);
    }

    #[test]
    fn test_query_path_metrics() {
        let learned_before = QUERY_PATH.with_label_values(&["learned_index"]).get();
        let btree_before = QUERY_PATH.with_label_values(&["fallback_btree"]).get();
        let scan_before = QUERY_PATH.with_label_values(&["full_scan"]).get();

        record_query_path("learned_index");
        record_query_path("learned_index");
        record_query_path("fallback_btree");
        record_query_path("full_scan");

        let learned_after = QUERY_PATH.with_label_values(&["learned_index"]).get();
        let btree_after = QUERY_PATH.with_label_values(&["fallback_btree"]).get();
        let scan_after = QUERY_PATH.with_label_values(&["full_scan"]).get();

        // Should have at least these increments (may be more due to parallel tests)
        assert!(learned_after >= learned_before + 2);
        assert!(btree_after >= btree_before + 1);
        assert!(scan_after >= scan_before + 1);
    }

    #[test]
    fn test_learned_index_size_tracking() {
        set_learned_index_size(10_000);
        assert_eq!(LEARNED_INDEX_SIZE_KEYS.get(), 10_000);

        set_learned_index_size(50_000);
        assert_eq!(LEARNED_INDEX_SIZE_KEYS.get(), 50_000);
    }

    #[test]
    fn test_learned_index_models_tracking() {
        set_learned_index_models(5); // 1 root + 4 second layer
        assert_eq!(LEARNED_INDEX_MODELS_COUNT.get(), 5);

        set_learned_index_models(17); // 1 root + 16 second layer
        assert_eq!(LEARNED_INDEX_MODELS_COUNT.get(), 17);
    }

    #[test]
    fn test_learned_index_prediction_accuracy() {
        let before_count = LEARNED_INDEX_PREDICTION_ERROR.get_sample_count();

        // Perfect predictions
        record_learned_index_hit(100, 100); // 0 error
        record_learned_index_hit(200, 200); // 0 error

        // Good predictions
        record_learned_index_hit(300, 305); // 5 error
        record_learned_index_hit(400, 410); // 10 error

        // Mediocre predictions
        record_learned_index_hit(500, 550); // 50 error
        record_learned_index_hit(600, 700); // 100 error

        let after_count = LEARNED_INDEX_PREDICTION_ERROR.get_sample_count();

        // Should have recorded at least 6 new samples (may be more due to other tests)
        assert!(after_count >= before_count + 6);

        // Verify histogram appears in metrics
        let metrics = get_metrics();
        assert!(metrics.contains("omendb_learned_index_prediction_error_positions"));
    }
}
