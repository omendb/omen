use anyhow::{Result, Context};
use crate::storage::{Page, PageId, PageManager};
use std::cmp::Ordering;
use std::sync::Arc;

const BTREE_ORDER: usize = 256;  // Number of keys per node (optimized for 16KB pages)
const VALUE_SIZE: usize = 64;    // Fixed value size for pointers/small values

#[derive(Debug, Clone)]
pub struct BTreeKey(Vec<u8>);

impl BTreeKey {
    pub fn new(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl PartialEq for BTreeKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BTreeKey {}

impl PartialOrd for BTreeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BTreeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeType {
    Internal = 1,
    Leaf = 2,
}

#[derive(Debug)]
struct BTreeNode {
    node_type: NodeType,
    page_id: PageId,
    keys: Vec<BTreeKey>,
    // For internal nodes: child page IDs
    // For leaf nodes: values
    children: Vec<PageId>,
    values: Vec<Vec<u8>>,
    next_leaf: Option<PageId>,  // For leaf nodes only
    parent: Option<PageId>,
}

impl BTreeNode {
    fn new_leaf(page_id: PageId) -> Self {
        Self {
            node_type: NodeType::Leaf,
            page_id,
            keys: Vec::with_capacity(BTREE_ORDER),
            children: Vec::new(),
            values: Vec::with_capacity(BTREE_ORDER),
            next_leaf: None,
            parent: None,
        }
    }
    
    fn new_internal(page_id: PageId) -> Self {
        Self {
            node_type: NodeType::Internal,
            page_id,
            keys: Vec::with_capacity(BTREE_ORDER - 1),
            children: Vec::with_capacity(BTREE_ORDER),
            values: Vec::new(),
            next_leaf: None,
            parent: None,
        }
    }
    
    fn is_full(&self) -> bool {
        match self.node_type {
            NodeType::Leaf => self.keys.len() >= BTREE_ORDER - 1,
            NodeType::Internal => self.keys.len() >= BTREE_ORDER - 1,
        }
    }
    
    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(16384);
        
        // Header
        data.push(self.node_type as u8);
        data.extend_from_slice(&(self.keys.len() as u16).to_le_bytes());
        
        // Parent page ID (8 bytes)
        let parent_id = self.parent.unwrap_or(0);
        data.extend_from_slice(&parent_id.to_le_bytes());
        
        // For leaf nodes: next_leaf pointer
        if self.node_type == NodeType::Leaf {
            let next = self.next_leaf.unwrap_or(0);
            data.extend_from_slice(&next.to_le_bytes());
        }
        
        // Keys (with length prefixes for variable-length support)
        for key in &self.keys {
            data.extend_from_slice(&(key.len() as u16).to_le_bytes());
            data.extend_from_slice(key.as_bytes());
        }
        
        // Children or Values
        match self.node_type {
            NodeType::Internal => {
                for &child in &self.children {
                    data.extend_from_slice(&child.to_le_bytes());
                }
            }
            NodeType::Leaf => {
                for value in &self.values {
                    data.extend_from_slice(&(value.len() as u16).to_le_bytes());
                    data.extend_from_slice(value);
                }
            }
        }
        
        // Pad to page size
        data.resize(16384, 0);
        data
    }
    
    fn deserialize(page_id: PageId, data: &[u8]) -> Result<Self> {
        let node_type = match data[0] {
            1 => NodeType::Internal,
            2 => NodeType::Leaf,
            _ => anyhow::bail!("Invalid node type: {}", data[0]),
        };
        
        let key_count = u16::from_le_bytes([data[1], data[2]]) as usize;
        let parent = u64::from_le_bytes(data[3..11].try_into()?);
        let parent = if parent == 0 { None } else { Some(parent) };
        
        let mut offset = 11;
        
        let next_leaf = if node_type == NodeType::Leaf {
            let next = u64::from_le_bytes(data[offset..offset+8].try_into()?);
            offset += 8;
            if next == 0 { None } else { Some(next) }
        } else {
            None
        };
        
        // Read keys (variable-length with length prefixes)
        let mut keys = Vec::with_capacity(key_count);
        for _ in 0..key_count {
            let key_len = u16::from_le_bytes([data[offset], data[offset+1]]) as usize;
            offset += 2;
            let key_data = data[offset..offset+key_len].to_vec();
            keys.push(BTreeKey(key_data));
            offset += key_len;
        }
        
        let mut children = Vec::new();
        let mut values = Vec::new();
        
        match node_type {
            NodeType::Internal => {
                // Read child pointers (one more than keys)
                for _ in 0..=key_count {
                    let child = u64::from_le_bytes(data[offset..offset+8].try_into()?);
                    children.push(child);
                    offset += 8;
                }
            }
            NodeType::Leaf => {
                // Read values
                for _ in 0..key_count {
                    let value_len = u16::from_le_bytes([data[offset], data[offset+1]]) as usize;
                    offset += 2;
                    let value = data[offset..offset+value_len].to_vec();
                    values.push(value);
                    offset += value_len;
                }
            }
        }
        
        Ok(BTreeNode {
            node_type,
            page_id,
            keys,
            children,
            values,
            next_leaf,
            parent,
        })
    }
}

pub struct BTree {
    page_manager: Arc<PageManager>,
    root_page_id: PageId,
}

impl BTree {
    pub fn root_page_id(&self) -> PageId {
        self.root_page_id
    }
    
