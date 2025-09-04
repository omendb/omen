"""Vector index implementations for OmenDB.

This package provides various index implementations for efficient vector search:

1. HnswIndex: Hierarchical Navigable Small World graph index for approximate nearest
   neighbor search with logarithmic query complexity.
   
2. ParallelHnswIndex: Multi-threaded implementation of HNSW that uses a thread pool
   for parallel vector insertion, significantly improving index construction time.
"""