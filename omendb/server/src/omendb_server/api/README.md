# OmenDB API Implementation

## Overview

This directory contains the API implementation for OmenDB vector database. The API provides a RESTful interface for interacting with the database, enabling vector storage, retrieval, and similarity search operations.

## Components

### Core API Components

- **api_server.mojo**: Core server implementation that interfaces with the vector store and index components.
- **rest_service.mojo**: REST API implementation using Lightbug API and EmberJson.
- **server.mojo**: Server configuration and initialization.
- **main.mojo**: Main entry point for starting the OmenDB server.

### Libraries Used

- **Lightbug API**: Framework for creating RESTful APIs in Mojo.
- **EmberJson**: Library for JSON serialization and deserialization.

## API Design

The API follows REST principles with JSON request/response format. It provides endpoints for:

- Vector management (create, read, update, delete)
- Vector search (similarity search with filtering)
- Batch operations
- Health checks

For detailed API documentation, see the [docs/README.md](docs/README.md) file.

## Implementation Notes

### REST Implementation

The REST service is implemented using Lightbug API, which provides a simple and intuitive way to define routes and handle HTTP requests in Mojo. The service registers routes for various vector operations and maps them to corresponding handlers.

### JSON Handling

EmberJson is used for JSON serialization and deserialization. The implementation provides utility functions for converting between OmenDB data structures (vectors, records) and JSON objects.

### Security Considerations

- Input validation is implemented to prevent invalid or malicious requests.
- The API includes appropriate status codes and error messages.

## Testing

The API implementation includes comprehensive tests:

- **test_api_server.mojo**: Tests for the core server implementation.
- **test_rest_service.mojo**: Tests for the REST service implementation.

## Examples

For examples of using the API, see the following files:

- **examples/basic_api_example.mojo**: Basic usage example of the OmenDB API.

## Running the Server

To run the OmenDB server with the REST API:

```shell
# Run with default settings
magic run mojo src/api/main.mojo

# Run API example
magic run api_example
```

## Future Work

- Add Python client SDK implementation
- Add authentication and authorization
- Implement rate limiting
- Add API documentation generation
- Implement Protocol Buffers for gRPC support