//! Resource management for OmenDB Server
//! 
//! Handles memory monitoring, connection pooling, and resource allocation
//! across tenants and operations.

use crate::config::Config;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// Resource manager for monitoring and controlling system resources
pub struct ResourceManager {
    /// Configuration
    config: Config,
    /// Memory monitor
    memory_monitor: Arc<MemoryMonitor>,
    /// Resource usage tracking
    usage_tracker: Arc<RwLock<ResourceUsage>>,
}

/// Memory monitoring component
pub struct MemoryMonitor {
    /// Current memory usage in bytes
    current_usage: Arc<RwLock<u64>>,
    /// Peak memory usage
    peak_usage: Arc<RwLock<u64>>,
    /// Memory limit in bytes
    memory_limit: u64,
}

/// System resource usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Total memory allocated (bytes)
    pub memory_allocated: u64,
    /// Memory in use (bytes)
    pub memory_used: u64,
    /// Number of active connections
    pub active_connections: usize,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Disk usage (bytes)
    pub disk_usage_bytes: u64,
    /// Network bytes sent
    pub network_bytes_sent: u64,
    /// Network bytes received
    pub network_bytes_received: u64,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Current usage in bytes
    pub used_bytes: u64,
    /// Peak usage in bytes
    pub peak_bytes: u64,
    /// Usage as percentage of limit
    pub used_percent: f64,
    /// Available memory in bytes
    pub available_bytes: u64,
}

impl ResourceManager {
    /// Create a new resource manager
    #[instrument(level = "info")]
    pub fn new(config: Config) -> Self {
        info!("Initializing resource manager");
        
        // Calculate memory limit (default to 80% of available memory)
        let memory_limit = Self::calculate_memory_limit();
        
        let memory_monitor = Arc::new(MemoryMonitor {
            current_usage: Arc::new(RwLock::new(0)),
            peak_usage: Arc::new(RwLock::new(0)),
            memory_limit,
        });

        ResourceManager {
            config,
            memory_monitor,
            usage_tracker: Arc::new(RwLock::new(ResourceUsage::default())),
        }
    }

    /// Start background monitoring tasks
    #[instrument(level = "info", skip(self))]
    pub async fn start_monitoring(&self) {
        info!("Starting resource monitoring tasks");
        
        // Start memory monitoring
        let memory_monitor = Arc::clone(&self.memory_monitor);
        let usage_tracker = Arc::clone(&self.usage_tracker);
        
        tokio::spawn(async move {
            Self::memory_monitoring_task(memory_monitor, usage_tracker).await;
        });

        // Start CPU monitoring
        let usage_tracker = Arc::clone(&self.usage_tracker);
        tokio::spawn(async move {
            Self::cpu_monitoring_task(usage_tracker).await;
        });
    }

    /// Get current memory usage
    #[instrument(level = "debug", skip(self))]
    pub async fn get_memory_usage(&self) -> MemoryUsage {
        let current = *self.memory_monitor.current_usage.read().await;
        let peak = *self.memory_monitor.peak_usage.read().await;
        let limit = self.memory_monitor.memory_limit;
        
        MemoryUsage {
            used_bytes: current,
            peak_bytes: peak,
            used_percent: (current as f64 / limit as f64) * 100.0,
            available_bytes: limit.saturating_sub(current),
        }
    }

    /// Get overall resource usage
    pub async fn get_resource_usage(&self) -> ResourceUsage {
        let usage = self.usage_tracker.read().await;
        usage.clone()
    }

    /// Check if system is under memory pressure
    pub async fn is_memory_pressure(&self) -> bool {
        let usage = self.get_memory_usage().await;
        usage.used_percent > 85.0 // Alert at 85% usage
    }

    /// Request memory allocation (for quota checking)
    #[instrument(level = "debug", skip(self))]
    pub async fn request_memory(&self, bytes: u64) -> Result<()> {
        let current = *self.memory_monitor.current_usage.read().await;
        let limit = self.memory_monitor.memory_limit;
        
        if current + bytes > limit {
            warn!(
                "Memory allocation denied: {} + {} > {} (limit)",
                current, bytes, limit
            );
            return Err(Error::ResourceLimit(format!(
                "Insufficient memory: requested {} bytes, available {} bytes",
                bytes,
                limit.saturating_sub(current)
            )));
        }

        // Update current usage
        {
            let mut current_usage = self.memory_monitor.current_usage.write().await;
            *current_usage += bytes;
            
            // Update peak if necessary
            let mut peak_usage = self.memory_monitor.peak_usage.write().await;
            if *current_usage > *peak_usage {
                *peak_usage = *current_usage;
            }
        }

        debug!("Allocated {} bytes of memory", bytes);
        Ok(())
    }

    /// Release memory allocation
    #[instrument(level = "debug", skip(self))]
    pub async fn release_memory(&self, bytes: u64) {
        let mut current_usage = self.memory_monitor.current_usage.write().await;
        *current_usage = current_usage.saturating_sub(bytes);
        debug!("Released {} bytes of memory", bytes);
    }

    /// Trigger garbage collection hint
    #[instrument(level = "info", skip(self))]
    pub async fn trigger_gc(&self) {
        info!("Triggering garbage collection hint");
        
        // In Rust, we don't have traditional GC, but we can:
        // 1. Force cleanup of caches
        // 2. Compact data structures
        // 3. Release unused resources
        
        // This would trigger cleanup in the Mojo engine via FFI
        warn!("Garbage collection triggered due to memory pressure");
    }

