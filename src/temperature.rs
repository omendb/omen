//! Temperature-based data classification for adaptive HTAP routing
//!
//! Tracks access patterns to classify data as hot/warm/cold:
//! - Hot data (>0.8): Frequently accessed, keep in ALEX index + memory
//! - Warm data (0.3-0.8): Moderate access, ALEX index + Parquet
//! - Cold data (<0.3): Rarely accessed, Parquet only
//!
//! Formula: T = α×Frequency + β×Recency
//! - Frequency: Access count / window_threshold (normalized to 0-1)
//! - Recency: 1 - (seconds_since_access / window_seconds) (normalized to 0-1)
//! - Default weights: α=0.6 (frequency), β=0.4 (recency)

use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Data tier based on temperature
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataTier {
    /// Hot data: >0.8 temperature (keep in ALEX + memory)
    Hot,

    /// Warm data: 0.3-0.8 temperature (ALEX + Parquet)
    Warm,

    /// Cold data: <0.3 temperature (Parquet only)
    Cold,
}

/// Key range for temperature tracking
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct KeyRange {
    /// Start of range (inclusive)
    pub start: i64,

    /// End of range (inclusive)
    pub end: i64,
}

impl KeyRange {
    /// Create new key range
    pub fn new(start: i64, end: i64) -> Self {
        Self { start, end }
    }

    /// Create single-key range
    pub fn point(key: i64) -> Self {
        Self { start: key, end: key }
    }

    /// Check if range contains a key
    pub fn contains(&self, key: i64) -> bool {
        key >= self.start && key <= self.end
    }

    /// Check if two ranges overlap
    pub fn overlaps(&self, other: &KeyRange) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

/// Access tracking data for a key range
#[derive(Debug, Clone)]
struct AccessData {
    /// Number of accesses in current window
    access_count: u64,

    /// Last access time
    last_access: Instant,

    /// Total accesses (all time)
    total_accesses: u64,
}

impl AccessData {
    fn new() -> Self {
        Self {
            access_count: 0,
            last_access: Instant::now(),
            total_accesses: 0,
        }
    }

    fn record_access(&mut self) {
        self.access_count += 1;
        self.total_accesses += 1;
        self.last_access = Instant::now();
    }

    fn reset_window(&mut self) {
        self.access_count = 0;
    }
}

/// Temperature model for tracking data access patterns
pub struct TemperatureModel {
    /// Access data per key range
    access_data: Arc<RwLock<HashMap<KeyRange, AccessData>>>,

    /// Time window for frequency calculation (default: 5 minutes)
    window_duration: Duration,

    /// Frequency threshold for normalization (accesses per window)
    frequency_threshold: u64,

    /// Weight for frequency component (default: 0.6)
    alpha: f64,

    /// Weight for recency component (default: 0.4)
    beta: f64,

    /// Hot threshold (default: 0.8)
    hot_threshold: f64,

    /// Cold threshold (default: 0.3)
    cold_threshold: f64,

