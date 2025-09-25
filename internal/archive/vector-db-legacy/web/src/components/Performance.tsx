import { Component } from 'solid-js';

const Performance: Component = () => {
  return (
    <section class="py-20 bg-content">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-16">
          <h2 class="text-section font-bold mb-4">
            Production Ready
          </h2>
          <p class="text-xl text-body content-width">
            Optimized for AI workloads. Tested on real-world applications.
          </p>
        </div>

        <div class="max-w-4xl mx-auto">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mb-12">
            {/* Key Performance */}
            <div class="metric-card">
              <h3 class="text-xl font-semibold text-omen-white mb-6">Cross-Platform Performance</h3>
              <div class="space-y-4">
                <div class="flex justify-between items-center">
                  <span class="text-muted">M3 Max</span>
                  <span class="text-2xl font-bold text-omen-indigo-400">157K/s</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-muted">i9-13900KF</span>
                  <span class="text-2xl font-bold text-omen-green">211K/s</span>
                </div>
                <div class="flex justify-between items-center">
                  <span class="text-muted">Startup Time</span>
                  <span class="text-2xl font-bold text-omen-green">0.001ms</span>
                </div>
              </div>
            </div>

            {/* Key Features */}
            <div class="metric-card">
              <h3 class="text-xl font-semibold text-omen-white mb-6">Production Features</h3>
              <div class="space-y-4">
                <div class="flex items-center">
                  <svg class="w-5 h-5 text-omen-green mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-omen-white">HNSW algorithm with automatic switching</span>
                </div>
                <div class="flex items-center">
                  <svg class="w-5 h-5 text-omen-green mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-omen-white">Zero dependencies, works offline</span>
                </div>
                <div class="flex items-center">
                  <svg class="w-5 h-5 text-omen-green mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-omen-white">Cross-platform (macOS, Linux)</span>
                </div>
                <div class="flex items-center">
                  <svg class="w-5 h-5 text-omen-green mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                  </svg>
                  <span class="text-omen-white">Memory safe and production tested</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Methodology Note */}
        <div class="mt-16 pt-8 border-t border-omen-gray-800">
          <div class="text-center">
            <p class="text-sm text-subtle mb-2">
              <strong>Test Configuration:</strong> 128-dimensional vectors, NumPy arrays, batch operations
            </p>
            <p class="text-xs text-subtle mb-2">
              <strong>Cross-Platform:</strong> M3 Max MacBook Pro, Intel i9-13900KF Fedora 42 | Verified August 2025
            </p>
            <p class="text-xs text-subtle">
              Reproducible benchmarks available in <a href="https://github.com/omendb/omendb/tree/main/benchmarks" class="text-omen-indigo-400 hover:text-omen-indigo-300">GitHub repository</a>
            </p>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Performance;