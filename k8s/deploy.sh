#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENVIRONMENT="${1:-development}"

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

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
    error "Invalid environment: $ENVIRONMENT. Must be one of: development, staging, production"
fi

# Check prerequisites
command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
command -v kustomize >/dev/null 2>&1 || error "kustomize is required but not installed"

# Validate kubectl context
CURRENT_CONTEXT=$(kubectl config current-context 2>/dev/null || echo "none")
log "Current kubectl context: $CURRENT_CONTEXT"

if [[ "$ENVIRONMENT" == "production" ]]; then
    warn "Deploying to PRODUCTION environment!"
    read -p "Are you sure you want to continue? (yes/no): " -r
    if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
        log "Deployment cancelled"
        exit 0
    fi
fi

# Deploy to environment
OVERLAY_PATH="$SCRIPT_DIR/overlays/$ENVIRONMENT"
if [[ ! -d "$OVERLAY_PATH" ]]; then
    error "Environment overlay not found: $OVERLAY_PATH"
fi

log "Deploying OmenDB to $ENVIRONMENT environment..."

# Create namespace if it doesn't exist
NAMESPACE=""
case "$ENVIRONMENT" in
    development) NAMESPACE="omendb-dev" ;;
    staging) NAMESPACE="omendb-staging" ;;
    production) NAMESPACE="omendb-prod" ;;
esac

if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    log "Creating namespace: $NAMESPACE"
    kubectl create namespace "$NAMESPACE"
fi

# Apply configuration
log "Applying Kubernetes manifests..."
kustomize build "$OVERLAY_PATH" | kubectl apply -f -

# Wait for deployment
log "Waiting for deployment to be ready..."
kubectl rollout status deployment/"${ENVIRONMENT%-*}-omendb" -n "$NAMESPACE" --timeout=300s

# Verify deployment
log "Verifying deployment..."
READY_PODS=$(kubectl get pods -n "$NAMESPACE" -l app=omendb --field-selector=status.phase=Running --no-headers | wc -l)
TOTAL_PODS=$(kubectl get pods -n "$NAMESPACE" -l app=omendb --no-headers | wc -l)

if [[ "$READY_PODS" -eq "$TOTAL_PODS" ]] && [[ "$READY_PODS" -gt 0 ]]; then
    success "Deployment successful! $READY_PODS/$TOTAL_PODS pods are running"
else
    warn "Deployment may have issues. $READY_PODS/$TOTAL_PODS pods are running"
fi

# Show service information
log "Service information:"
kubectl get svc -n "$NAMESPACE" -l app=omendb

# Show useful commands
log ""
log "Useful commands:"
log "  View pods:     kubectl get pods -n $NAMESPACE"
log "  View logs:     kubectl logs -n $NAMESPACE -l app=omendb"
log "  Port forward:  kubectl port-forward -n $NAMESPACE svc/${ENVIRONMENT%-*}-omendb 3000:3000"
log "  Delete:        kubectl delete -k $OVERLAY_PATH"