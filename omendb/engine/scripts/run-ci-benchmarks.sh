#!/bin/bash

# CI/CD Performance Regression Detection Script
# ============================================
# 
# Fast performance regression detection optimized for continuous integration.
# Executes in <5 minutes with focused testing on critical performance metrics.

set -e  # Exit on any error

echo "⚡ OmenDB CI/CD Performance Regression Detection"
echo "=============================================="
echo "Fast execution optimized for continuous integration"
echo "Target execution time: <5 minutes"
echo "Focus: Critical performance regression detection"
echo ""

# Configuration
BENCHMARK_DIR="test/benchmarks"
CI_RESULTS_DIR="/tmp/omendb_ci_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
CI_REPORT_FILE="${CI_RESULTS_DIR}/ci_performance_report_${TIMESTAMP}.txt"

# Create results directory
mkdir -p "${CI_RESULTS_DIR}"

echo "📁 CI Results directory: ${CI_RESULTS_DIR}"
echo "📄 CI Report file: ${CI_REPORT_FILE}"
echo ""

# Initialize CI report
cat > "${CI_REPORT_FILE}" << EOF
OmenDB CI/CD Performance Regression Report
==========================================
Generated: $(date)
Mode: Fast CI execution (<5 minutes)
Focus: Critical performance regression detection

EOF

# Start timing
CI_START_TIME=$(date +%s)

echo "⚡ PHASE 1: CI Performance Regression Suite"
echo "==========================================="
echo "Testing: Core performance metrics with regression detection"
echo "Scale: 1K vectors (CI optimized)"
echo "Metrics: Search latency, memory usage, throughput"
echo ""

echo "Running CI regression detection..." | tee -a "${CI_REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/ci_cd_regression_suite.mojo >> "${CI_REPORT_FILE}" 2>&1; then
    echo "✅ CI regression detection: PASSED" | tee -a "${CI_REPORT_FILE}"
    CI_REGRESSION_STATUS="PASSED"
else
    echo "❌ CI regression detection: FAILED" | tee -a "${CI_REPORT_FILE}"
    CI_REGRESSION_STATUS="FAILED"
fi
echo ""

echo "⚡ PHASE 2: Quick Vector Operations Check"
echo "========================================"
echo "Testing: Essential vector operations for basic functionality"
echo "Focus: Smoke test for critical path operations"
echo ""

# Quick smoke test of vector operations (reduced scale)
echo "Running quick vector operations check..." | tee -a "${CI_REPORT_FILE}"
if timeout 120 pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_vector_ops.mojo >> "${CI_REPORT_FILE}" 2>&1; then
    echo "✅ Quick vector operations: PASSED" | tee -a "${CI_REPORT_FILE}"
    VECTOR_OPS_STATUS="PASSED"
else
    echo "❌ Quick vector operations: FAILED (or timed out)" | tee -a "${CI_REPORT_FILE}"
    VECTOR_OPS_STATUS="FAILED"
fi
echo ""

echo "⚡ PHASE 3: Essential Search Validation"
echo "======================================"
echo "Testing: Basic search functionality and latency"
echo "Focus: Ensuring search operations work correctly"
echo ""

# Quick search validation (reduced scale)
echo "Running essential search validation..." | tee -a "${CI_REPORT_FILE}"
if timeout 120 pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_search_latency.mojo >> "${CI_REPORT_FILE}" 2>&1; then
    echo "✅ Essential search validation: PASSED" | tee -a "${CI_REPORT_FILE}"
    SEARCH_STATUS="PASSED"
else
    echo "❌ Essential search validation: FAILED (or timed out)" | tee -a "${CI_REPORT_FILE}"
    SEARCH_STATUS="FAILED"
fi
echo ""

# Calculate execution time
CI_END_TIME=$(date +%s)
CI_EXECUTION_TIME=$((CI_END_TIME - CI_START_TIME))

# Generate CI summary
echo "📋 CI PERFORMANCE SUMMARY" | tee -a "${CI_REPORT_FILE}"
echo "=========================" | tee -a "${CI_REPORT_FILE}"
echo "" | tee -a "${CI_REPORT_FILE}"

echo "⏱️  Execution Time: ${CI_EXECUTION_TIME} seconds" | tee -a "${CI_REPORT_FILE}"
echo "" | tee -a "${CI_REPORT_FILE}"

echo "📊 CI Test Results:" | tee -a "${CI_REPORT_FILE}"
echo "  Regression Detection: ${CI_REGRESSION_STATUS}" | tee -a "${CI_REPORT_FILE}"
echo "  Vector Operations: ${VECTOR_OPS_STATUS}" | tee -a "${CI_REPORT_FILE}"
echo "  Search Validation: ${SEARCH_STATUS}" | tee -a "${CI_REPORT_FILE}"
echo "" | tee -a "${CI_REPORT_FILE}"

# Count passed tests
PASSED_TESTS=0
if [ "${CI_REGRESSION_STATUS}" = "PASSED" ]; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
fi
if [ "${VECTOR_OPS_STATUS}" = "PASSED" ]; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
fi
if [ "${SEARCH_STATUS}" = "PASSED" ]; then
    PASSED_TESTS=$((PASSED_TESTS + 1))
