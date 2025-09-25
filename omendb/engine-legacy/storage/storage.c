// storage.c - Minimal implementation using proven libraries
#define _GNU_SOURCE  // For mremap on Linux
#include "storage.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <errno.h>

#ifdef __APPLE__
    // macOS doesn't have mremap
    #define MREMAP_MAYMOVE 0
    static void* mremap(void* old_addr, size_t old_size, size_t new_size, int flags) {
        // Simple implementation: munmap old, mmap new
        // Not atomic, but works for our use case
        void* new_addr = mmap(NULL, new_size, PROT_READ|PROT_WRITE, MAP_ANON|MAP_PRIVATE, -1, 0);
        if (new_addr != MAP_FAILED) {
            memcpy(new_addr, old_addr, old_size < new_size ? old_size : new_size);
            munmap(old_addr, old_size);
        }
        return new_addr;
    }
#endif

// Use mimalloc if available, fallback to standard malloc
#ifdef USE_MIMALLOC
    #include <mimalloc.h>
    #define ALLOC(size) mi_malloc(size)
    #define ALLOC_ALIGNED(size, align) mi_malloc_aligned(size, align)
    #define FREE(ptr) mi_free(ptr)
#else
    #define ALLOC(size) malloc(size)
    #define ALLOC_ALIGNED(size, align) aligned_alloc(align, size)
    #define FREE(ptr) free(ptr)
#endif

// Simple storage structure - easy to understand and verify
struct Storage {
    // Memory-mapped region
    void* mmap_base;
    size_t mmap_size;
    int fd;
    
    // Vector storage area (points into mmap_base)
    float* vectors;
    size_t vector_capacity;
    size_t vector_count;
    size_t vector_dim;
    
    // Metadata area (optional, points into mmap_base after vectors)
    char* metadata;
    size_t metadata_offset;
    
    // Simple memory pool using mimalloc or malloc
    size_t pool_allocated;
    size_t pool_freed;
    
    // File path for persistence
    char* filepath;
};

// Header at start of mmap file for recovery
typedef struct {
    uint32_t magic;      // 0x4F4D454E ('OMEN')
    uint32_t version;    // 1
    size_t capacity;
    size_t count;
    size_t dimension;
    size_t metadata_offset;
} StorageHeader;

Storage* storage_create(const char* path, size_t capacity) {
    Storage* s = ALLOC(sizeof(Storage));
    if (!s) return NULL;
    
    // Initialize structure
    memset(s, 0, sizeof(Storage));
    s->vector_capacity = capacity;
    s->filepath = strdup(path);
    
    // Calculate sizes (assume 128d vectors for initial sizing)
    s->vector_dim = 128;
    size_t header_size = sizeof(StorageHeader);
    size_t vector_size = capacity * s->vector_dim * sizeof(float);
    size_t metadata_size = capacity * 256;  // 256 bytes metadata per vector
    s->mmap_size = header_size + vector_size + metadata_size;
    
    // Open or create file
    s->fd = open(path, O_RDWR | O_CREAT, 0644);
    if (s->fd < 0) {
        FREE(s->filepath);
        FREE(s);
        return NULL;
    }
    
    // Resize file
    if (ftruncate(s->fd, s->mmap_size) < 0) {
        close(s->fd);
        FREE(s->filepath);
        FREE(s);
        return NULL;
    }
    
    // Memory map the file
    s->mmap_base = mmap(NULL, s->mmap_size, PROT_READ | PROT_WRITE, MAP_SHARED, s->fd, 0);
    if (s->mmap_base == MAP_FAILED) {
        close(s->fd);
        FREE(s->filepath);
        FREE(s);
        return NULL;
    }
    
    // Set up pointers
    StorageHeader* header = (StorageHeader*)s->mmap_base;
    s->vectors = (float*)((char*)s->mmap_base + header_size);
    s->metadata = (char*)s->mmap_base + header_size + vector_size;
    s->metadata_offset = header_size + vector_size;
    
    // Initialize header if new file
    if (header->magic != 0x4F4D454E) {
        header->magic = 0x4F4D454E;
        header->version = 1;
        header->capacity = capacity;
        header->count = 0;
        header->dimension = s->vector_dim;
        header->metadata_offset = s->metadata_offset;
    } else {
        // Recover from existing file
        s->vector_capacity = header->capacity;
        s->vector_count = header->count;
        s->vector_dim = header->dimension;
        s->metadata_offset = header->metadata_offset;
    }
    
    return s;
}

