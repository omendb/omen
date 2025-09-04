import { Component } from 'solid-js';
import CodeBlock from '../components/CodeBlock';

const Docs: Component = () => {
  return (
    <div class="min-h-screen py-12 bg-gray-50 dark:bg-gray-900">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">Documentation</h1>
          <p class="text-xl text-gray-600 dark:text-gray-400">Everything you need to get started with OmenDB</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
          <div class="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700">
            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">Quick Start</h3>
            <p class="text-gray-600 dark:text-gray-400">Get up and running with OmenDB in under 5 minutes.</p>
            <p class="text-sm text-gray-500 dark:text-gray-500 mt-4">Documentation coming soon</p>
          </div>

          <div class="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700">
            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">API Reference</h3>
            <p class="text-gray-600 dark:text-gray-400">Complete reference for all OmenDB methods and classes.</p>
            <p class="text-sm text-gray-500 dark:text-gray-500 mt-4">Documentation coming soon</p>
          </div>

          <a href="https://github.com/omendb/omenDB/tree/main/examples" target="_blank" rel="noopener noreferrer" class="bg-white dark:bg-gray-800 p-6 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-omen-primary/30 dark:hover:border-omen-accent/30 hover:shadow-lg transition-all">
            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">Examples</h3>
            <p class="text-gray-600 dark:text-gray-400">Real-world examples with PyTorch, TensorFlow, and OpenAI embeddings.</p>
          </a>
        </div>

        <div class="mt-12 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-8">
          <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Installation</h2>
          
          <div class="max-w-4xl">
            <p class="text-gray-600 dark:text-gray-400 mb-4">Choose your preferred package manager:</p>
            
            <div class="space-y-8">
              {/* pip */}
              <div>
                <h4 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">pip</h4>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-3">Coming soon to PyPI. For now, install from source:</p>
                <div class="space-y-3">
                  <div>
                    <div class="text-xs text-gray-500 dark:text-gray-500 mb-1">Clone the repository:</div>
                    <CodeBlock 
                      code="git clone https://github.com/omendb/omenDB.git"
                      language="bash"
                    />
                  </div>
                  <div>
                    <div class="text-xs text-gray-500 dark:text-gray-500 mb-1">Install in development mode:</div>
                    <CodeBlock 
                      code="cd omenDB && pip install -e ."
                      language="bash"
                    />
                  </div>
                </div>
              </div>

              {/* uv */}
              <div>
                <h4 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">uv (fast Python package manager)</h4>
                <CodeBlock 
                  code="uv add omendb"
                  language="bash"
                />
                <p class="text-xs text-gray-500 dark:text-gray-500 mt-2">Or: <code class="bg-gray-100 dark:bg-gray-800 px-1 rounded">uv pip install omendb</code></p>
              </div>

              {/* pixi */}
              <div>
                <h4 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">pixi</h4>
                <CodeBlock 
                  code="pixi add omendb"
                  language="bash"
                />
              </div>

              {/* poetry */}
              <div>
                <h4 class="text-lg font-semibold text-gray-900 dark:text-white mb-2">Poetry</h4>
                <CodeBlock 
                  code="poetry add omendb"
                  language="bash"
                />
              </div>
            </div>

            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mt-8 mb-4">Requirements</h3>
            <ul class="list-disc list-inside text-gray-600 dark:text-gray-400 space-y-2">
              <li>Python 3.8 or higher</li>
              <li>NumPy (optional, for compatibility)</li>
              <li>No external database required</li>
            </ul>

            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mt-8 mb-4">First Steps</h3>
            <div class="space-y-4">
              <div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">1. Import and create a database:</p>
                <CodeBlock 
                  code={`from omendb import DB

db = DB("my_vectors.omen")`}
                  language="python"
                />
              </div>
              
              <div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">2. Add vectors with IDs:</p>
                <CodeBlock 
                  code={`db.add("hello", [1.0, 2.0, 3.0])`}
                  language="python"
                />
              </div>
              
              <div>
                <p class="text-sm text-gray-600 dark:text-gray-400 mb-2">3. Search for similar vectors:</p>
                <CodeBlock 
                  code={`results = db.search([1.1, 2.1, 3.1], limit=5)
print(results[0].id)  # "hello"`}
                  language="python"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Docs;