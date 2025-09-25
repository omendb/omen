"""Lightweight helpers that keep the "advanced" SIMD API compiling.

Mojo's optimizer already emits vector instructions for simple, cache-friendly
loops, so we lean on idiomatic code instead of hand-written AVX intrinsics.
These helpers exist only so legacy call sites keep their names while delegating
work to the specialized kernels or a straightforward fallback loop.
"""

from math import sqrt
from memory import UnsafePointer
from sys.info import simdwidthof
from .specialized_kernels import (
    euclidean_distance_128d,
    euclidean_distance_256d,
    euclidean_distance_384d,
    euclidean_distance_512d,
    euclidean_distance_768d,
    euclidean_distance_1536d,
)

alias float_dtype = DType.float32

@always_inline
fn _distance_scalar(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32],
    dimension: Int
) -> Float32:
    var total = Float32(0)
    for i in range(dimension):
        var diff = a[i] - b[i]
        total += diff * diff
    return sqrt(total)

@always_inline
fn _distance_width[width: Int](
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32],
    dimension: Int
) -> Float32:
    var sum = SIMD[float_dtype, width](0)
    var i = 0
    while i + width <= dimension:
        var diff = a.load[width=width](i) - b.load[width=width](i)
        sum += diff * diff
        i += width
    var total = sum.reduce_add()
    while i < dimension:
        var diff = a[i] - b[i]
        total += diff * diff
        i += 1
    return sqrt(total)

@always_inline
fn euclidean_distance_128d_avx512(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    # Keep the legacy name but delegate to the tuned 128D kernel.
    return euclidean_distance_128d(a, b)

@always_inline
fn euclidean_distance_adaptive_simd(
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32],
    dimension: Int
) -> Float32:
    if dimension == 128:
        return euclidean_distance_128d(a, b)
    elif dimension == 256:
        return euclidean_distance_256d(a, b)
    elif dimension == 384:
        return euclidean_distance_384d(a, b)
    elif dimension == 512:
        return euclidean_distance_512d(a, b)
    elif dimension == 768:
        return euclidean_distance_768d(a, b)
    elif dimension == 1536:
        return euclidean_distance_1536d(a, b)

    var width = simdwidthof[float_dtype]()
    if width >= 32:
        return _distance_width[32](a, b, dimension)
    elif width >= 16:
        return _distance_width[16](a, b, dimension)
    elif width >= 8:
        return _distance_width[8](a, b, dimension)
    else:
        return _distance_scalar(a, b, dimension)

@always_inline
fn vectorized_candidate_distances(
    query: UnsafePointer[Float32],
    candidates: UnsafePointer[UnsafePointer[Float32]],
    n_candidates: Int,
    dimension: Int,
    distances: UnsafePointer[Float32]
):
    for i in range(n_candidates):
        distances[i] = euclidean_distance_adaptive_simd(query, candidates[i], dimension)

@always_inline
fn binary_hamming_distance_avx512(
    a: UnsafePointer[UInt8],
    b: UnsafePointer[UInt8],
    num_bytes: Int
) -> Int:
    var total = 0
    for i in range(num_bytes):
        total += _popcount_u8(a[i] ^ b[i])
    return total

@always_inline
fn _popcount_u8(value: UInt8) -> Int:
    var n = Int(value)
    n = n - ((n >> 1) & 0x55)
    n = (n & 0x33) + ((n >> 2) & 0x33)
    return (n + (n >> 4)) & 0x0F

fn get_simd_capabilities() -> String:
    var width = simdwidthof[float_dtype]()
    if width >= 32:
        return "Wide SIMD"
    elif width >= 16:
        return "Standard SIMD"
    elif width >= 8:
        return "Narrow SIMD"
    else:
        return "Scalar"
