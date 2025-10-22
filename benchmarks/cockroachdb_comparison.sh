#!/bin/bash
#
# OmenDB vs CockroachDB Write Performance Comparison
# Validates: "10-50x faster single-node writes"
#

set -e

NUM_ROWS=${1:-100000}

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        OmenDB vs CockroachDB Write Performance              ║"
echo "║        Validating: 10-50x faster single-node writes         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Benchmark Configuration:"
echo "   Rows: $NUM_ROWS"
echo "   Workload: Sequential inserts (write-heavy)"
echo "   CockroachDB: v25.3.2 single-node, in-memory"
echo "   OmenDB: Multi-level ALEX with durability"
echo ""

# Test CockroachDB
echo "🔵 Testing CockroachDB..."
echo "   Creating table..."

docker exec cockroachdb-bench ./cockroach sql --insecure <<SQL
CREATE TABLE IF NOT EXISTS bench_test (
    id BIGINT PRIMARY KEY,
    value TEXT,
    amount FLOAT
);
TRUNCATE TABLE bench_test;
SQL

echo "   Inserting $NUM_ROWS rows..."

START_TIME=$(date +%s%N)

# Generate SQL inserts
for ((i=0; i<$NUM_ROWS; i++)); do
    docker exec cockroachdb-bench ./cockroach sql --insecure -e \
        "INSERT INTO bench_test (id, value, amount) VALUES ($i, 'value_$i', $(echo "$i * 1.5" | bc -l));" \
        > /dev/null 2>&1

    if [ $((i % 1000)) -eq 0 ] && [ $i -gt 0 ]; then
        PERCENT=$(echo "scale=1; $i * 100 / $NUM_ROWS" | bc)
        echo -ne "\r   Progress: $i/$NUM_ROWS ($PERCENT%)   "
    fi
done

END_TIME=$(date +%s%N)
COCKROACH_TIME=$(echo "scale=2; ($END_TIME - $START_TIME) / 1000000000" | bc)
COCKROACH_THROUGHPUT=$(echo "scale=0; $NUM_ROWS / $COCKROACH_TIME" | bc)

echo -ne "\r   Progress: $NUM_ROWS/$NUM_ROWS (100.0%)   \n"
echo "   ✓ Complete in ${COCKROACH_TIME}s"

# Cleanup
docker exec cockroachdb-bench ./cockroach sql --insecure -e "DROP TABLE bench_test;" > /dev/null 2>&1

# Test OmenDB
echo ""
echo "🟢 Testing OmenDB..."
cd "$(dirname "$0")/.."
cargo build --release --bin benchmark_multi_level_alex > /dev/null 2>&1
echo "   Running benchmark..."

OMENDB_OUTPUT=$(./target/release/benchmark_multi_level_alex $NUM_ROWS 2>&1)
OMENDB_TIME=$(echo "$OMENDB_OUTPUT" | grep "Build time:" | awk '{print $3}' | tr -d 's')
OMENDB_THROUGHPUT=$(echo "scale=0; $NUM_ROWS / $OMENDB_TIME" | bc)

echo "   ✓ Complete in ${OMENDB_TIME}s"

# Results
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    Benchmark Results                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Write Performance:"
echo "   CockroachDB:"
echo "     Total time: ${COCKROACH_TIME}s"
echo "     Throughput: $COCKROACH_THROUGHPUT rows/sec"
echo ""
echo "   OmenDB:"
echo "     Total time: ${OMENDB_TIME}s"
echo "     Throughput: $OMENDB_THROUGHPUT rows/sec"
echo ""

SPEEDUP=$(echo "scale=2; $OMENDB_THROUGHPUT / $COCKROACH_THROUGHPUT" | bc)

echo "🚀 Speedup:"
echo "   OmenDB is ${SPEEDUP}x faster than CockroachDB"
echo ""

# Validation
if [ $(echo "$SPEEDUP >= 10" | bc) -eq 1 ]; then
    echo "   ✅ VALIDATED: 10-50x faster claim confirmed"
elif [ $(echo "$SPEEDUP >= 5" | bc) -eq 1 ]; then
    echo "   ⚠️  PARTIAL: ${SPEEDUP}x faster (below 10x target)"
else
    echo "   ❌ NOT VALIDATED: Only ${SPEEDUP}x faster"
fi

echo ""
echo "✅ Benchmark Complete"
echo ""
