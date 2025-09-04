//! Page compression module using LZ4 for fast compression/decompression

use anyhow::Result;
use lz4_flex;

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable/disable compression
    pub enabled: bool,
    /// Compression level (0 = fastest, higher = more compression)
    pub level: u32,
    /// Minimum page utilization to compress (avoid compressing sparse pages)
    pub min_utilization: f32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: 0, // LZ4 fast mode for best performance
            min_utilization: 0.1, // Only compress pages that are at least 10% full
        }
    }
}

/// Compressed page metadata
#[derive(Debug, Clone)]
pub struct CompressedPage {
    /// Original uncompressed size
    pub uncompressed_size: u32,
    /// Compressed data
    pub data: Vec<u8>,
}

impl CompressedPage {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f32 {
        self.data.len() as f32 / self.uncompressed_size as f32
    }
    
    /// Check if compression was effective (compressed size < 95% of original)
    pub fn is_effective(&self) -> bool {
        self.compression_ratio() < 0.95
    }
}

/// Page compressor using LZ4
pub struct PageCompressor {
    config: CompressionConfig,
}

impl PageCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }
    
    /// Compress page data if beneficial
    pub fn compress(&self, data: &[u8]) -> Result<Option<CompressedPage>> {
        if !self.config.enabled {
            return Ok(None);
        }
        
        // Check if page has sufficient data to be worth compressing
        let utilization = self.calculate_utilization(data);
        if utilization < self.config.min_utilization {
            return Ok(None);
        }
        
        // Compress using LZ4
        let compressed = if self.config.level == 0 {
            // Fast mode
            lz4_flex::compress(data)
        } else {
            // High compression mode (not yet supported by lz4_flex in this version)
            lz4_flex::compress(data)
        };
        
        let compressed_page = CompressedPage {
            uncompressed_size: data.len() as u32,
            data: compressed,
        };
        
        // Only return compressed version if it's actually smaller
        if compressed_page.is_effective() {
            Ok(Some(compressed_page))
        } else {
            Ok(None)
        }
    }
    
    /// Decompress page data
    pub fn decompress(&self, compressed: &CompressedPage) -> Result<Vec<u8>> {
        let decompressed = lz4_flex::decompress(&compressed.data, compressed.uncompressed_size as usize)
            .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;
        
        Ok(decompressed)
    }
    
    /// Calculate page utilization (how much of the page contains non-zero data)
    fn calculate_utilization(&self, data: &[u8]) -> f32 {
        let non_zero_bytes = data.iter().filter(|&&b| b != 0).count();
        non_zero_bytes as f32 / data.len() as f32
    }
}

/// Compressed page format for disk storage
/// 
/// Layout:
/// - 1 byte: flags (bit 0: compressed, bits 1-7: reserved)
/// - 4 bytes: uncompressed size (if compressed)
/// - 4 bytes: compressed size (if compressed)  
/// - N bytes: page data (compressed or uncompressed)
///
/// Important: The total serialized size must never exceed PAGE_SIZE
pub struct CompressedPageFormat;

impl CompressedPageFormat {
    const FLAG_COMPRESSED: u8 = 0x01;
    const HEADER_SIZE_UNCOMPRESSED: usize = 1;
    const HEADER_SIZE_COMPRESSED: usize = 9; // 1 + 4 + 4 bytes
    
    /// Maximum data size that can fit in a page after accounting for headers
    pub const MAX_UNCOMPRESSED_DATA_SIZE: usize = crate::storage::page_manager::PAGE_SIZE - Self::HEADER_SIZE_UNCOMPRESSED;
    pub const MAX_COMPRESSED_DATA_SIZE: usize = crate::storage::page_manager::PAGE_SIZE - Self::HEADER_SIZE_COMPRESSED;
    
    /// Serialize page to disk format (never exceeds PAGE_SIZE)
    pub fn serialize(data: &[u8], compressed: Option<&CompressedPage>) -> Vec<u8> {
        let mut result = Vec::new();
        
        if let Some(comp) = compressed {
            // Check if compressed data fits with header
            if Self::HEADER_SIZE_COMPRESSED + comp.data.len() <= crate::storage::page_manager::PAGE_SIZE {
                // Compressed format
                result.push(Self::FLAG_COMPRESSED);
                result.extend_from_slice(&comp.uncompressed_size.to_le_bytes());
                result.extend_from_slice(&(comp.data.len() as u32).to_le_bytes());
                result.extend_from_slice(&comp.data);
                return result;
            }
        }
        
        // Uncompressed format - truncate data if necessary to fit with header
        let max_data_size = Self::MAX_UNCOMPRESSED_DATA_SIZE;
        result.push(0);
        if data.len() <= max_data_size {
            result.extend_from_slice(data);
        } else {
            result.extend_from_slice(&data[..max_data_size]);
        }
        
        result
    }
    
