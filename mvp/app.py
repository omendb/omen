"""
pgAnalytics - PostgreSQL Real-Time Analytics
MVP for YC Application (6-week sprint)
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import duckdb
import asyncpg
import asyncio
from typing import Dict, List, Any
import json
from datetime import datetime

app = FastAPI(title="pgAnalytics", version="0.1.0")

class DatabaseConnection(BaseModel):
    postgres_url: str
    analytics_name: str = "analytics"

class QueryRequest(BaseModel):
    sql: str
    timeout: int = 30

class AnalyticsEngine:
    def __init__(self):
        self.duckdb_conn = duckdb.connect(":memory:")
        self.pg_connections: Dict[str, asyncpg.Connection] = {}
        self.sync_tasks: Dict[str, asyncio.Task] = {}

    async def connect_postgres(self, name: str, url: str):
        """Connect to PostgreSQL and start CDC"""
        try:
            conn = await asyncpg.connect(url)
            self.pg_connections[name] = conn

            # Create replication slot
            await self.setup_replication(conn)

            # Start sync task
            task = asyncio.create_task(self.sync_data(name, conn))
            self.sync_tasks[name] = task

            return {"status": "connected", "database": name}
        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

    async def setup_replication(self, conn: asyncpg.Connection):
        """Setup logical replication for CDC"""
        # Check if logical replication is enabled
        wal_level = await conn.fetchval("SHOW wal_level")
        if wal_level != 'logical':
            raise ValueError("PostgreSQL wal_level must be 'logical' for CDC")

        # Create publication if not exists
        await conn.execute("""
            DO $$
            BEGIN
                IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'pganalytics_pub') THEN
                    CREATE PUBLICATION pganalytics_pub FOR ALL TABLES;
                END IF;
            END $$;
        """)

    async def sync_data(self, name: str, conn: asyncpg.Connection):
        """Sync data from PostgreSQL to DuckDB"""
        while True:
            try:
                # Get all tables
                tables = await conn.fetch("""
                    SELECT table_name
                    FROM information_schema.tables
                    WHERE table_schema = 'public'
                """)

                for table in tables:
                    table_name = table['table_name']

                    # Get table data
                    rows = await conn.fetch(f"SELECT * FROM {table_name}")

                    if rows:
                        # Convert to pandas-like format for DuckDB
                        columns = list(rows[0].keys())
                        data = [list(row.values()) for row in rows]

                        # Create table in DuckDB
                        self.create_duckdb_table(name, table_name, columns, data)

                # Sleep before next sync
                await asyncio.sleep(1)

            except Exception as e:
                print(f"Sync error: {e}")
                await asyncio.sleep(5)

    def create_duckdb_table(self, db_name: str, table_name: str, columns: List[str], data: List[List]):
        """Create or update table in DuckDB"""
        full_table_name = f"{db_name}_{table_name}"

        # Drop if exists and recreate (MVP simplicity)
        self.duckdb_conn.execute(f"DROP TABLE IF EXISTS {full_table_name}")

        # Create table with inferred types
        if data:
            # Use first row to infer types
            col_defs = []
            for i, col in enumerate(columns):
                val = data[0][i]
                if isinstance(val, int):
                    col_type = "BIGINT"
                elif isinstance(val, float):
                    col_type = "DOUBLE"
                elif isinstance(val, bool):
                    col_type = "BOOLEAN"
                elif isinstance(val, datetime):
                    col_type = "TIMESTAMP"
                else:
                    col_type = "VARCHAR"
                col_defs.append(f"{col} {col_type}")

            create_sql = f"CREATE TABLE {full_table_name} ({', '.join(col_defs)})"
            self.duckdb_conn.execute(create_sql)

            # Insert data
            placeholders = ', '.join(['?' for _ in columns])
            insert_sql = f"INSERT INTO {full_table_name} VALUES ({placeholders})"
            self.duckdb_conn.executemany(insert_sql, data)

    def execute_query(self, sql: str) -> List[Dict]:
        """Execute analytical query on DuckDB"""
        try:
            result = self.duckdb_conn.execute(sql).fetchall()
            columns = [desc[0] for desc in self.duckdb_conn.description]
            return [dict(zip(columns, row)) for row in result]
        except Exception as e:
            raise HTTPException(status_code=400, detail=str(e))

# Global engine instance
engine = AnalyticsEngine()

@app.post("/connect")
async def connect_database(conn: DatabaseConnection):
    """Connect to a PostgreSQL database"""
    result = await engine.connect_postgres(conn.analytics_name, conn.postgres_url)
    return result

@app.post("/query")
async def execute_query(query: QueryRequest):
    """Execute analytical query"""
    result = engine.execute_query(query.sql)
    return {"data": result, "rows": len(result)}

@app.get("/status")
async def get_status():
    """Get system status"""
    return {
        "connected_databases": list(engine.pg_connections.keys()),
        "active_syncs": len(engine.sync_tasks),
        "duckdb_tables": engine.duckdb_conn.execute("SHOW TABLES").fetchall()
    }

@app.get("/benchmark/{table_name}")
async def benchmark_performance(table_name: str):
    """Compare PostgreSQL vs DuckDB performance"""
    import time

    # Test query
    test_query = f"SELECT COUNT(*), AVG(id), MAX(id) FROM {table_name}"

    # DuckDB performance
    start = time.time()
    duckdb_result = engine.execute_query(test_query)
    duckdb_time = time.time() - start

    # PostgreSQL performance (if connected)
    pg_time = None
    if engine.pg_connections:
        conn = list(engine.pg_connections.values())[0]
        start = time.time()
        await conn.fetch(test_query)
        pg_time = time.time() - start

    speedup = pg_time / duckdb_time if pg_time else None

    return {
        "duckdb_time": f"{duckdb_time:.3f}s",
        "postgres_time": f"{pg_time:.3f}s" if pg_time else "N/A",
        "speedup": f"{speedup:.1f}x" if speedup else "N/A",
        "verdict": "DuckDB is faster!" if speedup and speedup > 1 else "Similar performance"
    }

@app.get("/")
async def root():
    return {
        "name": "pgAnalytics",
        "tagline": "Turn any PostgreSQL into a real-time analytics database",
        "version": "0.1.0 (YC MVP)",
        "docs": "/docs"
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)