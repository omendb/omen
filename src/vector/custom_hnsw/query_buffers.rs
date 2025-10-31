// Thread-local query buffers for allocation-free search
//
// Reuses temporary buffers across queries to reduce allocations.
// From Week 8 profiling: 7.3M allocations identified (76% in search operations).
//
// Thread-local storage ensures:
// - No contention between threads
// - Amortizes allocation cost across queries
// - 10-15% performance improvement expected

use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};

use super::types::Candidate;

/// Reusable buffers for search operations
///
/// These are cleared and reused across queries to avoid allocations.
#[derive(Default)]
pub struct QueryBuffers {
    /// Visited nodes during graph traversal
    pub visited: HashSet<u32>,

    /// Candidate queue (min-heap)
    pub candidates: BinaryHeap<Reverse<Candidate>>,

    /// Working set (max-heap)
    pub working: BinaryHeap<Candidate>,

    /// Entry points for layer traversal
    pub entry_points: Vec<u32>,
}

impl QueryBuffers {
    /// Create new empty buffers
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all buffers for reuse
    pub fn clear(&mut self) {
        self.visited.clear();
        self.candidates.clear();
        self.working.clear();
        self.entry_points.clear();
    }

    /// Pre-allocate buffers for expected capacity
    pub fn with_capacity(ef: usize, num_levels: usize) -> Self {
        Self {
            visited: HashSet::with_capacity(ef * 2),
            candidates: BinaryHeap::with_capacity(ef),
            working: BinaryHeap::with_capacity(ef),
            entry_points: Vec::with_capacity(num_levels),
        }
    }
}

thread_local! {
    /// Thread-local query buffers
    ///
    /// Each thread gets its own buffers, avoiding contention and allocations.
    static QUERY_BUFFERS: RefCell<QueryBuffers> = RefCell::new(QueryBuffers::new());
}

/// Use thread-local buffers for a query
///
/// Automatically clears buffers before and after use.
pub fn with_buffers<F, R>(f: F) -> R
where
    F: FnOnce(&mut QueryBuffers) -> R,
{
    QUERY_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        buffers.clear();
        let result = f(&mut *buffers);
        buffers.clear(); // Clear again to release memory
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_buffers_creation() {
        let buffers = QueryBuffers::new();
        assert!(buffers.visited.is_empty());
        assert!(buffers.candidates.is_empty());
        assert!(buffers.working.is_empty());
        assert!(buffers.entry_points.is_empty());
    }

    #[test]
    fn test_query_buffers_clear() {
        let mut buffers = QueryBuffers::new();

        // Add some data
        buffers.visited.insert(1);
        buffers.entry_points.push(0);

        // Clear
        buffers.clear();

        assert!(buffers.visited.is_empty());
        assert!(buffers.entry_points.is_empty());
    }

    #[test]
    fn test_with_buffers() {
        // Use buffers
        let result = with_buffers(|buffers| {
            buffers.visited.insert(42);
            buffers.visited.len()
        });

        assert_eq!(result, 1);

        // Buffers should be cleared after use
        with_buffers(|buffers| {
            assert!(buffers.visited.is_empty());
        });
    }

    #[test]
    fn test_thread_local_isolation() {
        use std::thread;

        // Main thread
        with_buffers(|buffers| {
            buffers.visited.insert(1);
        });

        // Spawn new thread
        let handle = thread::spawn(|| {
            with_buffers(|buffers| {
                // Should not see main thread's data
                assert!(buffers.visited.is_empty());
                buffers.visited.insert(2);
            });
        });

        handle.join().unwrap();

        // Main thread should not see spawned thread's data
        with_buffers(|buffers| {
            assert!(buffers.visited.is_empty());
        });
    }
}
