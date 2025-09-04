"""
Apache Arrow & Parquet Integration for OmenDB

Provides efficient data interchange with the broader ML ecosystem:
- Import vectors from Parquet files (columnar storage)
- Export search results to Arrow/Parquet format
- Zero-copy operations where possible
- Integration with pandas, polars, duckdb

This enables OmenDB to work seamlessly with:
- Data lakes (Parquet files)
- Analytics databases (DuckDB, ClickHouse)
- ML pipelines (Ray, Dask)
- Data processing frameworks (Spark, Pandas)
"""

from typing import Optional, List, Dict, Union, Any
import os
from pathlib import Path

# Try to import Arrow/Parquet dependencies
try:
    import pyarrow as pa
    import pyarrow.parquet as pq
    import pyarrow.compute as pc

    ARROW_AVAILABLE = True
except ImportError:
    ARROW_AVAILABLE = False
    pa = None
    pq = None
    pc = None

# Optional polars integration
try:
    import polars as pl

    POLARS_AVAILABLE = True
except ImportError:
    POLARS_AVAILABLE = False
    pl = None

# Core imports
from .api import DB, SearchResult
from .exceptions import ValidationError, DatabaseError


class ArrowIntegration:
    """Apache Arrow integration for OmenDB."""

    def __init__(self, db: DB):
        """Initialize Arrow integration for a database.

        Args:
            db: OmenDB database instance
        """
        if not ARROW_AVAILABLE:
            raise ImportError(
                "Apache Arrow not available. Install with: pip install pyarrow"
            )

        self.db = db

    def import_from_parquet(
        self,
        file_path: Union[str, Path],
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]] = None,
        filters: Optional[List] = None,
        batch_size: Optional[int] = 10000,
    ) -> int:
        """Import vectors from Parquet file.

        Args:
            file_path: Path to Parquet file
            id_column: Column name containing vector IDs
            vector_column: Column name containing vector data (list of floats)
            metadata_columns: Optional list of columns to use as metadata
            filters: Optional Parquet filters for selective reading
            batch_size: Batch size for processing (None = load all at once)

        Returns:
            Number of vectors successfully imported
        """
        file_path = Path(file_path)
        if not file_path.exists():
            raise FileNotFoundError(f"Parquet file not found: {file_path}")

        try:
            # Read Parquet file with optional filtering
            table = pq.read_table(str(file_path), filters=filters)

            # Validate required columns
            schema = table.schema
            if id_column not in schema.names:
                raise ValidationError(
                    f"ID column '{id_column}' not found in Parquet file"
                )
            if vector_column not in schema.names:
                raise ValidationError(
                    f"Vector column '{vector_column}' not found in Parquet file"
                )

            # Process in batches for memory efficiency
            total_imported = 0

            if batch_size is None:
                # Process entire table at once
                total_imported = self._process_arrow_batch(
                    table, id_column, vector_column, metadata_columns
                )
            else:
                # Process in batches
                record_batches = table.to_batches(max_chunksize=batch_size)

                for batch in record_batches:
                    batch_table = pa.Table.from_batches([batch])
                    imported = self._process_arrow_batch(
                        batch_table, id_column, vector_column, metadata_columns
                    )
                    total_imported += imported

                    print(
                        f"   Imported batch: {imported} vectors (total: {total_imported})"
                    )

            return total_imported

        except Exception as e:
            raise DatabaseError(f"Failed to import from Parquet: {e}")

    def _process_arrow_batch(
        self,
        table: "pa.Table",
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]],
    ) -> int:
        """Process a single Arrow table batch."""

        # Extract IDs
        ids = table[id_column].to_pylist()

        # Extract vectors
        vectors = table[vector_column].to_pylist()

        # Extract metadata if specified
        metadata_list = []
        if metadata_columns:
            for i in range(len(ids)):
                metadata = {}
                for col in metadata_columns:
                    if col in table.schema.names:
                        value = table[col][i].as_py()
                        metadata[col] = str(value) if value is not None else ""
                metadata_list.append(metadata)
        else:
            metadata_list = [{}] * len(ids)

        # Prepare batch data
        batch_data = []
        for i, (vector_id, vector, metadata) in enumerate(
            zip(ids, vectors, metadata_list)
        ):
            if vector is None or len(vector) == 0:
                continue  # Skip empty vectors

            # Convert to proper format
            if not isinstance(vector, list):
                # Handle various vector formats (numpy arrays, etc.)
                if hasattr(vector, "tolist"):
                    vector = vector.tolist()
                else:
                    vector = list(vector)

            batch_data.append((str(vector_id), vector, metadata))

        # Add to database using batch operations
        if batch_data:
            results = self.db.add_batch(batch_data)
            return sum(results)

        return 0

    def export_to_parquet(
        self,
        file_path: Union[str, Path],
        query_vector: Optional[List[float]] = None,
        top_k: Optional[int] = None,
        where: Optional[Dict[str, str]] = None,
        include_vectors: bool = False,
        compression: str = "snappy",
    ) -> int:
        """Export search results to Parquet file.

        Args:
            file_path: Output Parquet file path
            query_vector: Vector to search for (if None, exports all vectors)
            top_k: Number of results to export
            where: Optional metadata filters
            include_vectors: Whether to include vector data in export
            compression: Parquet compression algorithm

        Returns:
            Number of records exported
        """
        try:
            if query_vector is not None:
                # Export search results
                results = self.db.query(
                    vector=query_vector, top_k=top_k or 100, where=where
                )

                # Convert to Arrow format
                data = {
                    "id": [r.id for r in results],
                    "similarity": [r.similarity for r in results],
                }

                # Add vector data if requested
                if include_vectors:
                    vectors = []
                    for result in results:
                        if result.id:
                            vector_data = self.db.get(result.id)
                            if vector_data:
                                vectors.append(
                                    vector_data[0]
                                )  # Get vector, not metadata
                            else:
                                vectors.append([])
                        else:
                            vectors.append([])
                    data["vector"] = vectors

                # Add metadata columns
                if results and results[0].metadata:
                    # Get all unique metadata keys
                    all_keys = set()
                    for result in results:
                        if result.metadata:
                            all_keys.update(result.metadata.keys())

                    # Add metadata columns
                    for key in all_keys:
                        data[f"metadata_{key}"] = [
                            result.metadata.get(key, "") if result.metadata else ""
                            for result in results
                        ]

            else:
                # Export all vectors (not implemented - would require iterator)
                raise NotImplementedError("Exporting all vectors not yet implemented")

            # Create Arrow table
            table = pa.table(data)

            # Write to Parquet
            pq.write_table(table, str(file_path), compression=compression)

            return len(results) if query_vector is not None else 0

        except Exception as e:
            raise DatabaseError(f"Failed to export to Parquet: {e}")

    def import_from_arrow_table(
        self,
        table: "pa.Table",
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]] = None,
    ) -> int:
        """Import vectors from Arrow table.

        Args:
            table: PyArrow Table containing vector data
            id_column: Column name containing vector IDs
            vector_column: Column name containing vector data
            metadata_columns: Optional list of columns to use as metadata

        Returns:
            Number of vectors successfully imported
        """
        return self._process_arrow_batch(
            table, id_column, vector_column, metadata_columns
        )

    def export_to_arrow_table(
        self,
        query_vector: Optional[List[float]] = None,
        top_k: Optional[int] = None,
        where: Optional[Dict[str, str]] = None,
        include_vectors: bool = False,
    ) -> "pa.Table":
        """Export search results to Arrow table.

        Args:
            query_vector: Vector to search for (if None, exports all vectors)
            top_k: Number of results to export
            where: Optional metadata filters
            include_vectors: Whether to include vector data in export

        Returns:
            PyArrow Table with search results
        """
        if query_vector is not None:
            results = self.db.query(
                vector=query_vector, top_k=top_k or 100, where=where
            )

            # Convert to Arrow format
            data = {
                "id": [r.id for r in results],
                "similarity": [r.similarity for r in results],
            }

            if include_vectors:
                vectors = []
                for result in results:
                    if result.id:
                        vector_data = self.db.get(result.id)
                        if vector_data:
                            vectors.append(vector_data[0])
                        else:
                            vectors.append([])
                    else:
                        vectors.append([])
                data["vector"] = vectors

            return pa.table(data)
        else:
            raise NotImplementedError("Exporting all vectors not yet implemented")


