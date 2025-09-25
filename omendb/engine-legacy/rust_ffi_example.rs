// Example Rust code showing how to use OmenDB via C FFI
//
// Build the Mojo library:
//   mojo build omendb/c_exports.mojo -o libomendb.so --emit shared-lib
//
// Build this Rust example:
//   rustc --edition 2021 -L . rust_ffi_example.rs
//
// Run:
//   LD_LIBRARY_PATH=. ./rust_ffi_example

use std::ffi::{c_char, c_float, c_int, CStr};
use std::ptr;

#[link(name = "omendb")]
extern "C" {
    fn omendb_init(dimension: c_int) -> c_int;
    
    fn omendb_add(
        id_ptr: *const u8,
        id_len: c_int,
        vector_ptr: *const c_float,
        dimension: c_int,
    ) -> c_int;
    
    fn omendb_search(
        query_ptr: *const c_float,
        k: c_int,
        result_ids: *mut c_int,
        result_distances: *mut c_float,
    ) -> c_int;
    
    fn omendb_clear() -> c_int;
    fn omendb_count() -> c_int;
    fn omendb_version() -> *const u8;
}

fn main() {
    unsafe {
        // Get version
        let version_ptr = omendb_version();
        let version = CStr::from_ptr(version_ptr as *const c_char)
            .to_string_lossy();
        println!("OmenDB Version: {}", version);
        
        // Initialize with 128 dimensions
        let dimension = 128;
        let init_result = omendb_init(dimension);
        println!("Initialization: {}", if init_result == 1 { "Success" } else { "Failed" });
        
        // Add some vectors
        for i in 0..10 {
            let id = format!("vec_{}", i);
            let mut vector = vec![0.0f32; dimension as usize];
            for j in 0..dimension {
                vector[j as usize] = (i as f32 * 0.1) + (j as f32 * 0.01);
            }
            
            let result = omendb_add(
                id.as_ptr(),
                id.len() as c_int,
                vector.as_ptr(),
                dimension,
            );
            
            if result != 1 {
                println!("Failed to add vector {}", i);
            }
        }
        
        println!("Added vectors. Count: {}", omendb_count());
        
        // Search for nearest neighbors
        let mut query = vec![0.5f32; dimension as usize];
        let k = 5;
        let mut result_ids = vec![0i32; k];
        let mut result_distances = vec![0.0f32; k];
        
        let num_results = omendb_search(
            query.as_ptr(),
            k as c_int,
            result_ids.as_mut_ptr(),
            result_distances.as_mut_ptr(),
        );
        
        println!("\nSearch results (found {}):", num_results);
        for i in 0..num_results as usize {
            println!("  ID: {}, Distance: {:.4}", 
                     result_ids[i], result_distances[i]);
        }
        
        // Clear database
        omendb_clear();
        println!("\nAfter clear. Count: {}", omendb_count());
    }
}

// For the actual Rust server integration:
//
// pub struct OmenDBFFI {
//     // Wrapper struct for the C FFI
// }
//
// impl OmenDBFFI {
//     pub fn new(dimension: usize) -> Result<Self, String> {
//         unsafe {
//             if omendb_init(dimension as c_int) != 1 {
//                 return Err("Failed to initialize OmenDB".into());
//             }
//         }
//         Ok(Self {})
//     }
//     
//     pub fn add_vector(&self, id: &str, vector: &[f32]) -> Result<(), String> {
//         unsafe {
//             let result = omendb_add(
//                 id.as_ptr(),
//                 id.len() as c_int,
//                 vector.as_ptr(),
//                 vector.len() as c_int,
//             );
//             
//             if result != 1 {
//                 return Err("Failed to add vector".into());
//             }
//         }
//         Ok(())
//     }
//     
//     pub fn search(&self, query: &[f32], k: usize) -> Vec<(i32, f32)> {
//         unsafe {
//             let mut ids = vec![0i32; k];
//             let mut distances = vec![0.0f32; k];
//             
//             let count = omendb_search(
//                 query.as_ptr(),
//                 k as c_int,
//                 ids.as_mut_ptr(),
//                 distances.as_mut_ptr(),
//             );
//             
//             ids.truncate(count as usize);
//             distances.truncate(count as usize);
//             
//             ids.into_iter()
//                 .zip(distances.into_iter())
//                 .collect()
//         }
//     }
// }