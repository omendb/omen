#!/bin/bash

# Production Benchmark Execution Script
# ====================================
# 
# Comprehensive production hardening validation for OmenDB embedded mode.
# Executes all benchmark suites and generates consolidated reports.

set -e  # Exit on any error

echo "🚀 OmenDB Production Benchmark Suite"
echo "===================================="
echo "Comprehensive embedded mode production validation"
echo "Target scale: 10K vectors (development maximum)"
echo "Focus: Performance, reliability, competitive analysis"
echo ""

# Configuration
BENCHMARK_DIR="test/benchmarks"
RESULTS_DIR="/tmp/omendb_benchmark_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="${RESULTS_DIR}/production_benchmark_report_${TIMESTAMP}.txt"

# Create results directory
mkdir -p "${RESULTS_DIR}"

echo "📁 Results directory: ${RESULTS_DIR}"
echo "📄 Report file: ${REPORT_FILE}"
echo ""

# Initialize report
cat > "${REPORT_FILE}" << EOF
OmenDB Production Benchmark Report
=================================
Generated: $(date)
Target Scale: 10K vectors (development maximum)
Focus: Production hardening and competitive validation

EOF

echo "📊 PHASE 1: Vector Operations Performance"
echo "========================================="
echo "Testing: Vector creation, distance calculations, memory operations"
echo "Enhanced: 10K+ scale validation with production targets"
echo ""

echo "Running vector operations benchmark..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_vector_ops.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Vector operations benchmark: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Vector operations benchmark: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 2: Search Latency Performance"  
echo "======================================"
echo "Testing: Linear search latency, percentile analysis, production targets"
echo "Enhanced: Real embedding patterns, competitive comparison"
echo ""

echo "Running search latency benchmark..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_search_latency.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Search latency benchmark: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Search latency benchmark: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 3: Memory Usage and Leak Detection"
echo "==========================================="
echo "Testing: Memory allocation efficiency, leak detection, scaling patterns"
echo "Enhanced: 10K scale memory validation"
echo ""

echo "Running memory usage benchmark..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_memory_usage.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Memory usage benchmark: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Memory usage benchmark: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 4: Compression Performance"
echo "==================================="
echo "Testing: Binary quantization, compression ratios, accuracy loss"
echo "Enhanced: Production compression targets validation"
echo ""

echo "Running compression benchmark..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/benchmark_compression.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Compression benchmark: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Compression benchmark: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 5: Production Hardening Suite"
echo "======================================"
echo "Testing: 10K scale validation, adversarial inputs, memory pressure"
echo "Focus: Real-world patterns, MLOps integration, edge cases"
echo ""

echo "Running production hardening suite..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/production_hardening_suite.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Production hardening suite: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Production hardening suite: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 6: Competitive Analysis"
echo "================================"
echo "Testing: Industry benchmark comparison, competitive positioning"
echo "Focus: Faiss, Pinecone, Weaviate, Chroma, Qdrant comparison"
echo ""

echo "Running competitive analysis..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/competitive_analysis_production.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Competitive analysis: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Competitive analysis: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

echo "📊 PHASE 7: Performance Regression Suite"
echo "========================================"
echo "Testing: Comprehensive regression detection, baseline establishment"
echo "Focus: Unified performance monitoring and validation"
echo ""

echo "Running performance regression suite..." | tee -a "${REPORT_FILE}"
if pixi run mojo -I omendb ${BENCHMARK_DIR}/performance_regression_suite.mojo >> "${REPORT_FILE}" 2>&1; then
    echo "✅ Performance regression suite: PASSED" | tee -a "${REPORT_FILE}"
else
    echo "❌ Performance regression suite: FAILED" | tee -a "${REPORT_FILE}"
    echo "Check ${REPORT_FILE} for details"
fi
echo ""

# Generate final summary
echo "📋 BENCHMARK SUMMARY" | tee -a "${REPORT_FILE}"
echo "===================" | tee -a "${REPORT_FILE}"
echo "" | tee -a "${REPORT_FILE}"

# Count passed/failed benchmarks from report
TOTAL_BENCHMARKS=7
PASSED_COUNT=$(grep -c "PASSED" "${REPORT_FILE}" || echo "0")
FAILED_COUNT=$(grep -c "FAILED" "${REPORT_FILE}" || echo "0")

echo "📊 Benchmark Results:" | tee -a "${REPORT_FILE}"
echo "  Total benchmarks: ${TOTAL_BENCHMARKS}" | tee -a "${REPORT_FILE}"
echo "  Passed: ${PASSED_COUNT}" | tee -a "${REPORT_FILE}"
echo "  Failed: ${FAILED_COUNT}" | tee -a "${REPORT_FILE}"
echo "" | tee -a "${REPORT_FILE}"