    /// Calculate system memory limit
    fn calculate_memory_limit() -> u64 {
        // Get total system memory
        let total_memory = Self::get_total_system_memory();
        
        // Use 80% of total memory as default limit
        let limit = (total_memory as f64 * 0.8) as u64;
        
        info!(
            "Calculated memory limit: {} MB (80% of {} MB total)",
            limit / 1024 / 1024,
            total_memory / 1024 / 1024
        );
        
        limit
    }

    /// Get total system memory (simplified implementation)
    fn get_total_system_memory() -> u64 {
        // This is a simplified implementation
        // In production, you'd use a proper system info crate
        #[cfg(target_os = "macos")]
        {
            // Default to 16GB for macOS (can be improved)
            16 * 1024 * 1024 * 1024
        }
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/meminfo on Linux
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
            // Default to 8GB if can't read
            8 * 1024 * 1024 * 1024
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // Default to 8GB for other systems
            8 * 1024 * 1024 * 1024
        }
    }

    /// Background task for memory monitoring
    async fn memory_monitoring_task(
        memory_monitor: Arc<MemoryMonitor>,
        usage_tracker: Arc<RwLock<ResourceUsage>>,
    ) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Get current memory usage from system
            let current_system_memory = Self::get_current_memory_usage();
            
            // Update tracking
            {
                let mut usage = usage_tracker.write().await;
                usage.memory_used = current_system_memory;
            }
            
            let current = *memory_monitor.current_usage.read().await;
            let limit = memory_monitor.memory_limit;
            let usage_percent = (current as f64 / limit as f64) * 100.0;
            
            if usage_percent > 90.0 {
                warn!(
                    "High memory usage: {:.1}% ({} MB / {} MB)",
                    usage_percent,
                    current / 1024 / 1024,
                    limit / 1024 / 1024
                );
            }
        }
    }

    /// Background task for CPU monitoring
    async fn cpu_monitoring_task(usage_tracker: Arc<RwLock<ResourceUsage>>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Get CPU usage (simplified)
            let cpu_usage = Self::get_cpu_usage();
            
            {
                let mut usage = usage_tracker.write().await;
                usage.cpu_usage_percent = cpu_usage;
            }
            
            if cpu_usage > 80.0 {
                warn!("High CPU usage: {:.1}%", cpu_usage);
            }
        }
    }

    /// Get current memory usage from the system
    fn get_current_memory_usage() -> u64 {
        // Simplified implementation
        // In production, use a proper system monitoring crate
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: return a reasonable estimate
        100 * 1024 * 1024 // 100MB
    }

    /// Get CPU usage percentage
    fn get_cpu_usage() -> f64 {
        // Simplified implementation
        // In production, use a proper system monitoring crate
        fastrand::f64() * 20.0 // Random value between 0-20% for testing
    }

    /// Health check for resource manager
    pub async fn health_check(&self) -> Result<()> {
        let usage = self.get_memory_usage().await;
        
        if usage.used_percent > 95.0 {
            return Err(Error::ResourceLimit(format!(
                "Critical memory usage: {:.1}%",
                usage.used_percent
            )));
        }
        
        Ok(())
    }
}

/// Resource allocation tracker for specific operations
pub struct ResourceAllocation {
    /// Manager reference
    manager: Arc<ResourceManager>,
    /// Allocated memory in bytes
    allocated_memory: u64,
}

impl ResourceAllocation {
    /// Create a new resource allocation
    pub async fn new(manager: Arc<ResourceManager>, memory_bytes: u64) -> Result<Self> {
        manager.request_memory(memory_bytes).await?;
        
        Ok(ResourceAllocation {
            manager,
            allocated_memory: memory_bytes,
        })
    }

    /// Get allocated memory
    pub fn allocated_memory(&self) -> u64 {
        self.allocated_memory
    }
}

impl Drop for ResourceAllocation {
    fn drop(&mut self) {
        // Release memory when allocation is dropped
        let manager = Arc::clone(&self.manager);
        let memory = self.allocated_memory;
        
        tokio::spawn(async move {
            manager.release_memory(memory).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_resource_manager_creation() {
        let config = Config::default();
        let manager = ResourceManager::new(config);
        
        let usage = manager.get_memory_usage().await;
        assert!(usage.used_bytes == 0);
        assert!(usage.peak_bytes == 0);
    }

    #[tokio::test]
    async fn test_memory_allocation() {
        let config = Config::default();
        let manager = ResourceManager::new(config);
        
        // Test allocation
        let result = manager.request_memory(1024).await;
        assert!(result.is_ok());
        
        let usage = manager.get_memory_usage().await;
        assert!(usage.used_bytes >= 1024);
        
        // Test release
        manager.release_memory(1024).await;
        let usage_after = manager.get_memory_usage().await;
        assert!(usage_after.used_bytes < usage.used_bytes);
    }

    #[tokio::test]
    async fn test_memory_limit() {
        let config = Config::default();
        let manager = ResourceManager::new(config);
        
        // Try to allocate more than the limit
        let limit = manager.memory_monitor.memory_limit;
        let result = manager.request_memory(limit + 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resource_allocation_raii() {
        let config = Config::default();
        let manager = Arc::new(ResourceManager::new(config));
        
        let initial_usage = manager.get_memory_usage().await.used_bytes;
        
        {
            let _allocation = ResourceAllocation::new(Arc::clone(&manager), 2048).await.unwrap();
            let during_usage = manager.get_memory_usage().await.used_bytes;
            assert!(during_usage >= initial_usage + 2048);
        }
        
        // Give time for the drop to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let final_usage = manager.get_memory_usage().await.used_bytes;
        assert!(final_usage < initial_usage + 2048);
    }
}