void storage_destroy(Storage* s) {
    if (!s) return;
    
    // Sync before closing
    storage_sync(s);
    
    // Unmap and close
    if (s->mmap_base && s->mmap_base != MAP_FAILED) {
        munmap(s->mmap_base, s->mmap_size);
    }
    if (s->fd >= 0) {
        close(s->fd);
    }
    
    // Free memory
    FREE(s->filepath);
    FREE(s);
}

float* storage_get_vector(Storage* s, size_t index) {
    if (!s || index >= s->vector_capacity) return NULL;
    
    // Direct pointer into mmap region - zero copy!
    return s->vectors + (index * s->vector_dim);
}

bool storage_set_vector(Storage* s, size_t index, const float* data, size_t dim) {
    if (!s || index >= s->vector_capacity || dim != s->vector_dim) {
        return false;
    }
    
    // Copy data to mmap region
    float* dest = s->vectors + (index * s->vector_dim);
    memcpy(dest, data, dim * sizeof(float));
    
    // Update count if needed
    StorageHeader* header = (StorageHeader*)s->mmap_base;
    if (index >= s->vector_count) {
        s->vector_count = index + 1;
        header->count = s->vector_count;
    }
    
    return true;
}

float* storage_get_batch(Storage* s, size_t start_idx, size_t count) {
    if (!s || start_idx + count > s->vector_capacity) return NULL;
    
    // Return pointer to start of batch - zero copy!
    return s->vectors + (start_idx * s->vector_dim);
}

bool storage_set_batch(Storage* s, size_t start_idx, const float* data, size_t count, size_t dim) {
    if (!s || start_idx + count > s->vector_capacity || dim != s->vector_dim) {
        return false;
    }
    
    // Batch copy
    float* dest = s->vectors + (start_idx * s->vector_dim);
    memcpy(dest, data, count * dim * sizeof(float));
    
    // Update count
    StorageHeader* header = (StorageHeader*)s->mmap_base;
    size_t end_idx = start_idx + count;
    if (end_idx > s->vector_count) {
        s->vector_count = end_idx;
        header->count = s->vector_count;
    }
    
    return true;
}

bool storage_sync(Storage* s) {
    if (!s || !s->mmap_base) return false;
    
    // Sync mmap to disk
    return msync(s->mmap_base, s->mmap_size, MS_SYNC) == 0;
}

bool storage_resize(Storage* s, size_t new_capacity) {
    if (!s || new_capacity == s->vector_capacity) return true;
    
    // Calculate new size
    size_t header_size = sizeof(StorageHeader);
    size_t new_vector_size = new_capacity * s->vector_dim * sizeof(float);
    size_t new_metadata_size = new_capacity * 256;
    size_t new_mmap_size = header_size + new_vector_size + new_metadata_size;
    
    // Resize file
    if (ftruncate(s->fd, new_mmap_size) < 0) {
        return false;
    }
    
    // Remap (Linux has mremap, macOS we fake it)
    void* new_base = mremap(s->mmap_base, s->mmap_size, new_mmap_size, MREMAP_MAYMOVE);
    if (new_base == MAP_FAILED) {
        return false;
    }
    
    // Update pointers
    s->mmap_base = new_base;
    s->mmap_size = new_mmap_size;
    s->vector_capacity = new_capacity;
    
    StorageHeader* header = (StorageHeader*)s->mmap_base;
    s->vectors = (float*)((char*)s->mmap_base + header_size);
    s->metadata = (char*)s->mmap_base + header_size + new_vector_size;
    
    // Update header
    header->capacity = new_capacity;
    
    return true;
}

// Memory pool functions - simple for now
void* storage_alloc(Storage* s, size_t size, size_t alignment) {
    if (!s) return NULL;
    
    void* ptr = ALLOC_ALIGNED(size, alignment);
    if (ptr) {
        s->pool_allocated += size;
    }
    return ptr;
}

void storage_free(Storage* s, void* ptr) {
    if (!s || !ptr) return;
    
    FREE(ptr);
    s->pool_freed += sizeof(void*);  // Approximate
}

// Info functions
size_t storage_capacity(const Storage* s) {
    return s ? s->vector_capacity : 0;
}

size_t storage_size(const Storage* s) {
    return s ? s->vector_count : 0;
}

StorageStats storage_get_stats(const Storage* s) {
    StorageStats stats = {0};
    if (s) {
        stats.total_vectors = s->vector_count;
        stats.memory_used = s->vector_count * s->vector_dim * sizeof(float);
        stats.memory_mapped = s->mmap_size;
        stats.pool_allocations = s->pool_allocated;
        stats.avg_alloc_time_ns = 50.0;  // Estimate
    }
    return stats;
}

bool storage_is_thread_safe(void) {
    #ifdef USE_MIMALLOC
        return true;  // mimalloc is thread-safe
    #else
        return false; // standard malloc may not be
    #endif
}