#!/usr/bin/env python3
"""
Build script for OmenDB Python bindings.

This script compiles the Mojo C interface into a shared library
and copies it to the Python package directory.
"""

import os
import sys
import subprocess
import platform
import shutil
from pathlib import Path


def get_library_name():
    """Get platform-specific library name."""
    if platform.system() == "Windows":
        return "omendb.dll"
    elif platform.system() == "Darwin":
        return "libomendb.dylib"
    else:
        return "libomendb.so"


def find_mojo_executable():
    """Find the Mojo executable."""
    # Try common locations
    possible_paths = [
        "mojo",
        "pixi run mojo",
        os.path.expanduser("~/.modular/pkg/packages.modular.com_mojo/bin/mojo"),
    ]

    for path in possible_paths:
        try:
            result = subprocess.run(
                path.split() + ["--version"], capture_output=True, text=True, check=True
            )
            print(f"Found Mojo: {path}")
            return path
        except (subprocess.CalledProcessError, FileNotFoundError):
            continue

    raise RuntimeError(
        "Could not find Mojo executable. Please ensure Mojo is installed and accessible."
    )


def build_shared_library():
    """Build the shared library from Mojo source."""
    project_root = Path(__file__).parent.parent

    # Try modern Python module approach first
    modern_source = project_root / "src" / "api" / "python_module.mojo"
    ffi_source = project_root / "src" / "api" / "python_bindings.mojo"
    build_dir = project_root / "build"

    # Create build directory
    build_dir.mkdir(exist_ok=True)

    # Find Mojo executable
    mojo_cmd = find_mojo_executable()

    # Try modern approach first
    if modern_source.exists():
        print("Attempting modern Python module build...")
        if build_modern_module(mojo_cmd, modern_source, build_dir, project_root):
            return True

    # Fallback to FFI approach
    print("Falling back to C FFI approach...")
    return build_ffi_library(mojo_cmd, ffi_source, build_dir, project_root)


def build_modern_module(mojo_cmd, source_path, build_dir, project_root):
    """Build using modern python.bindings approach."""
    # For modern bindings, we create a Python extension module
    output_name = "omendb"  # Python module name

    # Build command for Python extension module
    cmd = mojo_cmd.split() + [
        "build",
        str(source_path),
        "-o",
        str(build_dir / (output_name + ".so")),
    ]

    print(f"Building Python module: {' '.join(cmd)}")

    try:
        original_cwd = os.getcwd()
        os.chdir(project_root)

        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("Modern module build successful!")
        print(result.stdout)

        if result.stderr:
            print("Build info:")
            print(result.stderr)

        # Copy to package directory for import
        built_module = build_dir / (output_name + ".so")
        if built_module.exists():
            package_dir = Path(__file__).parent / "omendb"
            dest_module = package_dir / (output_name + ".so")
            shutil.copy2(built_module, dest_module)
            print(f"Copied module to {dest_module}")
            return True

    except subprocess.CalledProcessError as e:
        print(f"Modern build failed: {e}")
        print("STDOUT:", e.stdout)
        print("STDERR:", e.stderr)
        return False
    finally:
        os.chdir(original_cwd)

    return False


def build_ffi_library(mojo_cmd, source_path, build_dir, project_root):
    """Build using C FFI approach."""
    library_name = get_library_name()
    output_path = build_dir / library_name

    # Mojo build command for shared library
    cmd = mojo_cmd.split() + ["build", str(source_path), "-o", str(output_path)]

    print(f"Building C FFI library: {' '.join(cmd)}")

    try:
        original_cwd = os.getcwd()
        os.chdir(project_root)

        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("FFI library build successful!")
        print(result.stdout)

        if result.stderr:
            print("Build info:")
            print(result.stderr)

        return output_path if output_path.exists() else None

    except subprocess.CalledProcessError as e:
        print(f"FFI build failed: {e}")
        print("STDOUT:", e.stdout)
        print("STDERR:", e.stderr)
        return None
    finally:
        os.chdir(original_cwd)


def copy_library_to_package():
    """Copy built library to Python package directory."""
    library_name = get_library_name()
    project_root = Path(__file__).parent.parent
    build_dir = project_root / "build"
    package_dir = Path(__file__).parent / "omendb"

    source_lib = build_dir / library_name
    dest_lib = package_dir / library_name

    if not source_lib.exists():
        print(f"Error: Built library not found at {source_lib}")
        return False

    try:
        shutil.copy2(source_lib, dest_lib)
        print(f"Copied library to {dest_lib}")
        return True
    except Exception as e:
        print(f"Error copying library: {e}")
        return False


def create_stub_library():
    """Create a stub library for development/testing."""
    print("Creating stub library for development...")

    package_dir = Path(__file__).parent / "omendb"
    library_name = get_library_name()
    stub_path = package_dir / library_name

    # Create a minimal stub file
    with open(stub_path, "w") as f:
        f.write("# Stub library file for development\\n")
        f.write("# Replace with actual compiled shared library\\n")

    print(f"Created stub library at {stub_path}")
    print("Note: This is a placeholder. Replace with actual compiled library.")

    return True


def main():
    """Main build function."""
    print("Building OmenDB Python bindings...")
    print(f"Platform: {platform.system()}")
    print(f"Target library: {get_library_name()}")

    # Try to build shared library
    library_path = build_shared_library()

    if library_path and library_path.exists():
        # Copy to package directory
        if copy_library_to_package():
            print("\\nBuild completed successfully!")
            print("You can now install the package with: pip install -e .")
        else:
            print("\\nBuild failed during library copy.")
            return 1
    else:
        print("\\nShared library build failed or not supported.")
        print("Creating stub library for development...")
        if create_stub_library():
            print("\\nStub created. You can develop the Python interface,")
            print("but you'll need a real compiled library for functionality.")
        else:
            print("\\nFailed to create stub library.")
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
