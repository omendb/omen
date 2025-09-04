#!/usr/bin/env python3
"""
Cross-platform build script for OmenDB.

Generates platform-specific pyproject.toml configuration and handles
library bundling for different operating systems.
"""

import platform
import os
import sys
import shutil
from pathlib import Path
import subprocess
import json


class CrossPlatformBuilder:
    """Handles cross-platform building and packaging."""
    
    def __init__(self, project_root=None):
        self.project_root = Path(project_root) if project_root else Path.cwd()
        self.platform_info = self._get_platform_info()
        
    def _get_platform_info(self):
        """Get detailed platform information."""
        system = platform.system().lower()
        machine = platform.machine().lower()
        
        platform_map = {
            'darwin': 'macos',
            'linux': 'linux', 
            'windows': 'windows'
        }
        
        # Library extension mapping
        lib_extensions = {
            'macos': '.dylib',
            'linux': '.so',
            'windows': '.dll'
        }
        
        return {
            'system': platform_map.get(system, system),
            'machine': machine,
            'python_version': f"{sys.version_info.major}.{sys.version_info.minor}",
            'raw_system': system,
            'lib_extension': lib_extensions.get(platform_map.get(system, system), '.so')
        }
    
    def find_mojo_runtime_libraries(self):
        """Find Mojo runtime libraries for the current platform."""
        pixi_lib_dir = self.project_root / ".pixi" / "envs" / "default" / "lib"
        
        # Required Mojo runtime libraries (platform-independent names)
        required_libs = [
            'libAsyncRTRuntimeGlobals',
            'libMSupportGlobals', 
            'libKGENCompilerRTShared',
            'libAsyncRTMojoBindings'
        ]
        
        libraries = {}
        missing_libs = []
        
        if not pixi_lib_dir.exists():
            print(f"⚠️  Pixi library directory not found: {pixi_lib_dir}")
            return libraries, missing_libs
        
        ext = self.platform_info['lib_extension']
        
        for lib_name in required_libs:
            lib_file = lib_name + ext
            lib_path = pixi_lib_dir / lib_file
            
            if lib_path.exists():
                # Map to package-relative path
                package_path = f"omendb/lib/{lib_file}"
                libraries[str(lib_path)] = package_path
                print(f"✅ Found: {lib_file}")
            else:
                missing_libs.append(lib_file)
                print(f"❌ Missing: {lib_file}")
        
        return libraries, missing_libs
    
    def generate_platform_pyproject(self):
        """Generate platform-specific pyproject.toml content."""
        libraries, missing_libs = self.find_mojo_runtime_libraries()
        
        if missing_libs:
            print(f"⚠️  Missing libraries: {', '.join(missing_libs)}")
            print("   The package may not work correctly on this platform.")
        
        # Read base pyproject.toml
        base_pyproject_path = self.project_root / "pyproject.toml"
        
        if not base_pyproject_path.exists():
            raise FileNotFoundError(f"Base pyproject.toml not found: {base_pyproject_path}")
        
        # Generate force-include section
        force_include_lines = []
        for source_path, target_path in libraries.items():
            force_include_lines.append(f'"{source_path}" = "{target_path}"')
        
        # Base pyproject.toml content (platform-independent parts)
        base_content = f'''[build-system]
requires = ["hatchling", "setuptools>=61.0"]
build-backend = "hatchling.build"

[project]
name = "omendb"
version = "0.1.0"
description = "Embedded vector database with native performance"
readme = "README.md"
requires-python = ">=3.9"
license = {{text = "Apache-2.0"}}
authors = [
    {{name = "nijaru"}},
]
keywords = ["vector", "database", "embeddings", "similarity", "search", "ai"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: Apache Software License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10", 
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Database",
    "Topic :: Scientific/Engineering :: Artificial Intelligence",
]
dependencies = []

[project.urls]
Homepage = "https://omendb.io"
Repository = "https://github.com/nijaru/omendb"
Issues = "https://github.com/nijaru/omendb/issues"

[tool.hatch.build.targets.wheel]
packages = ["python/omendb"]

[tool.hatch.build.targets.wheel.sources]
"python" = ""

# Platform-specific library inclusion for {self.platform_info['system']} ({self.platform_info['machine']})
[tool.hatch.build.targets.wheel.force-include]
"omendb/" = "omendb/"'''
        
        # Add platform-specific libraries
        if force_include_lines:
            base_content += "\n" + "\n".join(force_include_lines)
        
        return base_content
    
    def generate_platform_pyproject_with_local_libs(self, copied_libraries):
        """Generate platform-specific pyproject.toml with local library references."""
        # Generate force-include section with local library paths
        force_include_lines = []
        for source_path, target_path in copied_libraries.items():
            force_include_lines.append(f'"{source_path}" = "{target_path}"')
        
        # Base pyproject.toml content (platform-independent parts)
        base_content = f'''[build-system]
requires = ["hatchling", "setuptools>=61.0"]
build-backend = "hatchling.build"

[project]
name = "omendb"
version = "0.1.0"
description = "Embedded vector database with native performance"
readme = "README.md"
requires-python = ">=3.9"
license = {{text = "Apache-2.0"}}
authors = [
    {{name = "nijaru"}},
]
keywords = ["vector", "database", "embeddings", "similarity", "search", "ai"]
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "License :: OSI Approved :: Apache Software License",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10", 
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Topic :: Database",
    "Topic :: Scientific/Engineering :: Artificial Intelligence",
]
dependencies = []

[project.urls]
Homepage = "https://omendb.io"
Repository = "https://github.com/nijaru/omendb"
Issues = "https://github.com/nijaru/omendb/issues"

[tool.hatch.build.targets.wheel]
packages = ["python/omendb"]

[tool.hatch.build.targets.wheel.sources]
"python" = ""

# Platform-specific library inclusion for {self.platform_info['system']} ({self.platform_info['machine']})
[tool.hatch.build.targets.wheel.force-include]'''
        
        # Add platform-specific libraries only (no source code)
        if force_include_lines:
            base_content += "\n" + "\n".join(force_include_lines)
        
        return base_content
    
    def copy_runtime_libraries(self):
        """Copy Mojo runtime libraries to a local libs directory."""
        local_libs_dir = self.project_root / "libs"
        local_libs_dir.mkdir(exist_ok=True)
        
        libraries, missing_libs = self.find_mojo_runtime_libraries()
        copied_libraries = {}
        
        for source_path, target_path in libraries.items():
            source = Path(source_path)
            lib_name = source.name
            local_lib_path = local_libs_dir / lib_name
            
            # Copy library to local directory
            try:
                shutil.copy2(source, local_lib_path)
                print(f"✅ Copied: {lib_name}")
                # Update target path to reference local copy
                copied_libraries[str(local_lib_path)] = target_path
            except Exception as e:
                print(f"❌ Failed to copy {lib_name}: {e}")
        
        return copied_libraries, missing_libs
    
    def create_platform_package(self, output_dir=None):
        """Create a platform-specific package."""
        output_dir = Path(output_dir) if output_dir else self.project_root / "dist"
        output_dir.mkdir(exist_ok=True)
        
        print(f"🔧 Building package for {self.platform_info['system']} ({self.platform_info['machine']})")
        
        # Copy runtime libraries to local directory
        print("📦 Copying runtime libraries...")
        copied_libraries, missing_libs = self.copy_runtime_libraries()
        
        if missing_libs:
            print(f"⚠️  Missing libraries: {', '.join(missing_libs)}")
        
        # Generate platform-specific pyproject.toml with local library paths
        platform_pyproject_content = self.generate_platform_pyproject_with_local_libs(copied_libraries)
        
        # Write platform-specific pyproject.toml
        platform_pyproject_path = self.project_root / f"pyproject.{self.platform_info['system']}.toml"
        with open(platform_pyproject_path, 'w') as f:
            f.write(platform_pyproject_content)
        
        print(f"✅ Generated: {platform_pyproject_path}")
        
        # Create platform-specific build using standard build process
        try:
            # Temporarily replace pyproject.toml
            original_pyproject = self.project_root / "pyproject.toml"
            backup_pyproject = self.project_root / "pyproject.toml.backup"
            
            # Backup original
            if original_pyproject.exists():
                shutil.copy2(original_pyproject, backup_pyproject)
            
            # Use platform-specific pyproject.toml
            shutil.copy2(platform_pyproject_path, original_pyproject)
            
            try:
                result = subprocess.run([
                    sys.executable, "-m", "build", 
                    "--outdir", str(output_dir)
                ], capture_output=True, text=True, cwd=self.project_root)
                
                if result.returncode == 0:
                    print(f"✅ Package built successfully in {output_dir}")
                    
                    # List generated files
                    built_files = list(output_dir.glob("*.whl")) + list(output_dir.glob("*.tar.gz"))
                    for file in built_files:
                        print(f"   📦 {file.name}")
                    
                    success = True
                else:
                    print(f"❌ Build failed:")
                    print(f"   stdout: {result.stdout}")
                    print(f"   stderr: {result.stderr}")
                    success = False
            
            finally:
                # Restore original pyproject.toml
                if backup_pyproject.exists():
                    shutil.copy2(backup_pyproject, original_pyproject)
                    backup_pyproject.unlink()
                
            return success
                
        except FileNotFoundError:
            print("❌ Build tool not found. Install with: pip install build")
            return False
    
    def validate_package(self, package_path):
        """Validate that a built package works correctly."""
        print(f"🧪 Validating package: {package_path}")
        
        # Create temporary environment for testing
        import tempfile
        import venv
        
        with tempfile.TemporaryDirectory() as temp_dir:
            venv_dir = Path(temp_dir) / "test_env"
            
            # Create virtual environment
            venv.create(venv_dir, with_pip=True)
            
            # Get python executable
            if self.platform_info['system'] == 'windows':
                python_exe = venv_dir / "Scripts" / "python.exe"
            else:
                python_exe = venv_dir / "bin" / "python"
            
            try:
                # Install the package
                result = subprocess.run([
                    str(python_exe), "-m", "pip", "install", str(package_path)
                ], capture_output=True, text=True)
                
                if result.returncode != 0:
                    print(f"❌ Package installation failed: {result.stderr}")
                    return False
                
                # Test import
                test_script = '''
import sys
sys.path.insert(0, ".")

try:
    from omendb import OmenDB
    print("✅ Import successful")
    
    # Basic functionality test
    db = OmenDB()
    db.add("test", [1.0, 2.0, 3.0])
    results = db.query([1.0, 2.0, 3.0])
    print(f"✅ Basic functionality: found {len(results)} results")
    
except Exception as e:
    print(f"❌ Test failed: {e}")
    sys.exit(1)
'''
                
                result = subprocess.run([
                    str(python_exe), "-c", test_script
                ], capture_output=True, text=True)
                
                if result.returncode == 0:
                    print("✅ Package validation successful")
                    print(result.stdout)
                    return True
                else:
                    print(f"❌ Package validation failed: {result.stderr}")
                    return False
                    
            except Exception as e:
                print(f"❌ Validation error: {e}")
                return False
    
    def create_cross_platform_workflow(self):
        """Create GitHub Actions workflow for cross-platform building."""
        workflow_content = '''name: Cross-Platform Build

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        python-version: ['3.9', '3.10', '3.11', '3.12']
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v4
      with:
        python-version: ${{ matrix.python-version }}
    
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install build pytest
    
    - name: Build cross-platform package
      run: |
        python build_cross_platform.py --build
    
    - name: Run cross-platform tests
      run: |
        python test_cross_platform.py
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: omendb-${{ matrix.os }}-py${{ matrix.python-version }}
        path: dist/
'''
        
        # Create .github/workflows directory
        workflow_dir = self.project_root / ".github" / "workflows"
        workflow_dir.mkdir(parents=True, exist_ok=True)
        
        workflow_path = workflow_dir / "cross-platform.yml"
        with open(workflow_path, 'w') as f:
            f.write(workflow_content)
        
        print(f"✅ Created GitHub Actions workflow: {workflow_path}")


