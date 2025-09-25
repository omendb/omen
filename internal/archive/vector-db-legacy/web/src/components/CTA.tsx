import { Component } from 'solid-js';
import CodeBlock from './CodeBlock';

const CTA: Component = () => {
  return (
    <section class="py-20 bg-gradient-to-r from-omen-primary to-blue-700">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
        <h2 class="text-3xl md:text-4xl font-bold text-white mb-6">
          Ready to Build Faster AI Applications?
        </h2>
        <p class="text-xl text-blue-100 mb-8 max-w-2xl mx-auto">
          Join developers using OmenDB for production-ready vector search in their AI applications.
        </p>
        
        <div class="space-y-6">
          <div class="max-w-md mx-auto">
            <p class="text-sm text-blue-100 mb-2">Install with pip:</p>
            <div class="bg-white/10 backdrop-blur rounded-lg p-1">
              <CodeBlock 
                code="pip install omendb"
                language="bash"
              />
            </div>
          </div>
          
          <div class="flex flex-col sm:flex-row gap-4 justify-center">
            <a
              href="https://github.com/omendb/omenDB#quick-start"
              class="inline-flex items-center px-8 py-4 bg-transparent text-white font-semibold rounded-lg border-2 border-white hover:bg-white/10 transition-colors"
            >
              View Quick Start Guide
              <svg class="ml-2 w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
              </svg>
            </a>
            <a
              href="https://github.com/omendb/omenDB/tree/main/examples"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center px-8 py-4 bg-transparent text-white font-semibold rounded-lg border-2 border-white hover:bg-white/10 transition-colors"
            >
              View Examples
            </a>
          </div>
        </div>
      </div>
    </section>
  );
};

export default CTA;