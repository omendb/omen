import { Component } from 'solid-js';

const Community: Component = () => {
  return (
    <div class="min-h-screen py-12 bg-gray-50 dark:bg-gray-900">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-12">
          <h1 class="text-4xl font-bold text-gray-900 dark:text-white mb-4">Community</h1>
          <p class="text-xl text-gray-600 dark:text-gray-400">Join the OmenDB community and help shape the future of vector databases</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mb-12">
          {/* GitHub */}
          <a href="https://github.com/nijaru/omendb" target="_blank" rel="noopener noreferrer" class="bg-white dark:bg-gray-800 p-8 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-omen-primary/30 dark:hover:border-omen-accent/30 hover:shadow-lg transition-all">
            <div class="flex items-center mb-4">
              <svg class="w-8 h-8 text-gray-900 dark:text-white mr-3" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              <h3 class="text-xl font-semibold dark:text-white">GitHub Repository</h3>
            </div>
            <p class="text-gray-600 dark:text-gray-400 mb-4">
              Star the repo, report issues, and contribute to development. We welcome PRs!
            </p>
            <span class="text-omen-primary dark:text-omen-accent font-medium">Visit GitHub →</span>
          </a>

          {/* Issues */}
          <a href="https://github.com/nijaru/omendb/issues" target="_blank" rel="noopener noreferrer" class="bg-white dark:bg-gray-800 p-8 rounded-xl border border-gray-200 dark:border-gray-700 hover:border-omen-primary/30 dark:hover:border-omen-accent/30 hover:shadow-lg transition-all">
            <div class="flex items-center mb-4">
              <svg class="w-8 h-8 text-omen-primary dark:text-omen-accent mr-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <h3 class="text-xl font-semibold dark:text-white">Report Issues</h3>
            </div>
            <p class="text-gray-600 dark:text-gray-400 mb-4">
              Found a bug or have a feature request? Let us know through GitHub issues.
            </p>
            <span class="text-omen-primary dark:text-omen-accent font-medium">Report Issue →</span>
          </a>
        </div>

        {/* Contributing */}
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-8 mb-8">
          <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Contributing</h2>
          
          <div class="prose prose-lg max-w-none text-gray-600 dark:text-gray-400">
            <p class="mb-4">
              OmenDB is an open-source project and we welcome contributions from the community. 
              Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated!
            </p>

            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mt-6 mb-3">How to Contribute</h3>
            <ol class="list-decimal list-inside space-y-2 mb-6">
              <li>Fork the repository on GitHub</li>
              <li>Create a feature branch (<code class="bg-gray-900 dark:bg-black text-gray-100 px-2 py-1 rounded font-mono text-sm">git checkout -b feature/amazing-feature</code>)</li>
              <li>Commit your changes (<code class="bg-gray-900 dark:bg-black text-gray-100 px-2 py-1 rounded font-mono text-sm">git commit -m 'Add amazing feature'</code>)</li>
              <li>Push to the branch (<code class="bg-gray-900 dark:bg-black text-gray-100 px-2 py-1 rounded font-mono text-sm">git push origin feature/amazing-feature</code>)</li>
              <li>Open a Pull Request</li>
            </ol>

            <h3 class="text-xl font-semibold text-gray-900 dark:text-white mb-3">Areas We Need Help</h3>
            <ul class="list-disc list-inside space-y-2">
              <li>Performance optimizations and benchmarking</li>
              <li>Integration with popular ML frameworks</li>
              <li>Documentation and tutorials</li>
              <li>Testing and bug fixes</li>
              <li>Language bindings (JavaScript, Go, Rust)</li>
            </ul>
          </div>
        </div>

        {/* Get Involved */}
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-8">
          <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Get Involved</h2>
          
          <div class="prose prose-lg max-w-none text-gray-600 dark:text-gray-400">
            <p class="mb-6">
              OmenDB is built by developers, for developers. We believe in open collaboration 
              and welcome contributions that improve the project.
            </p>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div class="text-center">
                <div class="text-3xl mb-2">🌟</div>
                <h3 class="font-semibold text-gray-900 dark:text-white mb-2">Star the Repo</h3>
                <p class="text-sm">Show your support and help others discover OmenDB</p>
              </div>
              
              <div class="text-center">
                <div class="text-3xl mb-2">🐛</div>
                <h3 class="font-semibold text-gray-900 dark:text-white mb-2">Report Issues</h3>
                <p class="text-sm">Help us improve by reporting bugs and suggesting features</p>
              </div>
              
              <div class="text-center">
                <div class="text-3xl mb-2">🚀</div>
                <h3 class="font-semibold text-gray-900 dark:text-white mb-2">Contribute Code</h3>
                <p class="text-sm">Submit PRs for bug fixes, features, or documentation</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Community;