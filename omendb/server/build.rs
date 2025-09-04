//! Build script for OmenDB Server
//! 
//! Simplified build script - skips gRPC generation for now and links C FFI library.

fn main() {
    // Link the Mojo C FFI library (using simplified batch version)
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rustc-link-lib=dylib=omendb_simple_batch_c_api");
    
    // For macOS, set rpath so it can find the library at runtime
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/");
    
    // For Linux
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    
    println!("cargo:warning=Skipping gRPC generation - implementing HTTP-only server first");
}