    pub fn create(page_manager: Arc<PageManager>) -> Result<Self> {
        // Allocate root page
        let root_page_id = page_manager.allocate_page()?;
        
        // Create empty leaf as root
        let root = BTreeNode::new_leaf(root_page_id);
        let page_data = root.serialize();
        
        let page = Page {
            id: root_page_id,
            data: page_data.try_into().map_err(|_| anyhow::anyhow!("Page data size mismatch"))?,
        };
        
        page_manager.write_page(&page)?;
        page_manager.sync()?;
        
        Ok(Self {
            page_manager,
            root_page_id,
        })
    }
    
    pub fn open(page_manager: Arc<PageManager>, root_page_id: PageId) -> Result<Self> {
        // Verify root page exists
        page_manager.read_page(root_page_id)
            .context("Root page does not exist")?;
        
        Ok(Self {
            page_manager,
            root_page_id,
        })
    }
    
    fn load_node(&self, page_id: PageId) -> Result<BTreeNode> {
        let page = self.page_manager.read_page(page_id)?;
        BTreeNode::deserialize(page_id, &page.data)
    }
    
    fn save_node(&self, node: &BTreeNode) -> Result<()> {
        let page_data = node.serialize();
        let page = Page {
            id: node.page_id,
            data: page_data.try_into().map_err(|_| anyhow::anyhow!("Page data size mismatch"))?,
        };
        self.page_manager.write_page(&page)?;
        Ok(())
    }
    
    pub fn search(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let search_key = BTreeKey::new(key);
        let mut current_page = self.root_page_id;
        
        loop {
            let node = self.load_node(current_page)?;
            
            match node.node_type {
                NodeType::Leaf => {
                    // Search in leaf node
                    for (i, node_key) in node.keys.iter().enumerate() {
                        if node_key == &search_key {
                            return Ok(Some(node.values[i].clone()));
                        }
                    }
                    return Ok(None);
                }
                NodeType::Internal => {
                    // Find child to descend into
                    let mut child_index = node.children.len() - 1;
                    for (i, node_key) in node.keys.iter().enumerate() {
                        if &search_key < node_key {
                            child_index = i;
                            break;
                        }
                    }
                    current_page = node.children[child_index];
                }
            }
        }
    }
    
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let insert_key = BTreeKey::new(key);
        let insert_value = value.to_vec();
        
        // Find leaf node for insertion
        let mut path = Vec::new();
        let mut current_page = self.root_page_id;
        
