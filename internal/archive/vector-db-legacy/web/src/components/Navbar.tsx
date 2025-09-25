import { Component, createSignal } from 'solid-js';
import { A } from '@solidjs/router';

const Navbar: Component = () => {
  const [isOpen, setIsOpen] = createSignal(false);

  return (
    <nav class="sticky top-0 z-50 bg-omen-black/90 backdrop-blur-md border-b border-omen-gray-800">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex justify-between h-16">
          <div class="flex items-center">
            <A href="/" class="flex items-center space-x-2">
              <svg class="w-8 h-8 text-omen-indigo-500" viewBox="0 0 40 40" fill="currentColor">
                <path d="M20 5L5 15v10l15 10 15-10V15L20 5z" opacity="0.9"/>
                <path d="M20 15l-5 3.33v6.67l5 3.33 5-3.33v-6.67L20 15z" fill="white"/>
              </svg>
              <span class="font-semibold text-xl text-omen-white">OmenDB</span>
            </A>
          </div>

          {/* Desktop Navigation */}
          <div class="hidden md:flex items-center space-x-8">
            <A href="/docs" class="text-omen-gray-300 hover:text-omen-indigo-400 transition-colors">
              Documentation
            </A>
            <a 
              href="https://github.com/omendb/omenDB" 
              target="_blank"
              rel="noopener noreferrer"
              class="text-omen-gray-300 hover:text-omen-indigo-400 transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://pypi.org/project/omendb/"
              target="_blank"
              rel="noopener noreferrer"
              class="bg-omen-indigo-500 text-omen-white px-4 py-2 rounded-lg hover:bg-omen-indigo-600 transition-colors font-medium"
            >
              Get Started
            </a>
          </div>

          {/* Mobile menu button */}
          <div class="md:hidden flex items-center">
            <button
              onClick={() => setIsOpen(!isOpen())}
              class="text-omen-gray-300 hover:text-omen-indigo-400"
            >
              <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                {isOpen() ? (
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Navigation */}
      {isOpen() && (
        <div class="md:hidden border-t border-omen-gray-800">
          <div class="px-2 pt-2 pb-3 space-y-1">
            <A
              href="/docs"
              class="block px-3 py-2 text-omen-gray-300 hover:text-omen-indigo-400 transition-colors"
              onClick={() => setIsOpen(false)}
            >
              Documentation
            </A>
            <a
              href="https://github.com/omendb/omenDB"
              target="_blank"
              rel="noopener noreferrer"
              class="block px-3 py-2 text-omen-gray-300 hover:text-omen-indigo-400 transition-colors"
            >
              GitHub
            </a>
            <a
              href="https://pypi.org/project/omendb/"
              target="_blank"
              rel="noopener noreferrer"
              class="block px-3 py-2 bg-omen-indigo-500 text-omen-white rounded-lg hover:bg-omen-indigo-600 transition-colors text-center mt-2 font-medium"
            >
              Get Started
            </a>
          </div>
        </div>
      )}
    </nav>
  );
};

export default Navbar;