    /// Deserialize page from disk format
    pub fn deserialize(data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            anyhow::bail!("Empty page data");
        }
        
        let flags = data[0];
        
        if flags & Self::FLAG_COMPRESSED != 0 {
            // Compressed format
            if data.len() < Self::HEADER_SIZE_COMPRESSED {
                anyhow::bail!("Invalid compressed page header");
            }
            
            let uncompressed_size = u32::from_le_bytes(data[1..5].try_into()?);
            let compressed_size = u32::from_le_bytes(data[5..9].try_into()?);
            
            if data.len() < Self::HEADER_SIZE_COMPRESSED + compressed_size as usize {
                anyhow::bail!("Invalid compressed page data length");
            }
            
            let compressed_data = &data[Self::HEADER_SIZE_COMPRESSED..Self::HEADER_SIZE_COMPRESSED + compressed_size as usize];
            
            let decompressed = lz4_flex::decompress(compressed_data, uncompressed_size as usize)
                .map_err(|e| anyhow::anyhow!("Decompression failed: {}", e))?;
            
            Ok(decompressed)
        } else {
            // Uncompressed format
            Ok(data[Self::HEADER_SIZE_UNCOMPRESSED..].to_vec())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert!(config.enabled);
        assert_eq!(config.level, 0);
        assert_eq!(config.min_utilization, 0.1);
    }

    #[test]
    fn test_page_compressor_compress_decompress() {
        let compressor = PageCompressor::new(CompressionConfig::default());
        
        // Create test data with some patterns (compressible)
        // Use >10% non-zero data to exceed min_utilization threshold
        let mut test_data = vec![0u8; 1024];
        for i in 0..150 {  // 150/1024 = ~14.6% utilization
            test_data[i] = b'A';
        }
        
        // Compress
        let compressed = compressor.compress(&test_data).unwrap();
        assert!(compressed.is_some());
        
        let compressed = compressed.unwrap();
        assert!(compressed.is_effective());
        
        // Decompress
        let decompressed = compressor.decompress(&compressed).unwrap();
        assert_eq!(decompressed, test_data);
    }

    #[test]
    fn test_compression_ratio() {
        let compressed = CompressedPage {
            uncompressed_size: 1000,
            data: vec![0u8; 500],
        };
        
        assert_eq!(compressed.compression_ratio(), 0.5);
        assert!(compressed.is_effective());
    }

    #[test]
    fn test_compressed_page_format() {
        let test_data = vec![b'A'; 1000];
        let compressor = PageCompressor::new(CompressionConfig::default());
        
        let compressed = compressor.compress(&test_data).unwrap();
        
        if let Some(comp) = compressed {
            // Serialize with compression
            let serialized = CompressedPageFormat::serialize(&test_data, Some(&comp));
            
            // Deserialize
            let deserialized = CompressedPageFormat::deserialize(&serialized).unwrap();
            assert_eq!(deserialized, test_data);
        }
        
        // Test uncompressed format
        let serialized = CompressedPageFormat::serialize(&test_data, None);
        let deserialized = CompressedPageFormat::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, test_data);
    }

    #[test]
    fn test_utilization_calculation() {
        let compressor = PageCompressor::new(CompressionConfig::default());
        
        // Empty page (0% utilization)
        let empty_page = vec![0u8; 1000];
        assert_eq!(compressor.calculate_utilization(&empty_page), 0.0);
        
        // Half full page (50% utilization)
        let mut half_full = vec![0u8; 1000];
        for i in 0..500 {
            half_full[i] = b'A';
        }
        assert_eq!(compressor.calculate_utilization(&half_full), 0.5);
        
        // Full page (100% utilization)
        let full_page = vec![b'A'; 1000];
        assert_eq!(compressor.calculate_utilization(&full_page), 1.0);
    }
}