def main():
    """Main cross-platform build function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Cross-platform build for OmenDB")
    parser.add_argument("--build", action="store_true", help="Build platform-specific package")
    parser.add_argument("--validate", help="Validate a built package")
    parser.add_argument("--workflow", action="store_true", help="Create GitHub Actions workflow")
    parser.add_argument("--output", help="Output directory for built packages")
    
    args = parser.parse_args()
    
    builder = CrossPlatformBuilder()
    
    print("🌐 OmenDB Cross-Platform Builder")
    print("=" * 50)
    print(f"Platform: {builder.platform_info['system']} ({builder.platform_info['machine']})")
    print(f"Python: {builder.platform_info['python_version']}")
    print()
    
    if args.workflow:
        builder.create_cross_platform_workflow()
        return
    
    if args.validate:
        success = builder.validate_package(args.validate)
        sys.exit(0 if success else 1)
    
    if args.build:
        success = builder.create_platform_package(args.output)
        sys.exit(0 if success else 1)
    
    # Default: show platform analysis
    libraries, missing_libs = builder.find_mojo_runtime_libraries()
    
    print(f"📋 Platform Analysis")
    print("-" * 30)
    print(f"System: {builder.platform_info['system']}")
    print(f"Library extension: {builder.platform_info['lib_extension']}")
    print(f"Libraries found: {len(libraries)}")
    print(f"Missing libraries: {len(missing_libs)}")
    
    if missing_libs:
        print()
        print("⚠️  Missing Libraries:")
        for lib in missing_libs:
            print(f"   {lib}")
    
    print()
    print("🔧 Usage:")
    print("  python build_cross_platform.py --build           # Build platform package")
    print("  python build_cross_platform.py --workflow        # Create CI workflow")
    print("  python build_cross_platform.py --validate <pkg>  # Validate package")


if __name__ == "__main__":
    main()