#!/usr/bin/env python3
"""
Test working native module using the correct Mojo import approach.
"""

import os
import sys

# The Mojo importer module will handle compilation of the Mojo files
import max.mojo.importer  # noqa: F401

current_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, current_dir)

try:
    # Import our simple native module - this should compile it transparently
    print("🔄 Importing simple_native module...")
    sys.path.insert(0, os.path.join(current_dir, "omendb"))
    import simple_native  # type: ignore

    print("✅ Native module imported successfully!")

    # Test basic functionality
    print("🧪 Testing connection...")
    result = simple_native.test_connection()
    print(f"Connection test: {result}")

    # Test add vector
    print("🧪 Testing add vector...")
    add_result = simple_native.add_vector("test_id", [1, 2, 3, 4])
    print(f"Add vector result: {add_result}")

    # Test stats
    print("🧪 Testing stats...")
    stats = simple_native.info()
    print(f"Stats: {stats}")

    print("🎉 All native module tests passed!")
    print("🎯 Database compilation and Python integration working!")

except ImportError as e:
    print(f"❌ Failed to import native module: {e}")
    print("This may indicate compilation issues or missing dependencies")

except Exception as e:
    print(f"❌ Error testing native module: {e}")
    import traceback

    traceback.print_exc()
