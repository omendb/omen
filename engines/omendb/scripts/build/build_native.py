#!/usr/bin/env python3
"""
Build script for OmenDB native module.
Compiles the Mojo native module into a Python extension.
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def build_native_module():
    """Build the native module from Mojo source."""
    print("🔧 Building OmenDB native module...")
    
    # Get paths
    root_dir = Path(__file__).parent.parent.parent  # Go up to project root
    mojo_source = root_dir / "omendb" / "native.mojo"
    python_dir = root_dir / "python" / "omendb"
    
    # Ensure the mojo source exists
    if not mojo_source.exists():
        print(f"❌ Mojo source not found: {mojo_source}")
        return False
    
    # Ensure python directory exists
    python_dir.mkdir(parents=True, exist_ok=True)
    
    # Try to build with pixi/mojo
    try:
        print(f"📦 Building {mojo_source} to native.so...")
        
        # Change to the omendb directory for building
        os.chdir(root_dir / "omendb")
        
        # Build the module as a Python extension
        result = subprocess.run([
            "pixi", "run", "mojo", "build", "native.mojo", 
            "-o", str(python_dir / "native.so"),
            "--target", "python-extension"
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print("✅ Native module built successfully!")
            return True
        else:
            print(f"❌ Build failed: {result.stderr}")
            
            # Try alternative build approach
            print("🔄 Trying alternative build approach...")
            result2 = subprocess.run([
                "pixi", "run", "mojo", "build", "native.mojo", 
                "-o", str(python_dir / "native.so")
            ], capture_output=True, text=True)
            
            if result2.returncode == 0:
                print("✅ Native module built successfully (alternative approach)!")
                return True
            else:
                print(f"❌ Alternative build also failed: {result2.stderr}")
                return False
                
    except FileNotFoundError:
        print("❌ Mojo compiler not found. Make sure pixi environment is activated.")
        return False
    except Exception as e:
        print(f"❌ Build error: {e}")
        return False

def verify_native_module():
    """Verify the built native module works."""
    print("🧪 Verifying native module...")
    
    python_dir = Path(__file__).parent.parent.parent / "python" / "omendb"
    native_so = python_dir / "native.so"
    
    if not native_so.exists():
        print(f"❌ Native module not found: {native_so}")
        return False
    
    # Test loading the module
    try:
        sys.path.insert(0, str(python_dir))
        import native
        print("✅ Native module loads successfully!")
        
        # Test basic functionality
        handle = native.create_database(3)
        print(f"✅ Database creation works: handle={handle}")
        
        # Test algorithm selection
        result = native.add_vector(handle, "test", [1.0, 2.0, 3.0])
        print(f"✅ Vector addition works: {result}")
        
        stats = native.get_stats(handle)
        print(f"📊 Stats: {stats}")
        
        # Check if we're using the right algorithm
        if 'algorithm' in stats:
            algorithm = stats['algorithm']
            if isinstance(algorithm, bytes):
                algorithm = algorithm.decode('utf-8')
            print(f"🧠 Algorithm in use: {algorithm}")
            
            if "BruteForce" in algorithm:
                print("✅ Adaptive algorithm selection working - using BruteForce for small dataset!")
            elif "RoarGraph" in algorithm:
                print("⚠️  Using RoarGraph - check if threshold logic is working")
            else:
                print(f"❓ Unknown algorithm: {algorithm}")
        
        return True
        
    except Exception as e:
        print(f"❌ Module verification failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def main():
    """Main build process."""
    print("🚀 OmenDB Native Module Build Process")
    print("=" * 50)
    
    # Build the native module
    if not build_native_module():
        print("❌ Build failed!")
        return False
    
    # Verify it works
    if not verify_native_module():
        print("❌ Verification failed!")
        return False
    
    print("🎉 Native module build complete and verified!")
    return True

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)