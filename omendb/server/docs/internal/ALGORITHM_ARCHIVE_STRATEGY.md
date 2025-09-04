# Algorithm Archive Strategy

## Current State
- **Public OmenDB**: Switching to HNSW for competitive pure vector search
- **Server Edition**: Archiving RoarGraph for future cross-modal features

## Why Archive RoarGraph?

### Performance Reality
- RoarGraph: 15-20K vec/s (with O(n²) training overhead)
- HNSW: 50K+ vec/s (O(log n) incremental)
- RoarGraph is 0.4-0.6x slower for pure vector search

### RoarGraph's Intended Purpose
- Designed for **cross-modal search** (text→image)
- Bipartite graph structure for different modalities
- Training queries make sense for cross-modal alignment
- Not optimal for single-modality vector search

## Archive Location
```
omendb-server/algorithms/archived/
├── roar_graph.mojo
├── true_roargraph_bipartite.mojo
├── projection_layers.mojo
└── README.md
```

## Future Server Features

### Cross-Modal Search (v0.3.0+)
```python
# Server-only feature
db = ServerDB(algorithm="roargraph", modalities=["text", "image"])
results = db.search_cross_modal(
    text_query="red sports car",
    image_collection="products"
)
```

### GPU Enhancement Strategy
- Accelerate RoarGraph's projection layers
- GPU batch distance computation
- Keep algorithmic structure, enhance compute

## Current Focus
- Use HNSW for competitive performance NOW
- Archive RoarGraph for future differentiation
- No need to maintain two algorithms in public repo