class PolarsIntegration:
    """Polars integration for OmenDB (high-performance DataFrames)."""

    def __init__(self, db: DB):
        """Initialize Polars integration.

        Args:
            db: OmenDB database instance
        """
        if not POLARS_AVAILABLE:
            raise ImportError("Polars not available. Install with: pip install polars")

        self.db = db

    def import_from_dataframe(
        self,
        df: "pl.DataFrame",
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]] = None,
    ) -> int:
        """Import vectors from Polars DataFrame.

        Args:
            df: Polars DataFrame containing vector data
            id_column: Column name containing vector IDs
            vector_column: Column name containing vector data
            metadata_columns: Optional list of columns to use as metadata

        Returns:
            Number of vectors successfully imported
        """
        # Convert to Arrow table and use Arrow integration
        arrow_table = df.to_arrow()
        arrow_integration = ArrowIntegration(self.db)
        return arrow_integration.import_from_arrow_table(
            arrow_table, id_column, vector_column, metadata_columns
        )

    def export_to_dataframe(
        self,
        query_vector: Optional[List[float]] = None,
        top_k: Optional[int] = None,
        where: Optional[Dict[str, str]] = None,
        include_vectors: bool = False,
    ) -> "pl.DataFrame":
        """Export search results to Polars DataFrame.

        Args:
            query_vector: Vector to search for
            top_k: Number of results to export
            where: Optional metadata filters
            include_vectors: Whether to include vector data

        Returns:
            Polars DataFrame with search results
        """
        arrow_integration = ArrowIntegration(self.db)
        arrow_table = arrow_integration.export_to_arrow_table(
            query_vector, top_k, where, include_vectors
        )
        return pl.from_arrow(arrow_table)


