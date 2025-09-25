# API Specification - OmenDB Server

## REST API

### Core Endpoints

#### Vector Operations
```
POST   /v1/vectors              # Add single vector
POST   /v1/vectors/batch        # Add multiple vectors
GET    /v1/vectors/{id}         # Get vector by ID
DELETE /v1/vectors/{id}         # Delete vector
POST   /v1/search               # Search similar vectors
```

#### Collection Management
```
POST   /v1/collections          # Create collection
GET    /v1/collections          # List collections
DELETE /v1/collections/{name}   # Delete collection
```

### Request/Response Format

#### Add Vector
```json
POST /v1/vectors
{
  "id": "vec_123",
  "vector": [0.1, 0.2, ...],
  "metadata": {"key": "value"}
}

Response:
{
  "id": "vec_123",
  "status": "created"
}
```

#### Search
```json
POST /v1/search
{
  "vector": [0.1, 0.2, ...],
  "top_k": 10,
  "filter": {"category": "products"}
}

Response:
{
  "results": [
    {
      "id": "vec_456",
      "distance": 0.23,
      "metadata": {...}
    }
  ]
}
```

## gRPC API

```protobuf
service OmenDB {
  rpc AddVector(AddVectorRequest) returns (AddVectorResponse);
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc StreamSearch(SearchRequest) returns (stream SearchResult);
}
```

## Authentication

- API Key authentication for platform tier
- JWT tokens for enterprise tier
- Rate limiting based on tier

## Performance SLAs

| Endpoint | P50 | P95 | P99 |
|----------|-----|-----|-----|
| Search   | 5ms | 10ms| 20ms|
| Add      | 2ms | 5ms | 10ms|
| Batch    | 50ms| 100ms| 200ms|