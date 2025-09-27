# OmenDB Production Container
# Multi-stage build for optimal size and security

# Build stage
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/omendb

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn placeholder() {}" > src/lib.rs

# Build dependencies
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binaries
RUN cargo build --release --bins

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r omendb \
    && useradd -r -g omendb -s /bin/false omendb

# Create directories for data and logs
RUN mkdir -p /var/lib/omendb/data \
    /var/log/omendb \
    /etc/omendb \
    && chown -R omendb:omendb /var/lib/omendb /var/log/omendb /etc/omendb

# Copy binaries from builder stage
COPY --from=builder /usr/src/omendb/target/release/omendb /usr/local/bin/
COPY --from=builder /usr/src/omendb/target/release/secure_server /usr/local/bin/
COPY --from=builder /usr/src/omendb/target/release/scale_test /usr/local/bin/
COPY --from=builder /usr/src/omendb/target/release/integration_test /usr/local/bin/

# Copy configuration files
COPY docker/omendb.toml /etc/omendb/
COPY docker/entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/entrypoint.sh

# Switch to non-root user
USER omendb

# Expose ports
EXPOSE 3000 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ready || exit 1

# Environment variables
ENV RUST_LOG=info \
    OMENDB_DATA_DIR=/var/lib/omendb/data \
    OMENDB_LOG_DIR=/var/log/omendb \
    OMENDB_HTTP_PORT=3000 \
    OMENDB_AUTH_DISABLED=false \
    OMENDB_ADMIN_USER=admin \
    OMENDB_ADMIN_PASSWORD=admin123

# Volume for persistent data
VOLUME ["/var/lib/omendb/data", "/var/log/omendb"]

# Default command
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["secure_server", "3000"]

# Labels for metadata
LABEL org.opencontainers.image.title="OmenDB" \
      org.opencontainers.image.description="High-performance learned index database" \
      org.opencontainers.image.version="0.1.0" \
      org.opencontainers.image.vendor="OmenDB Inc." \
      org.opencontainers.image.url="https://github.com/omendb/omendb" \
      org.opencontainers.image.documentation="https://docs.omendb.com" \
      org.opencontainers.image.source="https://github.com/omendb/omendb"