#!/usr/bin/env python3
"""
Platform-specific wheel building script for OmenDB.

This script handles the complexity of building wheels with the correct
native libraries for each target platform.
"""

import os
import sys
import platform
import shutil
from pathlib import Path


def get_platform_info():
    """Get current platform information."""
    system = platform.system().lower()
    machine = platform.machine().lower()
    
    if system == "darwin":
        return "macos", "dylib"
    elif system == "linux":
        return "linux", "so"  
    elif system == "windows":
        return "windows", "dll"
    else:
        raise ValueError(f"Unsupported platform: {system}")


def find_mojo_runtime_libs(platform_name, lib_extension):
    """Find Mojo runtime libraries for the current platform."""
    libs = []
    
    # Common Mojo runtime library names
    runtime_lib_names = [
        "libMSupportGlobals",
        "libAsyncRTRuntimeGlobals", 
        "libKGENCompilerRTShared",
        "libAsyncRTMojoBindings"
    ]
    
    # Look in pixi environment
    pixi_lib_dir = Path(".pixi/envs/default/lib")
    if pixi_lib_dir.exists():
        for lib_name in runtime_lib_names:
            lib_file = pixi_lib_dir / f"{lib_name}.{lib_extension}"
            if lib_file.exists():
                libs.append(str(lib_file))
                print(f"Found runtime library: {lib_file}")
    
    return libs


def find_native_module(platform_name, lib_extension):
    """Find the native module source (uses Mojo importer at runtime)."""
    # Look for native module source - OmenDB uses Mojo importer at runtime
    native_source = "omendb/native.mojo"
    
    if os.path.exists(native_source):
        print(f"Found native module source: {native_source}")
        print("ℹ️ OmenDB uses Mojo importer - no pre-compilation needed")
        return native_source
    
    print("❌ Native module source not found")
    return None


def create_platform_pyproject():
    """Create platform-specific pyproject.toml."""
    platform_name, lib_extension = get_platform_info()
    
    print(f"Building wheel for {platform_name} (lib extension: {lib_extension})")
    
    # Find libraries
    runtime_libs = find_mojo_runtime_libs(platform_name, lib_extension)
    native_module = find_native_module(platform_name, lib_extension)
    
    # Read base pyproject.toml
    base_pyproject = Path("pyproject.toml").read_text()
    
    # Platform-specific force-include section
    force_include_lines = []
    
    # Add native module source files (OmenDB uses Mojo importer)
    if native_module:
        # Include the entire omendb directory with Mojo source files
        force_include_lines.append(f'"omendb/" = "omendb/"')
    
    # Add runtime libraries
    for lib_path in runtime_libs:
        lib_name = Path(lib_path).name
        force_include_lines.append(f'"{lib_path}" = "omendb/lib/{lib_name}"')
    
    # Create new force-include section
    force_include_section = "\n".join(force_include_lines)
    
    # Replace the force-include section in pyproject.toml
    # Find the start and end of the force-include section
    lines = base_pyproject.split("\n")
    new_lines = []
    in_force_include = False
    
    for line in lines:
        if line.startswith("[tool.hatch.build.targets.wheel.force-include]"):
            new_lines.append(line)
            new_lines.extend(force_include_lines)
            in_force_include = True
        elif in_force_include and line.startswith("["):
            # End of force-include section
            in_force_include = False
            new_lines.append(line)
        elif not in_force_include:
            new_lines.append(line)
    
    # Write platform-specific pyproject.toml
    platform_pyproject = "\n".join(new_lines)
    Path("pyproject_platform.toml").write_text(platform_pyproject)
    
    print("Created platform-specific pyproject_platform.toml")
    return "pyproject_platform.toml"


def build_wheel():
    """Build the wheel using platform-specific configuration."""
    platform_pyproject = create_platform_pyproject()
    
    # Build wheel using the platform-specific config
    build_cmd = f"python -m build --wheel -C--config-file={platform_pyproject}"
    print(f"Running: {build_cmd}")
    
    exit_code = os.system(build_cmd)
    
    # Clean up
    if os.path.exists(platform_pyproject):
        os.remove(platform_pyproject)
    
    if exit_code == 0:
        print("✅ Wheel built successfully")
        # List built wheels
        dist_dir = Path("dist")
        if dist_dir.exists():
            wheels = list(dist_dir.glob("*.whl"))
            for wheel in wheels:
                print(f"Built: {wheel}")
    else:
        print("❌ Wheel build failed")
        sys.exit(1)


def validate_wheel():
    """Validate the built wheel."""
    print("\nValidating wheel...")
    
    # Find the latest wheel
    dist_dir = Path("dist")
    if not dist_dir.exists():
        print("❌ No dist directory found")
        return False
    
    wheels = sorted(dist_dir.glob("*.whl"), key=os.path.getmtime, reverse=True)
    if not wheels:
        print("❌ No wheels found")
        return False
    
    latest_wheel = wheels[0]
    print(f"Validating: {latest_wheel}")
    
    # Check wheel with twine
    check_cmd = f"python -m twine check {latest_wheel}"
    exit_code = os.system(check_cmd)
    
    if exit_code == 0:
        print("✅ Wheel validation passed")
        return True
    else:
        print("❌ Wheel validation failed")
        return False


def main():
    """Main entry point."""
    print("OmenDB Platform-Specific Wheel Builder")
    print("=" * 50)
    
    platform_name, lib_extension = get_platform_info()
    print(f"Target platform: {platform_name}")
    print(f"Library extension: {lib_extension}")
    
    # Check if we can build for this platform
    if platform_name == "windows":
        print("❌ Windows builds not yet supported (awaiting Mojo Windows runtime)")
        sys.exit(1)
    
    try:
        build_wheel()
        validate_wheel()
        print("\n🎉 Platform-specific wheel build completed successfully!")
        
    except Exception as e:
        print(f"❌ Build failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()