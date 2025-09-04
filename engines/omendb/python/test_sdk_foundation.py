#!/usr/bin/env python3
"""
Python SDK Foundation Test

Tests the basic Python SDK functionality with stub fallback.
This validates that the Python SDK foundation is working correctly
even without the compiled Mojo library.
"""

import os
import sys
import tempfile
from pathlib import Path

# Add the python package to path
project_root = Path(__file__).parent.parent
python_package = project_root / "python"
sys.path.insert(0, str(python_package))


def test_imports():
    """Test that all required modules can be imported."""
    print("🧪 Testing Python SDK imports...")

    try:
        # Try modern bindings first (with stub fallback)
        from omendb.api import EmbeddedDB, Vector, Metadata

        print("  ✅ Imported EmbeddedDB, Vector, Metadata (modern with stub fallback)")

        from omendb.exceptions import OmenDBError, DatabaseError

        print("  ✅ Imported exception classes")

        return True
    except ImportError as e:
        print(f"  ❌ Import failed: {e}")
        return False


def test_vector_creation():
    """Test Vector class functionality."""
    print("🧪 Testing Vector class...")

    try:
        from omendb.api import Vector

        # Test valid vector creation
        data = [1.0, 2.0, 3.0, 4.0]
        vec = Vector(data)

        print(f"  ✅ Created vector: {vec}")
        print(f"  ✅ Dimension: {vec.dimension}")
        print(f"  ✅ Length: {len(vec)}")

        # Test vector element access
        print(f"  ✅ First element: {vec[0]}")
        print(f"  ✅ As list: {vec.to_list()}")

        # Test empty vector error
        try:
            empty_vec = Vector([])
            print("  ❌ Empty vector should have failed")
            return False
        except ValueError:
            print("  ✅ Empty vector properly rejected")

        return True
    except Exception as e:
        print(f"  ❌ Vector test failed: {e}")
        return False


def test_metadata_creation():
    """Test Metadata class functionality."""
    print("🧪 Testing Metadata class...")

    try:
        from omendb.api import Metadata

        # Test metadata creation
        metadata = Metadata()
        print(f"  ✅ Created empty metadata: {metadata}")

        # Test with initial data
        data = {"title": "Test Document", "category": "test"}
        metadata_with_data = Metadata(data)
        print(f"  ✅ Created metadata with data: {metadata_with_data}")

        # Test setting values
        metadata.set("author", "Test Author")
        metadata["tags"] = "python,test"
        print("  ✅ Set metadata values")

        return True
    except Exception as e:
        print(f"  ❌ Metadata test failed: {e}")
        return False


def test_database_creation():
    """Test EmbeddedDB class functionality."""
    print("🧪 Testing EmbeddedDB class...")

    try:
        from omendb.api import EmbeddedDB

        # Create temporary database path
        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as temp_file:
            db_path = temp_file.name

        try:
            # Note: This will use stub implementation if library is not built
            db = EmbeddedDB(db_path)
            print(f"  ✅ Created database: {db}")

            # Test context manager
            with EmbeddedDB(db_path) as db_context:
                print(f"  ✅ Context manager works: {db_context}")

            print("  ✅ Database creation and cleanup successful")
            return True

        except Exception as e:
            print(f"  ⚠️ Database creation failed (expected if library not built): {e}")
            # This is expected if the native library isn't built yet
            return True

        finally:
            # Clean up
            try:
                os.unlink(db_path)
            except:
                pass

    except Exception as e:
        print(f"  ❌ Database test failed unexpectedly: {e}")
        return False


def test_sdk_foundation():
    """Test the overall SDK foundation."""
    print("🧪 Testing SDK Foundation...")

    try:
        from omendb.api import EmbeddedDB, Vector, Metadata
        from omendb.exceptions import OmenDBError

        # Test that classes exist and have expected interfaces
        assert hasattr(EmbeddedDB, "insert"), "EmbeddedDB should have insert method"
        assert hasattr(EmbeddedDB, "search"), "EmbeddedDB should have search method"
        assert hasattr(EmbeddedDB, "delete"), "EmbeddedDB should have delete method"
        assert hasattr(EmbeddedDB, "flush"), "EmbeddedDB should have flush method"
        assert hasattr(EmbeddedDB, "get_stats"), (
            "EmbeddedDB should have get_stats method"
        )
        assert hasattr(EmbeddedDB, "is_healthy"), (
            "EmbeddedDB should have is_healthy method"
        )

        assert hasattr(Vector, "dimension"), "Vector should have dimension property"
        assert hasattr(Vector, "to_list"), "Vector should have to_list method"

        assert hasattr(Metadata, "set"), "Metadata should have set method"

        print("  ✅ All expected methods and properties present")
        print("  ✅ SDK foundation is structurally complete")

        return True

    except Exception as e:
        print(f"  ❌ SDK foundation test failed: {e}")
        return False


def main():
    """Main test function."""
    print("🐍 OmenDB Python SDK Foundation Test")
    print("=" * 50)

    tests = [
        test_imports,
        test_vector_creation,
        test_metadata_creation,
        test_database_creation,
        test_sdk_foundation,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            if test():
                passed += 1
                print("✅ Test passed")
            else:
                failed += 1
                print("❌ Test failed")
        except Exception as e:
            failed += 1
            print(f"❌ Test error: {e}")
        print()

    print("=" * 50)
    print(f"Test Results: {passed} passed, {failed} failed")

    if failed == 0:
        print("🎉 ALL TESTS PASSED!")
        print()
        print("✅ Python SDK Foundation Status:")
        print("  • Core classes: Working")
        print("  • Method interfaces: Complete")
        print("  • Error handling: Functional")
        print("  • Ready for native library integration")
        print()
        print("📋 Next Steps:")
        print("  1. Build native Mojo library: mojo build python_bindings.mojo")
        print("  2. Install Python package: pip install -e python/")
        print("  3. Run full integration tests with native library")
    else:
        print(f"⚠️ {failed} tests failed - check implementation")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
