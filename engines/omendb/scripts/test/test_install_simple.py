#!/usr/bin/env python3
"""
Simple installation test that verifies the package structure is correct.
"""

import os
import sys

def test_package_structure():
    """Test that package has correct structure for installation."""
    print("🧪 PACKAGE STRUCTURE TEST")
    print("=" * 60)
    
    required_files = [
        "pyproject.toml",
        "README.md", 
        "LICENSE",
        "python/omendb/__init__.py",
        "python/omendb/api.py",
        "python/omendb/native.so",
    ]
    
    missing = []
    for file in required_files:
        if os.path.exists(file):
            print(f"✅ {file}")
        else:
            print(f"❌ {file} - MISSING!")
            missing.append(file)
    
    if missing:
        print(f"\n❌ Missing {len(missing)} required files!")
        return False
    
    print("\n✅ All required files present")
    return True

def test_imports():
    """Test that imports work correctly."""
    print("\n🔍 IMPORT TEST")
    print("=" * 60)
    
    # Add python dir to path
    sys.path.insert(0, "python")
    
    try:
        from omendb import DB
        print("✅ Basic import successful")
        
        # Test instantiation
        db = DB()
        print("✅ Database instantiation successful")
        
        # Test basic operation
        db.add("test", [1.0, 2.0, 3.0])
        print("✅ Vector addition successful")
        
        results = db.search([1.0, 2.0, 3.0], limit=5)
        print(f"✅ Query successful: {len(results)} results")
        
        return True
        
    except Exception as e:
        print(f"❌ Import test failed: {e}")
        import traceback
        traceback.print_exc()
        return False

def test_metadata():
    """Test package metadata."""
    print("\n📦 METADATA TEST")
    print("=" * 60)
    
    # Read pyproject.toml
    with open("pyproject.toml", "r") as f:
        content = f.read()
    
    checks = [
        ('name = "omendb"', "Package name"),
        ('version = "0.1.0"', "Version"),
        ('description =', "Description"),
        ('readme = "README.md"', "README reference"),
        ('license =', "License"),
    ]
    
    all_good = True
    for check, name in checks:
        if check in content:
            print(f"✅ {name} defined")
        else:
            print(f"❌ {name} missing!")
            all_good = False
    
    return all_good

def main():
    """Run all tests."""
    print("🚀 OMENDB INSTALLATION VERIFICATION")
    print("Checking package is ready for pip install")
    print("=" * 70)
    
    tests = [
        ("Package Structure", test_package_structure),
        ("Import Test", test_imports),
        ("Metadata Test", test_metadata),
    ]
    
    results = []
    for test_name, test_func in tests:
        try:
            success = test_func()
            results.append((test_name, success))
        except Exception as e:
            print(f"❌ {test_name} FAILED with exception: {e}")
            results.append((test_name, False))
    
    print("\n📋 FINAL RESULTS:")
    all_passed = True
    for test_name, success in results:
        status = "✅" if success else "❌"
        print(f"   {status} {test_name}")
        if not success:
            all_passed = False
    
    if all_passed:
        print("\n🎉 ALL TESTS PASSED!")
        print("   Package structure is ready for installation")
        print("\n📝 Next steps:")
        print("   1. Build wheel: python -m build")
        print("   2. Test install: pip install dist/*.whl")
        print("   3. Upload to PyPI: twine upload dist/*")
        return True
    else:
        print("\n❌ Some tests failed!")
        print("   Fix issues before building package")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)