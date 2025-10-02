//! Test the backup and restore functionality

use anyhow::Result;
use omendb::backup::BackupManager;
use std::fs;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 Testing OmenDB Backup & Restore Functionality");
    println!();

    // Setup test environment
    let data_dir = TempDir::new()?;
    let backup_dir = TempDir::new()?;

    println!("📁 Test directories:");
    println!("  Data:   {}", data_dir.path().display());
    println!("  Backup: {}", backup_dir.path().display());
    println!();

    // Create test data
    setup_test_data(&data_dir)?;

    // Initialize backup manager
    let mut backup_manager = BackupManager::new(data_dir.path(), backup_dir.path())?;

    // Test 1: Create full backup
    println!("1️⃣ Creating full backup...");
    let full_backup = backup_manager.create_full_backup("test_database")?;
    println!("✅ Full backup created: {}", full_backup.backup_id);
    println!("   Size: {} bytes", full_backup.total_size_bytes);
    println!();

    // Test 2: Verify backup
    println!("2️⃣ Verifying backup integrity...");
    let is_valid = backup_manager.verify_backup(&full_backup.backup_id)?;
    assert!(is_valid, "Backup verification failed");
    println!("✅ Backup verification successful");
    println!();

    // Test 3: List backups
    println!("3️⃣ Listing available backups...");
    let backups = backup_manager.list_backups();
    println!("   Found {} backup(s)", backups.len());
    for backup in &backups {
        println!(
            "   - {} [{}] - {}",
            backup.backup_id,
            format!("{:?}", backup.backup_type),
            backup.created_at.format("%Y-%m-%d %H:%M:%S")
        );
    }
    println!();

    // Test 4: Add more data and create incremental backup
    println!("4️⃣ Adding more data for incremental backup...");
    add_incremental_data(&data_dir)?;

    println!("5️⃣ Creating incremental backup...");
    let incremental_backup = backup_manager.create_incremental_backup("test_database")?;
    println!(
        "✅ Incremental backup created: {}",
        incremental_backup.backup_id
    );
    println!("   Depends on: {:?}", incremental_backup.depends_on_backup);
    println!();

    // Test 5: Show backup chain
    println!("6️⃣ Analyzing backup chain...");
    let chain = backup_manager.get_backup_chain(&incremental_backup.backup_id)?;
    println!("   Chain has {} backup(s):", chain.len());
    for (i, backup) in chain.iter().enumerate() {
        println!(
            "   {}. {} [{:?}]",
            i + 1,
            backup.backup_id,
            backup.backup_type
        );
    }
    println!();

    // Test 6: Simulate disaster and restore
    println!("7️⃣ Simulating disaster (clearing data directory)...");
    let original_files = count_files(&data_dir)?;
    println!("   Original files: {}", original_files);

    // Clear data directory
    for entry in fs::read_dir(data_dir.path())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    let after_disaster = count_files(&data_dir)?;
    println!("   Files after disaster: {}", after_disaster);
    println!();

    // Test 7: Restore from backup
    println!("8️⃣ Restoring from incremental backup...");
    backup_manager.restore_from_backup(&incremental_backup.backup_id, None)?;

    let after_restore = count_files(&data_dir)?;
    println!("✅ Restore completed");
    println!("   Files after restore: {}", after_restore);
    println!();

    // Test 8: Cleanup test
    println!("9️⃣ Testing backup cleanup...");
    let deleted = backup_manager.cleanup_old_backups(0)?; // Delete all backups older than 0 days
    println!("   Cleaned up {} backup(s)", deleted.len());
    for backup_id in deleted {
        println!("   - Deleted: {}", backup_id);
    }
    println!();

    // Final verification
    let final_backups = backup_manager.list_backups();
    println!("🎉 All tests completed successfully!");
    println!("   Remaining backups: {}", final_backups.len());

    Ok(())
}

fn setup_test_data(data_dir: &TempDir) -> Result<()> {
    // Create some test Parquet files
    fs::write(
        data_dir.path().join("series_001.parquet"),
        b"mock parquet data for series 1 with 1000 records",
    )?;
    fs::write(
        data_dir.path().join("series_002.parquet"),
        b"mock parquet data for series 2 with 2000 records",
    )?;

    // Create WAL directory with test log files
    let wal_dir = data_dir.path().join("wal");
    fs::create_dir_all(&wal_dir)?;

    fs::write(
        wal_dir.join("wal_000001.log"),
        b"mock wal entry 1: insert series_1 timestamp=1000000 value=42.5",
    )?;
    fs::write(
        wal_dir.join("wal_000002.log"),
        b"mock wal entry 2: insert series_2 timestamp=1000001 value=43.7",
    )?;

    Ok(())
}

fn add_incremental_data(data_dir: &TempDir) -> Result<()> {
    let wal_dir = data_dir.path().join("wal");

    // Add new WAL entries
    fs::write(
        wal_dir.join("wal_000003.log"),
        b"mock wal entry 3: insert series_1 timestamp=1000002 value=44.1",
    )?;
    fs::write(
        wal_dir.join("wal_000004.log"),
        b"mock wal entry 4: insert series_3 timestamp=1000003 value=45.9",
    )?;

    Ok(())
}

fn count_files(data_dir: &TempDir) -> Result<usize> {
    let mut count = 0;

    if data_dir.path().exists() {
        for entry in fs::read_dir(data_dir.path())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                // Count files in subdirectories
                for sub_entry in fs::read_dir(&path)? {
                    let sub_entry = sub_entry?;
                    if sub_entry.path().is_file() {
                        count += 1;
                    }
                }
            }
        }
    }

    Ok(count)
}
