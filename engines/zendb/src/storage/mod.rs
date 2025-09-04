//! Storage engine module
//! 
//! Implements hybrid B+Tree/LSM storage engine inspired by Bf-Tree research
//! with MVCC support using Hybrid Logical Clocks.

pub mod btree;
pub mod lsm_buffer;
pub mod hybrid_engine;
pub mod page_manager;
pub mod compression;
pub mod multiwriter;

use anyhow::Result;
pub use page_manager::{PageManager, PageId, Page, LRUCache};
pub use btree::BTree;

/// Storage engine trait for different deployment modes
pub trait StorageEngine: Send + Sync {
    /// Read a page by ID
    fn read_page(&self, page_id: PageId) -> Result<Page>;
    
    /// Write a page
    fn write_page(&self, page: &Page) -> Result<()>;
    
    /// Allocate a new page
    fn allocate_page(&self) -> Result<PageId>;
    
    /// Free a page
    fn free_page(&self, page_id: PageId) -> Result<()>;
    
    /// Sync all pending writes to disk
    fn sync(&self) -> Result<()>;
}

/// Embedded storage engine implementation
pub struct EmbeddedEngine {
    page_manager: PageManager,
}

impl EmbeddedEngine {
    pub fn open(path: &str) -> Result<Self> {
        let page_manager = PageManager::open(path)?;
        
        Ok(Self {
            page_manager,
        })
    }
}

impl StorageEngine for EmbeddedEngine {
    fn read_page(&self, page_id: PageId) -> Result<Page> {
        self.page_manager.read_page(page_id)
    }
    
    fn write_page(&self, page: &Page) -> Result<()> {
        self.page_manager.write_page(page)
    }
    
    fn allocate_page(&self) -> Result<PageId> {
        self.page_manager.allocate_page()
    }
    
    fn free_page(&self, page_id: PageId) -> Result<()> {
        self.page_manager.free_page(page_id)
    }
    
    fn sync(&self) -> Result<()> {
        self.page_manager.sync()
    }
}