        loop {
            let node = self.load_node(current_page)?;
            path.push(current_page);
            
            match node.node_type {
                NodeType::Leaf => {
                    // Insert into leaf
                    let mut node = node;
                    
                    // Find insertion position
                    let mut insert_pos = node.keys.len();
                    for (i, node_key) in node.keys.iter().enumerate() {
                        match insert_key.cmp(node_key) {
                            Ordering::Less => {
                                insert_pos = i;
                                break;
                            }
                            Ordering::Equal => {
                                // Update existing key
                                node.values[i] = insert_value;
                                self.save_node(&node)?;
                                return Ok(());
                            }
                            Ordering::Greater => continue,
                        }
                    }
                    
                    // Insert at position
                    node.keys.insert(insert_pos, insert_key.clone());
                    node.values.insert(insert_pos, insert_value);
                    
                    // Check if split needed
                    if node.is_full() {
                        self.split_leaf(node, path)?;
                    } else {
                        self.save_node(&node)?;
                    }
                    
                    return Ok(());
                }
                NodeType::Internal => {
                    // Find child to descend into
                    let mut child_index = node.children.len() - 1;
                    for (i, node_key) in node.keys.iter().enumerate() {
                        if &insert_key < node_key {
                            child_index = i;
                            break;
                        }
                    }
                    current_page = node.children[child_index];
                }
            }
        }
    }
    
    fn split_leaf(&mut self, mut node: BTreeNode, path: Vec<PageId>) -> Result<()> {
        let mid = node.keys.len() / 2;
        
        // Create new right node
        let right_page_id = self.page_manager.allocate_page()?;
        let mut right_node = BTreeNode::new_leaf(right_page_id);
        
        // Split keys and values
        right_node.keys = node.keys.split_off(mid);
        right_node.values = node.values.split_off(mid);
        
        // Update leaf pointers
        right_node.next_leaf = node.next_leaf;
        node.next_leaf = Some(right_page_id);
        
        // Get the middle key for parent
        let promote_key = right_node.keys[0].clone();
        
        // Save both nodes
        self.save_node(&node)?;
        self.save_node(&right_node)?;
        
        // Update parent or create new root
        if path.len() == 1 {
            // Create new root
            self.create_new_root(node.page_id, right_page_id, promote_key)?;
        } else {
            // Update parent node
            self.update_parent(path, right_page_id, promote_key)?
        }
        
        Ok(())
    }
    
    fn update_parent(&mut self, mut path: Vec<PageId>, new_child: PageId, promote_key: BTreeKey) -> Result<()> {
        // Remove current node from path
        path.pop();
        
        if path.is_empty() {
            // This shouldn't happen, but handle gracefully
            return self.create_new_root(self.root_page_id, new_child, promote_key);
        }
        
        let parent_id = *path.last().unwrap();
        let mut parent = self.load_node(parent_id)?;
        
        // Find position to insert new key and child
        let mut insert_pos = parent.keys.len();
        for (i, key) in parent.keys.iter().enumerate() {
            if promote_key < *key {
                insert_pos = i;
                break;
            }
        }
        
        // Insert key and child
        parent.keys.insert(insert_pos, promote_key);
        parent.children.insert(insert_pos + 1, new_child);
        
        // Check if parent needs splitting
        if parent.is_full() {
            self.split_internal(parent, path)?;
        } else {
            self.save_node(&parent)?;
        }
        
        Ok(())
    }
    
    fn split_internal(&mut self, mut node: BTreeNode, path: Vec<PageId>) -> Result<()> {
        let mid = node.keys.len() / 2;
        let promote_key = node.keys[mid].clone();
        
        // Create new right node
        let right_page_id = self.page_manager.allocate_page()?;
        let mut right_node = BTreeNode::new_internal(right_page_id);
        
        // Split keys and children
        // Right node gets keys after mid
        right_node.keys = node.keys.split_off(mid + 1);
        // Remove the promoted key
        node.keys.pop();
        
        // Split children (one more than keys)
        let child_split = mid + 1;
        right_node.children = node.children.split_off(child_split);
        
        // Save both nodes
        self.save_node(&node)?;
        self.save_node(&right_node)?;
        
        // Update parent or create new root
        if path.len() == 1 {
            self.create_new_root(node.page_id, right_page_id, promote_key)?;
        } else {
            self.update_parent(path, right_page_id, promote_key)?;
        }
        
        Ok(())
    }
    
