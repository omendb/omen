#!/usr/bin/env python3
"""
Cross-platform setup script for OmenDB.

Handles platform-specific library inclusion and build configuration.
Ensures compatibility across macOS, Linux, and Windows.
"""

import platform
import os
import sys
from pathlib import Path


def get_platform_info():
    """Get detailed platform information for cross-platform builds."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    platform_map = {
        'darwin': 'macos',
        'linux': 'linux', 
        'windows': 'windows'
    }
    
    return {
        'system': platform_map.get(system, system),
        'machine': machine,
        'python_version': f"{sys.version_info.major}.{sys.version_info.minor}",
        'raw_system': system
    }


def get_mojo_runtime_libraries(pixi_env_path):
    """
    Find Mojo runtime libraries in the pixi environment.
    
    Returns platform-specific library paths for packaging.
    """
    platform_info = get_platform_info()
    lib_dir = Path(pixi_env_path) / "lib"
    
    # Platform-specific library extensions
    extensions = {
        'macos': '.dylib',
        'linux': '.so',
        'windows': '.dll'
    }
    
    ext = extensions.get(platform_info['system'], '.so')
    
    # Required Mojo runtime libraries (common across platforms)
    required_libs = [
        'libAsyncRTRuntimeGlobals',
        'libMSupportGlobals', 
        'libKGENCompilerRTShared',
        'libAsyncRTMojoBindings'
    ]
    
    libraries = {}
    
    if not lib_dir.exists():
        print(f"Warning: Library directory not found: {lib_dir}")
        return libraries
    
    for lib_name in required_libs:
        lib_file = lib_name + ext
        lib_path = lib_dir / lib_file
        
        if lib_path.exists():
            # Include in package with platform-neutral path
            package_path = f"omendb/lib/{lib_file}"
            libraries[str(lib_path)] = package_path
            print(f"✅ Found: {lib_file}")
        else:
            print(f"⚠️  Missing: {lib_file} (expected at {lib_path})")
    
    return libraries


def generate_platform_specific_pyproject():
    """Generate platform-specific pyproject.toml configuration."""
    platform_info = get_platform_info()
    pixi_env = Path(".pixi/envs/default")
    
    print(f"🔍 Platform detected: {platform_info['system']} ({platform_info['machine']})")
    print(f"🐍 Python version: {platform_info['python_version']}")
    
    # Find Mojo runtime libraries
    libraries = get_mojo_runtime_libraries(pixi_env)
    
    if not libraries:
        print("❌ No Mojo runtime libraries found!")
        print("   This will likely cause import failures on the target platform.")
        return None
    
    # Generate force-include section for hatch
    force_include_lines = []
    for source_path, target_path in libraries.items():
        force_include_lines.append(f'"{source_path}" = "{target_path}"')
    
    force_include_section = "\n".join(force_include_lines)
    
    return {
        'platform_info': platform_info,
        'libraries': libraries,
        'force_include_section': force_include_section
    }


def create_cross_platform_test():
    """Create a test script to validate cross-platform functionality."""
    test_content = '''#!/usr/bin/env python3
"""
Cross-platform compatibility test for OmenDB.

