# Multi-stage Dockerfile for OmenDB Server
FROM rust:1-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY proto ./proto
COPY build.rs ./

# Build the application
RUN cargo build --release

# Runtime stage  
FROM ubuntu:lts

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app omendb

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/omendb-server /usr/local/bin/omendb-server

# Copy Mojo runtime libraries (placeholder - would be actual paths)
# COPY --from=mojo-runtime /usr/local/lib/libomendb.so /usr/local/lib/
# COPY --from=mojo-runtime /usr/local/bin/mojo /usr/local/bin/

# Create data directory
RUN mkdir -p /app/data && chown omendb:omendb /app/data

# Switch to app user
USER omendb

# Create default config
RUN echo '[server]\nhttp_port = 8080\ngrpc_port = 9090\n\n[engine]\ndimension = 128\npool_size = 10\n\n[storage]\ndata_dir = "/app/data"\nenable_hot_tier = true\nhot_tier_memory_mb = 1024\nenable_warm_tier = true\nwarm_tier_path = "/app/data/warm"\nenable_cold_tier = false\ncold_tier_path = "/app/data/cold"\n\n[metrics]\nenabled = true\nport = 9091\n\n[auth]\njwt_secret = "change-this-in-production"\nenable_api_keys = true' > /app/config.toml

# Expose ports
EXPOSE 8080 9090 9091

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["omendb-server", "--config", "/app/config.toml"]