    fn create_new_root(&mut self, left_child: PageId, right_child: PageId, key: BTreeKey) -> Result<()> {
        let new_root_id = self.page_manager.allocate_page()?;
        let mut new_root = BTreeNode::new_internal(new_root_id);
        
        new_root.keys.push(key);
        new_root.children.push(left_child);
        new_root.children.push(right_child);
        
        self.save_node(&new_root)?;
        self.root_page_id = new_root_id;
        
        // TODO: Update children's parent pointers
        
        Ok(())
    }
    
    pub fn delete(&mut self, key: &[u8]) -> Result<bool> {
        let delete_key = BTreeKey::new(key);
        let mut current_page = self.root_page_id;
        let mut path = Vec::new();
        
        // Find leaf containing the key
        loop {
            let node = self.load_node(current_page)?;
            path.push(current_page);
            
            match node.node_type {
                NodeType::Leaf => {
                    // Find and delete the key
                    let mut node = node;
                    
                    if let Some(pos) = node.keys.iter().position(|k| k == &delete_key) {
                        node.keys.remove(pos);
                        node.values.remove(pos);
                        
                        // Check if node needs rebalancing
                        let min_keys = BTREE_ORDER / 2 - 1;
                        if node.keys.len() < min_keys && path.len() > 1 {
                            self.save_node(&node)?;
                            // Handle underflow by merging or redistribution
                            self.handle_underflow(&mut path)?;
                            return Ok(true);
                        }
                        
                        self.save_node(&node)?;
                        return Ok(true);
                    } else {
                        return Ok(false); // Key not found
                    }
                }
                NodeType::Internal => {
                    // Find child to descend into
                    let mut child_index = node.children.len() - 1;
                    for (i, node_key) in node.keys.iter().enumerate() {
                        if &delete_key < node_key {
                            child_index = i;
                            break;
                        }
                    }
                    current_page = node.children[child_index];
                }
            }
        }
    }
    
    /// Handle underflow in a node by merging or redistributing with siblings
    fn handle_underflow(&mut self, path: &mut Vec<PageId>) -> Result<()> {
        if path.len() < 2 {
            return Ok(());  // Root node underflow is acceptable
        }
        
        let underflow_page_id = *path.last().unwrap();
        let parent_page_id = path[path.len() - 2];
        
        let parent = self.load_node(parent_page_id)?;
        
        // Find the index of underflowing child in parent
        let child_index = parent.children.iter()
            .position(|&child| child == underflow_page_id)
            .ok_or_else(|| anyhow::anyhow!("Child not found in parent"))?;
        
        // Try redistribution with left sibling first
        if child_index > 0 {
            let left_sibling_page_id = parent.children[child_index - 1];
            if self.try_redistribute_from_left(underflow_page_id, left_sibling_page_id, parent_page_id, child_index)? {
                return Ok(());
            }
        }
        
        // Try redistribution with right sibling
        if child_index + 1 < parent.children.len() {
            let right_sibling_page_id = parent.children[child_index + 1];
            if self.try_redistribute_from_right(underflow_page_id, right_sibling_page_id, parent_page_id, child_index)? {
                return Ok(());
            }
        }
        
        // No redistribution possible, merge with sibling
        if child_index > 0 {
            // Merge with left sibling
            let left_sibling_page_id = parent.children[child_index - 1];
            self.merge_with_left_sibling(underflow_page_id, left_sibling_page_id, parent_page_id, child_index, path)?;
        } else if child_index + 1 < parent.children.len() {
            // Merge with right sibling
            let right_sibling_page_id = parent.children[child_index + 1];
            self.merge_with_right_sibling(underflow_page_id, right_sibling_page_id, parent_page_id, child_index, path)?;
        }
        
        Ok(())
    }
    