# Calculate success rate
if [ "${TOTAL_BENCHMARKS}" -gt 0 ]; then
    SUCCESS_RATE=$((PASSED_COUNT * 100 / TOTAL_BENCHMARKS))
    echo "📈 Success Rate: ${SUCCESS_RATE}%" | tee -a "${REPORT_FILE}"
else
    SUCCESS_RATE=0
    echo "📈 Success Rate: 0%" | tee -a "${REPORT_FILE}"
fi
echo "" | tee -a "${REPORT_FILE}"

# Production readiness assessment
if [ "${SUCCESS_RATE}" -ge 85 ]; then
    echo "🎉 PRODUCTION READINESS: VALIDATED" | tee -a "${REPORT_FILE}"
    echo "✅ Embedded mode meets production hardening requirements" | tee -a "${REPORT_FILE}"
    echo "✅ Performance targets achieved at 10K vector scale" | tee -a "${REPORT_FILE}"
    echo "✅ Competitive positioning established" | tee -a "${REPORT_FILE}"
    echo "✅ Ready for real-world deployment validation" | tee -a "${REPORT_FILE}"
    OVERALL_STATUS="PRODUCTION_READY"
elif [ "${SUCCESS_RATE}" -ge 70 ]; then
    echo "⚠️  PRODUCTION READINESS: NEEDS IMPROVEMENT" | tee -a "${REPORT_FILE}"
    echo "📋 Most benchmarks passed but some issues remain" | tee -a "${REPORT_FILE}"
    echo "🔧 Address failing benchmarks before production deployment" | tee -a "${REPORT_FILE}"
    echo "📊 Performance generally acceptable with optimization needed" | tee -a "${REPORT_FILE}"
    OVERALL_STATUS="NEEDS_IMPROVEMENT"
else
    echo "❌ PRODUCTION READINESS: NOT READY" | tee -a "${REPORT_FILE}"
    echo "🚨 Significant performance or reliability issues detected" | tee -a "${REPORT_FILE}"
    echo "🔧 Major work required before production consideration" | tee -a "${REPORT_FILE}"
    echo "📋 Review and address all failing benchmarks" | tee -a "${REPORT_FILE}"
    OVERALL_STATUS="NOT_READY"
fi
echo "" | tee -a "${REPORT_FILE}"

echo "🎯 NEXT STEPS:" | tee -a "${REPORT_FILE}"
if [ "${OVERALL_STATUS}" = "PRODUCTION_READY" ]; then
    echo "  1. Proceed with real-world customer pilot testing" | tee -a "${REPORT_FILE}"
    echo "  2. Establish production monitoring and alerting" | tee -a "${REPORT_FILE}"
    echo "  3. Document performance characteristics for customers" | tee -a "${REPORT_FILE}"
    echo "  4. Begin competitive marketing positioning" | tee -a "${REPORT_FILE}"
elif [ "${OVERALL_STATUS}" = "NEEDS_IMPROVEMENT" ]; then
    echo "  1. Investigate and fix failing benchmark issues" | tee -a "${REPORT_FILE}"
    echo "  2. Re-run production benchmark suite" | tee -a "${REPORT_FILE}"
    echo "  3. Focus optimization on critical performance gaps" | tee -a "${REPORT_FILE}"
    echo "  4. Validate fixes with targeted testing" | tee -a "${REPORT_FILE}"
else
    echo "  1. Prioritize fixing critical performance issues" | tee -a "${REPORT_FILE}"
    echo "  2. Review system architecture for bottlenecks" | tee -a "${REPORT_FILE}"
    echo "  3. Implement comprehensive performance optimization" | tee -a "${REPORT_FILE}"
    echo "  4. Re-baseline all performance expectations" | tee -a "${REPORT_FILE}"
fi
echo "" | tee -a "${REPORT_FILE}"

echo "📁 Detailed Results: ${REPORT_FILE}" | tee -a "${REPORT_FILE}"
echo "⏰ Benchmark Completed: $(date)" | tee -a "${REPORT_FILE}"

echo ""
echo "🏁 PRODUCTION BENCHMARK SUITE COMPLETED"
echo "======================================"
echo "Status: ${OVERALL_STATUS}"
echo "Success Rate: ${SUCCESS_RATE}%"
echo "Detailed Report: ${REPORT_FILE}"
echo ""

# Exit with appropriate code
if [ "${SUCCESS_RATE}" -ge 85 ]; then
    echo "✅ Benchmark suite passed - ready for production validation"
    exit 0
elif [ "${SUCCESS_RATE}" -ge 70 ]; then
    echo "⚠️  Benchmark suite needs improvement - address issues before deployment"
    exit 1
else
    echo "❌ Benchmark suite failed - significant work required"
    exit 2
fi