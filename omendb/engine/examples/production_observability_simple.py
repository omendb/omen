#!/usr/bin/env python3
"""
Production Observability Example (Simplified)
===========================================

Demonstrates basic observability patterns with OmenDB using standard Python tools.
"""

import time
import json
import logging
import os
import numpy as np
from datetime import datetime

import omendb


def setup_logging():
    """Setup basic logging for production."""
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        handlers=[
            logging.FileHandler("omendb_production.log"),
            logging.StreamHandler(),
        ],
    )
    return logging.getLogger("omendb_production")


def main():
    """Demonstrate basic observability with OmenDB."""
    print("🚀 Production Observability Demo")
    print("=" * 50)

    # Setup logging
    logger = setup_logging()
    logger.info("Starting production observability demo")

    # Create database
    db_path = "observability_demo.omen"
    db = omendb.DB(db_path)
    logger.info(f"Created database: {db_path}")

    # Track basic metrics
    metrics = {
        "start_time": time.time(),
        "insert_count": 0,
        "query_count": 0,
        "errors": 0,
    }

    try:
        # Insert some test data
        print("\n📊 Inserting test data...")
        dimension = 128
        for i in range(100):
            vector = np.random.randn(dimension).tolist()
            metadata = {"id": str(i), "timestamp": str(time.time())}

            try:
                success = db.add(f"vec_{i}", vector, metadata)
                if success:
                    metrics["insert_count"] += 1
                    if i % 25 == 0:
                        logger.info(f"Inserted {i + 1} vectors")
            except Exception as e:
                logger.error(f"Insert error: {e}")
                metrics["errors"] += 1

        # Run some queries
        print("\n🔍 Running test queries...")
        for i in range(10):
            try:
                query_vec = np.random.randn(dimension).tolist()
                results = db.search(query_vec, limit=5)
                metrics["query_count"] += 1
                logger.info(f"Query {i + 1} returned {len(results)} results")
            except Exception as e:
                logger.error(f"Query error: {e}")
                metrics["errors"] += 1

        # Get database stats
        print("\n📈 Database Statistics:")
        stats = db.info()
        for key, value in stats.items():
            print(f"  {key}: {value}")
            logger.info(f"DB stat - {key}: {value}")

        # Calculate final metrics
        elapsed = time.time() - metrics["start_time"]
        metrics["elapsed_seconds"] = elapsed
        metrics["inserts_per_second"] = (
            metrics["insert_count"] / elapsed if elapsed > 0 else 0
        )
        metrics["queries_per_second"] = (
            metrics["query_count"] / elapsed if elapsed > 0 else 0
        )

        # Log final metrics
        print("\n📊 Performance Metrics:")
        print(f"  Total operations: {metrics['insert_count'] + metrics['query_count']}")
        print(f"  Insert rate: {metrics['inserts_per_second']:.2f} ops/sec")
        print(f"  Query rate: {metrics['queries_per_second']:.2f} ops/sec")
        print(f"  Error count: {metrics['errors']}")

        logger.info(f"Final metrics: {json.dumps(metrics, indent=2)}")

        # Simple health check
        print("\n🏥 Health Check:")
        if metrics["errors"] == 0:
            print("  ✅ System healthy - no errors")
            logger.info("Health check passed")
        else:
            print(f"  ⚠️  System has {metrics['errors']} errors")
            logger.warning(f"Health check: {metrics['errors']} errors detected")

        print("\n✅ Observability demo completed successfully!")

    except Exception as e:
        logger.error(f"Demo failed: {e}", exc_info=True)
        print(f"\n❌ Demo failed: {e}")

    finally:
        # Cleanup
        if os.path.exists(db_path):
            os.remove(db_path)
        if os.path.exists("omendb_production.log"):
            print("\n📝 Logs written to: omendb_production.log")


if __name__ == "__main__":
    main()
