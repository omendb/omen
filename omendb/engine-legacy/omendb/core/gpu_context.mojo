"""CPU-only stubs that keep legacy GPU call sites compiling.

Mojo does not currently expose CUDA/Metal bindings, so the previous version of
this module only simulated GPU behaviour. We keep the API surface but the
implementation always falls back to CPU memory and math. Callers in archived
code paths can continue to run without promising fictional acceleration.
"""

from math import sqrt
from memory import UnsafePointer
from collections import List

struct GPUContext:
    """Represents the (non-existent) GPU backend.

    The context always reports CPU mode so callers can branch safely.
    """
    var backend: String
    var device_id: Int
    var memory_limit: Int
    var is_available: Bool

    fn __init__(out self, device_id: Int = 0, memory_limit: Int = -1):
        self.backend = "cpu"
        self.device_id = device_id
        self.memory_limit = memory_limit
        self.is_available = False

    fn get_memory_info(self) -> Tuple[Int, Int]:
        return (0, 0)

    fn synchronize(self):
        pass

struct GPUTensor[dtype: DType]:
    """CPU-backed tensor that mimics a GPU allocation."""
    var data: UnsafePointer[Scalar[dtype]]
    var shape: List[Int]
    var gpu_context: GPUContext

    fn __init__(out self, shape: List[Int], gpu_context: GPUContext):
        self.shape = shape
        self.gpu_context = gpu_context
        self.data = UnsafePointer[Scalar[dtype]].alloc(self._total_elements())

    fn __del__(owned self):
        self.data.free()

    fn _total_elements(self) -> Int:
        var total = 1
        for i in range(len(self.shape)):
            total *= self.shape[i]
        return total

    fn copy_to_device(inout self, host_data: UnsafePointer[Scalar[dtype]]):
        let count = self._total_elements()
        for i in range(count):
            self.data[i] = host_data[i]

    fn copy_to_host(self, host_data: UnsafePointer[Scalar[dtype]]):
        let count = self._total_elements()
        for i in range(count):
            host_data[i] = self.data[i]

fn gpu_batch_distance_cosine(
    query: GPUTensor[DType.float32],
    vectors: GPUTensor[DType.float32]
) -> GPUTensor[DType.float32]:
    """CPU implementation of the legacy GPU cosine helper."""
    var result_shape = List[Int]()
    result_shape.append(vectors.shape[0] if len(vectors.shape) > 0 else 0)
    var result = GPUTensor[DType.float32](result_shape, query.gpu_context)

    if len(query.shape) == 0 or len(vectors.shape) < 2:
        return result

    let dim = query.shape[len(query.shape) - 1]
    let count = vectors.shape[0]

    for i in range(count):
        var dot = Float64(0.0)
        var q_norm = Float64(0.0)
        var v_norm = Float64(0.0)

        for j in range(dim):
            let q = Float64(query.data[j])
            let v = Float64(vectors.data[i * dim + j])
            dot += q * v
            q_norm += q * q
            v_norm += v * v

        var denom = sqrt(q_norm) * sqrt(v_norm)
        var value = Float32(0.0)
        if denom > 0.0:
            value = Float32(dot / denom)
        result.data[i] = value

    return result
