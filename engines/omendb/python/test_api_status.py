#!/usr/bin/env python3
"""
Test current Python API status and functionality
"""

try:
    import omendb

    print("✅ omendb module loaded")

    # Test DB access
    db = omendb.DB()
    print("✅ DB class accessible")

    # Test basic functionality
    db.add("test1", [1.0, 2.0, 3.0])
    print("✅ Vector added")

    results = db.search([1.0, 2.0, 3.0], limit=1)
    print(f"✅ Search returned {len(results)} results")

    if len(results) > 0:
        print(f"✅ Found match with score: {results[0].score}")
        if results[0].score > 0.99:
            print("✅ Exact match accuracy confirmed")
    else:
        print("❌ No results found")

    # Test uncommitted training API
    try:
        has_train = hasattr(db, "train_with_queries")
        print(f"Training API available: {has_train}")

        if has_train:
            # Test training method
            base_data = [("doc1", [1.0, 2.0, 3.0]), ("doc2", [4.0, 5.0, 6.0])]
            training_queries = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]
            result = db.train_with_queries(base_data, training_queries)
            print(f"✅ Training API callable, result: {result}")
        else:
            print("⚠️ Training API not available")

    except Exception as e:
        print(f"❌ Training API error: {e}")

    print("\n🎯 Python API Status Check Complete")

except Exception as e:
    print(f"❌ API test failed: {e}")
    import traceback

    traceback.print_exc()
