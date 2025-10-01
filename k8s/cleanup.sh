#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVIRONMENT="${1:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR:${NC} $1"
    exit 1
}

success() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] SUCCESS:${NC} $1"
}

show_usage() {
    echo "Usage: $0 <environment>"
    echo "Environments: development, staging, production, all"
    echo ""
    echo "Examples:"
    echo "  $0 development  # Remove development deployment"
    echo "  $0 all          # Remove all deployments"
    exit 1
}

cleanup_environment() {
    local env="$1"
    local overlay_path="$SCRIPT_DIR/overlays/$env"

    if [[ ! -d "$overlay_path" ]]; then
        error "Environment overlay not found: $overlay_path"
    fi

    local namespace=""
    case "$env" in
        development) namespace="omendb-dev" ;;
        staging) namespace="omendb-staging" ;;
        production) namespace="omendb-prod" ;;
    esac

    log "Cleaning up $env environment (namespace: $namespace)..."

    # Check if namespace exists
    if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
        warn "Namespace $namespace does not exist"
        return 0
    fi

    # Warn for production
    if [[ "$env" == "production" ]]; then
        warn "DELETING PRODUCTION DEPLOYMENT!"
        warn "This will permanently delete all data and configurations!"
        read -p "Type 'DELETE PRODUCTION' to confirm: " -r
        if [[ "$REPLY" != "DELETE PRODUCTION" ]]; then
            log "Cleanup cancelled"
            return 0
        fi
    fi

    # Delete resources
    log "Deleting OmenDB resources in $env..."
    kustomize build "$overlay_path" | kubectl delete -f - --ignore-not-found=true

    # Wait for pods to terminate
    log "Waiting for pods to terminate..."
    kubectl wait --for=delete pods -n "$namespace" -l app=omendb --timeout=120s || true

    # Optionally delete namespace
    read -p "Delete namespace $namespace? (y/n): " -r
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        kubectl delete namespace "$namespace" --ignore-not-found=true
        success "Namespace $namespace deleted"
    else
        log "Namespace $namespace preserved"
    fi

    success "Cleanup of $env environment complete"
}

# Validate input
if [[ -z "$ENVIRONMENT" ]]; then
    show_usage
fi

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
command -v kustomize >/dev/null 2>&1 || error "kustomize is required but not installed"

# Handle cleanup
case "$ENVIRONMENT" in
    development|staging|production)
        cleanup_environment "$ENVIRONMENT"
        ;;
    all)
        warn "This will clean up ALL OmenDB deployments!"
        read -p "Are you sure? (yes/no): " -r
        if [[ $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            cleanup_environment "development"
            cleanup_environment "staging"
            cleanup_environment "production"
        else
            log "Cleanup cancelled"
        fi
        ;;
    *)
        error "Invalid environment: $ENVIRONMENT"
        show_usage
        ;;
esac