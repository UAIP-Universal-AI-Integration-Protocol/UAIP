#!/bin/bash
# Deployment Automation Script for UAIP Hub
# Automates the deployment process to different environments

set -euo pipefail

# Configuration
ENVIRONMENT="${1:-dev}"
VERSION="${VERSION:-latest}"
NAMESPACE="${NAMESPACE:-uaip}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-300}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Display usage
usage() {
    cat <<EOF
Usage: $0 <environment> [options]

Environments:
  dev         Development (docker-compose)
  staging     Staging (Kubernetes)
  prod        Production (Kubernetes)

Options:
  VERSION=<version>               Docker image version (default: latest)
  NAMESPACE=<namespace>           Kubernetes namespace (default: uaip)
  DOCKER_REGISTRY=<registry>      Docker registry URL
  HEALTH_CHECK_TIMEOUT=<seconds>  Health check timeout (default: 300)

Examples:
  $0 dev                          # Deploy to development
  $0 staging VERSION=v1.2.3       # Deploy v1.2.3 to staging
  $0 prod VERSION=v1.2.3          # Deploy v1.2.3 to production

EOF
    exit 1
}

# Log function
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✓${NC} $*"
}

log_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ✗${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠${NC} $*"
}

# Check if required tools are installed
check_requirements() {
    local missing_tools=()

    case "$ENVIRONMENT" in
        dev)
            if ! command -v docker &> /dev/null; then
                missing_tools+=("docker")
            fi
            if ! command -v docker-compose &> /dev/null; then
                missing_tools+=("docker-compose")
            fi
            ;;
        staging|prod)
            if ! command -v kubectl &> /dev/null; then
                missing_tools+=("kubectl")
            fi
            ;;
    esac

    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
}

# Deploy to development (docker-compose)
deploy_dev() {
    log "Deploying to development environment..."

    # Build images
    log "Building Docker images..."
    docker-compose -f docker-compose.dev.yml build

    # Stop existing containers
    log "Stopping existing containers..."
    docker-compose -f docker-compose.dev.yml down

    # Start services
    log "Starting services..."
    docker-compose -f docker-compose.dev.yml up -d

    # Wait for services to be healthy
    log "Waiting for services to be healthy..."
    sleep 10

    # Run migrations
    log "Running database migrations..."
    docker-compose -f docker-compose.dev.yml exec -T postgres psql -U uaip -d uaip < migrations/001_initial_schema.sql || true
    docker-compose -f docker-compose.dev.yml exec -T postgres psql -U uaip -d uaip < migrations/002_rbac_tables.sql || true

    # Check health
    if check_health "http://localhost:8443"; then
        log_success "Development deployment complete!"
        log "Access points:"
        log "  - UAIP Hub:     http://localhost:8443"
        log "  - Prometheus:   http://localhost:9090"
        log "  - Grafana:      http://localhost:3000 (admin/admin)"
    else
        log_error "Deployment failed - health check did not pass"
        log "Check logs: docker-compose -f docker-compose.dev.yml logs"
        exit 1
    fi
}

