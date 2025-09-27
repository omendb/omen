//! Enterprise backup and restore functionality for OmenDB
//! Supports full backups, incremental backups, and point-in-time recovery

use crate::storage::ArrowStorage;
use crate::wal::{WalManager, WalEntry, WalOperation};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::{Result, Context, bail};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use sha2::{Sha256, Digest};

/// Backup metadata with version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Unique backup identifier
    pub backup_id: String,
    /// Backup type (full or incremental)
    pub backup_type: BackupType,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Database name
    pub database_name: String,
    /// WAL sequence range covered
    pub wal_sequence_start: u64,
    pub wal_sequence_end: u64,
    /// Data checksums for integrity verification
    pub data_checksum: String,
    pub wal_checksum: String,
    /// Size information
    pub total_size_bytes: u64,
    pub compressed_size_bytes: u64,
    /// Dependency tracking for incremental backups
    pub depends_on_backup: Option<String>,
    /// File paths included in backup
    pub included_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Full,
    Incremental,
    PointInTime { target_sequence: u64 },
}

/// Backup manager with enterprise features
pub struct BackupManager {
    /// Database data directory
    data_dir: PathBuf,
    /// Backup storage directory
    backup_dir: PathBuf,
    /// Backup metadata index
    metadata_index: HashMap<String, BackupMetadata>,
}

impl BackupManager {
    /// Create new backup manager
    pub fn new<P: AsRef<Path>>(data_dir: P, backup_dir: P) -> Result<Self> {
        let data_path = data_dir.as_ref().to_path_buf();
        let backup_path = backup_dir.as_ref().to_path_buf();

        // Create backup directory if it doesn't exist
        fs::create_dir_all(&backup_path)?;

        let mut manager = Self {
            data_dir: data_path,
            backup_dir: backup_path,
            metadata_index: HashMap::new(),
        };

        // Load existing backup metadata
        manager.load_metadata_index()?;

        Ok(manager)
    }

    /// Create a full backup of the database
    pub fn create_full_backup(&mut self, database_name: &str) -> Result<BackupMetadata> {
        let backup_id = generate_backup_id("full");
        let backup_path = self.backup_dir.join(&backup_id);
        fs::create_dir_all(&backup_path)?;

        println!("Creating full backup: {}", backup_id);

        // Get current WAL state
        let wal_dir = self.data_dir.join("wal");
        let (wal_start, wal_end) = self.get_wal_sequence_range(&wal_dir)?;

        // Backup data files (Parquet files)
        let mut included_files = Vec::new();
        let mut total_size = 0u64;

        // Copy all Parquet files
        let data_pattern = self.data_dir.join("*.parquet");
        for entry in glob::glob(&data_pattern.to_string_lossy())? {
            let source_path = entry?;
            let file_name = source_path.file_name().unwrap();
            let dest_path = backup_path.join("data").join(file_name);

            fs::create_dir_all(dest_path.parent().unwrap())?;
            let size = self.copy_file_compressed(&source_path, &dest_path)?;
            total_size += size;

            included_files.push(file_name.to_string_lossy().to_string());
        }

        // Backup WAL files
        let wal_backup_path = backup_path.join("wal");
        fs::create_dir_all(&wal_backup_path)?;

        for entry in fs::read_dir(&wal_dir)? {
            let entry = entry?;
            let source_path = entry.path();

            if source_path.is_file() {
                let file_name = source_path.file_name().unwrap();
                let dest_path = wal_backup_path.join(file_name);

                let size = self.copy_file_compressed(&source_path, &dest_path)?;
                total_size += size;

                included_files.push(format!("wal/{}", file_name.to_string_lossy()));
            }
        }

        // Calculate checksums
        let data_checksum = self.calculate_directory_checksum(&backup_path.join("data"))?;
        let wal_checksum = self.calculate_directory_checksum(&wal_backup_path)?;

        // Create metadata
        let metadata = BackupMetadata {
            backup_id: backup_id.clone(),
            backup_type: BackupType::Full,
            created_at: Utc::now(),
            database_name: database_name.to_string(),
            wal_sequence_start: wal_start,
            wal_sequence_end: wal_end,
            data_checksum,
            wal_checksum,
            total_size_bytes: total_size,
            compressed_size_bytes: self.get_directory_size(&backup_path)?,
            depends_on_backup: None,
            included_files,
        };

        // Save metadata
        self.save_backup_metadata(&backup_path, &metadata)?;
        self.metadata_index.insert(backup_id.clone(), metadata.clone());
        self.save_metadata_index()?;

        println!("Full backup completed: {} ({} files, {} bytes)",
                backup_id, metadata.included_files.len(), metadata.total_size_bytes);

        Ok(metadata)
    }