    /// Try to redistribute keys from left sibling
    fn try_redistribute_from_left(&mut self, underflow_page: PageId, left_sibling_page: PageId, 
                                  parent_page: PageId, child_index: usize) -> Result<bool> {
        let mut left_sibling = self.load_node(left_sibling_page)?;
        let mut underflow_node = self.load_node(underflow_page)?;
        let mut parent = self.load_node(parent_page)?;
        
        let min_keys = BTREE_ORDER / 2 - 1;
        
        // Check if left sibling has extra keys to spare
        if left_sibling.keys.len() <= min_keys {
            return Ok(false);
        }
        
        match underflow_node.node_type {
            NodeType::Leaf => {
                // Move last key/value from left sibling to first position of underflow node
                let key = left_sibling.keys.pop().unwrap();
                let value = left_sibling.values.pop().unwrap();
                
                underflow_node.keys.insert(0, key.clone());
                underflow_node.values.insert(0, value);
                
                // Update parent separator key
                parent.keys[child_index - 1] = key;
            }
            NodeType::Internal => {
                // Move last key and child from left sibling
                let key = left_sibling.keys.pop().unwrap();
                let child = left_sibling.children.pop().unwrap();
                
                // Insert parent separator key into underflow node
                underflow_node.keys.insert(0, parent.keys[child_index - 1].clone());
                underflow_node.children.insert(0, child);
                
                // Update parent separator key
                parent.keys[child_index - 1] = key;
            }
        }
        
        // Save all modified nodes
        self.save_node(&left_sibling)?;
        self.save_node(&underflow_node)?;
        self.save_node(&parent)?;
        
        Ok(true)
    }
    
    /// Try to redistribute keys from right sibling
    fn try_redistribute_from_right(&mut self, underflow_page: PageId, right_sibling_page: PageId,
                                   parent_page: PageId, child_index: usize) -> Result<bool> {
        let mut right_sibling = self.load_node(right_sibling_page)?;
        let mut underflow_node = self.load_node(underflow_page)?;
        let mut parent = self.load_node(parent_page)?;
        
        let min_keys = BTREE_ORDER / 2 - 1;
        
        // Check if right sibling has extra keys to spare
        if right_sibling.keys.len() <= min_keys {
            return Ok(false);
        }
        
        match underflow_node.node_type {
            NodeType::Leaf => {
                // Move first key/value from right sibling to last position of underflow node
                let key = right_sibling.keys.remove(0);
                let value = right_sibling.values.remove(0);
                
                underflow_node.keys.push(key);
                underflow_node.values.push(value);
                
                // Update parent separator key
                parent.keys[child_index] = right_sibling.keys[0].clone();
            }
            NodeType::Internal => {
                // Move first key and child from right sibling
                let key = right_sibling.keys.remove(0);
                let child = right_sibling.children.remove(0);
                
                // Insert parent separator key into underflow node
                underflow_node.keys.push(parent.keys[child_index].clone());
                underflow_node.children.push(child);
                
                // Update parent separator key
                parent.keys[child_index] = key;
            }
        }
        
        // Save all modified nodes
        self.save_node(&right_sibling)?;
        self.save_node(&underflow_node)?;
        self.save_node(&parent)?;
        
        Ok(true)
    }
    
    /// Merge underflow node with left sibling
    fn merge_with_left_sibling(&mut self, underflow_page: PageId, left_sibling_page: PageId,
                               parent_page: PageId, child_index: usize, path: &mut Vec<PageId>) -> Result<()> {
        let mut left_sibling = self.load_node(left_sibling_page)?;
        let underflow_node = self.load_node(underflow_page)?;
        let mut parent = self.load_node(parent_page)?;
        
        match underflow_node.node_type {
            NodeType::Leaf => {
                // Merge all keys/values from underflow node into left sibling
                left_sibling.keys.extend(underflow_node.keys);
                left_sibling.values.extend(underflow_node.values);
                left_sibling.next_leaf = underflow_node.next_leaf;
            }
            NodeType::Internal => {
                // Move separator key from parent to left sibling
                left_sibling.keys.push(parent.keys[child_index - 1].clone());
                
                // Merge keys and children from underflow node
                left_sibling.keys.extend(underflow_node.keys);
                left_sibling.children.extend(underflow_node.children);
            }
        }
        
        // Remove separator key and child pointer from parent
        parent.keys.remove(child_index - 1);
        parent.children.remove(child_index);
        
        // Free the underflow page
        self.page_manager.free_page(underflow_page)?;
        
        // Save modified nodes
        self.save_node(&left_sibling)?;
        self.save_node(&parent)?;
        
        // Remove underflow page from path
        path.pop();
        
        // Check if parent now needs rebalancing
        let min_keys = if parent.node_type == NodeType::Leaf { 
            BTREE_ORDER / 2 - 1 
        } else { 
            BTREE_ORDER / 2 - 1 
        };
        
        if parent.keys.len() < min_keys && path.len() > 1 {
            self.handle_underflow(path)?;
        }
        
        Ok(())
    }
    
