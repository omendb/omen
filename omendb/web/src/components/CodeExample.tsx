import { Component, createSignal } from 'solid-js';
import CodeBlock from './CodeBlock';

type TabKey = 'basic' | 'batch' | 'frameworks';

interface CodeExample {
  title: string;
  code: string;
}

const CodeExample: Component = () => {
  const [activeTab, setActiveTab] = createSignal<TabKey>('basic');

  const examples: Record<TabKey, CodeExample> = {
    basic: {
      title: 'Basic Usage',
      code: `from omendb import DB

# Instant startup (0.001ms)
db = DB("vectors.omen")

# Add vectors with metadata
db.add("doc1", [1.0, 2.0, 3.0], {"type": "article"})
db.add("doc2", [4.0, 5.0, 6.0], {"type": "report"})

# Fast similarity search
results = db.search([1.1, 2.1, 3.1], limit=5)

for result in results:
    print(f"ID: {result.id}, Score: {result.score}")`
    },
    batch: {
      title: 'High Performance',
      code: `from omendb import DB

db = DB("vectors.omen")

# Batch operations: 91K-210K vec/s (platform dependent)
vectors = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.6], [7.0, 8.0, 9.0]]
ids = ["id1", "id2", "id3"]
metadata = [{"category": "tech"}, {"category": "science"}, {"category": "tech"}]

db.add_batch(vectors=vectors, ids=ids, metadata=metadata)

# Query with metadata filtering  
results = db.search(
    [1.0, 2.0, 3.0], 
    limit=10,
    filter={"category": "tech"}
)`
    },
    frameworks: {
      title: 'Framework Integration',
      code: `from omendb import DB
import numpy as np
import torch

db = DB("vectors.omen")

# NumPy arrays - automatic conversion
np_vector = np.random.rand(128).astype(np.float32)
db.add("numpy_vec", np_vector)

# PyTorch tensors - seamless support
torch_vector = torch.randn(128)
db.add("torch_vec", torch_vector)

# Works with any framework that outputs lists or arrays`
    }
  };

  const currentExample = () => examples[activeTab()];

  return (
    <section class="py-20 bg-content">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="text-center mb-16">
          <h2 class="text-section font-bold mb-4">
            Simple, Pythonic API
          </h2>
          <p class="text-xl text-body max-w-3xl mx-auto">
            Get started in minutes with our intuitive API that follows Python best practices.
          </p>
        </div>

        <div class="max-w-4xl mx-auto">
          <div class="flex space-x-1 mb-6 bg-omen-gray-800 p-1 rounded-lg">
            {Object.entries(examples).map(([key, example]) => (
              <button
                onClick={() => setActiveTab(key as TabKey)}
                class={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
                  activeTab() === key
                    ? 'bg-omen-indigo-500 text-omen-white shadow-sm'
                    : 'text-omen-gray-300 hover:text-omen-white hover:bg-omen-gray-700'
                }`}
              >
                {example.title}
              </button>
            ))}
          </div>

          <div class="relative">
            <div class="absolute top-0 left-0 right-0 bg-omen-gray-900 rounded-t-xl px-6 py-3 border-b border-omen-gray-700">
              <div class="flex items-center justify-between">
                <div class="flex space-x-2">
                  <div class="w-3 h-3 bg-red-500 rounded-full"></div>
                  <div class="w-3 h-3 bg-yellow-500 rounded-full"></div>
                  <div class="w-3 h-3 bg-green-500 rounded-full"></div>
                </div>
                <span class="text-sm text-omen-gray-400">Python</span>
              </div>
            </div>
            <div class="pt-12">
              <CodeBlock 
                code={currentExample().code}
                language="python"
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default CodeExample;