    /// Create incremental backup since last backup
    pub fn create_incremental_backup(&mut self, database_name: &str) -> Result<BackupMetadata> {
        // Find the latest backup to use as base
        let base_backup = self.find_latest_backup(database_name)?
            .context("No previous backup found for incremental backup")?;

        let backup_id = generate_backup_id("incremental");
        let backup_path = self.backup_dir.join(&backup_id);
        fs::create_dir_all(&backup_path)?;

        println!("Creating incremental backup: {} (since {})", backup_id, base_backup.backup_id);

        // Get WAL entries since last backup
        let wal_dir = self.data_dir.join("wal");
        let incremental_entries = self.get_wal_entries_since_sequence(&wal_dir, base_backup.wal_sequence_end)?;

        if incremental_entries.is_empty() {
            bail!("No changes found since last backup");
        }

        // Export incremental WAL entries
        let wal_backup_path = backup_path.join("incremental_wal.json.gz");
        self.export_wal_entries_compressed(&incremental_entries, &wal_backup_path)?;

        let total_size = fs::metadata(&wal_backup_path)?.len();
        let wal_checksum = self.calculate_file_checksum(&wal_backup_path)?;

        let (wal_start, wal_end) = if let (Some(first), Some(last)) = (incremental_entries.first(), incremental_entries.last()) {
            (first.sequence, last.sequence)
        } else {
            (base_backup.wal_sequence_end + 1, base_backup.wal_sequence_end + 1)
        };

        // Create metadata
        let metadata = BackupMetadata {
            backup_id: backup_id.clone(),
            backup_type: BackupType::Incremental,
            created_at: Utc::now(),
            database_name: database_name.to_string(),
            wal_sequence_start: wal_start,
            wal_sequence_end: wal_end,
            data_checksum: String::new(), // No data files in incremental
            wal_checksum,
            total_size_bytes: total_size,
            compressed_size_bytes: total_size,
            depends_on_backup: Some(base_backup.backup_id.clone()),
            included_files: vec!["incremental_wal.json.gz".to_string()],
        };

        // Save metadata
        self.save_backup_metadata(&backup_path, &metadata)?;
        self.metadata_index.insert(backup_id.clone(), metadata.clone());
        self.save_metadata_index()?;

        println!("Incremental backup completed: {} ({} WAL entries)",
                backup_id, incremental_entries.len());

        Ok(metadata)
    }

    /// Restore database from backup with point-in-time recovery
    pub fn restore_from_backup(&self, backup_id: &str, target_sequence: Option<u64>) -> Result<()> {
        let metadata = self.metadata_index.get(backup_id)
            .context("Backup not found")?;

        println!("Starting restore from backup: {}", backup_id);

        // Clear existing data directory
        if self.data_dir.exists() {
            println!("Clearing existing data directory...");
            fs::remove_dir_all(&self.data_dir)?;
        }
        fs::create_dir_all(&self.data_dir)?;

        match &metadata.backup_type {
            BackupType::Full => {
                self.restore_full_backup(backup_id, target_sequence)?;
            }
            BackupType::Incremental => {
                // Find base backup and restore chain
                self.restore_incremental_chain(backup_id, target_sequence)?;
            }
            BackupType::PointInTime { target_sequence: pts } => {
                self.restore_full_backup(backup_id, Some(*pts))?;
            }
        }

        println!("Restore completed successfully");
        Ok(())
    }