    /// Merge underflow node with right sibling
    fn merge_with_right_sibling(&mut self, underflow_page: PageId, right_sibling_page: PageId,
                                parent_page: PageId, child_index: usize, path: &mut Vec<PageId>) -> Result<()> {
        let mut underflow_node = self.load_node(underflow_page)?;
        let right_sibling = self.load_node(right_sibling_page)?;
        let mut parent = self.load_node(parent_page)?;
        
        match underflow_node.node_type {
            NodeType::Leaf => {
                // Merge all keys/values from right sibling into underflow node
                underflow_node.keys.extend(right_sibling.keys);
                underflow_node.values.extend(right_sibling.values);
                underflow_node.next_leaf = right_sibling.next_leaf;
            }
            NodeType::Internal => {
                // Move separator key from parent to underflow node
                underflow_node.keys.push(parent.keys[child_index].clone());
                
                // Merge keys and children from right sibling
                underflow_node.keys.extend(right_sibling.keys);
                underflow_node.children.extend(right_sibling.children);
            }
        }
        
        // Remove separator key and child pointer from parent
        parent.keys.remove(child_index);
        parent.children.remove(child_index + 1);
        
        // Free the right sibling page
        self.page_manager.free_page(right_sibling_page)?;
        
        // Save modified nodes
        self.save_node(&underflow_node)?;
        self.save_node(&parent)?;
        
        // Check if parent now needs rebalancing
        let min_keys = if parent.node_type == NodeType::Leaf { 
            BTREE_ORDER / 2 - 1 
        } else { 
            BTREE_ORDER / 2 - 1 
        };
        
        if parent.keys.len() < min_keys && path.len() > 1 {
            self.handle_underflow(path)?;
        }
        
        Ok(())
    }
    
    /// Sync all pages to disk
    pub fn sync(&self) -> Result<()> {
        self.page_manager.sync()
    }
    
    pub fn range_scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let start_key = BTreeKey::new(start);
        let end_key = BTreeKey::new(end);
        let mut results = Vec::new();
        
        // Find starting leaf
        let mut current_page = self.root_page_id;
        loop {
            let node = self.load_node(current_page)?;
            
            match node.node_type {
                NodeType::Leaf => {
                    let mut current_leaf = Some(node);
                    
                    // Scan through leaves using next_leaf pointers
                    while let Some(leaf) = current_leaf {
                        for (i, key) in leaf.keys.iter().enumerate() {
                            if key >= &start_key && key <= &end_key {
                                results.push((key.as_bytes().to_vec(), leaf.values[i].clone()));
                            }
                            if key > &end_key {
                                return Ok(results);
                            }
                        }
                        
                        // Move to next leaf
                        current_leaf = if let Some(next_id) = leaf.next_leaf {
                            Some(self.load_node(next_id)?)
                        } else {
                            None
                        };
                    }
                    
                    return Ok(results);
                }
                NodeType::Internal => {
                    // Find child containing start key
                    let mut child_index = node.children.len() - 1;
                    for (i, node_key) in node.keys.iter().enumerate() {
                        if &start_key < node_key {
                            child_index = i;
                            break;
                        }
                    }
                    current_page = node.children[child_index];
                }
            }
        }
    }
}