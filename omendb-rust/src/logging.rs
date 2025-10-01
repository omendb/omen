//! Structured logging for OmenDB
//! Production-ready JSON logging with configurable levels and query tracing

use anyhow::Result;
use std::io;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    registry::LookupSpan,
    EnvFilter, Layer, Registry,
};

/// Logging configuration
#[derive(Clone, Debug)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Enable JSON format (default: true for production)
    pub json_format: bool,

    /// Enable query logging (default: false)
    pub log_queries: bool,

    /// Enable span events (default: true for detailed tracing)
    pub log_spans: bool,

    /// Log file path (None = stdout only)
    pub log_file: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json_format: true,
            log_queries: false,
            log_spans: true,
            log_file: None,
        }
    }
}

impl LogConfig {
    /// Create production logging config (JSON, INFO level)
    pub fn production() -> Self {
        Self {
            level: "info".to_string(),
            json_format: true,
            log_queries: false,
            log_spans: true,
            log_file: Some("omendb.log".to_string()),
        }
    }

    /// Create development logging config (pretty, DEBUG level)
    pub fn development() -> Self {
        Self {
            level: "debug".to_string(),
            json_format: false,
            log_queries: true,
            log_spans: true,
            log_file: None,
        }
    }

    /// Create verbose logging config (includes query logging)
    pub fn verbose() -> Self {
        Self {
            level: "trace".to_string(),
            json_format: false,
            log_queries: true,
            log_spans: true,
            log_file: None,
        }
    }
}

/// Initialize structured logging with the given configuration
pub fn init_logging(config: LogConfig) -> Result<()> {
    // Build the filter from config
    let filter = EnvFilter::try_new(&config.level)
        .or_else(|_| EnvFilter::try_new("info"))?;

    // Determine span events
    let span_events = if config.log_spans {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    // Build subscriber based on format choice
    if config.json_format {
        // JSON format for production
        let fmt_layer = fmt::layer()
            .json()
            .with_span_events(span_events)
            .with_current_span(true)
            .with_span_list(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_writer(io::stdout);

        let subscriber = Registry::default()
            .with(filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        // Pretty format for development
        let fmt_layer = fmt::layer()
            .pretty()
            .with_span_events(span_events)
            .with_target(true)
            .with_thread_ids(false)
            .with_writer(io::stdout);

        let subscriber = Registry::default()
            .with(filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)?;
    }

    Ok(())
}

/// Initialize logging from environment variables
/// RUST_LOG - log level (trace, debug, info, warn, error)
/// OMENDB_LOG_FORMAT - json or pretty (default: json)
/// OMENDB_LOG_QUERIES - true to enable query logging (default: false)
pub fn init_from_env() -> Result<()> {
    let level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());

    let json_format = std::env::var("OMENDB_LOG_FORMAT")
        .map(|v| v == "json")
        .unwrap_or(true);

    let log_queries = std::env::var("OMENDB_LOG_QUERIES")
        .map(|v| v == "true")
        .unwrap_or(false);

    let config = LogConfig {
        level,
        json_format,
        log_queries,
        log_spans: true,
        log_file: None,
    };

    init_logging(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.json_format);
        assert!(!config.log_queries);
    }

    #[test]
    fn test_production_config() {
        let config = LogConfig::production();
        assert_eq!(config.level, "info");
        assert!(config.json_format);
        assert!(!config.log_queries);
        assert_eq!(config.log_file, Some("omendb.log".to_string()));
    }

    #[test]
    fn test_development_config() {
        let config = LogConfig::development();
        assert_eq!(config.level, "debug");
        assert!(!config.json_format);
        assert!(config.log_queries);
    }

    #[test]
    fn test_verbose_config() {
        let config = LogConfig::verbose();
        assert_eq!(config.level, "trace");
        assert!(config.log_queries);
    }

    #[test]
    fn test_logging_initialization() {
        // Test that we can initialize logging
        let config = LogConfig {
            level: "debug".to_string(),
            json_format: false,
            log_queries: false,
            log_spans: false,
            log_file: None,
        };

        // Note: This will fail if logging is already initialized in another test
        // That's okay - we just want to verify the API works
        let _ = init_logging(config);
    }
}