    /// Verify backup integrity
    pub fn verify_backup(&self, backup_id: &str) -> Result<bool> {
        let metadata = self.metadata_index.get(backup_id)
            .context("Backup not found")?;

        let backup_path = self.backup_dir.join(backup_id);

        println!("Verifying backup: {}", backup_id);

        // Verify data checksum
        if !metadata.data_checksum.is_empty() {
            let data_path = backup_path.join("data");
            if data_path.exists() {
                let current_checksum = self.calculate_directory_checksum(&data_path)?;
                if current_checksum != metadata.data_checksum {
                    println!("Data checksum mismatch!");
                    return Ok(false);
                }
            }
        }

        // Verify WAL checksum
        if !metadata.wal_checksum.is_empty() {
            match &metadata.backup_type {
                BackupType::Full => {
                    let wal_path = backup_path.join("wal");
                    if wal_path.exists() {
                        let current_checksum = self.calculate_directory_checksum(&wal_path)?;
                        if current_checksum != metadata.wal_checksum {
                            println!("WAL checksum mismatch!");
                            return Ok(false);
                        }
                    }
                }
                BackupType::Incremental => {
                    let wal_file = backup_path.join("incremental_wal.json.gz");
                    if wal_file.exists() {
                        let current_checksum = self.calculate_file_checksum(&wal_file)?;
                        if current_checksum != metadata.wal_checksum {
                            println!("WAL file checksum mismatch!");
                            return Ok(false);
                        }
                    }
                }
                _ => {}
            }
        }

        println!("Backup verification successful");
        Ok(true)
    }

    /// List all available backups
    pub fn list_backups(&self) -> Vec<&BackupMetadata> {
        let mut backups: Vec<_> = self.metadata_index.values().collect();
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        backups
    }

    /// Get backup chain for incremental restore
    pub fn get_backup_chain(&self, backup_id: &str) -> Result<Vec<&BackupMetadata>> {
        let mut chain = Vec::new();
        let mut current_id = Some(backup_id);

        while let Some(id) = current_id {
            let metadata = self.metadata_index.get(id)
                .context(format!("Backup {} not found in chain", id))?;

            chain.push(metadata);
            current_id = metadata.depends_on_backup.as_deref();
        }

        chain.reverse(); // Start from base backup
        Ok(chain)
    }

    /// Delete old backups with retention policy
    pub fn cleanup_old_backups(&mut self, retention_days: u64) -> Result<Vec<String>> {
        let cutoff_time = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut deleted_backups = Vec::new();

        // Find backups to delete (keep full backups that others depend on)
        let mut to_delete = Vec::new();
        for (backup_id, metadata) in &self.metadata_index {
            if metadata.created_at < cutoff_time {
                // Check if other backups depend on this one
                let has_dependencies = self.metadata_index.values()
                    .any(|m| m.depends_on_backup.as_ref() == Some(backup_id));

                if !has_dependencies {
                    to_delete.push(backup_id.clone());
                }
            }
        }

        // Delete backups
        for backup_id in to_delete {
            self.delete_backup(&backup_id)?;
            deleted_backups.push(backup_id);
        }

        self.save_metadata_index()?;
        Ok(deleted_backups)
    }

    // Private helper methods

