// storage.h - Minimal C23 storage layer for OmenDB
// Designed to be easily replaced with pure Mojo later
#ifndef OMENDB_STORAGE_H
#define OMENDB_STORAGE_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

// Opaque handle - Mojo only sees pointers
typedef struct Storage Storage;

// Error codes that match Mojo expectations
enum StorageError {
    STORAGE_OK = 0,
    STORAGE_ERROR_IO = -1,
    STORAGE_ERROR_MEMORY = -2,
    STORAGE_ERROR_INVALID = -3,
};

// Core API - mirrors future Mojo API exactly
// When Mojo supports mmap, we just replace the implementation

// Lifecycle
[[nodiscard]] Storage* storage_create(const char* path, size_t capacity);
void storage_destroy(Storage* storage);

// Vector operations - zero-copy returns
[[nodiscard]] float* storage_get_vector(Storage* storage, size_t index);
[[nodiscard]] bool storage_set_vector(Storage* storage, size_t index, const float* data, size_t dim);

// Batch operations for efficiency
[[nodiscard]] float* storage_get_batch(Storage* storage, size_t start_idx, size_t count);
[[nodiscard]] bool storage_set_batch(Storage* storage, size_t start_idx, const float* data, size_t count, size_t dim);

// Metadata (optional, for future)
[[nodiscard]] const char* storage_get_metadata(Storage* storage, size_t index);
[[nodiscard]] bool storage_set_metadata(Storage* storage, size_t index, const char* json);

// Memory management
[[nodiscard]] size_t storage_capacity(const Storage* storage);
[[nodiscard]] size_t storage_size(const Storage* storage);
[[nodiscard]] bool storage_resize(Storage* storage, size_t new_capacity);

// Persistence
[[nodiscard]] bool storage_sync(Storage* storage);
[[nodiscard]] bool storage_checkpoint(Storage* storage, const char* path);

// Memory pool for allocations
[[nodiscard]] void* storage_alloc(Storage* storage, size_t size, size_t alignment);
void storage_free(Storage* storage, void* ptr);

// Statistics
typedef struct {
    size_t total_vectors;
    size_t memory_used;
    size_t memory_mapped;
    size_t pool_allocations;
    double avg_alloc_time_ns;
} StorageStats;

[[nodiscard]] StorageStats storage_get_stats(const Storage* storage);

// Thread safety info
[[nodiscard]] bool storage_is_thread_safe(void);

#endif // OMENDB_STORAGE_H