import { Component } from 'solid-js';
import CodeBlock from './CodeBlock';

const Hero: Component = () => {
  return (
    <section class="relative min-h-screen flex items-center bg-primary">
      {/* Subtle indigo gradient for depth */}
      <div class="absolute inset-0 bg-[radial-gradient(circle_at_50%_50%,rgba(99,102,241,0.08),transparent_50%)]"></div>
      
      <div class="relative hero-width px-6 sm:px-8 lg:px-12">
        <div class="text-center">
          {/* Main headline */}
          <h1 class="text-hero font-bold mb-8">
            High-Performance
            <span class="block">Embedded Vector Database</span>
          </h1>
          
          {/* Value proposition */}
          <p class="text-xl sm:text-2xl text-body mb-12 content-width">
            Zero configuration. Production performance. Python API.
          </p>

          {/* Terminal demo */}
          <div class="mb-12 content-width">
            <div class="max-w-md mx-auto">
              <div class="text-omen-gray-500 text-xs mb-2">Terminal</div>
              <CodeBlock
                code={`$ pip install omendb
$ python
>>> from omendb import DB
>>> db = DB()  # 0.001ms startup`}
                language="bash"
              />
            </div>
          </div>

          {/* Call to action */}
          <div class="flex flex-col sm:flex-row gap-4 justify-center mb-16">
            <a
              href="https://pypi.org/project/omendb/"
              class="btn-primary"
            >
              Get Started
              <svg class="ml-2 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
              </svg>
            </a>
            <a
              href="/docs"
              class="btn-primary"
            >
              View Quick Start Guide
            </a>
          </div>

          {/* Performance metrics */}
          <div class="grid grid-cols-1 md:grid-cols-3 gap-8 content-width">
            <div class="metric-card">
              <div class="text-3xl font-bold text-omen-white mb-2">99K+</div>
              <div class="text-muted">vectors/sec</div>
              <div class="text-sm text-subtle mt-1">batch mode</div>
            </div>
            <div class="metric-card">
              <div class="text-3xl font-bold text-omen-white mb-2">&lt;1ms</div>
              <div class="text-muted">query time</div>
              <div class="text-sm text-subtle mt-1">P50 latency</div>
            </div>
            <div class="metric-card">
              <div class="text-3xl font-bold text-omen-green mb-2">0.001ms</div>
              <div class="text-muted">constructor</div>
              <div class="text-sm text-subtle mt-1">instant ready</div>
            </div>
          </div>

          {/* Social proof */}
          <div class="mt-16 pt-16 border-t border-omen-gray-800">
            <p class="text-sm text-subtle mb-4">
              Optimized for AI applications. Ready for production.
            </p>
            <div class="flex items-center justify-center gap-8 text-muted">
              <span>✓ Memory Safe</span>
              <span>✓ Cross Platform</span>
              <span>✓ Zero Dependencies</span>
              <span>✓ Production Ready</span>
            </div>
          </div>
        </div>
      </div>

      {/* Scroll indicator */}
      <div class="absolute bottom-8 left-1/2 transform -translate-x-1/2">
        <div class="w-6 h-10 border-2 border-omen-gray-700 rounded-full flex justify-center">
          <div class="w-1 h-2 bg-omen-gray-600 rounded-full mt-2 animate-bounce"></div>
        </div>
      </div>
    </section>
  );
};

export default Hero;