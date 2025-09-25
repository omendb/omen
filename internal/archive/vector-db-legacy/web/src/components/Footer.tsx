import { Component } from 'solid-js';
import { A } from '@solidjs/router';
import CodeBlock from './CodeBlock';

const Footer: Component = () => {
  return (
    <footer class="bg-omen-gray-950 border-t border-omen-gray-800">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* Brand */}
          <div class="col-span-1">
            <div class="flex items-center space-x-2 mb-4">
              <svg class="w-8 h-8 text-omen-indigo-500" viewBox="0 0 40 40" fill="currentColor">
                <path d="M20 5L5 15v10l15 10 15-10V15L20 5z" opacity="0.9"/>
                <path d="M20 15l-5 3.33v6.67l5 3.33 5-3.33v-6.67L20 15z" fill="white"/>
              </svg>
              <span class="font-semibold text-xl text-omen-white">OmenDB</span>
            </div>
            <p class="text-muted text-sm">
              Production-ready vector database with instant startup
            </p>
          </div>

          {/* Resources */}
          <div>
            <h3 class="font-semibold text-omen-white mb-4">Resources</h3>
            <ul class="space-y-2 text-sm">
              <li>
                <A href="/docs" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  Documentation
                </A>
              </li>
              <li>
                <a href="https://github.com/omendb/omenDB/tree/main/examples" target="_blank" rel="noopener noreferrer" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  Examples
                </a>
              </li>
              <li>
                <a href="https://github.com/omendb/omenDB/tree/main/benchmarks" target="_blank" rel="noopener noreferrer" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  Benchmarks
                </a>
              </li>
            </ul>
          </div>

          {/* Community */}
          <div>
            <h3 class="font-semibold text-omen-white mb-4">Community</h3>
            <ul class="space-y-2 text-sm">
              <li>
                <a href="https://github.com/omendb/omenDB" target="_blank" rel="noopener noreferrer" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  GitHub
                </a>
              </li>
              <li>
                <a href="https://github.com/omendb/omenDB/issues" target="_blank" rel="noopener noreferrer" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  Issues
                </a>
              </li>
              <li>
                <a href="https://pypi.org/project/omendb/" target="_blank" rel="noopener noreferrer" class="text-muted hover:text-omen-indigo-400 transition-colors">
                  PyPI Package
                </a>
              </li>
            </ul>
          </div>

          {/* Install */}
          <div>
            <h3 class="font-semibold text-omen-white mb-4">Quick Install</h3>
            <CodeBlock
              code="$ pip install omendb"
              language="bash"
            />
          </div>
        </div>

        <div class="mt-8 pt-8 border-t border-omen-gray-800">
          <div class="flex flex-col md:flex-row justify-between items-center gap-4">
            <p class="text-sm text-subtle">
              © {new Date().getFullYear()} OmenDB Project. Elastic License 2.0.
            </p>
            <div class="flex items-center gap-4 text-subtle text-sm">
              <span>Built with Mojo</span>
              <span>•</span>
              <span>Python API</span>
              <span>•</span>
              <span>Intel Optimized</span>
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;