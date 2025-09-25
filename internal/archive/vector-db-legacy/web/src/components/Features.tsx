import { Component } from 'solid-js';

const Features: Component = () => {
  const features = [
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      ),
      title: "Instant Startup",
      description: "0.001ms initialization. No index loading or service startup required. Ready immediately."
    },
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 002 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
        </svg>
      ),
      title: "HNSW Algorithm",
      description: "Industry-standard graph indexing with automatic optimization. Scales from thousands to millions of vectors."
    },
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
        </svg>
      ),
      title: "Memory Safe",
      description: "Battle-tested in production. Automatic memory management. Comprehensive test coverage."
    },
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
        </svg>
      ),
      title: "Zero Dependencies",
      description: "Self-contained database. No external services or special hardware required. Works offline."
    },
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
        </svg>
      ),
      title: "Framework Integration",
      description: "Works with NumPy, PyTorch, TensorFlow, JAX. Automatic tensor conversion. Industry-standard API."
    },
    {
      icon: (
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
      ),
      title: "Cross Platform",
      description: "Develop on macOS and Linux. Deploy to production Linux servers. Simple pip install."
    }
  ];

  return (
    <section class="py-20 bg-content">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-16">
          <h2 class="text-section font-bold mb-4">
            Built for AI Applications
          </h2>
          <p class="text-xl text-body content-width">
            Production-ready vector database combining embedded simplicity 
            with enterprise performance.
          </p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature) => (
            <div class="metric-card hover:border-omen-indigo-500/30 transition-all">
              <div class="w-12 h-12 bg-omen-indigo-500/10 text-omen-indigo-400 rounded-lg flex items-center justify-center mb-4">
                {feature.icon}
              </div>
              <h3 class="text-xl font-semibold text-omen-white mb-2">{feature.title}</h3>
              <p class="text-body">{feature.description}</p>
            </div>
          ))}
        </div>

        {/* Use Cases */}
        <div class="mt-16 pt-16 border-t border-omen-gray-800">
          <div class="text-center mb-12">
            <h3 class="text-2xl font-bold text-omen-white mb-4">Production Use Cases</h3>
            <p class="text-body">Real-world applications powered by OmenDB performance</p>
          </div>
          
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div class="text-center">
              <div class="w-16 h-16 bg-omen-gray-800 rounded-lg flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-omen-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              </div>
              <h4 class="font-semibold text-omen-white mb-2">RAG Systems</h4>
              <p class="text-sm text-muted">Document search and context retrieval</p>
            </div>
            
            <div class="text-center">
              <div class="w-16 h-16 bg-omen-gray-800 rounded-lg flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-omen-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
                </svg>
              </div>
              <h4 class="font-semibold text-omen-white mb-2">Recommendations</h4>
              <p class="text-sm text-muted">Product and content similarity</p>
            </div>
            
            <div class="text-center">
              <div class="w-16 h-16 bg-omen-gray-800 rounded-lg flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-omen-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <h4 class="font-semibold text-omen-white mb-2">Semantic Search</h4>
              <p class="text-sm text-muted">Natural language queries</p>
            </div>
            
            <div class="text-center">
              <div class="w-16 h-16 bg-omen-gray-800 rounded-lg flex items-center justify-center mx-auto mb-4">
                <svg class="w-8 h-8 text-omen-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
                </svg>
              </div>
              <h4 class="font-semibold text-omen-white mb-2">Edge AI</h4>
              <p class="text-sm text-muted">Offline applications</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default Features;