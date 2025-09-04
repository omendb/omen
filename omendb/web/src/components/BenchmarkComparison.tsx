import { Component } from 'solid-js';

const BenchmarkComparison: Component = () => {
  return (
    <section class="py-16 bg-content">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <h2 class="text-3xl font-bold text-omen-white mb-8">
          Why Choose OmenDB?
        </h2>
        
        <div class="grid grid-cols-1 md:grid-cols-3 gap-8 mb-12">
          <div class="metric-card">
            <div class="text-3xl font-bold text-omen-green mb-2">0.001ms</div>
            <div class="text-lg text-omen-white mb-2">Startup Time</div>
            <div class="text-sm text-subtle">No index loading required</div>
          </div>
          <div class="metric-card">
            <div class="text-3xl font-bold text-omen-indigo-400 mb-2">99K+/s</div>
            <div class="text-lg text-omen-white mb-2">Ingestion Rate</div>
            <div class="text-sm text-subtle">Batch operations</div>
          </div>
          <div class="metric-card">
            <div class="text-3xl font-bold text-omen-green mb-2">&lt;1ms</div>
            <div class="text-lg text-omen-white mb-2">Query Latency</div>
            <div class="text-sm text-subtle">P50 latency</div>
          </div>
        </div>

        <div class="bg-omen-indigo-500/10 p-6 rounded-lg border border-omen-indigo-500/20">
          <p class="text-omen-indigo-300 mb-2">
            <strong>Standard Configuration:</strong> 128-dimensional embeddings
          </p>
          <p class="text-sm text-subtle">
            Results vary by hardware and workload
          </p>
        </div>
      </div>
    </section>
  );
};

export default BenchmarkComparison;