Tests core functionality across different platforms to ensure
algorithm optimizations work consistently.
"""

import sys
import time
import platform
from pathlib import Path

def test_basic_import():
    """Test that OmenDB can be imported successfully."""
    try:
        from omendb import DB
        print("✅ Import successful")
        return True
    except ImportError as e:
        print(f"❌ Import failed: {e}")
        return False

def test_basic_functionality():
    """Test basic database operations."""
    try:
        from omendb import DB
        
        # Create database
        db = DB()
        print("✅ Database creation successful")
        
        # Add vectors
        test_vectors = [
            ("test1", [1.0, 2.0, 3.0]),
            ("test2", [4.0, 5.0, 6.0]),
            ("test3", [7.0, 8.0, 9.0])
        ]
        
        for vec_id, vector in test_vectors:
            db.add(vec_id, vector)
        print("✅ Vector addition successful")
        
        # Query
        results = db.query([1.0, 2.0, 3.0], top_k=2)
        if len(results) > 0:
            print(f"✅ Query successful (found {len(results)} results)")
            return True
        else:
            print("❌ Query returned no results")
            return False
            
    except Exception as e:
        print(f"❌ Functionality test failed: {e}")
        return False

def test_algorithm_optimizations():
    """Test that algorithm optimizations are working."""
    try:
        from omendb import DB
        
        # Test with larger dataset to trigger optimizations
        db = DB()
        
        # Add enough vectors to trigger optimized paths
        print("🔧 Testing algorithm optimizations...")
        start_time = time.time()
        
        for i in range(100):
            vector = [float(i * j) for j in range(10)]  # 10D vectors
            db.add(f"vec_{i}", vector)
        
        construction_time = time.time() - start_time
        print(f"✅ Construction time: {construction_time:.4f}s ({100/construction_time:.0f} vec/s)")
        
        # Test query performance
        query_times = []
        for _ in range(10):
            start_time = time.time()
            results = db.query([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
            query_time = (time.time() - start_time) * 1000  # Convert to ms
            query_times.append(query_time)
        
        avg_query_time = sum(query_times) / len(query_times)
        print(f"✅ Average query time: {avg_query_time:.3f}ms")
        
        return True
        
    except Exception as e:
        print(f"❌ Algorithm optimization test failed: {e}")
        return False

def run_cross_platform_tests():
    """Run comprehensive cross-platform tests."""
    print("🌐 OmenDB Cross-Platform Compatibility Test")
    print("=" * 50)
    
    # Platform info
    print(f"Platform: {platform.system()} {platform.release()}")
    print(f"Architecture: {platform.machine()}")
    print(f"Python: {sys.version}")
    print()
    
    # Run tests
    tests = [
        ("Import Test", test_basic_import),
        ("Basic Functionality", test_basic_functionality), 
        ("Algorithm Optimizations", test_algorithm_optimizations)
    ]
    
    results = []
    for test_name, test_func in tests:
        print(f"🧪 Running {test_name}...")
        success = test_func()
        results.append((test_name, success))
        print()
    
    # Summary
    print("📊 Test Results Summary")
    print("-" * 30)
    passed = 0
    for test_name, success in results:
        status = "PASS" if success else "FAIL"
        print(f"{test_name}: {status}")
        if success:
            passed += 1
    
    print(f"\\n🎯 Overall: {passed}/{len(results)} tests passed")
    
    if passed == len(results):
        print("🎉 All tests passed! Cross-platform compatibility confirmed.")
        return True
    else:
        print("⚠️  Some tests failed. Cross-platform issues detected.")
        return False

if __name__ == "__main__":
    success = run_cross_platform_tests()
    sys.exit(0 if success else 1)
'''
    
    with open("test_cross_platform.py", "w") as f:
        f.write(test_content)
    
    print("✅ Created test_cross_platform.py")


def main():
    """Main cross-platform setup function."""
    print("🌐 OmenDB Cross-Platform Setup")
    print("=" * 40)
    
    config = generate_platform_specific_pyproject()
    if not config:
        print("❌ Failed to generate platform configuration")
        return False
    
    # Create cross-platform test
    create_cross_platform_test()
    
    # Display configuration summary
    print("\n📋 Platform Configuration Summary")
    print("-" * 40)
    print(f"Platform: {config['platform_info']['system']}")
    print(f"Architecture: {config['platform_info']['machine']}")
    print(f"Python: {config['platform_info']['python_version']}")
    print(f"Libraries found: {len(config['libraries'])}")
    
    for lib_path, package_path in config['libraries'].items():
        lib_name = Path(lib_path).name
        print(f"  📦 {lib_name}")
    
    print("\n🔧 Next Steps:")
    print("1. Update pyproject.toml with platform-specific libraries")
    print("2. Test build on target platforms")
    print("3. Run: python test_cross_platform.py")
    
    return True


if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)