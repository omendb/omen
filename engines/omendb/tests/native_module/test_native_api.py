#!/usr/bin/env python3
import sys

sys.path.insert(0, "python")

from omendb.api import _native

print("Testing native module function signatures...")

# Test load_database
try:
    result = _native.load_database("test.db")
    print("load_database(1 arg) worked:", result)
except Exception as e:
    print("load_database(1 arg) failed:", type(e).__name__, str(e))

try:
    result = _native.load_database(0, "test.db")
    print("load_database(2 args) worked:", result)
except Exception as e:
    print("load_database(2 args) failed:", type(e).__name__, str(e))

# Test other functions
try:
    result = _native.test_connection()
    print("test_connection() worked:", result)
except Exception as e:
    print("test_connection() failed:", type(e).__name__, str(e))

try:
    result = _native.get_stats(0)
    print("get_stats(0) worked")
except Exception as e:
    print("get_stats(0) failed:", type(e).__name__, str(e))

try:
    result = _native.get_stats()
    print("get_stats() worked")
except Exception as e:
    print("get_stats() failed:", type(e).__name__, str(e))
