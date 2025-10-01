#!/bin/bash
# OmenDB Container Entrypoint Script
# Handles initialization, configuration, and graceful shutdown

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Signal handlers for graceful shutdown
shutdown_handler() {
    log_info "Received shutdown signal, stopping OmenDB..."
    if [[ -n "${OMENDB_PID:-}" ]]; then
        kill -TERM "$OMENDB_PID" 2>/dev/null || true
        wait "$OMENDB_PID" 2>/dev/null || true
    fi
    log_info "OmenDB stopped gracefully"
    exit 0
}

# Set up signal traps
trap shutdown_handler SIGTERM SIGINT

# Function to wait for dependencies
wait_for_dependency() {
    local host=$1
    local port=$2
    local service=$3
    local max_attempts=30
    local attempt=1

    log_info "Waiting for $service at $host:$port..."

    while ! nc -z "$host" "$port" 2>/dev/null; do
        if [[ $attempt -ge $max_attempts ]]; then
            log_error "Failed to connect to $service after $max_attempts attempts"
            exit 1
        fi
        log_info "Attempt $attempt/$max_attempts: $service not ready, waiting..."
        sleep 2
        ((attempt++))
    done

    log_success "$service is ready!"
}

# Function to validate configuration
validate_config() {
    log_info "Validating configuration..."

    # Check required directories exist
    local required_dirs=(
        "$OMENDB_DATA_DIR"
        "$OMENDB_LOG_DIR"
    )

    for dir in "${required_dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            log_error "Required directory does not exist: $dir"
            exit 1
        fi
        if [[ ! -w "$dir" ]]; then
            log_error "Directory is not writable: $dir"
            exit 1
        fi
    done

    # Check configuration file exists
    if [[ ! -f "/etc/omendb/omendb.toml" ]]; then
        log_warn "Configuration file not found, using defaults"
    fi

    # Validate environment variables
    if [[ -z "${OMENDB_HTTP_PORT:-}" ]]; then
        export OMENDB_HTTP_PORT=3000
        log_info "Using default HTTP port: $OMENDB_HTTP_PORT"
    fi

    log_success "Configuration validation complete"
}

# Function to initialize database
initialize_database() {
    log_info "Initializing OmenDB database..."

    # Create subdirectories if they don't exist
    mkdir -p "$OMENDB_DATA_DIR/wal"
    mkdir -p "$OMENDB_DATA_DIR/storage"
    mkdir -p "$OMENDB_LOG_DIR/archive"

    # Check if this is a fresh installation
    if [[ ! -f "$OMENDB_DATA_DIR/.initialized" ]]; then
        log_info "Fresh installation detected, setting up initial data..."

        # Create initial marker file
        echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$OMENDB_DATA_DIR/.initialized"

        log_success "Database initialization complete"
    else
        log_info "Existing database detected, checking for recovery..."

        # Check for WAL files that need recovery
        if find "$OMENDB_DATA_DIR/wal" -name "*.wal" -type f | grep -q .; then
            log_info "WAL files found, database will recover on startup"
        fi
    fi
}

# Function to setup monitoring
setup_monitoring() {
    log_info "Setting up monitoring and health checks..."

    # Create monitoring directories
    mkdir -p /var/lib/omendb/metrics

    # Set up log rotation if logrotate is available
    if command -v logrotate >/dev/null 2>&1; then
        cat > /tmp/omendb-logrotate.conf << EOF
$OMENDB_LOG_DIR/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 omendb omendb
    postrotate
        killall -USR1 omendb 2>/dev/null || true
    endscript
}
EOF
        log_info "Log rotation configured"
    fi

    log_success "Monitoring setup complete"
}

# Function to run health check
health_check() {
    local max_attempts=10
    local attempt=1

    log_info "Running initial health check..."

    # Wait for server to start
    sleep 5

    while [[ $attempt -le $max_attempts ]]; do
        if curl -sf "http://localhost:${OMENDB_HTTP_PORT}/ready" >/dev/null 2>&1; then
            log_success "Health check passed"
            return 0
        fi

        log_info "Health check attempt $attempt/$max_attempts failed, retrying..."
        sleep 2
        ((attempt++))
    done

    log_error "Health check failed after $max_attempts attempts"
    return 1
}

# Function to print startup banner
print_banner() {
    cat << 'EOF'
╔══════════════════════════════════════════════════════════════════╗
║                            OMENDB                                ║
║                  High-Performance Learned Index Database         ║
║                                                                  ║
║  🚀 Production Container Ready                                   ║
║  🔒 Security Enabled                                             ║
║  📊 Monitoring Active                                            ║
║  💾 Persistence Configured                                       ║
╚══════════════════════════════════════════════════════════════════╝
EOF
}

# Function to print configuration summary
print_config() {
    log_info "OmenDB Configuration Summary:"
    echo "  Data Directory: $OMENDB_DATA_DIR"
    echo "  Log Directory: $OMENDB_LOG_DIR"
    echo "  HTTP Port: $OMENDB_HTTP_PORT"
    echo "  Authentication: ${OMENDB_AUTH_DISABLED:-false}"
    echo "  Admin User: ${OMENDB_ADMIN_USER:-admin}"
    echo "  Log Level: ${RUST_LOG:-info}"
    echo ""
}

# Main execution
main() {
    print_banner
    print_config

    # Check for dependencies if specified
    if [[ -n "${OMENDB_WAIT_FOR:-}" ]]; then
        IFS=',' read -ra DEPS <<< "$OMENDB_WAIT_FOR"
        for dep in "${DEPS[@]}"; do
            IFS=':' read -ra DEP_PARTS <<< "$dep"
            wait_for_dependency "${DEP_PARTS[0]}" "${DEP_PARTS[1]}" "${DEP_PARTS[2]:-service}"
        done
    fi

    # Initialize system
    validate_config
    initialize_database
    setup_monitoring

    # Determine which binary to run
    local binary="${1:-secure_server}"
    local args=("${@:2}")

    case "$binary" in
        "secure_server")
            log_info "Starting OmenDB Secure Server..."
            exec /usr/local/bin/secure_server "${args[@]}" &
            ;;
        "omendb")
            log_info "Starting OmenDB Main Service..."
            exec /usr/local/bin/omendb "${args[@]}" &
            ;;
        "scale_test")
            log_info "Running OmenDB Scale Test..."
            exec /usr/local/bin/scale_test "${args[@]}"
            ;;
        "integration_test")
            log_info "Running OmenDB Integration Test..."
            exec /usr/local/bin/integration_test "${args[@]}"
            ;;
        *)
            log_error "Unknown binary: $binary"
            log_info "Available binaries: secure_server, omendb, scale_test, integration_test"
            exit 1
            ;;
    esac

    # Store PID for signal handling
    OMENDB_PID=$!

    # Run health check in background for server binaries
    if [[ "$binary" == "secure_server" || "$binary" == "omendb" ]]; then
        (
            health_check || {
                log_error "Initial health check failed, stopping container"
                kill -TERM $OMENDB_PID
                exit 1
            }
        ) &
    fi

    log_success "OmenDB started successfully (PID: $OMENDB_PID)"

    # Wait for the main process
    wait $OMENDB_PID
}

# Run main function with all arguments
main "$@"