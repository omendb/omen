#!/bin/bash
set -euo pipefail

# OmenDB Server Deployment Script
# Deploys OmenDB server to Kubernetes cluster

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
K8S_DIR="$(dirname "$SCRIPT_DIR")/k8s"
NAMESPACE="omendb-system"
ENVIRONMENT="${ENVIRONMENT:-staging}"
IMAGE_TAG="${IMAGE_TAG:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Deploy OmenDB server to Kubernetes

OPTIONS:
    -e, --environment ENVIRONMENT   Deployment environment (staging|production) [default: staging]
    -t, --tag TAG                  Docker image tag [default: latest]
    -n, --namespace NAMESPACE      Kubernetes namespace [default: omendb-system]
    -d, --dry-run                  Show what would be deployed without applying
    -h, --help                     Show this help message

EXAMPLES:
    $0                             # Deploy to staging
    $0 -e production -t v0.1.0     # Deploy to production with specific tag
    $0 --dry-run                   # Preview deployment

PREREQUISITES:
    - kubectl configured for target cluster
    - Docker image available: omendb/server:TAG
    - Cluster has required storage classes
    - Ingress controller installed (for external access)
EOF
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found. Please install kubectl."
        exit 1
    fi
    
    # Check cluster connection
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster. Check your kubeconfig."
        exit 1
    fi
    
    # Check kustomize
    if ! command -v kustomize &> /dev/null; then
        log_warn "kustomize not found. Using kubectl apply -k instead."
    fi
    
    log_info "Prerequisites check passed."
}

create_secrets() {
    log_info "Creating secrets..."
    
    # Generate JWT secret if not exists
    if ! kubectl get secret omendb-secrets -n "$NAMESPACE" &> /dev/null; then
        JWT_SECRET=$(openssl rand -base64 32)
        DB_PASSWORD=$(openssl rand -base64 16)
        
        kubectl create secret generic omendb-secrets \
            --namespace="$NAMESPACE" \
            --from-literal=jwt-secret="$JWT_SECRET" \
            --from-literal=db-password="$DB_PASSWORD" \
            --dry-run=client -o yaml | kubectl apply -f -
        
        log_info "Generated new secrets"
    else
        log_info "Using existing secrets"
    fi
}

deploy_monitoring() {
    if [[ "$ENVIRONMENT" == "production" ]]; then
        log_info "Setting up monitoring for production..."
        
        # Apply ServiceMonitor and PrometheusRules
        kubectl apply -f "$K8S_DIR/monitoring.yaml" -n "$NAMESPACE"
        
        log_info "Monitoring configuration applied"
    fi
}

wait_for_deployment() {
    log_info "Waiting for deployment to be ready..."
    
    kubectl rollout status deployment/omendb-server -n "$NAMESPACE" --timeout=600s
    
    # Wait for pods to be ready
    kubectl wait --for=condition=ready pod -l app=omendb-server -n "$NAMESPACE" --timeout=300s
    
    log_info "Deployment is ready!"
}

show_status() {
    log_info "Deployment status:"
    kubectl get pods,svc,ingress -n "$NAMESPACE" -l app=omendb-server
    
    log_info "Health check:"
    if kubectl get ingress omendb-server -n "$NAMESPACE" &> /dev/null; then
        INGRESS_HOST=$(kubectl get ingress omendb-server -n "$NAMESPACE" -o jsonpath='{.spec.rules[0].host}')
        log_info "External endpoint: https://$INGRESS_HOST/health"
    else
        log_info "Port-forward for testing: kubectl port-forward svc/omendb-server 8080:80 -n $NAMESPACE"
    fi
}

main() {
    local dry_run=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--environment)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -n|--namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            -d|--dry-run)
                dry_run=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    # Validate environment
    if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
        log_error "Invalid environment: $ENVIRONMENT. Must be 'staging' or 'production'."
        exit 1
    fi
    
    log_info "Starting OmenDB deployment..."
    log_info "Environment: $ENVIRONMENT"
    log_info "Image tag: $IMAGE_TAG"
    log_info "Namespace: $NAMESPACE"
    
    if [[ "$dry_run" == "true" ]]; then
        log_info "DRY RUN MODE - No changes will be applied"
    fi
    
    check_prerequisites
    
    # Create namespace
    if [[ "$dry_run" == "false" ]]; then
        kubectl apply -f "$K8S_DIR/namespace.yaml"
        create_secrets
    fi
    
    # Build kustomization command
    local kustomize_cmd="kubectl apply -k $K8S_DIR"
    
    if [[ "$dry_run" == "true" ]]; then
        kustomize_cmd+=" --dry-run=client"
    fi
    
    # Apply environment-specific configuration
    cd "$K8S_DIR"
    
    # Update image tag
    if command -v kustomize &> /dev/null; then
        kustomize edit set image omendb/server:$IMAGE_TAG
    fi
    
    log_info "Applying Kubernetes manifests..."
    eval "$kustomize_cmd"
    
    if [[ "$dry_run" == "false" ]]; then
        deploy_monitoring
        wait_for_deployment
        show_status
        
        log_info "🎉 OmenDB server deployed successfully!"
        log_info "Next steps:"
        log_info "  1. Update DNS to point to the load balancer"
        log_info "  2. Configure monitoring alerts"
        log_info "  3. Set up backup procedures"
        log_info "  4. Run integration tests"
    else
        log_info "Dry run completed. Review the output above."
    fi
}

main "$@"