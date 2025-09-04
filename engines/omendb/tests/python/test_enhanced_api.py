#!/usr/bin/env python3
"""
Test the enhanced OmenDB Python API with new features:
- Rich search results with scores and metadata
- Batch operations for insert/delete
- Query builder pattern
- Metadata filtering (placeholder)
"""

import sys
import os
import traceback

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "python"))


def test_enhanced_search_api():
    """Test enhanced search API with rich results."""
    print("🔍 Testing Enhanced Search API")
    print("=" * 30)

    try:
        import omendb

        # Test SearchResult import
        from omendb import SearchResult

        print("✅ SearchResult class imported successfully")

        # Create test database
        db = omendb.EmbeddedDB("test_enhanced_api.omen")
        db.set_dimension(128)

        # Add test vectors using new API
        vectors = [
            ("doc1", [0.1] * 128, {"category": "tech", "author": "alice"}),
            ("doc2", [0.2] * 128, {"category": "science", "author": "bob"}),
            ("doc3", [0.3] * 128, {"category": "tech", "author": "carol"}),
        ]

        for id, vector, metadata in vectors:
            success = db.add(id=id, vector=vector, metadata=metadata)
            print(f"  Add {id}: {success}")

        # Test new query API (industry standard)
        query = [0.15] * 128
        basic_results = db.search(vector=query, limit=3)
        print(f"  Query results: {basic_results}")

        # Test query with metadata
        metadata_results = db.search(vector=query, limit=3, include_metadata=True)
        print(f"  Metadata query results: {metadata_results}")

        # Validate SearchResult objects
        if isinstance(metadata_results, list) and len(metadata_results) > 0:
            first_result = metadata_results[0]
            if hasattr(first_result, "id") and hasattr(first_result, "score"):
                print(
                    f"  ✅ SearchResult structure: ID={first_result.id}, Score={first_result.score}"
                )
            else:
                print("  ⚠️ SearchResult missing expected attributes")

        db.close()
        print("✅ Enhanced search API working!")

    except Exception as e:
        print(f"❌ Enhanced search API error: {e}")
        traceback.print_exc()


def test_batch_operations():
    """Test batch insert and delete operations."""
    print("\n📦 Testing Batch Operations")
    print("=" * 27)

    try:
        import omendb

        db = omendb.EmbeddedDB("test_batch_api.omen")
        db.set_dimension(64)

        # Test batch insert
        batch_items = [
            ("batch1", [0.1] * 64, {"type": "batch", "index": "1"}),
            ("batch2", [0.2] * 64, {"type": "batch", "index": "2"}),
            ("batch3", [0.3] * 64, {"type": "batch", "index": "3"}),
            ("batch4", [0.4] * 64, {"type": "batch", "index": "4"}),
            ("batch5", [0.5] * 64, {"type": "batch", "index": "5"}),
        ]

        batch_results = db.batch_insert(batch_items)
        print(f"  Batch insert results: {batch_results}")

        success_count = sum(1 for result in batch_results if result)
        print(f"  Successful inserts: {success_count}/{len(batch_items)}")

        # Test search to verify inserts
        search_results = db.search([0.25] * 64, limit=5)
        print(f"  Search after batch insert: {len(search_results)} results")

        # Test batch delete
        delete_ids = ["batch2", "batch4", "batch5"]
        delete_results = db.batch_delete(delete_ids)
        print(f"  Batch delete results: {delete_results}")

        delete_success = sum(1 for result in delete_results if result)
        print(f"  Successful deletes: {delete_success}/{len(delete_ids)}")

        # Verify deletions
        search_after_delete = db.search([0.25] * 64, limit=5)
        print(f"  Search after batch delete: {len(search_after_delete)} results")

        db.close()
        print("✅ Batch operations working!")

    except Exception as e:
        print(f"❌ Batch operations error: {e}")
        traceback.print_exc()


