#!/usr/bin/env python3
"""
Test startup time claims - is 0.001ms real or a bug?

This tests the actual time to create a usable database object
and perform the first operation.
"""

import time
import os
import sys
import tempfile
import shutil

# Add the python directory to the path
current_dir = os.path.dirname(os.path.abspath(__file__))
root_dir = os.path.dirname(os.path.dirname(current_dir))
python_dir = os.path.join(root_dir, "python")
sys.path.insert(0, python_dir)


def test_omendb_startup():
    """Test OmenDB startup time."""
    print("Testing OmenDB startup time...")

    # Test 1: Time to create DB object
    start = time.perf_counter()
    from omendb import DB

    db = DB()
    create_time = (time.perf_counter() - start) * 1000
    print(f"  Object creation: {create_time:.3f}ms")

    # Test 2: Time to first operation (add)
    start = time.perf_counter()
    db.add("test", [1.0] * 128)
    first_add_time = (time.perf_counter() - start) * 1000
    print(f"  First add: {first_add_time:.3f}ms")

    # Test 3: Time to first query
    db2 = DB()
    db2.add("test", [1.0] * 128)
    start = time.perf_counter()
    results = db2.search([1.0] * 128)
    first_search_time = (time.perf_counter() - start) * 1000
    print(f"  First search: {first_search_time:.3f}ms")

    # Test 4: With persistence
    with tempfile.NamedTemporaryFile(suffix=".omendb", delete=False) as tmp:
        db_path = tmp.name

    start = time.perf_counter()
    db3 = DB(db_path=db_path)
    db3.add("test", [1.0] * 128)
    persist_time = (time.perf_counter() - start) * 1000
    print(f"  With persistence: {persist_time:.3f}ms")

    if os.path.exists(db_path):
        os.unlink(db_path)

    return create_time


def test_chromadb_startup():
    """Test ChromaDB startup time."""
    try:
        import chromadb
    except ImportError:
        print("ChromaDB not installed")
        return None

    print("\nTesting ChromaDB startup time...")

    # Test 1: Time to create client
    start = time.perf_counter()
    client = chromadb.Client()
    client_time = (time.perf_counter() - start) * 1000
    print(f"  Client creation: {client_time:.3f}ms")

    # Test 2: Time to create collection
    import uuid

    collection_name = f"test_{uuid.uuid4().hex[:8]}"
    start = time.perf_counter()
    collection = client.create_collection(collection_name)
    collection_time = (time.perf_counter() - start) * 1000
    print(f"  Collection creation: {collection_time:.3f}ms")

    # Test 3: Time to first operation
    start = time.perf_counter()
    collection.add(ids=["test"], embeddings=[[1.0] * 128])
    first_add_time = (time.perf_counter() - start) * 1000
    print(f"  First add: {first_add_time:.3f}ms")

    # Test 4: With persistence
    with tempfile.TemporaryDirectory() as tmpdir:
        start = time.perf_counter()
        persistent_client = chromadb.PersistentClient(path=tmpdir)
        persist_collection_name = f"test_{uuid.uuid4().hex[:8]}"
        collection = persistent_client.create_collection(persist_collection_name)
        collection.add(ids=["test"], embeddings=[[1.0] * 128])
        persist_time = (time.perf_counter() - start) * 1000
        print(f"  With persistence: {persist_time:.3f}ms")

    return client_time + collection_time


def test_lancedb_startup():
    """Test LanceDB startup time."""
    try:
        import lancedb
    except ImportError:
        print("LanceDB not installed")
        return None

    print("\nTesting LanceDB startup time...")

    with tempfile.TemporaryDirectory() as tmpdir:
        # Test 1: Time to connect
        start = time.perf_counter()
        db = lancedb.connect(tmpdir)
        connect_time = (time.perf_counter() - start) * 1000
        print(f"  Connection: {connect_time:.3f}ms")

        # Test 2: Time to create table
        start = time.perf_counter()
        table = db.create_table("test", [{"id": "test", "vector": [1.0] * 128}])
        table_time = (time.perf_counter() - start) * 1000
        print(f"  Table creation: {table_time:.3f}ms")

        # Test 3: Time to first query
        start = time.perf_counter()
        results = table.search([1.0] * 128).limit(1).to_pandas()
        search_time = (time.perf_counter() - start) * 1000
        print(f"  First search: {search_time:.3f}ms")

    return connect_time + table_time


def main():
    """Run all startup tests."""
    print("=" * 60)
    print("STARTUP TIME COMPARISON")
    print("=" * 60)

    # Run tests multiple times and take average
    omendb_times = []
    chromadb_times = []
    lancedb_times = []

    for i in range(5):
        if i > 0:  # Skip first run (warmup)
            omendb_time = test_omendb_startup()
            omendb_times.append(omendb_time)

            chromadb_time = test_chromadb_startup()
            if chromadb_time:
                chromadb_times.append(chromadb_time)

            lancedb_time = test_lancedb_startup()
            if lancedb_time:
                lancedb_times.append(lancedb_time)
        else:
            # Warmup run
            test_omendb_startup()
            test_chromadb_startup()
            test_lancedb_startup()

    print("\n" + "=" * 60)
    print("AVERAGE STARTUP TIMES (over 4 runs)")
    print("=" * 60)

    if omendb_times:
        avg_omendb = sum(omendb_times) / len(omendb_times)
        print(f"OmenDB:   {avg_omendb:.3f}ms")

    if chromadb_times:
        avg_chromadb = sum(chromadb_times) / len(chromadb_times)
        print(f"ChromaDB: {avg_chromadb:.3f}ms")

    if lancedb_times:
        avg_lancedb = sum(lancedb_times) / len(lancedb_times)
        print(f"LanceDB:  {avg_lancedb:.3f}ms")

    print("\n" + "=" * 60)
    print("REALITY CHECK")
    print("=" * 60)

    if omendb_times:
        if avg_omendb < 1.0:
            print(f"✅ OmenDB startup IS sub-millisecond ({avg_omendb:.3f}ms)")
        else:
            print(f"❌ OmenDB startup is NOT instant ({avg_omendb:.3f}ms)")

    if chromadb_times and omendb_times:
        ratio = avg_chromadb / avg_omendb
        print(f"📊 OmenDB is {ratio:.1f}x faster to start than ChromaDB")

    if lancedb_times and omendb_times:
        ratio = avg_lancedb / avg_omendb
        print(f"📊 OmenDB is {ratio:.1f}x faster to start than LanceDB")


if __name__ == "__main__":
    main()