fi

TOTAL_CI_TESTS=3
CI_SUCCESS_RATE=$((PASSED_TESTS * 100 / TOTAL_CI_TESTS))

echo "📈 CI Success Rate: ${CI_SUCCESS_RATE}% (${PASSED_TESTS}/${TOTAL_CI_TESTS})" | tee -a "${CI_REPORT_FILE}"
echo "" | tee -a "${CI_REPORT_FILE}"

# CI decision logic
if [ "${CI_SUCCESS_RATE}" -ge 100 ]; then
    echo "✅ CI STATUS: ALL TESTS PASSED" | tee -a "${CI_REPORT_FILE}"
    echo "🚀 Build ready for deployment" | tee -a "${CI_REPORT_FILE}"
    echo "📊 No performance regressions detected" | tee -a "${CI_REPORT_FILE}"
    echo "🎯 All critical functionality working" | tee -a "${CI_REPORT_FILE}"
    CI_FINAL_STATUS="SUCCESS"
elif [ "${CI_SUCCESS_RATE}" -ge 67 ]; then
    echo "⚠️  CI STATUS: PARTIAL SUCCESS" | tee -a "${CI_REPORT_FILE}"
    echo "🔍 Some tests failed - investigate before deployment" | tee -a "${CI_REPORT_FILE}"
    echo "📋 Check individual test results for specific issues" | tee -a "${CI_REPORT_FILE}"
    if [ "${CI_REGRESSION_STATUS}" = "FAILED" ]; then
        echo "🚨 Performance regression detected - review recent changes" | tee -a "${CI_REPORT_FILE}"
    fi
    CI_FINAL_STATUS="WARNING"
else
    echo "❌ CI STATUS: TESTS FAILED" | tee -a "${CI_REPORT_FILE}"
    echo "🚨 Build not ready for deployment" | tee -a "${CI_REPORT_FILE}"
    echo "🔧 Fix failing tests before proceeding" | tee -a "${CI_REPORT_FILE}"
    if [ "${CI_REGRESSION_STATUS}" = "FAILED" ]; then
        echo "📉 Critical performance regression - immediate attention required" | tee -a "${CI_REPORT_FILE}"
    fi
    CI_FINAL_STATUS="FAILURE"
fi
echo "" | tee -a "${CI_REPORT_FILE}"

# Execution time validation
if [ "${CI_EXECUTION_TIME}" -le 300 ]; then  # 5 minutes
    echo "⏱️  CI Execution Time: OPTIMAL (${CI_EXECUTION_TIME}s ≤ 300s)" | tee -a "${CI_REPORT_FILE}"
elif [ "${CI_EXECUTION_TIME}" -le 600 ]; then  # 10 minutes
    echo "⏱️  CI Execution Time: ACCEPTABLE (${CI_EXECUTION_TIME}s ≤ 600s)" | tee -a "${CI_REPORT_FILE}"
else
    echo "⏱️  CI Execution Time: TOO SLOW (${CI_EXECUTION_TIME}s > 600s)" | tee -a "${CI_REPORT_FILE}"
    echo "🔧 Optimize CI tests for faster feedback" | tee -a "${CI_REPORT_FILE}"
fi
echo "" | tee -a "${CI_REPORT_FILE}"

echo "📁 Detailed CI Report: ${CI_REPORT_FILE}" | tee -a "${CI_REPORT_FILE}"
echo "⏰ CI Completed: $(date)" | tee -a "${CI_REPORT_FILE}"

echo ""
echo "🏁 CI/CD PERFORMANCE REGRESSION DETECTION COMPLETED"
echo "=================================================="
echo "Status: ${CI_FINAL_STATUS}"
echo "Success Rate: ${CI_SUCCESS_RATE}%"
echo "Execution Time: ${CI_EXECUTION_TIME} seconds"
echo "Detailed Report: ${CI_REPORT_FILE}"
echo ""

# CI-specific recommendations
if [ "${CI_FINAL_STATUS}" = "SUCCESS" ]; then
    echo "✅ CI passed - safe to merge/deploy"
    echo "📊 Performance within expected bounds"
    echo "🚀 No regressions detected"
elif [ "${CI_FINAL_STATUS}" = "WARNING" ]; then
    echo "⚠️  CI warning - review before merge/deploy"
    echo "🔍 Some performance concerns detected"
    echo "📋 Manual review recommended"
else
    echo "❌ CI failed - do not merge/deploy"
    echo "🚨 Performance regressions or critical failures"
    echo "🔧 Fix issues before proceeding"
fi

echo ""

# Exit with appropriate CI status codes
if [ "${CI_FINAL_STATUS}" = "SUCCESS" ]; then
    exit 0  # Success
elif [ "${CI_FINAL_STATUS}" = "WARNING" ]; then
    exit 1  # Warning - manual review needed
else
    exit 2  # Failure - do not proceed
fi