    /// Last window reset time
    last_reset: Arc<RwLock<Instant>>,
}

impl TemperatureModel {
    /// Create new temperature model with default parameters
    pub fn new() -> Self {
        Self {
            access_data: Arc::new(RwLock::new(HashMap::new())),
            window_duration: Duration::from_secs(300), // 5 minutes
            frequency_threshold: 1000,
            alpha: 0.6,
            beta: 0.4,
            hot_threshold: 0.8,
            cold_threshold: 0.3,
            last_reset: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create with custom parameters
    pub fn with_params(
        window_seconds: u64,
        frequency_threshold: u64,
        alpha: f64,
        beta: f64,
        hot_threshold: f64,
        cold_threshold: f64,
    ) -> Self {
        Self {
            access_data: Arc::new(RwLock::new(HashMap::new())),
            window_duration: Duration::from_secs(window_seconds),
            frequency_threshold,
            alpha,
            beta,
            hot_threshold,
            cold_threshold,
            last_reset: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record access to a key
    pub fn record_access(&self, key: &Value) {
        if let Some(key_int) = self.value_to_i64(key) {
            let range = KeyRange::point(key_int);
            self.record_range_access(&range);
        }
    }

    /// Record access to a key range
    pub fn record_range_access(&self, range: &KeyRange) {
        self.check_window_reset();

        let mut data = self.access_data.write().unwrap();
        data.entry(range.clone())
            .or_insert_with(AccessData::new)
            .record_access();
    }

    /// Get temperature for a key (0.0 to 1.0)
    pub fn get_temperature(&self, key: &Value) -> f64 {
        if let Some(key_int) = self.value_to_i64(key) {
            let range = KeyRange::point(key_int);
            self.get_range_temperature(&range)
        } else {
            0.0 // Unknown type, assume cold
        }
    }

    /// Get temperature for a key range
    pub fn get_range_temperature(&self, range: &KeyRange) -> f64 {
        let data = self.access_data.read().unwrap();

        // Find all ranges that overlap with query range
        let mut total_freq = 0u64;
        let mut min_recency_secs = u64::MAX;
        let mut found_any = false;

        for (tracked_range, access_data) in data.iter() {
            if tracked_range.overlaps(range) {
                total_freq += access_data.access_count;
                let recency = access_data.last_access.elapsed().as_secs();
                min_recency_secs = min_recency_secs.min(recency);
                found_any = true;
            }
        }

        if !found_any {
            return 0.0; // No access data, assume cold
        }

        // Calculate frequency score (0-1)
        let freq_score = (total_freq as f64 / self.frequency_threshold as f64).min(1.0);

        // Calculate recency score (0-1)
        let window_secs = self.window_duration.as_secs();
        let recency_score = (1.0 - (min_recency_secs as f64 / window_secs as f64)).max(0.0);

        // Combined temperature: T = α×Frequency + β×Recency
        self.alpha * freq_score + self.beta * recency_score
    }

    /// Classify key as hot/warm/cold
    pub fn classify(&self, key: &Value) -> DataTier {
        let temp = self.get_temperature(key);
        self.classify_temperature(temp)
    }

    /// Classify range as hot/warm/cold
    pub fn classify_range(&self, range: &KeyRange) -> DataTier {
        let temp = self.get_range_temperature(range);
        self.classify_temperature(temp)
    }

    /// Classify temperature value
    fn classify_temperature(&self, temp: f64) -> DataTier {
        if temp > self.hot_threshold {
            DataTier::Hot
        } else if temp > self.cold_threshold {
            DataTier::Warm
        } else {
            DataTier::Cold
        }
    }

    /// Check if key is hot
    pub fn is_hot(&self, key: &Value) -> bool {
        matches!(self.classify(key), DataTier::Hot)
    }

    /// Check if key is cold
    pub fn is_cold(&self, key: &Value) -> bool {
        matches!(self.classify(key), DataTier::Cold)
    }

    /// Reset window if duration has passed
    fn check_window_reset(&self) {
        let mut last_reset = self.last_reset.write().unwrap();
        if last_reset.elapsed() >= self.window_duration {
            // Reset window
            let mut data = self.access_data.write().unwrap();
            for access_data in data.values_mut() {
                access_data.reset_window();
            }
            *last_reset = Instant::now();
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> TemperatureStats {
        let data = self.access_data.read().unwrap();

        let mut hot_count = 0;
        let mut warm_count = 0;
        let mut cold_count = 0;
        let mut total_accesses = 0u64;

        for (range, access_data) in data.iter() {
            total_accesses += access_data.total_accesses;
            let temp = self.get_range_temperature(range);
            match self.classify_temperature(temp) {
                DataTier::Hot => hot_count += 1,
                DataTier::Warm => warm_count += 1,
                DataTier::Cold => cold_count += 1,
            }
        }

        TemperatureStats {
            tracked_ranges: data.len(),
            hot_count,
            warm_count,
            cold_count,
            total_accesses,
        }
    }

    /// Clear all tracking data
    pub fn clear(&self) {
        self.access_data.write().unwrap().clear();
        *self.last_reset.write().unwrap() = Instant::now();
    }

    /// Convert Value to i64 for range tracking
    fn value_to_i64(&self, value: &Value) -> Option<i64> {
        match value {
            Value::Int64(v) => Some(*v),
            Value::UInt64(v) => Some(*v as i64),
            _ => None,
        }
    }
}

impl Default for TemperatureModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Temperature tracking statistics
#[derive(Debug, Clone)]
pub struct TemperatureStats {
    /// Number of tracked key ranges
    pub tracked_ranges: usize,

    /// Number of hot ranges
    pub hot_count: usize,

    /// Number of warm ranges
    pub warm_count: usize,

    /// Number of cold ranges
    pub cold_count: usize,

    /// Total accesses (all time)
    pub total_accesses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_key_range() {
        let range = KeyRange::new(10, 20);
        assert!(range.contains(15));
        assert!(!range.contains(5));
        assert!(!range.contains(25));

        let point = KeyRange::point(42);
        assert!(point.contains(42));
        assert!(!point.contains(41));
    }

    #[test]
    fn test_range_overlap() {
        let range1 = KeyRange::new(10, 20);
        let range2 = KeyRange::new(15, 25);
        let range3 = KeyRange::new(30, 40);

        assert!(range1.overlaps(&range2));
        assert!(!range1.overlaps(&range3));
    }

    #[test]
    fn test_temperature_calculation() {
        let model = TemperatureModel::with_params(
            60,   // 1 minute window
            100,  // 100 accesses = 1.0 frequency score
            0.6,  // alpha
            0.4,  // beta
            0.8,  // hot threshold
            0.3,  // cold threshold
        );

        let key = Value::Int64(42);

        // No accesses: cold
        assert_eq!(model.classify(&key), DataTier::Cold);

        // Access 50 times (0.5 frequency score)
        for _ in 0..50 {
            model.record_access(&key);
        }

        let temp = model.get_temperature(&key);
        // Frequency: 50/100 = 0.5, Recency: ~1.0 (just accessed)
        // T = 0.6*0.5 + 0.4*1.0 = 0.3 + 0.4 = 0.7 (warm)
        assert!(temp > 0.6 && temp < 0.8);
        assert_eq!(model.classify(&key), DataTier::Warm);

        // Access 100 more times (1.0 frequency score)
        for _ in 0..100 {
            model.record_access(&key);
        }

        let temp = model.get_temperature(&key);
        // Frequency: 150/100 = 1.0 (capped), Recency: ~1.0
        // T = 0.6*1.0 + 0.4*1.0 = 1.0 (hot)
        assert!(temp >= 0.8);
        assert_eq!(model.classify(&key), DataTier::Hot);
    }

    #[test]
    fn test_recency_decay() {
        let model = TemperatureModel::with_params(
            2,    // 2 second window
            100,  // frequency threshold
            0.6,  // alpha
            0.4,  // beta
            0.8,  // hot threshold
            0.3,  // cold threshold
        );

        let key = Value::Int64(42);

        // Access once
        model.record_access(&key);
        let temp_initial = model.get_temperature(&key);
        assert!(temp_initial > 0.0);

        // Wait 1 second (half window)
        thread::sleep(Duration::from_secs(1));
        let temp_mid = model.get_temperature(&key);
        assert!(temp_mid < temp_initial); // Temperature decayed

        // Wait another second (full window)
        thread::sleep(Duration::from_secs(1));
        let temp_end = model.get_temperature(&key);
        assert!(temp_end < temp_mid); // Further decay
    }

    #[test]
    fn test_range_temperature() {
        let model = TemperatureModel::new();

        // Access individual keys
        model.record_access(&Value::Int64(10));
        model.record_access(&Value::Int64(15));
        model.record_access(&Value::Int64(20));

        // Check range temperature
        let range = KeyRange::new(10, 20);
        let temp = model.get_range_temperature(&range);
        assert!(temp > 0.0); // Should aggregate accesses
    }

    #[test]
    fn test_statistics() {
        let model = TemperatureModel::with_params(60, 100, 0.6, 0.4, 0.8, 0.3);

        // Create hot data (150 accesses = 1.0 freq, recent = 1.0 recency => T = 1.0)
        for _ in 0..150 {
            model.record_access(&Value::Int64(1));
        }

        // Create warm data (50 accesses = 0.5 freq, recent = 1.0 recency => T = 0.7)
        for _ in 0..50 {
            model.record_access(&Value::Int64(2));
        }

        // Create cold data (1 access = 0.01 freq, but recent = 1.0 recency => T = 0.406, still warm!)
        // To be truly cold, data needs both low frequency AND low recency
        // Let's not access key 3, so it has temp = 0

        let stats = model.get_stats();
        assert_eq!(stats.tracked_ranges, 2); // Only 2 keys accessed
        assert_eq!(stats.hot_count, 1); // key 1
        assert_eq!(stats.warm_count, 1); // key 2 (note: key with 1 access would also be warm)
        assert_eq!(stats.cold_count, 0); // No cold data (need no accesses or very old access)
        assert_eq!(stats.total_accesses, 200);
    }

    #[test]
    fn test_window_reset() {
        let model = TemperatureModel::with_params(
            1,    // 1 second window
            100,  // frequency threshold
            0.6, 0.4, 0.8, 0.3,
        );

        let key = Value::Int64(42);

        // Access 100 times
        for _ in 0..100 {
            model.record_access(&key);
        }

        let temp_before = model.get_temperature(&key);
        assert!(temp_before > 0.8); // Hot

        // Wait for window reset
        thread::sleep(Duration::from_millis(1100));

        // Record new access (triggers window reset)
        model.record_access(&key);

        // Frequency count should be reset (now only 1 access in new window)
        let temp_after = model.get_temperature(&key);
        assert!(temp_after < temp_before); // Temperature dropped
    }

    #[test]
    fn test_clear() {
        let model = TemperatureModel::new();

        model.record_access(&Value::Int64(1));
        model.record_access(&Value::Int64(2));

        let stats_before = model.get_stats();
        assert_eq!(stats_before.tracked_ranges, 2);

        model.clear();

        let stats_after = model.get_stats();
        assert_eq!(stats_after.tracked_ranges, 0);
    }
}
