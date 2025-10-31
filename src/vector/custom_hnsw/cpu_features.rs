// Runtime CPU feature detection for optimal SIMD selection
//
// This allows the same binary to use AVX512 on capable CPUs (Fedora i9-13900KF)
// while gracefully falling back to AVX2 or scalar on other hardware (M3 Max).

use std::sync::OnceLock;

/// CPU capabilities detected at runtime
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    pub avx512f: bool,
    pub avx2: bool,
    pub sse2: bool,
    pub neon: bool,
}

static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

/// Detect CPU features once at startup
pub fn detect() -> CpuFeatures {
    *CPU_FEATURES.get_or_init(|| {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            CpuFeatures {
                avx512f: is_x86_feature_detected!("avx512f"),
                avx2: is_x86_feature_detected!("avx2"),
                sse2: is_x86_feature_detected!("sse2"),
                neon: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            CpuFeatures {
                avx512f: false,
                avx2: false,
                sse2: false,
                neon: cfg!(target_feature = "neon"),
            }
        }

        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "aarch64"
        )))]
        {
            CpuFeatures {
                avx512f: false,
                avx2: false,
                sse2: false,
                neon: false,
            }
        }
    })
}

/// Get optimal SIMD lane count for current CPU
pub fn optimal_lanes() -> usize {
    let features = detect();

    if features.avx512f {
        16 // AVX-512: 16 x f32
    } else if features.avx2 {
        8 // AVX2: 8 x f32
    } else if features.sse2 || features.neon {
        4 // SSE2/NEON: 4 x f32
    } else {
        1 // Scalar fallback
    }
}

/// Print detected CPU features
pub fn print_features() {
    let features = detect();
    println!("CPU Features:");
    println!("  AVX-512: {}", if features.avx512f { "✅" } else { "❌" });
    println!("  AVX2:    {}", if features.avx2 { "✅" } else { "❌" });
    println!("  SSE2:    {}", if features.sse2 { "✅" } else { "❌" });
    println!("  NEON:    {}", if features.neon { "✅" } else { "❌" });
    println!("  Optimal lanes: {}", optimal_lanes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_detection() {
        let features = detect();

        // Should detect something on any platform
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // Most x86_64 CPUs have at least SSE2
            assert!(features.sse2 || features.avx2 || features.avx512f);
        }

        #[cfg(target_arch = "aarch64")]
        {
            // ARM64 typically has NEON
            assert!(features.neon);
        }
    }

    #[test]
    fn test_optimal_lanes() {
        let lanes = optimal_lanes();
        assert!(lanes == 1 || lanes == 4 || lanes == 8 || lanes == 16);
    }

    #[test]
    fn test_features_cached() {
        // Multiple calls should return same instance
        let f1 = detect();
        let f2 = detect();
        assert_eq!(f1.avx512f, f2.avx512f);
        assert_eq!(f1.avx2, f2.avx2);
    }
}
