#!/bin/bash
# Test coverage script for OmenDB

echo "Installing cargo-tarpaulin for coverage reporting..."
cargo install cargo-tarpaulin --locked

echo "Running tests with coverage..."
cargo tarpaulin --out Html --out Xml --output-dir target/coverage --exclude-files "*/tests/*" --exclude-files "*/bench*" --exclude-files "*/main.rs" --timeout 120

echo "Coverage report generated at target/coverage/tarpaulin-report.html"
echo "XML report at target/coverage/cobertura.xml"

# Extract coverage percentage
if [ -f target/coverage/cobertura.xml ]; then
    coverage=$(grep -o 'line-rate="[0-9.]*"' target/coverage/cobertura.xml | head -1 | grep -o '[0-9.]*')
    percentage=$(echo "$coverage * 100" | bc)
    echo ""
    echo "===================="
    echo "Code Coverage: ${percentage}%"
    echo "===================="

    # Fail if below threshold
    threshold=60
    if (( $(echo "$percentage < $threshold" | bc -l) )); then
        echo "❌ Coverage ${percentage}% is below threshold of ${threshold}%"
        exit 1
    else
        echo "✅ Coverage ${percentage}% meets threshold of ${threshold}%"
    fi
fi