#!/usr/bin/env python3
"""
PyPI Distribution Script for OmenDB

Handles building, testing, and uploading OmenDB packages to PyPI and TestPyPI.
Includes comprehensive validation and safety checks.
"""

import subprocess
import sys
import os
import tempfile
import shutil
from pathlib import Path
import json
import time


class PyPIDistributor:
    """Handles PyPI package distribution workflow."""
    
    def __init__(self, project_root=None):
        self.project_root = Path(project_root) if project_root else Path.cwd()
        self.dist_dir = self.project_root / "dist"
        
    def clean_dist(self):
        """Clean existing distribution artifacts."""
        if self.dist_dir.exists():
            shutil.rmtree(self.dist_dir)
            print("✅ Cleaned existing dist/ directory")
        
    def build_package(self):
        """Build package using cross-platform builder."""
        print("🔧 Building package...")
        
        # Use our cross-platform builder
        build_script = self.project_root / "build_cross_platform.py"
        if not build_script.exists():
            raise FileNotFoundError("build_cross_platform.py not found")
        
        result = subprocess.run([
            sys.executable, str(build_script), 
            "--build", "--output", str(self.dist_dir)
        ], capture_output=True, text=True, cwd=self.project_root)
        
        if result.returncode != 0:
            print(f"❌ Build failed:")
            print(f"   stdout: {result.stdout}")
            print(f"   stderr: {result.stderr}")
            return False
        
        print("✅ Package built successfully")
        
        # List generated files
        built_files = list(self.dist_dir.glob("*.whl")) + list(self.dist_dir.glob("*.tar.gz"))
        for file in built_files:
            print(f"   📦 {file.name}")
        
        return len(built_files) > 0
    
    def check_package(self):
        """Check package metadata and content."""
        print("🔍 Checking package...")
        
        # Find wheel file
        wheel_files = list(self.dist_dir.glob("*.whl"))
        if not wheel_files:
            print("❌ No wheel files found")
            return False
        
        wheel_file = wheel_files[0]
        
        # Use twine to check the package
        result = subprocess.run([
            "twine", "check", str(wheel_file)
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"❌ Package check failed:")
            print(f"   {result.stderr}")
            return False
        
        print("✅ Package metadata validation passed")
        
        # Check package contents
        print("📋 Package contents:")
        result = subprocess.run([
            sys.executable, "-m", "zipfile", "-l", str(wheel_file)
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            lines = result.stdout.split('\n')
            for line in lines[:15]:  # Show first 15 lines
                if line.strip():
                    print(f"   {line}")
            if len(lines) > 15:
                print(f"   ... and {len(lines) - 15} more files")
        
        return True
    
    def upload_to_testpypi(self):
        """Upload package to TestPyPI for testing."""
        print("🧪 Uploading to TestPyPI...")
        
        # Find wheel file
        wheel_files = list(self.dist_dir.glob("*.whl"))
        if not wheel_files:
            print("❌ No wheel files found")
            return False
        
        wheel_file = wheel_files[0]
        
        # Upload to TestPyPI
        result = subprocess.run([
            "twine", "upload", 
            "--repository", "testpypi",
            str(wheel_file)
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"❌ TestPyPI upload failed:")
            print(f"   {result.stderr}")
            if "already exists" in result.stderr.lower():
                print("   (Package version already exists - this is expected for testing)")
                return True
            return False
        
        print("✅ Package uploaded to TestPyPI successfully")
        return True
    
    def test_installation_from_testpypi(self):
        """Test installing the package from TestPyPI."""
        print("🧪 Testing installation from TestPyPI...")
        
        # Create temporary environment for testing
        with tempfile.TemporaryDirectory() as temp_dir:
            test_dir = Path(temp_dir)
            
            # Create a simple test script
            test_script = test_dir / "test_install.py"
            test_content = '''
import sys
import subprocess
import time

# Install from TestPyPI
print("Installing omendb from TestPyPI...")
result = subprocess.run([
    sys.executable, "-m", "pip", "install", 
    "--index-url", "https://test.pypi.org/simple/",
    "--extra-index-url", "https://pypi.org/simple/",
    "omendb"
], capture_output=True, text=True)

if result.returncode != 0:
    print(f"❌ Installation failed: {result.stderr}")
    sys.exit(1)

print("✅ Installation successful")

# Test basic functionality
try:
    from omendb import OmenDB
    print("✅ Import successful")
    
    # Basic functionality test
    db = OmenDB()
    db.add("test1", [1.0, 2.0, 3.0])
    db.add("test2", [4.0, 5.0, 6.0])
    results = db.query([1.0, 2.0, 3.0], top_k=2)
    print(f"✅ Basic functionality: found {len(results)} results")
    
    print("🎉 TestPyPI package validation successful!")
    
except Exception as e:
    print(f"❌ Functionality test failed: {e}")
    sys.exit(1)
'''
            
            with open(test_script, 'w') as f:
                f.write(test_content)
            
            # Run the test
            result = subprocess.run([
                sys.executable, str(test_script)
            ], capture_output=True, text=True, cwd=test_dir)
            
            print(result.stdout)
            if result.stderr:
                print(f"Stderr: {result.stderr}")
            
            if result.returncode == 0:
                print("✅ TestPyPI installation test passed")
                return True
            else:
                print("❌ TestPyPI installation test failed")
                return False
    
    def upload_to_pypi(self):
        """Upload package to production PyPI."""
        print("🚀 Uploading to production PyPI...")
        
        # Find wheel file
        wheel_files = list(self.dist_dir.glob("*.whl"))
        if not wheel_files:
            print("❌ No wheel files found")
            return False
        
        wheel_file = wheel_files[0]
        
        # Confirm upload
        response = input(f"❗ Are you sure you want to upload {wheel_file.name} to production PyPI? (yes/no): ")
        if response.lower() != 'yes':
            print("❌ Upload cancelled")
            return False
        
        # Upload to PyPI
        result = subprocess.run([
            "twine", "upload", str(wheel_file)
        ], capture_output=True, text=True)
        
        if result.returncode != 0:
            print(f"❌ PyPI upload failed:")
            print(f"   {result.stderr}")
            return False
        
        print("✅ Package uploaded to PyPI successfully")
        print(f"🎉 OmenDB is now available: pip install omendb")
        return True
    
    def generate_distribution_report(self):
        """Generate a comprehensive distribution report."""
        report = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "project_root": str(self.project_root),
            "built_files": [],
            "package_info": {}
        }
        
        # List built files
        if self.dist_dir.exists():
            for file in self.dist_dir.glob("*"):
                report["built_files"].append({
                    "name": file.name,
                    "size": file.stat().st_size,
                    "type": file.suffix
                })
        
        # Package information
        wheel_files = list(self.dist_dir.glob("*.whl")) if self.dist_dir.exists() else []
        if wheel_files:
            wheel_file = wheel_files[0]
            report["package_info"] = {
                "wheel_file": wheel_file.name,
                "wheel_size": wheel_file.stat().st_size
            }
        
        # Save report
        report_file = self.project_root / "distribution_report.json"
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        print(f"📊 Distribution report saved: {report_file}")
        return report


def main():
    """Main PyPI distribution function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="PyPI distribution for OmenDB")
    parser.add_argument("--clean", action="store_true", help="Clean dist directory")
    parser.add_argument("--build", action="store_true", help="Build package")
    parser.add_argument("--check", action="store_true", help="Check package")
    parser.add_argument("--test-upload", action="store_true", help="Upload to TestPyPI")
    parser.add_argument("--test-install", action="store_true", help="Test installation from TestPyPI")
    parser.add_argument("--upload", action="store_true", help="Upload to production PyPI")
    parser.add_argument("--full-test", action="store_true", help="Run complete testing workflow")
    parser.add_argument("--report", action="store_true", help="Generate distribution report")
    
    args = parser.parse_args()
    
    distributor = PyPIDistributor()
    
    print("🚀 OmenDB PyPI Distribution")
    print("=" * 50)
    
    success = True
    
    if args.clean or args.full_test:
        distributor.clean_dist()
    
    if args.build or args.full_test:
        success = success and distributor.build_package()
    
    if args.check or args.full_test:
        success = success and distributor.check_package()
    
    if args.test_upload or args.full_test:
        success = success and distributor.upload_to_testpypi()
    
    if args.test_install or args.full_test:
        success = success and distributor.test_installation_from_testpypi()
    
    if args.upload and not args.full_test:
        success = success and distributor.upload_to_pypi()
    
    if args.report or args.full_test:
        distributor.generate_distribution_report()
    
    if args.full_test:
        if success:
            print("\n🎉 Full testing workflow completed successfully!")
            print("   Package is ready for production PyPI upload.")
            print("   Run with --upload to publish to PyPI.")
        else:
            print("\n❌ Testing workflow failed. Check the errors above.")
    
    # Default: show help
    if not any(vars(args).values()):
        print("🔧 Usage examples:")
        print("  python pypi_distribution.py --full-test    # Complete testing workflow")
        print("  python pypi_distribution.py --build        # Build package only")
        print("  python pypi_distribution.py --upload       # Upload to production PyPI")
        print()
        print("📋 Recommended workflow:")
        print("  1. python pypi_distribution.py --full-test")
        print("  2. python pypi_distribution.py --upload")
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()