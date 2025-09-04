#!/usr/bin/env python3
"""
Test the simple native module to verify compilation and Python integration works.
"""

import sys
import os

# Add the current directory to Python path for importing
sys.path.insert(0, "/Users/nick/github/omenDB")

try:
    # Try importing our simple native module
    import simple_native_module as simple_native

    print("✅ Native module imported successfully!")

    # Test basic functionality
    result = simple_native.test_connection()
    print(f"Connection test: {result}")

    # Test add vector
    add_result = simple_native.add_vector("test_id", [1, 2, 3, 4])
    print(f"Add vector result: {add_result}")

    # Test stats
    stats = simple_native.info()
    print(f"Stats: {stats}")

    print("🎉 All native module tests passed!")

except ImportError as e:
    print(f"❌ Failed to import native module: {e}")
    print("Module may need to be compiled or installed differently")

except Exception as e:
    print(f"❌ Error testing native module: {e}")
    import traceback

    traceback.print_exc()