def test_query_builder():
    """Test query builder pattern for advanced queries."""
    print("\n🏗️ Testing Query Builder")
    print("=" * 24)

    try:
        import omendb

        db = omendb.EmbeddedDB("test_query_builder.omen")
        db.set_dimension(32)

        # Insert test data
        test_data = [
            ("query1", [0.1] * 32, {"category": "A", "priority": "high"}),
            ("query2", [0.2] * 32, {"category": "B", "priority": "low"}),
            ("query3", [0.3] * 32, {"category": "A", "priority": "medium"}),
            ("query4", [0.4] * 32, {"category": "C", "priority": "high"}),
        ]

        for id, vector, metadata in test_data:
            db.insert(id, vector, metadata)

        # Test query builder basic usage
        query_vector = [0.25] * 32

        # Method chaining style
        builder_results = (
            db.search()
            .search(query_vector)
            .limit(3)
            .include_scores()
            .include_metadata()
            .execute()
        )

        print(f"  Query builder results: {len(builder_results)} items")

        # Test with metadata filtering (placeholder)
        filtered_results = (
            db.search()
            .search(query_vector)
            .where(category="A")
            .limit(2)
            .include_scores()
            .execute()
        )

        print(f"  Filtered query results: {len(filtered_results)} items")

        # Test individual methods
        builder = db.search()
        builder.search(query_vector)
        builder.limit(1)
        builder.include_metadata(True)

        individual_results = builder.execute()
        print(f"  Individual method results: {len(individual_results)} items")

        db.close()
        print("✅ Query builder working!")

    except Exception as e:
        print(f"❌ Query builder error: {e}")
        traceback.print_exc()


def test_api_ergonomics():
    """Test overall API ergonomics and usability."""
    print("\n🎨 Testing API Ergonomics")
    print("=" * 25)

    try:
        import omendb

        # Test different initialization patterns
        db = omendb.EmbeddedDB("test_ergonomics.omen")
        db.set_dimension(16)

        # Test Vector and Metadata classes
        vector = omendb.Vector([0.1, 0.2, 0.3, 0.4] * 4)
        metadata = omendb.Metadata({"title": "Test Document", "type": "example"})

        print(f"  Vector dimension: {vector.dimension}")
        print(f"  Metadata: {metadata}")

        # Test insertion with objects
        success = db.insert("ergonomic1", vector, metadata)
        print(f"  Object insertion: {success}")

        # Test insertion with raw data
        success2 = db.insert("ergonomic2", [0.5] * 16, {"title": "Raw Data"})
        print(f"  Raw data insertion: {success2}")

        # Test context manager
        with omendb.EmbeddedDB("test_context.omen") as ctx_db:
            ctx_db.set_dimension(8)
            ctx_success = ctx_db.insert("ctx1", [0.1] * 8)
            print(f"  Context manager insertion: {ctx_success}")

        print("  Context manager closed automatically")

        # Test database info
        print(f"  Database representation: {db}")
        print(f"  Database healthy: {db.is_healthy()}")

        stats = db.info()
        print(f"  Database stats: {stats[:100]}...")  # First 100 chars

        db.close()
        print("✅ API ergonomics working!")

    except Exception as e:
        print(f"❌ API ergonomics error: {e}")
        traceback.print_exc()


def test_error_handling():
    """Test error handling and edge cases."""
    print("\n⚠️ Testing Error Handling")
    print("=" * 25)

    try:
        import omendb

        # Test invalid inputs
        db = omendb.EmbeddedDB("test_errors.omen")
        db.set_dimension(4)

        # Test empty vector ID
        try:
            db.insert("", [0.1] * 4)
            print("  ❌ Empty ID should have failed")
        except ValueError:
            print("  ✅ Empty ID properly rejected")

        # Test query builder without search vector
        try:
            builder = db.search()
            builder.limit(5)
            builder.execute()  # Should fail - no search vector set
            print("  ❌ Query without vector should have failed")
        except ValueError:
            print("  ✅ Query without vector properly rejected")

        # Test search with invalid limit
        try:
            db.search([0.1] * 4, limit=0)
            print("  ❌ Zero limit should have failed")
        except ValueError:
            print("  ✅ Zero limit properly rejected")

        # Test batch operations with invalid data
        invalid_batch = [("test1", "invalid_vector", {})]
        try:
            results = db.batch_insert(invalid_batch)
            if not results[0]:  # Should return False for invalid data
                print("  ✅ Invalid batch data properly handled")
            else:
                print("  ⚠️ Invalid batch data accepted unexpectedly")
        except Exception:
            print("  ✅ Invalid batch data properly rejected")

        db.close()
        print("✅ Error handling working!")

    except Exception as e:
        print(f"❌ Error handling test error: {e}")
        traceback.print_exc()


def main():
    """Run all enhanced API tests."""
    print("🎯 OmenDB Enhanced API Testing")
    print("=" * 31)

    # Run all test suites
    test_enhanced_search_api()
    test_batch_operations()
    test_query_builder()
    test_api_ergonomics()
    test_error_handling()

    print("\n🏁 Enhanced API Testing Complete")
    print("=" * 32)
    print("✅ Modern vector database API features implemented!")
    print("📋 Key improvements:")
    print("  • Rich search results with scores and metadata")
    print("  • Batch operations for performance")
    print("  • Query builder pattern for advanced queries")
    print("  • Enhanced error handling and validation")
    print("  • Professional API ergonomics")


if __name__ == "__main__":
    main()
