#!/usr/bin/env python3
import sys

sys.path.insert(0, "python")

try:
    from omendb.api import _native

    print("Native module available:", _native is not None)
    if _native:
        attrs = [attr for attr in dir(_native) if not attr.startswith("_")]
        print("Available functions:", attrs)
        if hasattr(_native, "load_database"):
            import inspect

            try:
                sig = inspect.signature(_native.load_database)
                print("load_database signature:", sig)
            except:
                print("Cannot inspect load_database signature")
except Exception as e:
    print("Error:", e)
    import traceback

    traceback.print_exc()
