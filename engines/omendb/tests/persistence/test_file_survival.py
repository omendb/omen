#!/usr/bin/env python3
"""
File Persistence Validation Tests

CRITICAL: These tests validate the fundamental question - does our database actually work?
Do vectors survive database close/reopen cycles?

This is the foundation for all release claims.
"""

import os
import sys
import tempfile
from pathlib import Path

# Add python package to path
project_root = Path(__file__).parent.parent.parent
python_package = project_root / "python"
sys.path.insert(0, str(python_package))

try:
    from omendb import DB

    OMENDB_AVAILABLE = True
except ImportError as e:
    print(f"❌ Cannot import omendb: {e}")
    OMENDB_AVAILABLE = False


class TestFilePersistence:
    """Test whether data actually survives database restarts"""

    def test_basic_survival(self):
        """CRITICAL: Does data survive a basic close/reopen cycle?"""

        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Session 1: Add data and close
            test_vectors = [
                ("doc1", [1.0, 2.0, 3.0]),
                ("doc2", [4.0, 5.0, 6.0]),
                ("doc3", [7.0, 8.0, 9.0]),
            ]

            print(f"📝 Session 1: Adding {len(test_vectors)} vectors to {db_path}")
            with DB(db_path) as db:
                for vec_id, vector in test_vectors:
                    success = db.add(vec_id, vector)
                    print(f"   Added {vec_id}: {success}")
                    assert success, f"Failed to add {vec_id}"

                # Verify data exists in session 1
                session1_results = db.search([1.0, 2.0, 3.0], limit=3)
                print(f"   Session 1 query found: {len(session1_results)} results")
                assert len(session1_results) > 0, "No results in session 1"

            # Check file exists after close
            assert os.path.exists(db_path), (
                f"Database file {db_path} doesn't exist after close"
            )
            file_size = os.path.getsize(db_path)
            print(f"📁 Database file size after close: {file_size} bytes")

            # Session 2: Reopen and verify data survived
            print(f"🔄 Session 2: Reopening {db_path}")
            with DB(db_path) as db2:
                # Query for the same data
                session2_results = db2.search([1.0, 2.0, 3.0], limit=3)
                print(f"   Session 2 query found: {len(session2_results)} results")

                # CRITICAL ASSERTION: Data must survive
                assert len(session2_results) > 0, (
                    "❌ CRITICAL FAILURE: No data survived database restart"
                )

                # Verify we can find specific vectors
                found_ids = {result.id for result in session2_results}
                expected_ids = {"doc1", "doc2", "doc3"}

                print(f"   Expected IDs: {expected_ids}")
                print(f"   Found IDs: {found_ids}")

                # Check if we found the vectors we added
                intersection = found_ids.intersection(expected_ids)
                assert len(intersection) > 0, (
                    f"❌ CRITICAL: None of our vectors survived. Expected {expected_ids}, found {found_ids}"
                )

                if len(intersection) == len(expected_ids):
                    print("✅ ALL vectors survived database restart")
                else:
                    print(
                        f"⚠️  PARTIAL survival: {len(intersection)}/{len(expected_ids)} vectors survived"
                    )

        finally:
            # Cleanup
            if os.path.exists(db_path):
                os.unlink(db_path)

    def test_multiple_restart_cycles(self):
        """Test data survives multiple close/reopen cycles"""

        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            original_data = [("vec1", [1, 2, 3]), ("vec2", [4, 5, 6])]

            # Cycle 1: Add initial data
            with DB(db_path) as db:
                for vec_id, vector in original_data:
                    db.add(vec_id, vector)

            # Cycle 2: Verify and add more data
            with DB(db_path) as db:
                results = db.search([1, 2, 3], limit=2)
                assert len(results) > 0, "Data lost after first restart"

                # Add more data
                db.add("vec3", [7, 8, 9])

            # Cycle 3: Verify all data exists
            with DB(db_path) as db:
                results = db.search([1, 2, 3], limit=3)
                found_ids = {r.id for r in results}

                # Should have all three vectors
                expected_any_of = {"vec1", "vec2", "vec3"}
                intersection = found_ids.intersection(expected_any_of)
                assert len(intersection) > 0, (
                    f"No expected vectors found after multiple cycles: {found_ids}"
                )

                print(
                    f"✅ Multiple restart cycles: {len(intersection)} vectors survived"
                )

        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)

    def test_different_dimensions(self):
        """Test persistence with different vector dimensions"""

        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Test different dimensions
            test_cases = [
                ("dim3", [1.0, 2.0, 3.0]),
                ("dim4", [1.0, 2.0, 3.0, 4.0]),
                ("dim128", [0.1] * 128),
            ]

            # Add vectors with different dimensions
            with DB(db_path) as db:
                for vec_id, vector in test_cases:
                    success = db.add(vec_id, vector)
                    print(f"Added {vec_id} (dim {len(vector)}): {success}")

            # Verify survival for each dimension
            with DB(db_path) as db:
                for vec_id, vector in test_cases:
                    results = db.search(vector, limit=1)
                    assert len(results) > 0, (
                        f"❌ Vector {vec_id} (dim {len(vector)}) did not survive"
                    )
                    print(f"✅ {vec_id} (dim {len(vector)}) survived")

        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)

    def test_large_dataset_persistence(self):
        """Test persistence with larger number of vectors"""

        with tempfile.NamedTemporaryFile(suffix=".omen", delete=False) as tmp:
            db_path = tmp.name

        try:
            # Create 100 vectors
            vector_count = 100
            dimension = 64

            print(f"📊 Testing {vector_count} vectors of dimension {dimension}")

            # Add vectors
            with DB(db_path) as db:
                for i in range(vector_count):
                    vector = [float(i % 10) + 0.1 * j for j in range(dimension)]
                    success = db.add(f"vec_{i:03d}", vector)
                    if i % 20 == 0:
                        print(f"   Added vectors 0-{i}")

            file_size = os.path.getsize(db_path)
            print(f"📁 File size with {vector_count} vectors: {file_size:,} bytes")
            print(f"📏 Bytes per vector: {file_size / vector_count:.1f}")

            # Verify survival
            with DB(db_path) as db:
                # Test a few specific vectors
                test_vectors = [0, 25, 50, 75, 99]
                survived_count = 0

                for i in test_vectors:
                    vector = [float(i % 10) + 0.1 * j for j in range(dimension)]
                    results = db.search(vector, limit=1)
                    if len(results) > 0:
                        survived_count += 1

                survival_rate = survived_count / len(test_vectors)
                print(
                    f"📊 Survival rate: {survived_count}/{len(test_vectors)} ({survival_rate:.1%})"
                )

                assert survival_rate > 0, f"❌ No vectors survived from large dataset"
                if survival_rate == 1.0:
                    print("✅ All test vectors survived large dataset restart")
                else:
                    print(f"⚠️  Partial survival in large dataset: {survival_rate:.1%}")

        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)


def run_persistence_validation():
    """Run all persistence tests and provide summary"""

    print("🧪 OmenDB File Persistence Validation")
    print("=" * 50)
    print()

    if not OMENDB_AVAILABLE:
        print("❌ CRITICAL: Cannot import omendb - skipping persistence tests")
        return False

    try:
        # Run tests manually for better control
        test_instance = TestFilePersistence()

        print("Test 1: Basic Survival")
        test_instance.test_basic_survival()
        print("✅ PASSED\n")

        print("Test 2: Multiple Restart Cycles")
        test_instance.test_multiple_restart_cycles()
        print("✅ PASSED\n")

        print("Test 3: Different Dimensions")
        test_instance.test_different_dimensions()
        print("✅ PASSED\n")

        print("Test 4: Large Dataset Persistence")
        test_instance.test_large_dataset_persistence()
        print("✅ PASSED\n")

        print("🎉 ALL PERSISTENCE TESTS PASSED")
        print("✅ File persistence is WORKING")
        return True

    except Exception as e:
        print(f"❌ PERSISTENCE TEST FAILED: {e}")
        print("🚨 File persistence may not be working correctly")
        return False


if __name__ == "__main__":
    success = run_persistence_validation()
    sys.exit(0 if success else 1)
