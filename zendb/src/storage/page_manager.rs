use anyhow::{Result, Context};
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use crate::wal::{WALManager, WALEntryType};
use crate::storage::compression::{PageCompressor, CompressionConfig, CompressedPageFormat};

pub const PAGE_SIZE: usize = 16384; // 16KB pages
pub const HEADER_SIZE: usize = 4096; // 4KB header
pub const DEFAULT_CACHE_SIZE: usize = 1024; // Default cache size in pages (~16MB)

pub type PageId = u64;

#[derive(Debug, Clone)]
pub struct Page {
    pub id: PageId,
    pub data: Vec<u8>,
}

/// Structure for storing free page linked list nodes in the data file
#[derive(Debug, Clone)]
struct FreePageNode {
    next_free_page: PageId,  // 0 means end of list
    _padding: [u8; PAGE_SIZE - 8],  // Fill rest of page
}

impl FreePageNode {
    fn new(next_free_page: PageId) -> Self {
        Self {
            next_free_page,
            _padding: [0u8; PAGE_SIZE - 8],
        }
    }
    
    fn to_page_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(PAGE_SIZE);
        data.extend_from_slice(&self.next_free_page.to_le_bytes());
        data.extend_from_slice(&self._padding);
        data
    }
    
    fn from_page_data(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            anyhow::bail!("Invalid free page node data");
        }
        
        let next_free_page = u64::from_le_bytes(data[0..8].try_into()?);
        let mut padding = [0u8; PAGE_SIZE - 8];
        if data.len() >= PAGE_SIZE {
            padding.copy_from_slice(&data[8..PAGE_SIZE]);
        }
        
        Ok(Self {
            next_free_page,
            _padding: padding,
        })
    }
}

impl Page {
    pub fn new(id: PageId) -> Self {
        Self {
            id,
            data: vec![0; PAGE_SIZE],
        }
    }
}

/// LRU Cache for pages with configurable size limit
pub struct LRUCache {
    cache: HashMap<PageId, Arc<Page>>,
    access_order: VecDeque<PageId>,
    max_size: usize,
}

impl LRUCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
        }
    }
    
    pub fn get(&mut self, page_id: &PageId) -> Option<Arc<Page>> {
        if let Some(page) = self.cache.get(page_id).cloned() {
            // Move to back (most recently used)
            self.move_to_back(*page_id);
            Some(page)
        } else {
            None
        }
    }
    
    pub fn insert(&mut self, page_id: PageId, page: Arc<Page>) {
        // If already exists, just update access order
        if self.cache.contains_key(&page_id) {
            self.cache.insert(page_id, page);
            self.move_to_back(page_id);
            return;
        }
        
        // If at capacity, evict least recently used
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }
        
        // Insert new page
        self.cache.insert(page_id, page);
        self.access_order.push_back(page_id);
    }
    
    fn remove(&mut self, page_id: &PageId) {
        if self.cache.remove(page_id).is_some() {
            // Remove from access order
            if let Some(pos) = self.access_order.iter().position(|&id| id == *page_id) {
                self.access_order.remove(pos);
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }
    
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    /// Move page to back of access order (most recently used)
    fn move_to_back(&mut self, page_id: PageId) {
        // Remove from current position
        if let Some(pos) = self.access_order.iter().position(|&id| id == page_id) {
            self.access_order.remove(pos);
        }
        // Add to back
        self.access_order.push_back(page_id);
    }
    
    /// Evict least recently used page
    fn evict_lru(&mut self) {
        if let Some(lru_page_id) = self.access_order.pop_front() {
            self.cache.remove(&lru_page_id);
        }
    }
    
    /// Get cache statistics
    fn stats(&self) -> (usize, usize, f64) {
        let used = self.cache.len();
        let capacity = self.max_size;
        let utilization = used as f64 / capacity as f64;
        (used, capacity, utilization)
    }
}

#[derive(Clone)]
struct FileHeader {
    magic: [u8; 8],
    version: u32,
    page_count: u64,
    free_list_head: PageId,
    checksum: u32,  // CRC32 of header
}

impl FileHeader {
    fn new() -> Self {
        let mut header = Self {
            magic: *b"ZENDB001",
            version: 1,
            page_count: 0,
            free_list_head: 0,
            checksum: 0,
        };
        header.checksum = header.calculate_checksum();
        header
    }
    
    fn calculate_checksum(&self) -> u32 {
        // Simple checksum for header validation
        let mut sum = 0u32;
        for byte in &self.magic {
            sum = sum.wrapping_add(*byte as u32);
        }
        sum = sum.wrapping_add(self.version);
        sum = sum.wrapping_add(self.page_count as u32);
        sum = sum.wrapping_add(self.free_list_head as u32);
        sum
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.page_count.to_le_bytes());
        bytes.extend_from_slice(&self.free_list_head.to_le_bytes());
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        bytes.resize(HEADER_SIZE, 0);
        bytes
    }
    
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 32 {
            anyhow::bail!("Invalid header size");
        }
        
        let mut magic = [0u8; 8];
        magic.copy_from_slice(&bytes[0..8]);
        
        let header = Self {
            magic,
            version: u32::from_le_bytes(bytes[8..12].try_into()?),
            page_count: u64::from_le_bytes(bytes[12..20].try_into()?),
            free_list_head: u64::from_le_bytes(bytes[20..28].try_into()?),
            checksum: u32::from_le_bytes(bytes[28..32].try_into()?),
        };
        
        // Validate checksum
        let expected = header.calculate_checksum();
        if header.checksum != expected {
            anyhow::bail!("Header checksum mismatch: got {}, expected {}", header.checksum, expected);
        }
        
        Ok(header)
    }
}

