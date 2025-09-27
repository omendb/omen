//! Disaggregated storage backend for cloud-native architecture
//! Supports S3, GCS, Azure, and local filesystems

use async_trait::async_trait;
use arrow::record_batch::RecordBatch;
use parquet::arrow::{ArrowWriter, ParquetRecordBatchReaderBuilder};
use parquet::file::properties::{WriterProperties, EnabledStatistics};
use parquet::basic::{Compression, Encoding};
use std::sync::Arc;
use anyhow::{Result, Context};
use bytes::Bytes;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

/// Metadata for stored objects
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub created_at: i64,
    pub min_timestamp: Option<i64>,
    pub max_timestamp: Option<i64>,
    pub row_count: Option<u64>,
    pub compression: String,
}

/// Trait for cloud storage backends
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store an object
    async fn put(&self, key: &str, data: Bytes) -> Result<()>;

    /// Retrieve an object
    async fn get(&self, key: &str) -> Result<Bytes>;

    /// List objects with prefix
    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMetadata>>;

    /// Delete an object
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if object exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Get object metadata without downloading
    async fn head(&self, key: &str) -> Result<ObjectMetadata>;
}

/// S3-compatible storage backend (AWS, MinIO, etc.)
pub struct S3Backend {
    bucket: String,
    prefix: String,
    client: aws_sdk_s3::Client,
}

impl S3Backend {
    pub async fn new(bucket: String, prefix: String) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        Ok(Self {
            bucket,
            prefix,
            client,
        })
    }

    fn full_key(&self, key: &str) -> String {
        format!("{}/{}", self.prefix, key)
    }
}

#[async_trait]
impl StorageBackend for S3Backend {
    async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .body(data.into())
            .send()
            .await
            .context("Failed to put object to S3")?;

        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await
            .context("Failed to get object from S3")?;

        let data = response.body.collect().await?;
        Ok(data.into_bytes())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        let response = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(format!("{}/{}", self.prefix, prefix))
            .send()
            .await?;

        let mut metadata = Vec::new();

        if let Some(contents) = response.contents {
            for object in contents {
                if let Some(key) = object.key {
                    metadata.push(ObjectMetadata {
                        key: key.clone(),
                        size: object.size.unwrap_or(0) as u64,
                        created_at: object.last_modified
                            .map(|t| t.secs())
                            .unwrap_or(0),
                        min_timestamp: None,
                        max_timestamp: None,
                        row_count: None,
                        compression: "snappy".to_string(),
                    });
                }
            }
        }

        Ok(metadata)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn head(&self, key: &str) -> Result<ObjectMetadata> {
        let response = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(self.full_key(key))
            .send()
            .await?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: response.content_length.unwrap_or(0) as u64,
            created_at: response.last_modified
                .map(|t| t.secs())
                .unwrap_or(0),
            min_timestamp: None,
            max_timestamp: None,
            row_count: None,
            compression: "snappy".to_string(),
        })
    }
}

/// Local filesystem backend for development/testing
pub struct LocalBackend {
    base_path: PathBuf,
}

impl LocalBackend {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    fn full_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

#[async_trait]
impl StorageBackend for LocalBackend {
    async fn put(&self, key: &str, data: Bytes) -> Result<()> {
        let path = self.full_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, &data).await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Bytes> {
        let path = self.full_path(key);
        let mut file = fs::File::open(&path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(Bytes::from(buffer))
    }

    async fn list(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        let mut metadata = Vec::new();
        let prefix_path = self.base_path.join(prefix);

        if prefix_path.exists() {
            let mut entries = fs::read_dir(prefix_path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    let meta = entry.metadata().await?;
                    metadata.push(ObjectMetadata {
                        key: path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                        size: meta.len(),
                        created_at: meta.modified()?
                            .duration_since(std::time::UNIX_EPOCH)?
                            .as_secs() as i64,
                        min_timestamp: None,
                        max_timestamp: None,
                        row_count: None,
                        compression: "none".to_string(),
                    });
                }
            }
        }

        Ok(metadata)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.full_path(key);
        fs::remove_file(path).await?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.full_path(key).exists())
    }

    async fn head(&self, key: &str) -> Result<ObjectMetadata> {
        let path = self.full_path(key);
        let meta = fs::metadata(&path).await?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: meta.len(),
            created_at: meta.modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64,
            min_timestamp: None,
            max_timestamp: None,
            row_count: None,
            compression: "none".to_string(),
        })
    }
}

/// Parquet-based table storage on top of object storage
pub struct ParquetTableStorage {
    backend: Arc<dyn StorageBackend>,
    table_name: String,
    schema: arrow::datatypes::SchemaRef,
}

