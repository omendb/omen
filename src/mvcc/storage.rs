// Versioned storage types for MVCC
//
// Provides encoding/decoding for multi-version key-value storage in RocksDB:
// - VersionedKey: (key, txn_id) composite key
// - VersionedValue: value with begin_ts and optional end_ts
//
// Key format ensures newer versions come first in iteration (inverted txn_id)

use anyhow::{anyhow, Result};

/// Versioned key for RocksDB storage
///
/// Format: key_bytes + inverted_txn_id (8 bytes, big-endian)
/// Inversion ensures newer versions come first in RocksDB iteration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedKey {
    pub key: Vec<u8>,
    pub txn_id: u64,
}

impl VersionedKey {
    /// Create a new versioned key
    pub fn new(key: Vec<u8>, txn_id: u64) -> Self {
        Self { key, txn_id }
    }

    /// Encode as RocksDB key
    ///
    /// Format: key_bytes + inverted_txn_id (big-endian)
    /// Inverted txn_id ensures newer versions sort first
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = self.key.clone();

        // Invert txn_id so newer versions come first
        let inverted_txn_id = u64::MAX - self.txn_id;
        encoded.extend_from_slice(&inverted_txn_id.to_be_bytes());

        encoded
    }

    /// Decode from RocksDB key bytes
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 8 {
            return Err(anyhow!("Invalid versioned key: too short"));
        }

        let key_len = bytes.len() - 8;
        let key = bytes[..key_len].to_vec();

        let inverted_bytes: [u8; 8] = bytes[key_len..]
            .try_into()
            .map_err(|_| anyhow!("Invalid txn_id bytes"))?;
        let inverted_txn_id = u64::from_be_bytes(inverted_bytes);
        let txn_id = u64::MAX - inverted_txn_id;

        Ok(Self { key, txn_id })
    }

    /// Create prefix for scanning all versions of a key
    pub fn prefix(key: &[u8]) -> Vec<u8> {
        key.to_vec()
    }
}

/// Versioned value for MVCC storage
///
/// Each version stores:
/// - value: actual data
/// - begin_ts: transaction ID that created this version
/// - end_ts: optional transaction ID that deleted/updated this version
#[derive(Debug, Clone, PartialEq)]
pub struct VersionedValue {
    pub value: Vec<u8>,
    pub begin_ts: u64,
    pub end_ts: Option<u64>,
}

impl VersionedValue {
    /// Create a new versioned value
    pub fn new(value: Vec<u8>, begin_ts: u64) -> Self {
        Self {
            value,
            begin_ts,
            end_ts: None,
        }
    }

    /// Create a deleted version (tombstone)
    pub fn tombstone(begin_ts: u64, end_ts: u64) -> Self {
        Self {
            value: Vec::new(),
            begin_ts,
            end_ts: Some(end_ts),
        }
    }

    /// Mark this version as deleted
    pub fn mark_deleted(&mut self, end_ts: u64) {
        self.end_ts = Some(end_ts);
    }

    /// Check if this version is deleted
    pub fn is_deleted(&self) -> bool {
        self.end_ts.is_some()
    }

    /// Encode as RocksDB value
    ///
    /// Format: value_len (4 bytes) | value | begin_ts (8 bytes) | end_ts_flag (1 byte) | end_ts (8 bytes if flag=1)
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::new();

        // Value length (4 bytes)
        let value_len = self.value.len() as u32;
        encoded.extend_from_slice(&value_len.to_be_bytes());

        // Value bytes
        encoded.extend_from_slice(&self.value);

        // Begin timestamp (8 bytes)
        encoded.extend_from_slice(&self.begin_ts.to_be_bytes());

        // End timestamp (1 byte flag + optional 8 bytes)
        if let Some(end_ts) = self.end_ts {
            encoded.push(1); // Has end_ts
            encoded.extend_from_slice(&end_ts.to_be_bytes());
        } else {
            encoded.push(0); // No end_ts
        }

