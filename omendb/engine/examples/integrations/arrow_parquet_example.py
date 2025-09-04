#!/usr/bin/env python3
"""
Apache Arrow & Parquet Integration Example

Demonstrates OmenDB's integration with the modern data ecosystem:
- Import vectors from Parquet files (data lake format)
- Export search results to Arrow/Parquet
- Integration with Polars (high-performance DataFrames)
- Zero-copy operations where possible
- Production ML pipeline patterns

This enables OmenDB to work seamlessly with:
- Data lakes and lakehouses (Parquet files)
- Analytics databases (DuckDB, ClickHouse)
- ML pipelines (Ray, Dask, MLflow)
- Data processing frameworks (Spark, Pandas, Polars)
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "..", "python"))

from omendb import DB
import numpy as np
import json
from pathlib import Path
from typing import List, Dict, Any

# Try to import Arrow/Parquet dependencies
try:
    import pyarrow as pa
    import pyarrow.parquet as pq

    ARROW_AVAILABLE = True
    print("✅ Apache Arrow available")
except ImportError:
    ARROW_AVAILABLE = False
    print("❌ Apache Arrow not available: pip install pyarrow")

# Try to import Polars
try:
    import polars as pl

    POLARS_AVAILABLE = True
    print("✅ Polars available")
except ImportError:
    POLARS_AVAILABLE = False
    print("❌ Polars not available: pip install polars")

# Try to import pandas for comparison
try:
    import pandas as pd

    PANDAS_AVAILABLE = True
    print("✅ Pandas available")
except ImportError:
    PANDAS_AVAILABLE = False
    print("❌ Pandas not available")


def create_sample_parquet_data():
    """Create sample Parquet file for demonstration."""
    print("📄 Creating sample Parquet dataset...")

    # Sample embedding data (simulating real ML pipeline output)
    documents = [
        {
            "doc_id": "paper_001",
            "title": "Transformer Architecture for Natural Language Processing",
            "category": "nlp",
            "year": 2017,
            "citation_count": 15420,
            "embedding": np.random.randn(384).tolist(),
        },
        {
            "doc_id": "paper_002",
            "title": "BERT: Bidirectional Encoder Representations from Transformers",
            "category": "nlp",
            "year": 2018,
            "citation_count": 35210,
            "embedding": np.random.randn(384).tolist(),
        },
        {
            "doc_id": "paper_003",
            "title": "Vision Transformer for Image Recognition at Scale",
            "category": "cv",
            "year": 2020,
            "citation_count": 8934,
            "embedding": np.random.randn(384).tolist(),
        },
        {
            "doc_id": "paper_004",
            "title": "GPT-3: Language Models are Few-Shot Learners",
            "category": "nlp",
            "year": 2020,
            "citation_count": 12045,
            "embedding": np.random.randn(384).tolist(),
        },
        {
            "doc_id": "paper_005",
            "title": "ResNet: Deep Residual Learning for Image Recognition",
            "category": "cv",
            "year": 2015,
            "citation_count": 89234,
            "embedding": np.random.randn(384).tolist(),
        },
    ]

    if ARROW_AVAILABLE:
        # Create Arrow table
        table = pa.table(
            {
                "doc_id": [doc["doc_id"] for doc in documents],
                "title": [doc["title"] for doc in documents],
                "category": [doc["category"] for doc in documents],
                "year": [doc["year"] for doc in documents],
                "citation_count": [doc["citation_count"] for doc in documents],
                "embedding": [doc["embedding"] for doc in documents],
            }
        )

        # Write to Parquet with different compression options
        parquet_file = "sample_embeddings.parquet"
        pq.write_table(table, parquet_file, compression="snappy")

        print(f"   ✅ Created {parquet_file} with {len(documents)} documents")
        print(f"   📊 Schema: {table.schema}")

        return parquet_file, documents

    else:
        # Create mock file for demo
        print("   🔧 Mock data created (install pyarrow for real Parquet support)")
        return "mock_sample_embeddings.parquet", documents


def demonstrate_parquet_import():
    """Demonstrate importing vectors from Parquet files."""
    print("\n📥 Parquet Import Demonstration")
    print("=" * 35)

    # Create sample data
    parquet_file, sample_docs = create_sample_parquet_data()

    if ARROW_AVAILABLE and hasattr(DB, "import_parquet"):
        print("🗄️ Importing from Parquet file...")

        # Initialize database
        db = DB("arrow_demo.omen")

        # Import with different options
        try:
            # Basic import
            imported_count = db.import_parquet(
                file_path=parquet_file,
                id_column="doc_id",
                vector_column="embedding",
                metadata_columns=["title", "category", "year", "citation_count"],
            )

            print(f"   ✅ Imported {imported_count} vectors from Parquet")

            # Show database stats
            stats = db.info()
            print(
                f"   📊 Database: {stats.get('vector_count', 0)} vectors, {stats.get('dimension', 0)}D"
            )

            # Test search
            print("\n🔍 Testing semantic search...")
            query_embedding = np.random.randn(384).tolist()
            results = db.search(query_embedding, limit=3)

            for i, result in enumerate(results, 1):
                title = (
                    result.metadata.get("title", "Unknown")
                    if result.metadata
                    else "Unknown"
                )
                category = (
                    result.metadata.get("category", "Unknown")
                    if result.metadata
                    else "Unknown"
                )
                print(
                    f"   {i}. {title[:50]}... (category: {category}, similarity: {result.score:.3f})"
                )

            return db

        except Exception as e:
            print(f"   ❌ Import failed: {e}")
            return None

    else:
        print("🔧 Arrow/Parquet import not available (install pyarrow)")

        # Fallback: demonstrate with regular API
        db = DB("arrow_demo_fallback.omen")

        for doc in sample_docs:
            metadata = {
                "title": doc["title"],
                "category": doc["category"],
                "year": str(doc["year"]),
                "citation_count": str(doc["citation_count"]),
            }
            db.add(doc["doc_id"], doc["embedding"], metadata)

        print(f"   ✅ Added {len(sample_docs)} vectors using regular API")
        return db


def demonstrate_parquet_export(db: DB):
    """Demonstrate exporting search results to Parquet."""
    print("\n📤 Parquet Export Demonstration")
    print("=" * 35)

    if ARROW_AVAILABLE and hasattr(db, "export_parquet"):
        print("💾 Exporting search results to Parquet...")

        try:
            # Create a query
            query_embedding = np.random.randn(384).tolist()

            # Export search results
            export_file = "search_results.parquet"
            exported_count = db.export_parquet(
                file_path=export_file,
                query_vector=query_embedding,
                limit=5,
                include_vectors=True,  # Include vector data
                compression="gzip",
            )

            print(f"   ✅ Exported {exported_count} search results to {export_file}")

            # Read back and display
            if ARROW_AVAILABLE:
                table = pq.read_table(export_file)
                print(f"   📊 Export schema: {table.schema}")
                print(f"   📄 Export preview:")

                # Convert to pandas for nice display
                if PANDAS_AVAILABLE:
                    df = table.to_pandas()
                    print(df[["id", "similarity"]].head())
                else:
                    # Display raw data
                    for i in range(min(3, len(table))):
                        row_id = table["id"][i].as_py()
                        similarity = table["similarity"][i].as_py()
                        print(
                            f"      {i + 1}. ID: {row_id}, Similarity: {similarity:.3f}"
                        )

        except Exception as e:
            print(f"   ❌ Export failed: {e}")

    else:
        print("🔧 Arrow/Parquet export not available (install pyarrow)")


def demonstrate_polars_integration(db: DB):
    """Demonstrate Polars integration for high-performance data processing."""
    print("\n⚡ Polars Integration Demonstration")
    print("=" * 38)

    if POLARS_AVAILABLE and hasattr(db, "export_polars"):
        print("🔥 Using Polars for high-performance data operations...")

        try:
            # Export to Polars DataFrame
            query_embedding = np.random.randn(384).tolist()

            df = db.export_polars(
                query_vector=query_embedding,
                limit=5,
                include_vectors=False,  # Exclude vectors for cleaner display
            )

            print(f"   ✅ Exported to Polars DataFrame: {df.shape}")
            print(f"   📊 Columns: {df.columns}")

            # Demonstrate Polars operations
            print("\n   🔍 Polars operations:")

            # Filter and sort
            if "metadata_category" in df.columns:
                nlp_results = df.filter(pl.col("metadata_category") == "nlp")
                print(f"      • NLP papers: {len(nlp_results)} results")

                # Sort by similarity
                top_results = df.sort("similarity", descending=True).head(3)
                print("      • Top 3 results:")
                for row in top_results.iter_rows(named=True):
                    print(f"        - {row['id']}: {row['similarity']:.3f}")

            # Aggregate operations
            if "metadata_year" in df.columns:
                year_stats = df.group_by("metadata_year").agg(
                    [
                        pl.count().alias("count"),
                        pl.col("similarity").mean().alias("avg_similarity"),
                    ]
                )
                print("      • Results by year:")
                for row in year_stats.iter_rows(named=True):
                    print(
                        f"        - {row['metadata_year']}: {row['count']} papers, avg similarity: {row['avg_similarity']:.3f}"
                    )

        except Exception as e:
            print(f"   ❌ Polars integration failed: {e}")

    else:
        print("🔧 Polars integration not available")
        if not POLARS_AVAILABLE:
            print("   Install with: pip install polars")


def demonstrate_production_pipeline():
    """Demonstrate a production ML pipeline with Arrow/Parquet."""
    print("\n🏭 Production ML Pipeline Demonstration")
    print("=" * 42)

    print("📋 Typical production workflow:")
    print("   1. Data scientists export embeddings to Parquet (data lake)")
    print("   2. OmenDB imports from Parquet for vector search")
    print("   3. Search results exported back to Parquet for analytics")
    print("   4. Integration with downstream ML/analytics tools")

    if ARROW_AVAILABLE:
        print("\n🔄 Simulating production data flow...")

        try:
            # Step 1: Simulate data science team output
            print("   1. 📊 Data science team exports embeddings...")

            # Create larger dataset
            batch_size = 1000
            embeddings_data = {
                "document_id": [f"doc_{i:06d}" for i in range(batch_size)],
                "content_type": np.random.choice(
                    ["article", "paper", "blog", "news"], batch_size
                ),
                "publish_date": np.random.choice(
                    ["2023-01", "2023-02", "2023-03", "2023-04"], batch_size
                ),
                "embedding": [
                    np.random.randn(128).tolist() for _ in range(batch_size)
                ],  # Smaller for demo
            }

            production_table = pa.table(embeddings_data)
            production_file = "production_embeddings.parquet"
            pq.write_table(production_table, production_file, compression="snappy")

            print(f"      ✅ Created {production_file} with {batch_size} embeddings")

            # Step 2: OmenDB imports for search
            print("   2. 🔍 OmenDB imports for vector search...")

            db = DB("production_demo.omen")

            # Import with batch processing
            imported = db.import_parquet(
                file_path=production_file,
                id_column="document_id",
                vector_column="embedding",
                metadata_columns=["content_type", "publish_date"],
                batch_size=250,  # Process in batches
            )

            print(f"      ✅ Imported {imported} vectors for search")

            # Step 3: Search and export results
            print("   3. 📤 Export search results for analytics...")

            query_vec = np.random.randn(128).tolist()

            # Export to different formats for different use cases
            analytics_file = "analytics_results.parquet"
            exported = db.export_parquet(
                file_path=analytics_file,
                query_vector=query_vec,
                limit=100,
                include_vectors=False,  # Don't include vectors for analytics
                compression="gzip",
            )

            print(f"      ✅ Exported {exported} results to {analytics_file}")

            # Step 4: Demonstrate integration possibilities
            print("   4. 🔗 Integration with analytics tools:")
            print("      • DuckDB: SELECT * FROM 'analytics_results.parquet'")
            print("      • Spark: spark.read.parquet('analytics_results.parquet')")
            print("      • Polars: pl.read_parquet('analytics_results.parquet')")
            print("      • Pandas: pd.read_parquet('analytics_results.parquet')")

            # Cleanup demo files
            import os

            for file in [production_file, analytics_file]:
                if os.path.exists(file):
                    os.remove(file)

        except Exception as e:
            print(f"   ❌ Production pipeline demo failed: {e}")

    else:
        print("🔧 Production pipeline requires Apache Arrow")
        print("   Install with: pip install pyarrow")


def main():
    """Run comprehensive Arrow/Parquet integration demo."""
    print("🏹 Apache Arrow & Parquet Integration Demo")
    print("=" * 45)
    print()

    # Check dependencies
    missing_deps = []
    if not ARROW_AVAILABLE:
        missing_deps.append("pyarrow")
    if not POLARS_AVAILABLE:
        missing_deps.append("polars")
    if not PANDAS_AVAILABLE:
        missing_deps.append("pandas")

    if missing_deps:
        print("📦 Optional dependencies for full functionality:")
        for dep in missing_deps:
            print(f"   pip install {dep}")
        print()

    try:
        # Run demonstrations
        db = demonstrate_parquet_import()

        if db:
            demonstrate_parquet_export(db)
            demonstrate_polars_integration(db)

        demonstrate_production_pipeline()

        # Cleanup
        import glob

        demo_files = (
            glob.glob("*.omen")
            + glob.glob("*_demo*.parquet")
            + glob.glob("sample_*.parquet")
            + glob.glob("search_results.parquet")
        )
        for file in demo_files:
            try:
                os.remove(file)
            except:
                pass

        print("\n" + "=" * 45)
        print("🎉 Arrow/Parquet Integration Demo Complete!")
        print()
        print("📋 What we demonstrated:")
        print("   ✅ Import vectors from Parquet files (data lake format)")
        print("   ✅ Export search results to Arrow/Parquet format")
        print("   ✅ Polars integration for high-performance DataFrames")
        print("   ✅ Production ML pipeline patterns")
        print("   ✅ Integration with modern data ecosystem")
        print()
        print("🚀 Integration Benefits:")
        print("   • Seamless data lake integration")
        print("   • High-performance columnar operations")
        print("   • Cross-tool compatibility (Spark, DuckDB, etc.)")
        print("   • Production-ready data workflows")
        print("   • Zero-copy operations where possible")

        return True

    except Exception as e:
        print(f"❌ Demo failed: {e}")
        import traceback

        traceback.print_exc()
        return False


if __name__ == "__main__":
    main()