impl ParquetTableStorage {
    pub fn new(
        backend: Arc<dyn StorageBackend>,
        table_name: String,
        schema: arrow::datatypes::SchemaRef,
    ) -> Self {
        Self {
            backend,
            table_name,
            schema,
        }
    }

    /// Write a batch of data as Parquet
    pub async fn write_batch(&self, batch: RecordBatch, partition: &str) -> Result<()> {
        // Generate unique file name
        let timestamp = chrono::Utc::now().timestamp_micros();
        let key = format!("{}/{}/data_{}.parquet", self.table_name, partition, timestamp);

        // Convert to Parquet with optimal settings
        let mut buffer = Vec::new();
        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .set_encoding(Encoding::DELTA_BINARY_PACKED)
            .set_statistics_enabled(EnabledStatistics::Page)
            .set_max_row_group_size(100_000)
            .build();

        let mut writer = ArrowWriter::try_new(&mut buffer, self.schema.clone(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;

        // Upload to storage
        self.backend.put(&key, Bytes::from(buffer)).await?;

        Ok(())
    }

    /// Read data for a time range
    pub async fn read_range(
        &self,
        partition: &str,
        min_timestamp: i64,
        max_timestamp: i64,
    ) -> Result<Vec<RecordBatch>> {
        // List all files in partition
        let prefix = format!("{}/{}/", self.table_name, partition);
        let files = self.backend.list(&prefix).await?;

        let mut all_batches = Vec::new();

        // Read relevant files (would use metadata for pruning in production)
        for file_meta in files {
            // Skip files outside time range (if metadata available)
            if let (Some(file_min), Some(file_max)) = (file_meta.min_timestamp, file_meta.max_timestamp) {
                if file_max < min_timestamp || file_min > max_timestamp {
                    continue;
                }
            }

            // Download and read Parquet file
            let data = self.backend.get(&file_meta.key).await?;
            let reader = ParquetRecordBatchReaderBuilder::try_new(data.as_ref())?
                .build()?;

            for batch_result in reader {
                let batch = batch_result?;
                // Filter batch by timestamp (would push down to Parquet in production)
                all_batches.push(batch);
            }
        }

        Ok(all_batches)
    }

    /// Compact small files into larger ones
    pub async fn compact_partition(&self, partition: &str) -> Result<()> {
        let prefix = format!("{}/{}/", self.table_name, partition);
        let files = self.backend.list(&prefix).await?;

        // Group small files (< 100MB)
        let small_files: Vec<_> = files
            .iter()
            .filter(|f| f.size < 100_000_000)
            .collect();

        if small_files.len() < 2 {
            return Ok(()); // Nothing to compact
        }

        // Read all small files
        let mut all_batches = Vec::new();
        for file_meta in &small_files {
            let data = self.backend.get(&file_meta.key).await?;
            let reader = ParquetRecordBatchReaderBuilder::try_new(data.as_ref())?
                .build()?;

            for batch_result in reader {
                all_batches.push(batch_result?);
            }
        }

        // Combine into larger batches
        if !all_batches.is_empty() {
            // Write combined file
            let timestamp = chrono::Utc::now().timestamp_micros();
            let compacted_key = format!("{}/{}/compacted_{}.parquet", self.table_name, partition, timestamp);

            let mut buffer = Vec::new();
            let props = WriterProperties::builder()
                .set_compression(Compression::ZSTD(3))
                .set_max_row_group_size(500_000)
                .build();

            let mut writer = ArrowWriter::try_new(&mut buffer, self.schema.clone(), Some(props))?;
            for batch in all_batches {
                writer.write(&batch)?;
            }
            writer.close()?;

            // Upload compacted file
            self.backend.put(&compacted_key, Bytes::from(buffer)).await?;

            // Delete original small files
            for file_meta in small_files {
                self.backend.delete(&file_meta.key).await?;
            }
        }

        Ok(())
    }
}

/// Factory for creating storage backends
pub fn create_storage_backend(config: &StorageConfig) -> Result<Arc<dyn StorageBackend>> {
    match config {
        StorageConfig::S3 { bucket, prefix } => {
            // This would need tokio runtime context
            unimplemented!("S3 backend requires async initialization")
        }
        StorageConfig::Local { path } => {
            Ok(Arc::new(LocalBackend::new(PathBuf::from(path))?))
        }
        StorageConfig::GCS { .. } => {
            unimplemented!("GCS backend not yet implemented")
        }
        StorageConfig::Azure { .. } => {
            unimplemented!("Azure backend not yet implemented")
        }
    }
}

#[derive(Debug, Clone)]
pub enum StorageConfig {
    S3 {
        bucket: String,
        prefix: String,
    },
    Local {
        path: String,
    },
    GCS {
        bucket: String,
        prefix: String,
    },
    Azure {
        container: String,
        prefix: String,
    },
}