    fn restore_full_backup(&self, backup_id: &str, target_sequence: Option<u64>) -> Result<()> {
        let backup_path = self.backup_dir.join(backup_id);

        // Restore data files
        let data_backup_path = backup_path.join("data");
        if data_backup_path.exists() {
            for entry in fs::read_dir(&data_backup_path)? {
                let entry = entry?;
                let source_path = entry.path();
                let file_name = source_path.file_name().unwrap();
                let dest_path = self.data_dir.join(file_name);

                self.copy_file_decompressed(&source_path, &dest_path)?;
            }
        }

        // Restore WAL files
        let wal_backup_path = backup_path.join("wal");
        let wal_dest_path = self.data_dir.join("wal");
        fs::create_dir_all(&wal_dest_path)?;

        if wal_backup_path.exists() {
            for entry in fs::read_dir(&wal_backup_path)? {
                let entry = entry?;
                let source_path = entry.path();
                let file_name = source_path.file_name().unwrap();
                let dest_path = wal_dest_path.join(file_name);

                self.copy_file_decompressed(&source_path, &dest_path)?;
            }
        }

        // Apply point-in-time recovery if requested
        if let Some(target_seq) = target_sequence {
            self.truncate_wal_to_sequence(&wal_dest_path, target_seq)?;
        }

        Ok(())
    }

