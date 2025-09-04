import sys

sys.path.insert(0, "python")

try:
    import omendb.native as native

    print("Available functions in native module:")
    for name in dir(native):
        if not name.startswith("_"):
            print(f"  - {name}")
except Exception as e:
    print(f"Error: {e}")
