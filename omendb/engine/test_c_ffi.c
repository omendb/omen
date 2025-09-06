#include <stdio.h>
#include <stdlib.h>

// C function declarations matching our Mojo exports
extern int omendb_init(int dimension);
extern int omendb_add(const char* id_ptr, int id_len, const float* vector_ptr, int dimension);
extern int omendb_search(const float* query_ptr, int k, int* result_ids, float* result_distances);
extern int omendb_clear(void);
extern int omendb_count(void);
extern const char* omendb_version(void);

int main() {
    printf("Testing OmenDB C FFI...\n");
    
    // Test version
    const char* version = omendb_version();
    printf("Version: %s\n", version);
    
    // Initialize
    int dimension = 128;
    int result = omendb_init(dimension);
    printf("Init result: %d\n", result);
    
    if (result != 1) {
        printf("Failed to initialize\n");
        return 1;
    }
    
    // Add a test vector
    float vector[128];
    for (int i = 0; i < 128; i++) {
        vector[i] = i * 0.01f;
    }
    
    const char* id = "test_vec_0";
    result = omendb_add((const char*)id, 10, vector, dimension);
    printf("Add result: %d\n", result);
    
    // Check count
    int count = omendb_count();
    printf("Vector count: %d\n", count);
    
    // Search
    int result_ids[5];
    float result_distances[5];
    int found = omendb_search(vector, 5, result_ids, result_distances);
    printf("Found %d results\n", found);
    
    for (int i = 0; i < found; i++) {
        printf("  ID: %d, Distance: %.4f\n", result_ids[i], result_distances[i]);
    }
    
    printf("C FFI test completed!\n");
    return 0;
}