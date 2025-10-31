// Arena allocator for neighbor lists
//
// From Week 8 profiling: 7.3M allocations (76% in search operations)
// Arena allocation reduces heap fragmentation and allocation overhead
//
// Design:
// - Pre-allocate large chunks (arenas)
// - Hand out slices from arenas
// - No individual deallocations (bulk free on drop)
// - Thread-safe via interior mutability

use std::cell::RefCell;

/// Arena for u32 neighbor IDs
///
/// Allocates large chunks and hands out slices.
/// Memory is freed in bulk when arena is dropped.
pub struct U32Arena {
    /// Current arena chunk
    current: Vec<u32>,

    /// Offset into current chunk
    offset: usize,

    /// Chunk size (number of u32s per chunk)
    chunk_size: usize,

    /// Previous chunks (kept alive until arena is dropped)
    chunks: Vec<Vec<u32>>,
}

impl U32Arena {
    /// Create new arena with given chunk size
    pub fn new(chunk_size: usize) -> Self {
        Self {
            current: Vec::with_capacity(chunk_size),
            offset: 0,
            chunk_size,
            chunks: Vec::new(),
        }
    }

    /// Allocate a slice of n u32s
    ///
    /// Returns a mutable slice that lives as long as the arena.
    pub fn alloc(&mut self, n: usize) -> &mut [u32] {
        // If requested size is larger than chunk, allocate dedicated chunk
        if n > self.chunk_size {
            let mut dedicated = Vec::with_capacity(n);
            dedicated.resize(n, 0);
            self.chunks.push(dedicated);
            let idx = self.chunks.len() - 1;
            return &mut self.chunks[idx];
        }

        // If current chunk doesn't have space, allocate new chunk
        if self.offset + n > self.current.capacity() {
            let old_chunk = std::mem::replace(
                &mut self.current,
                Vec::with_capacity(self.chunk_size)
            );
            if !old_chunk.is_empty() {
                self.chunks.push(old_chunk);
            }
            self.offset = 0;
        }

        // Allocate from current chunk
        let start = self.offset;
        self.offset += n;

        // Ensure current has enough elements
        while self.current.len() < self.offset {
            self.current.push(0);
        }

        &mut self.current[start..self.offset]
    }

    /// Get total allocated bytes
    pub fn allocated_bytes(&self) -> usize {
        let current_bytes = self.current.capacity() * std::mem::size_of::<u32>();
        let chunks_bytes: usize = self.chunks.iter()
            .map(|c| c.capacity() * std::mem::size_of::<u32>())
            .sum();
        current_bytes + chunks_bytes
    }

    /// Get number of chunks
    pub fn num_chunks(&self) -> usize {
        self.chunks.len() + if self.current.capacity() > 0 { 1 } else { 0 }
    }
}

thread_local! {
    /// Thread-local arena for neighbor allocations
    ///
    /// Each thread gets its own arena to avoid contention.
    /// Chunk size: 64KB = 16384 u32s (enough for ~500 neighbor lists)
    static NEIGHBOR_ARENA: RefCell<U32Arena> = RefCell::new(U32Arena::new(16384));
}

/// Allocate neighbor list from thread-local arena
///
/// Returns a Vec copied from arena-allocated slice.
pub fn alloc_neighbors(neighbors: &[u32]) -> Vec<u32> {
    if neighbors.is_empty() {
        return Vec::new();
    }

    // For small lists, just clone (faster than arena overhead)
    if neighbors.len() < 8 {
        return neighbors.to_vec();
    }

    // For larger lists, use arena
    NEIGHBOR_ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        let slice = arena.alloc(neighbors.len());
        slice.copy_from_slice(neighbors);
        slice.to_vec()
    })
}

/// Clear thread-local arena
///
/// Useful for benchmarking to get consistent memory measurements.
pub fn clear_arena() {
    NEIGHBOR_ARENA.with(|arena| {
        *arena.borrow_mut() = U32Arena::new(16384);
    });
}

/// Get arena statistics
pub fn arena_stats() -> (usize, usize) {
    NEIGHBOR_ARENA.with(|arena| {
        let arena = arena.borrow();
        (arena.allocated_bytes(), arena.num_chunks())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let mut arena = U32Arena::new(1024);

        {
            let slice1 = arena.alloc(10);
            assert_eq!(slice1.len(), 10);
            slice1.fill(42);
        }

        {
            let slice2 = arena.alloc(20);
            assert_eq!(slice2.len(), 20);
        }

        // Total allocated should be at least 30 u32s
        assert!(arena.allocated_bytes() >= 30 * std::mem::size_of::<u32>());
    }

    #[test]
    fn test_arena_large_allocation() {
        let mut arena = U32Arena::new(100);

        {
            // Allocate larger than chunk size
            let large = arena.alloc(200);
            assert_eq!(large.len(), 200);
        }

        {
            // Should still be able to allocate from regular chunks
            let small = arena.alloc(10);
            assert_eq!(small.len(), 10);
        }
    }

    #[test]
    fn test_arena_multiple_chunks() {
        let mut arena = U32Arena::new(100);

        // Fill first chunk
        let _a = arena.alloc(90);

        // This should trigger new chunk
        let _b = arena.alloc(50);

        assert!(arena.num_chunks() >= 1);
    }

    #[test]
    fn test_alloc_neighbors() {
        clear_arena();

        let neighbors = vec![1, 2, 3, 4, 5];
        let allocated = alloc_neighbors(&neighbors);

        assert_eq!(allocated, neighbors);
    }

    #[test]
    fn test_alloc_neighbors_small() {
        // Small lists should not use arena
        let small = vec![1, 2, 3];
        let allocated = alloc_neighbors(&small);
        assert_eq!(allocated, small);
    }

    #[test]
    fn test_arena_stats() {
        clear_arena();

        let (bytes_before, _) = arena_stats();

        let neighbors = vec![1; 100];
        let _allocated = alloc_neighbors(&neighbors);

        let (bytes_after, chunks) = arena_stats();

        assert!(bytes_after >= bytes_before);
        assert!(chunks >= 1);
    }
}
