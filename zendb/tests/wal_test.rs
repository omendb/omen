//! Tests for Write-Ahead Logging (WAL) functionality

use zendb::storage::{PageManager, Page};
use zendb::wal::{WALManager, WALEntryType};
use tempfile::TempDir;
use anyhow::Result;

#[tokio::test]
async fn test_page_manager_with_wal() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    
    // Create page manager with WAL enabled
    let pm = PageManager::open_with_wal(&db_path, true)?;
    
    // Allocate a page
    let page_id = pm.allocate_page()?;
    
    // Create test data
    let mut page = Page::new(page_id);
    page.data[0..10].copy_from_slice(b"test data ");
    
    // Write the page (should go to WAL first, then storage)
    pm.write_page(&page)?;
    
    // Sync to ensure data is persisted
    pm.sync()?;
    
    // Read the page back
    let read_page = pm.read_page(page_id)?;
    assert_eq!(&read_page.data[0..10], b"test data ");
    
    Ok(())
}

#[tokio::test]
async fn test_wal_recovery() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_recovery.db");
    let wal_path = format!("{}.wal", db_path.to_string_lossy());
    
    // First, create some data with WAL
    let page_id = {
        let pm = PageManager::open_with_wal(&db_path, true)?;
        
        // Allocate and write a page
        let page_id = pm.allocate_page()?;
        let mut page = Page::new(page_id);
        page.data[0..13].copy_from_slice(b"original data");
        
        pm.write_page(&page)?;
        pm.sync()?;
        
        // Modify the page again
        page.data[0..13].copy_from_slice(b"modified data");
        pm.write_page(&page)?;
        
        // Don't sync here to simulate a crash
        page_id
    };
    
    // Now simulate recovery by reopening with WAL replay
    {
        let pm = PageManager::open_with_wal(&db_path, true)?;
        
        // Perform WAL recovery
        pm.recover_from_wal()?;
        
        // The page should have the last written data from WAL
        let recovered_page = pm.read_page(page_id)?;
        assert_eq!(&recovered_page.data[0..13], b"modified data");
    }
    
    Ok(())
}

#[test]
fn test_wal_manager_basic_operations() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("test.wal");
    
    let wal = WALManager::new(&wal_path)?;
    
    // Write some entries
    let lsn1 = wal.write_entry(1, WALEntryType::TxnBegin { txn_id: 1 })?;
    let lsn2 = wal.write_entry(1, WALEntryType::PageWrite { 
        page_id: 42, 
        data: vec![1, 2, 3, 4, 5] 
    })?;
    let lsn3 = wal.write_entry(1, WALEntryType::TxnCommit { txn_id: 1 })?;
    
    // LSNs should be sequential
    assert_eq!(lsn1, 1);
    assert_eq!(lsn2, 2);
    assert_eq!(lsn3, 3);
    
    // Create checkpoint
    let checkpoint_lsn = wal.checkpoint()?;
    assert!(checkpoint_lsn > lsn3);
    
    // Test replay functionality
    let mut replayed_entries = Vec::new();
    wal.replay(|entry| {
        replayed_entries.push((entry.lsn, entry.txn_id, entry.entry_type.clone()));
        Ok(())
    })?;
    
    // Should replay all committed transaction entries
    assert_eq!(replayed_entries.len(), 4); // 3 + 1 checkpoint
    
    // Verify the entries
    assert!(matches!(replayed_entries[0].2, WALEntryType::TxnBegin { txn_id: 1 }));
    assert!(matches!(replayed_entries[1].2, WALEntryType::PageWrite { page_id: 42, .. }));
    assert!(matches!(replayed_entries[2].2, WALEntryType::TxnCommit { txn_id: 1 }));
    
    Ok(())
}

#[test]
fn test_wal_transaction_isolation() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("isolation_test.wal");
    
    let wal = WALManager::new(&wal_path)?;
    
    // Write two transactions: one committed, one aborted
    
    // Transaction 1 (will be committed)
    wal.write_entry(1, WALEntryType::TxnBegin { txn_id: 1 })?;
    wal.write_entry(1, WALEntryType::PageWrite { 
        page_id: 1, 
        data: vec![1, 1, 1] 
    })?;
    wal.write_entry(1, WALEntryType::TxnCommit { txn_id: 1 })?;
    
    // Transaction 2 (will be aborted)
    wal.write_entry(2, WALEntryType::TxnBegin { txn_id: 2 })?;
    wal.write_entry(2, WALEntryType::PageWrite { 
        page_id: 2, 
        data: vec![2, 2, 2] 
    })?;
    wal.write_entry(2, WALEntryType::TxnAbort { txn_id: 2 })?;
    
    // Transaction 3 (not committed - simulates crash)
    wal.write_entry(3, WALEntryType::TxnBegin { txn_id: 3 })?;
    wal.write_entry(3, WALEntryType::PageWrite { 
        page_id: 3, 
        data: vec![3, 3, 3] 
    })?;
    
    wal.sync()?;
    
    // Replay should only include committed transactions
    let mut page_writes = Vec::new();
    wal.replay(|entry| {
        if let WALEntryType::PageWrite { page_id, data } = &entry.entry_type {
            // Only committed transactions should be replayed
            if entry.txn_id == 1 { // Only committed transaction
                page_writes.push((*page_id, data.clone()));
            } else {
                panic!("Non-committed transaction {} was replayed!", entry.txn_id);
            }
        }
        Ok(())
    })?;
    
    // Should only have page write from committed transaction 1
    assert_eq!(page_writes.len(), 1);
    assert_eq!(page_writes[0].0, 1);
    assert_eq!(page_writes[0].1, vec![1, 1, 1]);
    
    Ok(())
}

#[test] 
fn test_wal_checksum_verification() -> Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let wal_path = temp_dir.path().join("checksum_test.wal");
    
    let wal = WALManager::new(&wal_path)?;
    
    // Write a transaction with begin, page write, and commit
    wal.write_entry(1, WALEntryType::TxnBegin { txn_id: 1 })?;
    wal.write_entry(1, WALEntryType::PageWrite {
        page_id: 42,
        data: vec![1, 2, 3, 4, 5],
    })?;
    wal.write_entry(1, WALEntryType::TxnCommit { txn_id: 1 })?;
    
    wal.sync()?;
    
    // WAL replay should succeed with valid checksums
    let mut entry_count = 0;
    let result = wal.replay(|_entry| {
        entry_count += 1;
        Ok(())
    });
    
    assert!(result.is_ok());
    assert_eq!(entry_count, 3); // begin + page write + commit
    
    Ok(())
}