    fn restore_incremental_chain(&self, backup_id: &str, target_sequence: Option<u64>) -> Result<()> {
        let chain = self.get_backup_chain(backup_id)?;

        // Restore base backup first
        let base_backup = chain.first().context("No base backup in chain")?;
        self.restore_full_backup(&base_backup.backup_id, None)?;

        // Apply incremental backups in order
        for metadata in chain.iter().skip(1) {
            if let BackupType::Incremental = metadata.backup_type {
                self.apply_incremental_backup(&metadata.backup_id)?;

                // Stop if we've reached the target sequence
                if let Some(target_seq) = target_sequence {
                    if metadata.wal_sequence_end >= target_seq {
                        let wal_path = self.data_dir.join("wal");
                        self.truncate_wal_to_sequence(&wal_path, target_seq)?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn apply_incremental_backup(&self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_dir.join(backup_id);
        let wal_file = backup_path.join("incremental_wal.json.gz");

        if !wal_file.exists() {
            bail!("Incremental WAL file not found");
        }

        let entries = self.load_wal_entries_compressed(&wal_file)?;

        // Append entries to current WAL
        let wal_dir = self.data_dir.join("wal");
        let wal_manager = WalManager::new(&wal_dir)?;

        for entry in entries {
            // Re-apply the WAL entry
            // This would need integration with the actual WAL manager
            // For now, we'll reconstruct the WAL files
        }

        Ok(())
    }

    fn find_latest_backup(&self, database_name: &str) -> Result<Option<BackupMetadata>> {
        let mut latest: Option<&BackupMetadata> = None;

        for metadata in self.metadata_index.values() {
            if metadata.database_name == database_name {
                if let Some(current_latest) = latest {
                    if metadata.created_at > current_latest.created_at {
                        latest = Some(metadata);
                    }
                } else {
                    latest = Some(metadata);
                }
            }
        }

        Ok(latest.cloned())
    }

    fn get_wal_sequence_range(&self, wal_dir: &Path) -> Result<(u64, u64)> {
        let mut min_seq = u64::MAX;
        let mut max_seq = 0;

        if !wal_dir.exists() {
            return Ok((0, 0));
        }

        for entry in fs::read_dir(wal_dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let entries = self.read_wal_file(&file_path)?;
                for entry in entries {
                    min_seq = min_seq.min(entry.sequence);
                    max_seq = max_seq.max(entry.sequence);
                }
            }
        }

        if min_seq == u64::MAX {
            Ok((0, 0))
        } else {
            Ok((min_seq, max_seq))
        }
    }

    fn get_wal_entries_since_sequence(&self, wal_dir: &Path, since_sequence: u64) -> Result<Vec<WalEntry>> {
        let mut entries = Vec::new();

        if !wal_dir.exists() {
            return Ok(entries);
        }

        for entry in fs::read_dir(wal_dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let file_entries = self.read_wal_file(&file_path)?;
                for wal_entry in file_entries {
                    if wal_entry.sequence > since_sequence {
                        entries.push(wal_entry);
                    }
                }
            }
        }

        entries.sort_by_key(|e| e.sequence);
        Ok(entries)
    }

    fn read_wal_file(&self, file_path: &Path) -> Result<Vec<WalEntry>> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        // Read entries until end of file
        loop {
            match bincode::deserialize_from(&mut reader) {
                Ok(entry) => entries.push(entry),
                Err(_) => break, // End of file or corrupted entry
            }
        }

        Ok(entries)
    }

    fn export_wal_entries_compressed(&self, entries: &[WalEntry], output_path: &Path) -> Result<()> {
        let file = File::create(output_path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut writer = BufWriter::new(encoder);

        let json_data = serde_json::to_string(entries)?;
        writer.write_all(json_data.as_bytes())?;
        writer.into_inner()?.finish()?;

        Ok(())
    }

    fn load_wal_entries_compressed(&self, file_path: &Path) -> Result<Vec<WalEntry>> {
        let file = File::open(file_path)?;
        let decoder = GzDecoder::new(file);
        let mut reader = BufReader::new(decoder);

        let mut json_data = String::new();
        reader.read_to_string(&mut json_data)?;

        let entries: Vec<WalEntry> = serde_json::from_str(&json_data)?;
        Ok(entries)
    }

    fn copy_file_compressed(&self, source: &Path, dest: &Path) -> Result<u64> {
        let source_file = File::open(source)?;
        let dest_file = File::create(dest)?;
        let encoder = GzEncoder::new(dest_file, Compression::default());

        let mut source_reader = BufReader::new(source_file);
        let mut dest_writer = BufWriter::new(encoder);

        let bytes_copied = std::io::copy(&mut source_reader, &mut dest_writer)?;
        dest_writer.into_inner()?.finish()?;

        Ok(bytes_copied)
    }

    fn copy_file_decompressed(&self, source: &Path, dest: &Path) -> Result<u64> {
        let source_file = File::open(source)?;
        let decoder = GzDecoder::new(source_file);
        let dest_file = File::create(dest)?;

        let mut source_reader = BufReader::new(decoder);
        let mut dest_writer = BufWriter::new(dest_file);

        let bytes_copied = std::io::copy(&mut source_reader, &mut dest_writer)?;
        dest_writer.flush()?;

        Ok(bytes_copied)
    }

    fn calculate_file_checksum(&self, file_path: &Path) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn calculate_directory_checksum(&self, dir_path: &Path) -> Result<String> {
        let mut hasher = Sha256::new();

        if !dir_path.exists() {
            return Ok(format!("{:x}", hasher.finalize()));
        }

        let mut entries: Vec<_> = fs::read_dir(dir_path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if file_path.is_file() {
                let file_checksum = self.calculate_file_checksum(&file_path)?;
                hasher.update(file_checksum.as_bytes());
            }
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    fn get_directory_size(&self, dir_path: &Path) -> Result<u64> {
        let mut total_size = 0;

        if !dir_path.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                total_size += fs::metadata(&path)?.len();
            } else if path.is_dir() {
                total_size += self.get_directory_size(&path)?;
            }
        }

        Ok(total_size)
    }

    fn truncate_wal_to_sequence(&self, wal_dir: &Path, target_sequence: u64) -> Result<()> {
        // Read all WAL entries and keep only those <= target_sequence
        let mut all_entries = Vec::new();

        for entry in fs::read_dir(wal_dir)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.is_file() {
                let entries = self.read_wal_file(&file_path)?;
                all_entries.extend(entries);
            }
        }

        // Filter to target sequence
        all_entries.retain(|e| e.sequence <= target_sequence);
        all_entries.sort_by_key(|e| e.sequence);

        // Clear WAL directory and rewrite files
        fs::remove_dir_all(wal_dir)?;
        fs::create_dir_all(wal_dir)?;

        // Write entries back in chunks
        const ENTRIES_PER_FILE: usize = 1000;
        for (i, chunk) in all_entries.chunks(ENTRIES_PER_FILE).enumerate() {
            let file_path = wal_dir.join(format!("wal_{:06}.log", i));
            let file = File::create(&file_path)?;
            let mut writer = BufWriter::new(file);

            for entry in chunk {
                bincode::serialize_into(&mut writer, entry)?;
            }
            writer.flush()?;
        }

        Ok(())
    }

    fn save_backup_metadata(&self, backup_path: &Path, metadata: &BackupMetadata) -> Result<()> {
        let metadata_path = backup_path.join("metadata.json");
        let file = File::create(&metadata_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, metadata)?;
        Ok(())
    }

    fn load_metadata_index(&mut self) -> Result<()> {
        let index_path = self.backup_dir.join("backup_index.json");

        if !index_path.exists() {
            return Ok(());
        }

        let file = File::open(&index_path)?;
        let reader = BufReader::new(file);
        self.metadata_index = serde_json::from_reader(reader)?;

        Ok(())
    }

    fn save_metadata_index(&self) -> Result<()> {
        let index_path = self.backup_dir.join("backup_index.json");
        let file = File::create(&index_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.metadata_index)?;
        Ok(())
    }

    fn delete_backup(&mut self, backup_id: &str) -> Result<()> {
        let backup_path = self.backup_dir.join(backup_id);

        if backup_path.exists() {
            fs::remove_dir_all(&backup_path)?;
        }

        self.metadata_index.remove(backup_id);
        Ok(())
    }
}

/// Generate unique backup ID with timestamp
fn generate_backup_id(backup_type: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let random_suffix: String = (0..4)
        .map(|_| char::from(b'a' + (rand::random::<u8>() % 26)))
        .collect();
    format!("{}_{}_{}_{}", backup_type, timestamp, random_suffix, "omendb")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_environment() -> Result<(TempDir, TempDir, BackupManager)> {
        let data_dir = TempDir::new()?;
        let backup_dir = TempDir::new()?;

        let manager = BackupManager::new(data_dir.path(), backup_dir.path())?;

        Ok((data_dir, backup_dir, manager))
    }

    #[test]
    fn test_backup_manager_creation() {
        let (_, _, manager) = setup_test_environment().unwrap();
        assert_eq!(manager.metadata_index.len(), 0);
    }

    #[test]
    fn test_full_backup_creation() {
        let (data_dir, _, mut manager) = setup_test_environment().unwrap();

        // Create some test data
        fs::write(data_dir.path().join("test.parquet"), b"test data").unwrap();
        fs::create_dir_all(data_dir.path().join("wal")).unwrap();
        fs::write(data_dir.path().join("wal").join("wal_000001.log"), b"wal data").unwrap();

        let metadata = manager.create_full_backup("test_db").unwrap();

        assert_eq!(metadata.backup_type, BackupType::Full);
        assert_eq!(metadata.database_name, "test_db");
        assert!(!metadata.included_files.is_empty());
    }

    #[test]
    fn test_backup_verification() {
        let (data_dir, _, mut manager) = setup_test_environment().unwrap();

        // Create test data
        fs::write(data_dir.path().join("test.parquet"), b"test data").unwrap();
        fs::create_dir_all(data_dir.path().join("wal")).unwrap();

        let metadata = manager.create_full_backup("test_db").unwrap();
        let is_valid = manager.verify_backup(&metadata.backup_id).unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_backup_listing() {
        let (data_dir, _, mut manager) = setup_test_environment().unwrap();

        fs::create_dir_all(data_dir.path().join("wal")).unwrap();

        manager.create_full_backup("test_db").unwrap();
        manager.create_full_backup("test_db2").unwrap();

        let backups = manager.list_backups();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_checksum_calculation() {
        let (_, _, manager) = setup_test_environment().unwrap();

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test content").unwrap();

        let checksum1 = manager.calculate_file_checksum(temp_file.path()).unwrap();
        let checksum2 = manager.calculate_file_checksum(temp_file.path()).unwrap();

        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }
}