/// PageManager handles low-level page allocation and I/O
/// 
/// Current limitations:
/// - TODO(#4): No compression support
/// - TODO(#5): Single-writer limitation (readers are concurrent)
pub struct PageManager {
    file: Arc<RwLock<File>>,
    mmap: Arc<RwLock<Option<MmapMut>>>,
    header: Arc<RwLock<FileHeader>>,
    free_pages: Arc<RwLock<Vec<PageId>>>,
    page_cache: Arc<RwLock<LRUCache>>,
    wal: Option<Arc<WALManager>>,
    compressor: PageCompressor,
}

impl PageManager {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let exists = path.exists();
        
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Failed to open database file")?;
        
        let header = if !exists {
            let header = FileHeader::new();
            file.write_all(&header.to_bytes())?;
            file.sync_all()?;
            header
        } else {
            let mut header_bytes = vec![0u8; HEADER_SIZE];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut header_bytes)?;
            FileHeader::from_bytes(&header_bytes)?
        };
        
        if header.magic != *b"ZENDB001" {
            anyhow::bail!("Invalid database file format");
        }
        
        let file_len = file.metadata()?.len();
        if file_len < HEADER_SIZE as u64 {
            file.set_len(HEADER_SIZE as u64)?;
        }
        
        let mmap = if file_len > HEADER_SIZE as u64 {
            unsafe {
                Some(MmapOptions::new()
                    .offset(HEADER_SIZE as u64)
                    .map_mut(&file)?)
            }
        } else {
            None
        };
        
        let page_manager = Self {
            file: Arc::new(RwLock::new(file)),
            mmap: Arc::new(RwLock::new(mmap)),
            header: Arc::new(RwLock::new(header)),
            free_pages: Arc::new(RwLock::new(Vec::new())),
            page_cache: Arc::new(RwLock::new(LRUCache::new(DEFAULT_CACHE_SIZE))),
            wal: None,
            compressor: PageCompressor::new(CompressionConfig::default()),
        };
        
        // Load free list from persistent storage on startup
        page_manager.load_free_list()?;
        
        Ok(page_manager)
    }
    
    /// Open PageManager with WAL enabled for crash recovery
    pub fn open_with_wal<P: AsRef<Path>>(db_path: P, enable_wal: bool) -> Result<Self> {
        let mut pm = Self::open(&db_path)?;
        
        if enable_wal {
            // Create WAL file path by appending .wal to the database file
            let db_path = db_path.as_ref();
            let wal_path = format!("{}.wal", db_path.to_string_lossy());
            let wal_manager = WALManager::new(&wal_path)
                .context("Failed to initialize WAL")?;
            pm.wal = Some(Arc::new(wal_manager));
        }
        
        Ok(pm)
    }
    
    /// Recovery from WAL
    pub fn recover_from_wal(&self) -> Result<()> {
        if let Some(ref wal) = self.wal {
            wal.replay(|entry| {
                match &entry.entry_type {
                    WALEntryType::PageWrite { page_id, data } => {
                        // Apply page write from WAL
                        let page = Page {
                            id: *page_id as u64,
                            data: data.clone(),
                        };
                        // Write directly to storage without logging to WAL again
                        self.write_page_direct(&page)?;
                    },
                    _ => {
                        // Other entry types handled by transaction manager
                    }
                }
                Ok(())
            })?;
        }
        Ok(())
    }
    
    pub fn read_page(&self, page_id: PageId) -> Result<Page> {
        // Check if page exists
        let header = self.header.read();
        if page_id >= header.page_count {
            anyhow::bail!("Page {} does not exist (only {} pages)", page_id, header.page_count);
        }
        drop(header);
        
        // Check cache first (LRU cache requires write access to update access order)
        {
            let mut cache = self.page_cache.write();
            if let Some(page) = cache.get(&page_id) {
                return Ok((*page).clone());
            }
        }
        
        let offset = HEADER_SIZE + (page_id as usize * PAGE_SIZE);
        let mut page = Page::new(page_id);
        
        if let Some(ref mmap) = *self.mmap.read() {
            let page_offset = page_id as usize * PAGE_SIZE;
            if page_offset + PAGE_SIZE <= mmap.len() {
                page.data.copy_from_slice(&mmap[page_offset..page_offset + PAGE_SIZE]);
            } else {
                anyhow::bail!("Page {} out of bounds", page_id);
            }
        } else {
            let mut file = self.file.write();
            file.seek(SeekFrom::Start(offset as u64))?;
            file.read_exact(&mut page.data)?;
        }
        
        let page = Arc::new(page);
        self.page_cache.write().insert(page_id, page.clone());
        Ok((*page).clone())
    }
    
    pub fn write_page(&self, page: &Page) -> Result<()> {
        // Write to WAL first if enabled
        if let Some(ref wal) = self.wal {
            wal.write_entry(0, WALEntryType::PageWrite {
                page_id: page.id as u32,
                data: page.data.clone(),
            })?;
        }
        
        // Then write to storage
        self.write_page_direct(page)
    }
    
    /// Write page directly to storage without WAL (used during recovery)
    fn write_page_direct(&self, page: &Page) -> Result<()> {
        // Ensure file is large enough
        let header = self.header.read();
        if page.id >= header.page_count {
            anyhow::bail!("Cannot write page {} when only {} pages allocated", page.id, header.page_count);
        }
        drop(header);
        
        // Check if this is a B+Tree node (node type 1 or 2 in first byte)
        // Skip compression for B+Tree nodes to maintain format compatibility
        let is_btree_node = !page.data.is_empty() && 
                           (page.data[0] == 1 || page.data[0] == 2);
        
        let disk_data = if is_btree_node {
            // Don't compress B+Tree nodes to maintain format compatibility
            page.data.clone()
        } else {
            // Try to compress other page types
            let compressed = self.compressor.compress(&page.data)?;
            CompressedPageFormat::serialize(&page.data, compressed.as_ref())
        };
        
        // Pad to full page size for consistent addressing
        let mut padded_data = disk_data;
        padded_data.resize(PAGE_SIZE, 0);
        
        loop {
            let offset = HEADER_SIZE + (page.id as usize * PAGE_SIZE);
            
            if let Some(ref mut mmap) = *self.mmap.write() {
                let page_offset = page.id as usize * PAGE_SIZE;
                if page_offset + PAGE_SIZE <= mmap.len() {
                    mmap[page_offset..page_offset + PAGE_SIZE].copy_from_slice(&padded_data);
                    break;
                } else {
                    self.extend_file(page.id)?;
                    // Loop will retry with remapped mmap
                }
            } else {
                let mut file = self.file.write();
                file.seek(SeekFrom::Start(offset as u64))?;
                file.write_all(&padded_data)?;
                break;
            }
        }
        
        // Store uncompressed page in cache for fast access
        self.page_cache.write().insert(page.id, Arc::new(page.clone()));
        Ok(())
    }
    
    /// Sync all changes to disk
    pub fn sync(&self) -> Result<()> {
        // Sync WAL first
        if let Some(ref wal) = self.wal {
            wal.sync()?;
        }
        
        // Then sync file
        if let Some(ref mmap) = *self.mmap.read() {
            mmap.flush()?;
        } else {
            self.file.read().sync_all()?;
        }
        
        Ok(())
    }
    
    /// Create a checkpoint in the WAL
    pub fn checkpoint(&self) -> Result<()> {
        if let Some(ref wal) = self.wal {
            wal.checkpoint()?;
        }
        Ok(())
    }
    
    pub fn allocate_page(&self) -> Result<PageId> {
        let mut free_pages = self.free_pages.write();
        
        if let Some(page_id) = free_pages.pop() {
            // We're reusing a free page, need to update persistent free list head
            let page = self.read_page_direct(page_id)?;
            let free_node = FreePageNode::from_page_data(&page.data)?;
            
            // Update header to point to next free page
            let mut header = self.header.write();
            header.free_list_head = free_node.next_free_page;
            self.update_header(&header)?;
            
            return Ok(page_id);
        }
        drop(free_pages);
        
        let mut header = self.header.write();
        let page_id = header.page_count;
        header.page_count += 1;
        
        // Extend file if needed
        self.extend_file(page_id)?;
        
        // Persist header with new page count
        self.update_header(&header)?;
        
        Ok(page_id)
    }
    
    pub fn free_page(&self, page_id: PageId) -> Result<()> {
        let header = self.header.read();
        if page_id >= header.page_count {
            anyhow::bail!("Cannot free non-existent page {}", page_id);
        }
        let current_free_head = header.free_list_head;
        drop(header);
        
        // Add to in-memory free list for immediate availability
        self.free_pages.write().push(page_id);
        self.page_cache.write().remove(&page_id);
        
        // Persist the free page to the linked list on disk
        let free_node = FreePageNode::new(current_free_head);
        let page = Page {
            id: page_id,
            data: free_node.to_page_data(),
        };
        
        // Write the free page node to storage
        self.write_page_direct(&page)?;
        
        // Update header to point to this page as the new head of free list
        let mut header = self.header.write();
        header.free_list_head = page_id;
        self.update_header(&header)?;
        
        Ok(())
    }
    
    fn extend_file(&self, page_id: PageId) -> Result<()> {
        let required_size = HEADER_SIZE as u64 + ((page_id + 1) * PAGE_SIZE as u64);
        let file = self.file.write();
        let current_size = file.metadata()?.len();
        
        if required_size > current_size {
            file.set_len(required_size)?;
            
            drop(file);
            
            let mut mmap_guard = self.mmap.write();
            *mmap_guard = unsafe {
                Some(MmapOptions::new()
                    .offset(HEADER_SIZE as u64)
                    .map_mut(&*self.file.read())?)
            };
        }
        
        Ok(())
    }
    
    fn update_header(&self, header: &FileHeader) -> Result<()> {
        // Recalculate checksum with current values
        let mut updated_header = header.clone();
        updated_header.checksum = updated_header.calculate_checksum();
        
        let mut file = self.file.write();
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&updated_header.to_bytes())?;
        Ok(())
    }
    
    /// Load the persistent free list from disk into memory
    fn load_free_list(&self) -> Result<()> {
        let header = self.header.read();
        let mut current_page = header.free_list_head;
        drop(header);
        
        if current_page == 0 {
            // No free pages
            return Ok(());
        }
        
        let mut free_pages = Vec::new();
        
        // Traverse the linked list of free pages
        while current_page != 0 {
            let page = self.read_page_direct(current_page)?;
            let free_node = FreePageNode::from_page_data(&page.data)?;
            
            free_pages.push(current_page);
            current_page = free_node.next_free_page;
        }
        
        // Load into in-memory free list (reversed to maintain LIFO order)
        free_pages.reverse();
        *self.free_pages.write() = free_pages;
        
        Ok(())
    }
    
    /// Read page directly from storage without caching (used during startup)
    fn read_page_direct(&self, page_id: PageId) -> Result<Page> {
        let header = self.header.read();
        if page_id >= header.page_count {
            anyhow::bail!("Page {} does not exist (only {} pages)", page_id, header.page_count);
        }
        drop(header);
        
        let offset = HEADER_SIZE + (page_id as usize * PAGE_SIZE);
        let mut disk_data = vec![0u8; PAGE_SIZE];
        
        if let Some(ref mmap) = *self.mmap.read() {
            let page_offset = page_id as usize * PAGE_SIZE;
            if page_offset + PAGE_SIZE <= mmap.len() {
                disk_data.copy_from_slice(&mmap[page_offset..page_offset + PAGE_SIZE]);
            } else {
                anyhow::bail!("Page {} out of bounds", page_id);
            }
        } else {
            let mut file = self.file.write();
            file.seek(SeekFrom::Start(offset as u64))?;
            file.read_exact(&mut disk_data)?;
        }
        
        // Check if this looks like a B+Tree node (uncompressed)
        // B+Tree nodes have node type 1 or 2 in first byte
        let is_btree_node = !disk_data.is_empty() && 
                           (disk_data[0] == 1 || disk_data[0] == 2);
        
        let final_data = if is_btree_node {
            // B+Tree nodes are stored uncompressed
            disk_data
        } else {
            // Other pages may be compressed
            let page_data = CompressedPageFormat::deserialize(&disk_data)
                .context(format!("Failed to decompress page {}", page_id))?;
            
            // Ensure page data is correct size
            let mut data = page_data;
            if data.len() < PAGE_SIZE {
                data.resize(PAGE_SIZE, 0);
            } else if data.len() > PAGE_SIZE {
                data.truncate(PAGE_SIZE);
            }
            data
        };
        
        let page = Page {
            id: page_id,
            data: final_data,
        };
        
        Ok(page)
    }
    
    /// Get cache statistics for monitoring
    pub fn cache_stats(&self) -> (usize, usize, f64) {
        self.page_cache.read().stats()
    }
    
    /// Clear the page cache (useful for testing)
    pub fn clear_cache(&self) {
        self.page_cache.write().clear();
    }
}