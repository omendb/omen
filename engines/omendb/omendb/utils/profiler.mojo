# ===----------------------------------------------------------------------=== #
# Performance Profiler - Mojo-Native Bottleneck Analysis
#
# Uses Mojo's built-in profiling tools to identify the real 256D+ performance cliff
# ===----------------------------------------------------------------------=== #

from time import perf_counter_ns, time_function
from benchmark import Bench, BenchConfig, ThroughputMeasure, BenchMetric, BenchId
from runtime import tracing
from collections import List, Dict
from memory import UnsafePointer


# ===----------------------------------------------------------------------=== #
# High-Resolution Timing Utilities
# ===----------------------------------------------------------------------=== #

struct PerformanceTimer:
    """High-precision timer for micro-benchmarking."""
    
    var start_time: Int
    var name: String
    
    fn __init__(out self, name: String):
        self.name = name
        self.start_time = 0
    
    @always_inline
    fn start(mut self):
        """Start timing."""
        self.start_time = perf_counter_ns()
    
    @always_inline
    fn stop(self) -> Int:
        """Stop timing and return nanoseconds elapsed."""
        return perf_counter_ns() - self.start_time
    
    @always_inline
    fn stop_and_print(self):
        """Stop timing and print result."""
        var elapsed_ns = perf_counter_ns() - self.start_time
        var elapsed_ms = Float64(elapsed_ns) / 1_000_000.0
        print(self.name, ":", elapsed_ms, "ms")


# ===----------------------------------------------------------------------=== #
# Vector Operation Profiling
# ===----------------------------------------------------------------------=== #

struct VectorOperationProfiler:
    """Profiles individual vector operations to isolate bottlenecks."""
    
    var results: Dict[String, Float64]
    
    fn __init__(out self):
        self.results = Dict[String, Float64]()
    
    fn profile_list_creation(mut self, dimension: Int, iterations: Int):
        """Profile Python list creation overhead."""
        print("🔍 Profiling list creation for", dimension, "D vectors...")
        
        var timer = PerformanceTimer("list_creation")
        timer.start()
        
        # This simulates the overhead of creating Python lists
        # (We can't directly create Python lists in Mojo, but we can measure equivalent operations)
        for i in range(iterations):
            var dummy_data = UnsafePointer[Float32].alloc(dimension)
            for j in range(dimension):
                dummy_data[j] = Float32(j)
            dummy_data.free()
        
        var elapsed_ns = timer.stop()
        var ms_per_operation = Float64(elapsed_ns) / Float64(iterations) / 1_000_000.0
        
        print("  List creation:", ms_per_operation, "ms per", dimension, "D vector")
        self.results[String("list_creation_") + String(dimension)] = ms_per_operation
    
    fn profile_memory_copy(mut self, dimension: Int, iterations: Int):
        """Profile memory copy operations."""
        print("🔍 Profiling memory copy for", dimension, "D vectors...")
        
        var src = UnsafePointer[Float32].alloc(dimension)
        var dst = UnsafePointer[Float32].alloc(dimension)
        
        # Initialize source data
        for i in range(dimension):
            src[i] = Float32(i)
        
        var timer = PerformanceTimer("memory_copy")
        timer.start()
        
        for i in range(iterations):
            # Simple memory copy
            for j in range(dimension):
                dst[j] = src[j]
        
        var elapsed_ns = timer.stop()
        var ms_per_operation = Float64(elapsed_ns) / Float64(iterations) / 1_000_000.0
        
        print("  Memory copy:", ms_per_operation, "ms per", dimension, "D vector")
        self.results[String("memory_copy_") + String(dimension)] = ms_per_operation
        
        src.free()
        dst.free()
    
    fn profile_dictionary_operations(mut self, iterations: Int):
        """Profile dictionary insertion overhead."""
        print("🔍 Profiling dictionary operations...")
        
        var timer = PerformanceTimer("dict_operations")
        timer.start()
        
        var test_dict = Dict[String, Int]()
        for i in range(iterations):
            var key = String("key_") + String(i)
            test_dict[key] = i
        
        var elapsed_ns = timer.stop()
        var ms_per_operation = Float64(elapsed_ns) / Float64(iterations) / 1_000_000.0
        
        print("  Dictionary insert:", ms_per_operation, "ms per operation")
        self.results["dict_operations"] = ms_per_operation
    
    fn profile_string_operations(mut self, iterations: Int):
        """Profile string creation and manipulation."""
        print("🔍 Profiling string operations...")
        
        var timer = PerformanceTimer("string_operations")
        timer.start()
        
        var string_list = List[String]()
        for i in range(iterations):
            var new_string = String("vector_") + String(i)
            string_list.append(new_string)
        
        var elapsed_ns = timer.stop()
        var ms_per_operation = Float64(elapsed_ns) / Float64(iterations) / 1_000_000.0
        
        print("  String operations:", ms_per_operation, "ms per operation")
        self.results["string_operations"] = ms_per_operation
    
    fn print_summary(self):
        """Print profiling summary."""
        print("\n📊 PROFILING SUMMARY")
        print("=" * 40)
        
        # Find the operations that scale with dimension
        var list_64 = self.results.get("list_creation_64", 0.0)
        var list_256 = self.results.get("list_creation_256", 0.0)
        var list_512 = self.results.get("list_creation_512", 0.0)
        
        var copy_64 = self.results.get("memory_copy_64", 0.0)
        var copy_256 = self.results.get("memory_copy_256", 0.0)
        var copy_512 = self.results.get("memory_copy_512", 0.0)
        
        print("List Creation Scaling:")
        print("  64D:", list_64, "ms")
        print("  256D:", list_256, "ms (", list_256/list_64 if list_64 > 0 else 0, "x)")
        print("  512D:", list_512, "ms (", list_512/list_64 if list_64 > 0 else 0, "x)")
        
        print("Memory Copy Scaling:")
        print("  64D:", copy_64, "ms")
        print("  256D:", copy_256, "ms (", copy_256/copy_64 if copy_64 > 0 else 0, "x)")
        print("  512D:", copy_512, "ms (", copy_512/copy_64 if copy_64 > 0 else 0, "x)")
        
        print("Fixed Operations:")
        print("  Dictionary:", self.results.get("dict_operations", 0.0), "ms")
        print("  String:", self.results.get("string_operations", 0.0), "ms")