        encoded
    }

    /// Decode from RocksDB value bytes
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 13 {
            // Minimum: 4 (value_len) + 0 (value) + 8 (begin_ts) + 1 (flag)
            return Err(anyhow!("Invalid versioned value: too short"));
        }

        let mut offset = 0;

        // Read value length
        let value_len_bytes: [u8; 4] = bytes[offset..offset + 4]
            .try_into()
            .map_err(|_| anyhow!("Invalid value length"))?;
        let value_len = u32::from_be_bytes(value_len_bytes) as usize;
        offset += 4;

        // Read value
        if offset + value_len > bytes.len() {
            return Err(anyhow!("Invalid value: length mismatch"));
        }
        let value = bytes[offset..offset + value_len].to_vec();
        offset += value_len;

        // Read begin_ts
        if offset + 8 > bytes.len() {
            return Err(anyhow!("Invalid begin_ts"));
        }
        let begin_ts_bytes: [u8; 8] = bytes[offset..offset + 8]
            .try_into()
            .map_err(|_| anyhow!("Invalid begin_ts bytes"))?;
        let begin_ts = u64::from_be_bytes(begin_ts_bytes);
        offset += 8;

        // Read end_ts flag
        if offset >= bytes.len() {
            return Err(anyhow!("Missing end_ts flag"));
        }
        let end_ts_flag = bytes[offset];
        offset += 1;

        // Read optional end_ts
        let end_ts = if end_ts_flag == 1 {
            if offset + 8 > bytes.len() {
                return Err(anyhow!("Invalid end_ts"));
            }
            let end_ts_bytes: [u8; 8] = bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| anyhow!("Invalid end_ts bytes"))?;
            Some(u64::from_be_bytes(end_ts_bytes))
        } else {
            None
        };

        Ok(Self {
            value,
            begin_ts,
            end_ts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_key_encode_decode() {
        let key = VersionedKey::new(b"test_key".to_vec(), 42);
        let encoded = key.encode();
        let decoded = VersionedKey::decode(&encoded).unwrap();

        assert_eq!(decoded.key, b"test_key");
        assert_eq!(decoded.txn_id, 42);
        assert_eq!(decoded, key);
    }

    #[test]
    fn test_versioned_key_ordering() {
        // Newer versions should sort before older versions
        let key1 = VersionedKey::new(b"key".to_vec(), 1);
        let key2 = VersionedKey::new(b"key".to_vec(), 2);
        let key3 = VersionedKey::new(b"key".to_vec(), 3);

        let encoded1 = key1.encode();
        let encoded2 = key2.encode();
        let encoded3 = key3.encode();

        // In RocksDB, later versions should come first (reverse order)
        assert!(encoded3 < encoded2);
        assert!(encoded2 < encoded1);
    }

    #[test]
    fn test_versioned_key_prefix() {
        let prefix = VersionedKey::prefix(b"key");
        assert_eq!(prefix, b"key");

        // All versions should start with this prefix
        let key1 = VersionedKey::new(b"key".to_vec(), 1);
        let key2 = VersionedKey::new(b"key".to_vec(), 100);

        assert!(key1.encode().starts_with(&prefix));
        assert!(key2.encode().starts_with(&prefix));
    }

    #[test]
    fn test_versioned_value_encode_decode() {
        let value = VersionedValue::new(b"test_value".to_vec(), 100);
        let encoded = value.encode();
        let decoded = VersionedValue::decode(&encoded).unwrap();

        assert_eq!(decoded.value, b"test_value");
        assert_eq!(decoded.begin_ts, 100);
        assert_eq!(decoded.end_ts, None);
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_versioned_value_with_end_ts() {
        let mut value = VersionedValue::new(b"data".to_vec(), 50);
        value.mark_deleted(75);

        let encoded = value.encode();
        let decoded = VersionedValue::decode(&encoded).unwrap();

        assert_eq!(decoded.value, b"data");
        assert_eq!(decoded.begin_ts, 50);
        assert_eq!(decoded.end_ts, Some(75));
        assert!(decoded.is_deleted());
    }

    #[test]
    fn test_versioned_value_tombstone() {
        let tombstone = VersionedValue::tombstone(10, 20);

        assert_eq!(tombstone.value, Vec::<u8>::new());
        assert_eq!(tombstone.begin_ts, 10);
        assert_eq!(tombstone.end_ts, Some(20));
        assert!(tombstone.is_deleted());

        // Roundtrip
        let encoded = tombstone.encode();
        let decoded = VersionedValue::decode(&encoded).unwrap();
        assert_eq!(decoded, tombstone);
    }

    #[test]
    fn test_versioned_value_empty() {
        let value = VersionedValue::new(Vec::new(), 1);
        let encoded = value.encode();
        let decoded = VersionedValue::decode(&encoded).unwrap();

        assert_eq!(decoded.value, Vec::<u8>::new());
        assert_eq!(decoded.begin_ts, 1);
        assert_eq!(decoded.end_ts, None);
    }

    #[test]
    fn test_versioned_value_large() {
        let large_data = vec![0u8; 10000];
        let value = VersionedValue::new(large_data.clone(), 999);
        let encoded = value.encode();
        let decoded = VersionedValue::decode(&encoded).unwrap();

        assert_eq!(decoded.value, large_data);
        assert_eq!(decoded.begin_ts, 999);
    }

    #[test]
    fn test_decode_invalid_key() {
        // Too short
        let result = VersionedKey::decode(b"short");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_value() {
        // Too short
        let result = VersionedValue::decode(b"short");
        assert!(result.is_err());

        // Invalid value length
        let mut invalid = vec![0u8; 13];
        invalid[0..4].copy_from_slice(&u32::MAX.to_be_bytes());
        let result = VersionedValue::decode(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_versions_same_key() {
        // Simulate multiple versions of the same key
        let versions = vec![
            VersionedKey::new(b"user:123".to_vec(), 1),
            VersionedKey::new(b"user:123".to_vec(), 5),
            VersionedKey::new(b"user:123".to_vec(), 10),
        ];

        let mut encoded: Vec<Vec<u8>> = versions.iter()
            .map(|v| v.encode())
            .collect();

        // Sort as RocksDB would
        encoded.sort();

        // Decode and verify order (newest first)
        let decoded: Vec<VersionedKey> = encoded.iter()
            .map(|e| VersionedKey::decode(e).unwrap())
            .collect();

        assert_eq!(decoded[0].txn_id, 10); // Newest first
        assert_eq!(decoded[1].txn_id, 5);
        assert_eq!(decoded[2].txn_id, 1);  // Oldest last
    }
}
