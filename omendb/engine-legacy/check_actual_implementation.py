#!/usr/bin/env python3
"""
Check what's actually happening in our implementation.
"""

import os
import re

def check_simd_usage():
    """Check how SIMD is actually being used."""

    print("🔍 CHECKING ACTUAL SIMD USAGE IN HNSW")
    print("=" * 60)

    # Read the HNSW implementation
    hnsw_path = "omendb/algorithms/hnsw.mojo"

    with open(hnsw_path, 'r') as f:
        content = f.read()

    # Look for distance calculations
    print("\n📊 Distance calculation patterns found:")
    print("-" * 40)

    patterns = [
        r'sum \+= .*\* .*',  # Accumulation pattern
        r'SIMD\[.*\]',  # SIMD type usage
        r'\.load\[width=.*\]',  # SIMD load
        r'reduce_add\(\)',  # SIMD reduction
        r'for .* in range.*:.*\n.*sum',  # Loop patterns
        r'euclidean_distance',  # Function calls
        r'_simple_euclidean_distance'  # Internal function
    ]

    for pattern in patterns:
        matches = re.findall(pattern, content)
        if matches:
            print(f"\n✅ Found '{pattern}':")
            for match in matches[:3]:  # Show first 3
                print(f"   {match.strip()}")
        else:
            print(f"\n❌ NOT FOUND: '{pattern}'")

    # Check if we're using specialized kernels
    print("\n\n🔍 CHECKING SPECIALIZED KERNEL USAGE:")
    print("-" * 40)

    if 'euclidean_distance_128d' in content:
        print("✅ Using specialized 128D kernel")
    else:
        print("❌ NOT using specialized 128D kernel")

    if 'euclidean_distance_adaptive_simd' in content:
        print("✅ Using adaptive SIMD kernel")
    else:
        print("❌ NOT using adaptive SIMD kernel")

    # Check for explicit SIMD types
    simd_count = content.count('SIMD[')
    print(f"\n📊 SIMD type declarations: {simd_count}")

    # Check for vectorize usage
    vectorize_count = content.count('vectorize[')
    print(f"📊 Vectorize usage: {vectorize_count}")

    # Look for the actual distance function
    print("\n\n🔍 ACTUAL DISTANCE FUNCTION:")
    print("-" * 40)

    # Find distance function implementations
    dist_funcs = re.findall(r'fn .*distance.*\([\s\S]*?\n\s*return', content)

    if dist_funcs:
        print(f"Found {len(dist_funcs)} distance functions")
        for i, func in enumerate(dist_funcs[:2]):  # Show first 2
            print(f"\nFunction {i+1}:")
            lines = func.split('\n')[:10]  # First 10 lines
            for line in lines:
                print(f"  {line}")
    else:
        print("❌ No explicit distance functions found!")

    return content

def check_ffi_patterns():
    """Check how we're crossing FFI boundary."""

    print("\n\n🔍 CHECKING FFI PATTERNS")
    print("=" * 60)

    native_path = "omendb/native.mojo"

    try:
        with open(native_path, 'r') as f:
            content = f.read()

        # Check for batch operations
        print("\n📊 Batch operation support:")
        print("-" * 40)

        batch_patterns = [
            'add_vector_batch',
            'search_batch',
            'insert_bulk',
            'batch_size'
        ]

        for pattern in batch_patterns:
            if pattern in content:
                print(f"✅ Has {pattern}")
            else:
                print(f"❌ Missing {pattern}")

        # Check for NumPy zero-copy
        if 'numpy' in content.lower() or 'zero.copy' in content.lower():
            print("\n✅ Has NumPy zero-copy patterns")
        else:
            print("\n❌ No NumPy zero-copy patterns")

    except FileNotFoundError:
        print(f"❌ File not found: {native_path}")

def check_memory_alignment():
    """Check if we're aligning memory properly."""

    print("\n\n🔍 CHECKING MEMORY ALIGNMENT")
    print("=" * 60)

    # Check for alignment patterns
    files_to_check = [
        "omendb/algorithms/hnsw.mojo",
        "omendb/utils/memory_pool.mojo"
    ]

    for filepath in files_to_check:
        if os.path.exists(filepath):
            with open(filepath, 'r') as f:
                content = f.read()

            print(f"\n📊 {filepath}:")
            print("-" * 40)

            alignment_patterns = [
                ('aligned_alloc', 'Using aligned allocation'),
                ('align.*64', 'Aligning to 64 bytes'),
                ('align.*32', 'Aligning to 32 bytes'),
                ('align.*16', 'Aligning to 16 bytes'),
                ('AlignedBuffer', 'Using AlignedBuffer'),
                ('cache.line', 'Cache line aware')
            ]

            for pattern, description in alignment_patterns:
                if re.search(pattern, content, re.IGNORECASE):
                    print(f"✅ {description}")
                else:
                    print(f"❌ Not {description.lower()}")

if __name__ == "__main__":
    print("🧠 CHECKING ACTUAL IMPLEMENTATION")
    print("=" * 80)

    content = check_simd_usage()
    check_ffi_patterns()
    check_memory_alignment()

    print("\n\n📝 SUMMARY:")
    print("=" * 60)

    # Count actual SIMD usage
    simd_count = content.count('SIMD[')

    if simd_count > 10:
        print("✅ SIMD types are being used")
    else:
        print(f"⚠️ Only {simd_count} SIMD declarations - might not be using SIMD properly")

    print("\n🤔 Key Questions:")
    print("1. Are we using the specialized kernels?")
    print("2. Are we batching FFI operations?")
    print("3. Is memory properly aligned?")
    print("4. Are loops written for auto-vectorization?")