# Convenience functions for the main DB class
def add_arrow_methods_to_db():
    """Add Arrow/Parquet methods to the main DB class."""

    def import_parquet(
        self,
        file_path: Union[str, Path],
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]] = None,
        filters: Optional[List] = None,
        batch_size: Optional[int] = 10000,
    ) -> int:
        """Import vectors from Parquet file.

        This is a convenience method that creates an ArrowIntegration instance
        and calls import_from_parquet.
        """
        if not ARROW_AVAILABLE:
            raise ImportError(
                "Apache Arrow not available. Install with: pip install pyarrow"
            )

        integration = ArrowIntegration(self)
        return integration.import_from_parquet(
            file_path, id_column, vector_column, metadata_columns, filters, batch_size
        )

    def export_parquet(
        self,
        file_path: Union[str, Path],
        query_vector: Optional[List[float]] = None,
        top_k: Optional[int] = None,
        where: Optional[Dict[str, str]] = None,
        include_vectors: bool = False,
        compression: str = "snappy",
    ) -> int:
        """Export search results to Parquet file.

        This is a convenience method that creates an ArrowIntegration instance
        and calls export_to_parquet.
        """
        if not ARROW_AVAILABLE:
            raise ImportError(
                "Apache Arrow not available. Install with: pip install pyarrow"
            )

        integration = ArrowIntegration(self)
        return integration.export_to_parquet(
            file_path, query_vector, top_k, where, include_vectors, compression
        )

    def import_polars(
        self,
        df: "pl.DataFrame",
        id_column: str,
        vector_column: str,
        metadata_columns: Optional[List[str]] = None,
    ) -> int:
        """Import vectors from Polars DataFrame."""
        if not POLARS_AVAILABLE:
            raise ImportError("Polars not available. Install with: pip install polars")

        integration = PolarsIntegration(self)
        return integration.import_from_dataframe(
            df, id_column, vector_column, metadata_columns
        )

    def export_polars(
        self,
        query_vector: Optional[List[float]] = None,
        top_k: Optional[int] = None,
        where: Optional[Dict[str, str]] = None,
        include_vectors: bool = False,
    ) -> "pl.DataFrame":
        """Export search results to Polars DataFrame."""
        if not POLARS_AVAILABLE:
            raise ImportError("Polars not available. Install with: pip install polars")

        integration = PolarsIntegration(self)
        return integration.export_to_dataframe(
            query_vector, top_k, where, include_vectors
        )

    # Add methods to DB class
    DB.import_parquet = import_parquet
    DB.export_parquet = export_parquet
    DB.import_polars = import_polars
    DB.export_polars = export_polars


# Auto-register methods if Arrow is available
if ARROW_AVAILABLE:
    add_arrow_methods_to_db()