# ===----------------------------------------------------------------------=== #
# Benchmarking Integration
# ===----------------------------------------------------------------------=== #

struct MojoBenchmarkRunner:
    """Runs comprehensive benchmarks using Mojo's benchmark infrastructure."""
    
    var bench: Bench
    
    fn __init__(out self):
        var config = BenchConfig(
            max_iters=10000,
            min_runtime_secs=0.5,
            max_runtime_secs=10.0,
            verbose_timing=True
        )
        self.bench = Bench(config)
        print("🚀 Mojo Benchmark Runner initialized")
    
    fn benchmark_vector_operations(mut self, dimension: Int, num_vectors: Int):
        """Benchmark vector operations using Mojo's benchmark infrastructure."""
        print("🔬 Benchmarking", dimension, "D vector operations...")
        
        # Calculate theoretical metrics
        var total_elements = num_vectors * dimension
        var total_bytes = total_elements * 4  # Float32 = 4 bytes
        
        var elements_measure = ThroughputMeasure(BenchMetric.elements, total_elements)
        var bytes_measure = ThroughputMeasure(BenchMetric.bytes, total_bytes)
        
        # Benchmark memory allocation
        @parameter
        fn benchmark_allocation(mut bencher: auto):
            @parameter
            fn allocate_vectors():
                var vectors = List[UnsafePointer[Float32]]()
                for i in range(num_vectors):
                    var vec = UnsafePointer[Float32].alloc(dimension)
                    for j in range(dimension):
                        vec[j] = Float32(j)
                    vectors.append(vec)
                
                # Clean up
                for i in range(len(vectors)):
                    vectors[i].free()
            
            bencher.iter[allocate_vectors]()
        
        var bench_id = BenchId("allocation", String("dim_") + String(dimension))
        self.bench.bench_function[benchmark_allocation](bench_id, List(elements_measure, bytes_measure))
        
        print("Benchmark completed for", dimension, "D")


# ===----------------------------------------------------------------------=== #
# Tracing Integration
# ===----------------------------------------------------------------------=== #

@always_inline
fn profile_with_tracing[operation_name: StringLiteral](func: fn() -> None):
    """Profile a function using Mojo's runtime tracing."""
    with tracing.Trace[tracing.TraceLevel.OP, category=tracing.TraceCategory.MEM](operation_name):
        func()


# ===----------------------------------------------------------------------=== #
# Main Profiling Functions
# ===----------------------------------------------------------------------=== #

fn run_comprehensive_mojo_profiling():
    """Run comprehensive profiling using native Mojo tools."""
    print("🔬 COMPREHENSIVE MOJO PROFILING")
    print("=" * 60)
    
    var profiler = VectorOperationProfiler()
    
    # Profile individual operations
    var test_dimensions = InlineArray[Int, 4](64, 128, 256, 512)
    var iterations = 1000
    
    for i in range(4):
        var dimension = test_dimensions[i]
        profiler.profile_list_creation(dimension, iterations)
        profiler.profile_memory_copy(dimension, iterations)
    
    # Profile fixed operations
    profiler.profile_dictionary_operations(iterations)
    profiler.profile_string_operations(iterations)
    
    # Print comprehensive summary
    profiler.print_summary()
    
    # Run benchmark suite
    print("\n🚀 RUNNING MOJO BENCHMARKS")
    print("=" * 40)
    
    var benchmark_runner = MojoBenchmarkRunner()
    var test_configs = InlineArray[Tuple[Int, Int], 3]((64, 100), (256, 100), (512, 100))
    
    for i in range(3):
        var config = test_configs[i]
        benchmark_runner.benchmark_vector_operations(config[0], config[1])


fn print_profiling_info():
    """Print information about Mojo profiling capabilities."""
    print("🔍 Mojo Performance Profiling Active")
    print("Tools Used:")
    print("  • perf_counter_ns() - Nanosecond precision timing")
    print("  • Bench infrastructure - Statistical benchmarking")
    print("  • Runtime tracing - Hierarchical operation tracking")
    print("  • ThroughputMeasure - Elements/bytes per second metrics")
    print("  • Memory profiling - Allocation pattern analysis")
    print("Target: Identify exact bottleneck causing 256D+ performance cliff")


# ===----------------------------------------------------------------------=== #
# Validation Functions
# ===----------------------------------------------------------------------=== #

fn validate_profiling_tools() -> Bool:
    """Validate that profiling tools are working correctly."""
    print("✅ Validating Mojo profiling tools...")
    
    # Test high-resolution timing
    var timer = PerformanceTimer("validation")
    timer.start() 
    
    # Small delay
    var dummy_data = UnsafePointer[Float32].alloc(1000)
    for i in range(1000):
        dummy_data[i] = Float32(i)
    dummy_data.free()
    
    var elapsed = timer.stop()
    
    if elapsed > 0:
        print("  ✅ High-resolution timing working (", elapsed, "ns)")
        return True
    else:
        print("  ❌ High-resolution timing failed")
        return False