# Deploy to Kubernetes (staging/prod)
deploy_k8s() {
    local env=$1
    log "Deploying to ${env} environment..."

    # Check kubectl context
    local current_context=$(kubectl config current-context)
    log "Current kubectl context: ${YELLOW}${current_context}${NC}"

    if [ "$env" = "prod" ]; then
        log_warning "Deploying to PRODUCTION!"
        read -p "Are you sure you want to continue? (yes/no): " CONFIRM
        if [ "$CONFIRM" != "yes" ]; then
            log "Deployment cancelled"
            exit 0
        fi
    fi

    # Build and push Docker image
    if [ -n "$DOCKER_REGISTRY" ]; then
        local image_tag="${DOCKER_REGISTRY}/uaip-hub:${VERSION}"
        log "Building Docker image: ${image_tag}"
        docker build -t "$image_tag" .

        log "Pushing Docker image..."
        docker push "$image_tag"

        # Update image in deployment
        log "Updating deployment image..."
        kubectl set image deployment/uaip-hub uaip-hub="$image_tag" -n "$NAMESPACE"
    else
        log_warning "No Docker registry specified, skipping image build/push"
    fi

    # Apply Kubernetes manifests
    log "Applying Kubernetes manifests..."
    kubectl apply -f k8s/namespace.yaml
    kubectl apply -f k8s/configmap.yaml
    kubectl apply -f k8s/serviceaccount.yaml

    # Check if secrets exist
    if ! kubectl get secret uaip-secrets -n "$NAMESPACE" &> /dev/null; then
        log_error "Secrets not found! Create secrets first:"
        log "  kubectl create secret generic uaip-secrets \\"
        log "    --from-literal=database-url=... \\"
        log "    --from-literal=redis-url=... \\"
        log "    --from-literal=jwt-secret=... \\"
        log "    -n $NAMESPACE"
        exit 1
    fi

    kubectl apply -f k8s/deployment.yaml
    kubectl apply -f k8s/service.yaml
    kubectl apply -f k8s/hpa.yaml

    # Watch rollout
    log "Watching rollout status..."
    if kubectl rollout status deployment/uaip-hub -n "$NAMESPACE" --timeout="${HEALTH_CHECK_TIMEOUT}s"; then
        log_success "Rollout completed successfully"
    else
        log_error "Rollout failed or timed out"
        log "Check status: kubectl get pods -n $NAMESPACE"
        log "Check logs: kubectl logs -n $NAMESPACE -l app=uaip-hub"
        exit 1
    fi

    # Check pod health
    log "Checking pod health..."
    local ready_pods=$(kubectl get pods -n "$NAMESPACE" -l app=uaip-hub -o jsonpath='{.items[*].status.conditions[?(@.type=="Ready")].status}' | grep -o "True" | wc -l)
    local total_pods=$(kubectl get pods -n "$NAMESPACE" -l app=uaip-hub --no-headers | wc -l)

    if [ "$ready_pods" -eq "$total_pods" ] && [ "$total_pods" -gt 0 ]; then
        log_success "All pods are ready (${ready_pods}/${total_pods})"
    else
        log_error "Some pods are not ready (${ready_pods}/${total_pods})"
        kubectl get pods -n "$NAMESPACE" -l app=uaip-hub
        exit 1
    fi

    # Display deployment info
    log_success "Kubernetes deployment complete!"
    log "Deployment info:"
    kubectl get deployment uaip-hub -n "$NAMESPACE"
    log ""
    log "Service info:"
    kubectl get service uaip-hub -n "$NAMESPACE"
    log ""
    log "Useful commands:"
    log "  View logs:   kubectl logs -n $NAMESPACE -l app=uaip-hub -f"
    log "  Get pods:    kubectl get pods -n $NAMESPACE"
    log "  Describe:    kubectl describe deployment uaip-hub -n $NAMESPACE"
    log "  Rollback:    kubectl rollout undo deployment/uaip-hub -n $NAMESPACE"
}

# Check health endpoint
check_health() {
    local url=$1
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s -o /dev/null -w "%{http_code}" "${url}/api/v1/system/health" | grep -q "^2"; then
            return 0
        fi
        ((attempt++))
        sleep 2
    done

    return 1
}

# Main deployment logic
main() {
    # Display header
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  UAIP Hub Deployment${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    log "Environment: ${YELLOW}${ENVIRONMENT}${NC}"
    log "Version: ${YELLOW}${VERSION}${NC}"
    log "Namespace: ${YELLOW}${NAMESPACE}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Validate environment
    case "$ENVIRONMENT" in
        dev|development)
            ENVIRONMENT="dev"
            ;;
        staging|stg)
            ENVIRONMENT="staging"
            ;;
        prod|production)
            ENVIRONMENT="prod"
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            log_error "Invalid environment: $ENVIRONMENT"
            usage
            ;;
    esac

    # Check requirements
    check_requirements

    # Deploy based on environment
    case "$ENVIRONMENT" in
        dev)
            deploy_dev
            ;;
        staging|prod)
            deploy_k8s "$ENVIRONMENT"
            ;;
    esac

    log_success "Deployment complete! 🚀"
}

# Run main function
main
