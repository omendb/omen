#!/usr/bin/env python3
"""
Test CRUD Operations
===================

Comprehensive tests for Create, Read, Update, Delete operations in OmenDB.
"""

import numpy as np
import tempfile
import os
import sys
from omendb import DB


class TestCRUDOperations:
    """Test all CRUD operations."""

    def setup_method(self):
        """Create a temporary database for each test."""
        self.temp_dir = tempfile.mkdtemp()
        self.db_path = os.path.join(self.temp_dir, "test.omen")
        self.db = DB(self.db_path)
        self.dimension = 128

    def teardown_method(self):
        """Clean up temporary files."""
        import shutil

        shutil.rmtree(self.temp_dir, ignore_errors=True)

    def test_create_operations(self):
        """Test CREATE operations (add)."""
        # Test single add
        vector1 = np.random.rand(self.dimension).astype(np.float32).tolist()
        success = self.db.add("vec1", vector1, {"type": "test"})
        assert success == True

        # Test upsert with same ID (should update)
        vector2 = np.random.rand(self.dimension).astype(np.float32).tolist()
        success = self.db.add("vec1", vector2, {"type": "updated"})
        assert success == True

        # Test batch add
        batch_data = [
            (f"vec_{i}", np.random.rand(self.dimension).tolist(), {"idx": str(i)})
            for i in range(10)
        ]
        results = self.db.add_batch(batch_data)
        assert len(results) == 10
        assert all(results)

        # Verify count
        stats = self.db.info()
        assert stats["vector_count"] == 11  # vec1 + vec_0 through vec_9

    def test_read_operations(self):
        """Test READ operations (query, get_vector, exists)."""
        # Add test data
        vectors = []
        for i in range(5):
            vec = np.random.rand(self.dimension).astype(np.float32).tolist()
            vectors.append(vec)
            self.db.add(f"vec_{i}", vec, {"idx": str(i)})

        # Test query
        query_vec = vectors[0]
        results = self.db.search(query_vec, limit=3)
        assert len(results) == 3
        assert results[0].id == "vec_0"  # Should find itself as most similar

        # Test get_vector
        result = self.db.get("vec_0")
        retrieved = result[0] if result else None
        assert retrieved is not None
        assert len(retrieved) == self.dimension
        np.testing.assert_array_almost_equal(retrieved, vectors[0], decimal=5)

        # Test get non-existent vector
        result = self.db.get("non_existent")
        retrieved = result[0] if result else None
        assert retrieved is None

        # Test exists
        assert self.db.exists("vec_0") == True
        assert self.db.exists("vec_4") == True
        assert self.db.exists("non_existent") == False

    def test_update_operations(self):
        """Test UPDATE operations."""
        # Add initial vector
        vector1 = np.random.rand(self.dimension).astype(np.float32).tolist()
        self.db.add("vec1", vector1, {"version": "1"})

        # Update vector data
        vector2 = np.random.rand(self.dimension).astype(np.float32).tolist()
        success = self.db.add("vec1", vector2, {"version": "2"})
        assert success == True

        # Verify update
        result = self.db.get("vec1")
        retrieved = result[0] if result else None
        np.testing.assert_array_almost_equal(retrieved, vector2, decimal=5)

        # Verify metadata was updated
        result = self.db.get("vec1")
        metadata = result[1] if result else {}
        assert metadata["version"] == "2"

        # Test add non-existent vector (creates new with upsert behavior)
        success = self.db.add("non_existent", vector1)
        assert success == True  # add() always succeeds (upsert behavior)

        # Test batch update using add_batch (which does upsert)
        # First add vec2
        self.db.add("vec2", vector1, {"version": "0"})

        # Now batch update using columnar format
        ids = ["vec1", "vec2"]
        vectors = [vector1, vector2]
        metadata = [{"version": "3"}, {"version": "1"}]

        results = self.db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
        assert len(results) == 2
        assert "vec1" in results  # vec1 updated
        assert "vec2" in results  # vec2 updated

        # Verify updates
        result = self.db.get("vec1")
        metadata1 = result[1] if result else {}
        assert metadata1["version"] == "3"
        result = self.db.get("vec2")
        metadata2 = result[1] if result else {}
        assert metadata2["version"] == "1"

    def test_delete_operations(self):
        """Test DELETE operations."""
        # Add test data
        for i in range(5):
            vec = np.random.rand(self.dimension).tolist()
            self.db.add(f"vec_{i}", vec, {"idx": str(i)})

        # Test single delete
        success = self.db.delete("vec_0")
        assert success == True
        assert self.db.exists("vec_0") == False

        # Test delete non-existent
        success = self.db.delete("vec_0")  # Already deleted
        assert success == False

        # Test batch delete
        delete_ids = ["vec_1", "vec_2", "non_existent", "vec_3"]
        results = self.db.delete_batch(delete_ids)
        assert len(results) == 4
        assert results[0] == True  # vec_1 deleted
        assert results[1] == True  # vec_2 deleted
        assert results[2] == False  # non_existent
        assert results[3] == True  # vec_3 deleted

        # Verify deletions
        assert self.db.exists("vec_1") == False
        assert self.db.exists("vec_2") == False
        assert self.db.exists("vec_3") == False
        assert self.db.exists("vec_4") == True  # This one wasn't deleted

        # Verify count
        stats = self.db.info()
        assert stats["vector_count"] == "1"  # Only vec_4 remains

    def test_metadata_operations(self):
        """Test metadata CRUD operations."""
        # Add with metadata
        vec = np.random.rand(self.dimension).tolist()
        self.db.add("vec1", vec, {"key1": "value1", "key2": "value2"})

        # Get metadata
        result = self.db.get("vec1")
        metadata = result[1] if result else {}
        assert metadata["key1"] == "value1"
        assert metadata["key2"] == "value2"

        # Update with new metadata
        self.db.add("vec1", vec, {"key1": "updated", "key3": "new"})
        result = self.db.get("vec1")
        metadata = result[1] if result else {}
        assert metadata["key1"] == "updated"
        assert metadata.get("key2") is None  # Old key removed
        assert metadata["key3"] == "new"

        # Get metadata for non-existent vector
        result = self.db.get("non_existent")
        metadata = result[1] if result else {}
        assert metadata == {}

    def test_edge_cases(self):
        """Test edge cases and error conditions."""
        # Test empty vector
        try:
            self.db.add("empty", [])
            assert False, "Should have raised exception for empty vector"
        except:
            pass  # Expected

        # Test wrong dimension
        wrong_dim_vec = np.random.rand(64).tolist()
        success = self.db.add("wrong_dim", wrong_dim_vec)
        assert success == False

        # Test very long ID
        long_id = "x" * 1000
        vec = np.random.rand(self.dimension).tolist()
        success = self.db.add(long_id, vec)
        assert success == True
        assert self.db.exists(long_id) == True

        # Test special characters in ID
        special_id = "vec-123_test.doc#1"
        success = self.db.add(special_id, vec)
        assert success == True
        assert self.db.exists(special_id) == True

        # Test unicode in metadata
        unicode_metadata = {"text": "Hello 世界 🌍", "emoji": "🚀", "special": "café"}
        success = self.db.add("unicode_vec", vec, unicode_metadata)
        assert success == True
        result = self.db.get("unicode_vec")
        retrieved_metadata = result[1] if result else {}
        assert retrieved_metadata["text"] == unicode_metadata["text"]
        assert retrieved_metadata["emoji"] == unicode_metadata["emoji"]

    def test_persistence(self):
        """Test that CRUD operations persist across database reopening."""
        # Add data
        vec1 = np.random.rand(self.dimension).tolist()
        vec2 = np.random.rand(self.dimension).tolist()
        self.db.add("persist1", vec1, {"v": "1"})
        self.db.add("persist2", vec2, {"v": "2"})

        # Delete one
        self.db.delete("persist2")

        # Close and reopen database
        del self.db
        self.db = DB(self.db_path)

        # Verify data persisted correctly
        assert self.db.exists("persist1") == True
        assert self.db.exists("persist2") == False

        result = self.db.get("persist1")
        retrieved = result[0] if result else None
        np.testing.assert_array_almost_equal(retrieved, vec1, decimal=5)

        result = self.db.get("persist1")
        metadata = result[1] if result else {}
        assert metadata["v"] == "1"

    def test_query_after_modifications(self):
        """Test that queries work correctly after CRUD operations."""
        # Add initial vectors
        vectors = []
        for i in range(10):
            vec = np.random.rand(self.dimension).tolist()
            vectors.append(vec)
            self.db.add(f"vec_{i}", vec)

        # Delete some vectors
        self.db.delete("vec_2")
        self.db.delete("vec_5")
        self.db.delete("vec_8")

        # Update a vector
        new_vec = np.random.rand(self.dimension).tolist()
        self.db.add("vec_3", new_vec)
        vectors[3] = new_vec  # Update our reference

        # Query and verify results don't include deleted vectors
        query_vec = vectors[4]
        results = self.db.search(query_vec, limit=10)

        result_ids = [r[0] for r in results]
        assert "vec_2" not in result_ids
        assert "vec_5" not in result_ids
        assert "vec_8" not in result_ids
        assert "vec_4" in result_ids  # Should find itself

        # Verify we get exactly 7 results (10 - 3 deleted)
        assert len(results) == 7


def run_tests():
    """Run all tests and report results."""
    test_class = TestCRUDOperations()
    test_methods = [
        ("Create Operations", test_class.test_create_operations),
        ("Read Operations", test_class.test_read_operations),
        ("Update Operations", test_class.test_update_operations),
        ("Delete Operations", test_class.test_delete_operations),
        ("Metadata Operations", test_class.test_metadata_operations),
        ("Edge Cases", test_class.test_edge_cases),
        ("Persistence", test_class.test_persistence),
        ("Query After Modifications", test_class.test_query_after_modifications),
    ]

    passed = 0
    failed = 0

    print("🧪 Running CRUD Operations Tests")
    print("=" * 50)

    for test_name, test_method in test_methods:
        test_class.setup_method()
        try:
            test_method()
            print(f"✅ {test_name}: PASSED")
            passed += 1
        except Exception as e:
            import traceback

            print(f"❌ {test_name}: FAILED - {str(e)}")
            print(f"   Traceback: {traceback.format_exc()}")
            failed += 1
        finally:
            test_class.teardown_method()

    print("\n" + "=" * 50)
    print(f"Results: {passed} passed, {failed} failed")

    return failed == 0


if __name__ == "__main__":
    success = run_tests()
    sys.exit(0 if success else 1)
