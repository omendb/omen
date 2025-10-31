// Software prefetching for cache optimization
//
// Prefetching hides memory latency by loading data into cache before it's needed.
// This is critical for HNSW where we have random access patterns during graph traversal.
//
// From Week 8 profiling:
// - 23.41% LLC cache misses (very high)
// - 40% of query time in memory fetches
// - Prefetching can provide 10-20% improvement

/// Prefetch hint: data will be accessed soon (read)
///
/// This uses platform-specific intrinsics to hint the CPU to load data into cache.
/// On x86/x86_64, this compiles to `prefetcht0` instruction.
#[inline(always)]
pub fn prefetch_read<T>(ptr: *const T) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_feature = "sse")]
        unsafe {
            use std::arch::x86_64::_mm_prefetch;
            _mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T0);
        }
    }

    // On other architectures (ARM, etc.), this is a no-op
    // The optimizer will remove this function call
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        let _ = ptr; // Suppress unused variable warning
    }
}

/// Prefetch multiple items in a slice
///
/// Prefetches the first cache line of each item in the slice.
/// Useful for prefetching a list of neighbor node IDs.
#[inline(always)]
pub fn prefetch_slice<T>(slice: &[T]) {
    for item in slice {
        prefetch_read(item as *const T);
    }
}

/// Prefetch vector data for distance computation
///
/// Prefetches the vector data that will be accessed during distance calculation.
/// For 1536D vectors (6KB), this prefetches multiple cache lines.
#[inline(always)]
pub fn prefetch_vector(data: &[f32]) {
    // Prefetch first cache line (most important)
    if !data.is_empty() {
        prefetch_read(data.as_ptr());
    }

    // For large vectors, prefetch additional cache lines
    // Each cache line is 64 bytes = 16 f32 values
    if data.len() > 16 {
        // Prefetch middle of vector
        prefetch_read(unsafe { data.as_ptr().add(data.len() / 2) });
    }

    if data.len() > 32 {
        // Prefetch end of vector (for very large vectors)
        prefetch_read(unsafe { data.as_ptr().add(data.len() - 16) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_read() {
        let data = vec![1, 2, 3, 4, 5];
        // Should not panic or crash
        prefetch_read(data.as_ptr());
    }

    #[test]
    fn test_prefetch_slice() {
        let data = vec![1, 2, 3, 4, 5];
        // Should not panic or crash
        prefetch_slice(&data);
    }

    #[test]
    fn test_prefetch_vector() {
        // Small vector
        let small = vec![1.0; 16];
        prefetch_vector(&small);

        // Medium vector
        let medium = vec![1.0; 128];
        prefetch_vector(&medium);

        // Large vector (OpenAI embeddings)
        let large = vec![1.0; 1536];
        prefetch_vector(&large);
    }

    #[test]
    fn test_prefetch_empty() {
        let empty: Vec<f32> = vec![];
        prefetch_vector(&empty);
    }
}
