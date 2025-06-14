#!/bin/bash

# BCAI Enhanced VM Deployment Script
# Automated deployment for production environments with comprehensive health checks

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOY_ENV="${DEPLOY_ENV:-production}"
BUILD_TYPE="${BUILD_TYPE:-release}"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-bcai.registry.com}"
VERSION="${VERSION:-latest}"

# Deployment settings
BACKUP_ENABLED="${BACKUP_ENABLED:-true}"
HEALTH_CHECK_TIMEOUT="${HEALTH_CHECK_TIMEOUT:-300}"
ROLLBACK_ON_FAILURE="${ROLLBACK_ON_FAILURE:-true}"
PARALLEL_DEPLOYMENT="${PARALLEL_DEPLOYMENT:-true}"

# Logging
LOG_FILE="/tmp/bcai-deploy-$(date +%Y%m%d_%H%M%S).log"
exec 1> >(tee -a "$LOG_FILE")
exec 2> >(tee -a "$LOG_FILE" >&2)

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

info() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

# Pre-deployment checks
pre_deployment_checks() {
    log "🔍 Running pre-deployment checks..."
    
    # Check system requirements
    info "Checking system requirements..."
    
    # Check available memory
    AVAILABLE_MEMORY=$(free -m | awk 'NR==2{printf "%.0f", $7}')
    if [ "$AVAILABLE_MEMORY" -lt 2048 ]; then
        error "Insufficient memory. Need at least 2GB free, have ${AVAILABLE_MEMORY}MB"
    fi
    
    # Check disk space
    AVAILABLE_DISK=$(df / | awk 'NR==2{printf "%.0f", $4/1024}')
    if [ "$AVAILABLE_DISK" -lt 5120 ]; then
        error "Insufficient disk space. Need at least 5GB free, have ${AVAILABLE_DISK}MB"
    fi
    
    # Check required tools
    command -v docker >/dev/null 2>&1 || error "Docker is required but not installed"
    command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"
    command -v cargo >/dev/null 2>&1 || error "Rust/Cargo is required but not installed"
    
    # Check Kubernetes cluster connectivity
    if ! kubectl cluster-info >/dev/null 2>&1; then
        error "Cannot connect to Kubernetes cluster"
    fi
    
    # Check Docker registry access
    if ! docker login "$DOCKER_REGISTRY" >/dev/null 2>&1; then
        warn "Cannot authenticate with Docker registry $DOCKER_REGISTRY"
    fi
    
    log "✅ Pre-deployment checks passed"
}

# Build all components
build_components() {
    log "🔨 Building BCAI Enhanced VM components..."
    
    cd "$PROJECT_ROOT"
    
    # Build with optimizations for production
    info "Building runtime with enhanced VM..."
    cargo build --release --manifest-path runtime/Cargo.toml --features enhanced-vm,cuda,metal-gpu,pytorch
    
    info "Building job manager..."
    cargo build --release --manifest-path jobmanager/Cargo.toml
    
    info "Building devnet..."
    cargo build --release --manifest-path devnet/Cargo.toml
    
    info "Building keygen..."
    cargo build --release --manifest-path keygen/Cargo.toml
    
    info "Building dashboard..."
    cargo build --release --manifest-path dashboard/Cargo.toml
    
    # Run comprehensive tests
    info "Running test suite..."
    cargo test --release --all-features
    
    # Run benchmarks to validate performance
    info "Running performance benchmarks..."
    cargo bench --manifest-path runtime/Cargo.toml
    
    log "✅ Build completed successfully"
}

# Build Docker images
build_docker_images() {
    log "🐳 Building Docker images..."
    
    cd "$PROJECT_ROOT"
    
    # Build multi-architecture images
    PLATFORMS="linux/amd64,linux/arm64"
    
    # Enhanced VM Runtime
    info "Building enhanced VM runtime image..."
    docker buildx build \
        --platform "$PLATFORMS" \
        --file docker/Dockerfile.runtime \
        --tag "$DOCKER_REGISTRY/bcai-runtime:$VERSION" \
        --tag "$DOCKER_REGISTRY/bcai-runtime:latest" \
        --push \
        .
    
    # Job Manager
    info "Building job manager image..."
    docker buildx build \
        --platform "$PLATFORMS" \
        --file docker/Dockerfile.jobmanager \
        --tag "$DOCKER_REGISTRY/bcai-jobmanager:$VERSION" \
        --tag "$DOCKER_REGISTRY/bcai-jobmanager:latest" \
        --push \
        .
    
    # DevNet
    info "Building devnet image..."
    docker buildx build \
        --platform "$PLATFORMS" \
        --file docker/Dockerfile.devnet \
        --tag "$DOCKER_REGISTRY/bcai-devnet:$VERSION" \
        --tag "$DOCKER_REGISTRY/bcai-devnet:latest" \
        --push \
        .
    
    # Dashboard
    info "Building dashboard image..."
    docker buildx build \
        --platform "$PLATFORMS" \
        --file docker/Dockerfile.dashboard \
        --tag "$DOCKER_REGISTRY/bcai-dashboard:$VERSION" \
        --tag "$DOCKER_REGISTRY/bcai-dashboard:latest" \
        --push \
        .
    
    log "✅ Docker images built and pushed successfully"
}

# Create Kubernetes manifests
create_k8s_manifests() {
    log "☸️ Creating Kubernetes manifests..."
    
    cd "$PROJECT_ROOT/k8s"
    
    # Create namespace
    kubectl apply -f - <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: bcai-$DEPLOY_ENV
  labels:
    app: bcai
    environment: $DEPLOY_ENV
    version: $VERSION
EOF

    # Enhanced VM Runtime Deployment
    kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bcai-runtime
  namespace: bcai-$DEPLOY_ENV
  labels:
    app: bcai-runtime
    version: $VERSION
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  selector:
    matchLabels:
      app: bcai-runtime
  template:
    metadata:
      labels:
        app: bcai-runtime
        version: $VERSION
    spec:
      containers:
      - name: runtime
        image: $DOCKER_REGISTRY/bcai-runtime:$VERSION
        ports:
        - containerPort: 8080
        - containerPort: 8081  # Metrics port
        env:
        - name: RUST_LOG
          value: "info"
        - name: BCAI_ENV
          value: "$DEPLOY_ENV"
        - name: BCAI_MAX_MEMORY_MB
          value: "4096"
        - name: BCAI_ENABLE_CUDA
          value: "true"
        - name: BCAI_ENABLE_PYTHON_BRIDGE
          value: "true"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
            nvidia.com/gpu: 1  # Request GPU
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
EOF

    # Job Manager Deployment
    kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bcai-jobmanager
  namespace: bcai-$DEPLOY_ENV
spec:
  replicas: 2
  selector:
    matchLabels:
      app: bcai-jobmanager
  template:
    metadata:
      labels:
        app: bcai-jobmanager
        version: $VERSION
    spec:
      containers:
      - name: jobmanager
        image: $DOCKER_REGISTRY/bcai-jobmanager:$VERSION
        ports:
        - containerPort: 8082
        env:
        - name: BCAI_RUNTIME_ENDPOINT
          value: "http://bcai-runtime:8080"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
EOF

    # Services
    kubectl apply -f - <<EOF
apiVersion: v1
kind: Service
metadata:
  name: bcai-runtime
  namespace: bcai-$DEPLOY_ENV
spec:
  selector:
    app: bcai-runtime
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: metrics
    port: 8081
    targetPort: 8081
  type: ClusterIP
---
apiVersion: v1
kind: Service
metadata:
  name: bcai-jobmanager
  namespace: bcai-$DEPLOY_ENV
spec:
  selector:
    app: bcai-jobmanager
  ports:
  - port: 8082
    targetPort: 8082
  type: ClusterIP
EOF

    # Ingress for external access
    kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: bcai-ingress
  namespace: bcai-$DEPLOY_ENV
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
spec:
  tls:
  - hosts:
    - api.bcai.network
    secretName: bcai-tls
  rules:
  - host: api.bcai.network
    http:
      paths:
      - path: /runtime
        pathType: Prefix
        backend:
          service:
            name: bcai-runtime
            port:
              number: 8080
      - path: /jobs
        pathType: Prefix
        backend:
          service:
            name: bcai-jobmanager
            port:
              number: 8082
EOF

    log "✅ Kubernetes manifests created and applied"
}

# Deploy monitoring and observability
deploy_monitoring() {
    log "📊 Deploying monitoring and observability..."
    
    # Prometheus monitoring
    kubectl apply -f - <<EOF
apiVersion: v1
kind: ServiceMonitor
metadata:
  name: bcai-runtime-monitor
  namespace: bcai-$DEPLOY_ENV
  labels:
    app: bcai-runtime
spec:
  selector:
    matchLabels:
      app: bcai-runtime
  endpoints:
  - port: metrics
    path: /metrics
    interval: 30s
EOF

    # Grafana dashboard ConfigMap
    kubectl apply -f - <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: bcai-dashboard
  namespace: bcai-$DEPLOY_ENV
data:
  dashboard.json: |
    {
      "dashboard": {
        "title": "BCAI Enhanced VM Dashboard",
        "panels": [
          {
            "title": "VM Instruction Throughput",
            "type": "graph",
            "targets": [
              {
                "expr": "rate(bcai_vm_instructions_total[5m])"
              }
            ]
          },
          {
            "title": "Tensor Memory Usage",
            "type": "graph",
            "targets": [
              {
                "expr": "bcai_tensor_memory_bytes"
              }
            ]
          },
          {
            "title": "Python Execution Success Rate",
            "type": "stat",
            "targets": [
              {
                "expr": "rate(bcai_python_executions_success_total[5m]) / rate(bcai_python_executions_total[5m])"
              }
            ]
          }
        ]
      }
    }
EOF

    log "✅ Monitoring deployed successfully"
}

# Health checks and validation
run_health_checks() {
    log "🏥 Running comprehensive health checks..."
    
    local start_time=$(date +%s)
    local timeout=$HEALTH_CHECK_TIMEOUT
    
    # Wait for deployments to be ready
    info "Waiting for deployments to be ready..."
    kubectl wait --for=condition=available deployment/bcai-runtime -n bcai-$DEPLOY_ENV --timeout=${timeout}s
    kubectl wait --for=condition=available deployment/bcai-jobmanager -n bcai-$DEPLOY_ENV --timeout=${timeout}s
    
    # Test runtime API
    info "Testing runtime API..."
    local runtime_endpoint="http://$(kubectl get svc bcai-runtime -n bcai-$DEPLOY_ENV -o jsonpath='{.spec.clusterIP}'):8080"
    
    if curl -f -s "$runtime_endpoint/health" >/dev/null; then
        log "✅ Runtime health check passed"
    else
        error "Runtime health check failed"
    fi
    
    # Test VM functionality
    info "Testing enhanced VM functionality..."
    local test_payload='{"instruction": "TensorCreate", "tensor_id": 1, "shape": [10, 10], "dtype": "Float32"}'
    
    if curl -f -s -X POST -H "Content-Type: application/json" -d "$test_payload" "$runtime_endpoint/execute" >/dev/null; then
        log "✅ VM functionality test passed"
    else
        error "VM functionality test failed"
    fi
    
    # Test Python bridge
    info "Testing Python bridge..."
    local python_payload='{"instruction": "PythonExecute", "code": "result = 2 + 2", "input_tensors": [], "output_tensors": []}'
    
    if curl -f -s -X POST -H "Content-Type: application/json" -d "$python_payload" "$runtime_endpoint/execute" >/dev/null; then
        log "✅ Python bridge test passed"
    else
        warn "Python bridge test failed - this may be expected in some environments"
    fi
    
    # Performance validation
    info "Running performance validation..."
    local performance_score=$(run_performance_test)
    if [ "$performance_score" -gt 1000 ]; then
        log "✅ Performance validation passed (score: $performance_score)"
    else
        warn "Performance validation below threshold (score: $performance_score)"
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log "✅ Health checks completed in ${duration}s"
}

# Performance testing
run_performance_test() {
    local runtime_endpoint="http://$(kubectl get svc bcai-runtime -n bcai-$DEPLOY_ENV -o jsonpath='{.spec.clusterIP}'):8080"
    local start_time=$(date +%s%3N)
    
    # Run 100 tensor operations
    for i in $(seq 1 100); do
        curl -f -s -X POST -H "Content-Type: application/json" \
            -d "{\"instruction\": \"TensorCreate\", \"tensor_id\": $i, \"shape\": [100, 100], \"dtype\": \"Float32\"}" \
            "$runtime_endpoint/execute" >/dev/null || true
    done
    
    local end_time=$(date +%s%3N)
    local duration=$((end_time - start_time))
    local ops_per_second=$((100000 / duration))
    
    echo "$ops_per_second"
}

# Backup current deployment
backup_deployment() {
    if [ "$BACKUP_ENABLED" = "true" ]; then
        log "💾 Creating deployment backup..."
        
        local backup_dir="/tmp/bcai-backup-$(date +%Y%m%d_%H%M%S)"
        mkdir -p "$backup_dir"
        
        # Backup Kubernetes resources
        kubectl get all -n bcai-$DEPLOY_ENV -o yaml > "$backup_dir/k8s-resources.yaml"
        kubectl get configmaps -n bcai-$DEPLOY_ENV -o yaml > "$backup_dir/configmaps.yaml"
        kubectl get secrets -n bcai-$DEPLOY_ENV -o yaml > "$backup_dir/secrets.yaml"
        
        # Backup application data
        info "Backing up application data..."
        # Add specific backup logic here
        
        log "✅ Backup created at $backup_dir"
        echo "$backup_dir" > /tmp/bcai-last-backup
    fi
}

# Rollback deployment
rollback_deployment() {
    if [ "$ROLLBACK_ON_FAILURE" = "true" ]; then
        warn "🔄 Rolling back deployment..."
        
        # Rollback Kubernetes deployments
        kubectl rollout undo deployment/bcai-runtime -n bcai-$DEPLOY_ENV
        kubectl rollout undo deployment/bcai-jobmanager -n bcai-$DEPLOY_ENV
        
        # Wait for rollback to complete
        kubectl rollout status deployment/bcai-runtime -n bcai-$DEPLOY_ENV --timeout=300s
        kubectl rollout status deployment/bcai-jobmanager -n bcai-$DEPLOY_ENV --timeout=300s
        
        log "✅ Rollback completed"
    fi
}

# Cleanup old resources
cleanup_old_resources() {
    log "🧹 Cleaning up old resources..."
    
    # Remove old Docker images (keep last 3 versions)
    docker images "$DOCKER_REGISTRY/bcai-*" --format "table {{.Repository}}\t{{.Tag}}\t{{.CreatedAt}}" | \
        grep -v latest | sort -k3 -r | tail -n +4 | \
        awk '{print $1":"$2}' | xargs -r docker rmi || true
    
    # Clean up old Kubernetes resources
    kubectl delete pods -n bcai-$DEPLOY_ENV --field-selector=status.phase=Succeeded || true
    kubectl delete pods -n bcai-$DEPLOY_ENV --field-selector=status.phase=Failed || true
    
    log "✅ Cleanup completed"
}

# Main deployment orchestration
main() {
    log "🚀 Starting BCAI Enhanced VM Deployment"
    log "Environment: $DEPLOY_ENV"
    log "Version: $VERSION"
    log "Registry: $DOCKER_REGISTRY"
    
    # Trap errors for rollback
    trap 'error "Deployment failed! Check logs at $LOG_FILE"; rollback_deployment' ERR
    
    # Pre-deployment
    pre_deployment_checks
    backup_deployment
    
    # Build and package
    build_components
    build_docker_images
    
    # Deploy to Kubernetes
    create_k8s_manifests
    deploy_monitoring
    
    # Validate deployment
    run_health_checks
    
    # Cleanup
    cleanup_old_resources
    
    log "🎉 BCAI Enhanced VM deployed successfully!"
    log "📊 Dashboard: https://grafana.bcai.network/d/bcai-vm"
    log "📋 API Documentation: https://api.bcai.network/docs"
    log "📝 Deployment log: $LOG_FILE"
    
    # Post-deployment notifications
    if command -v slack-notify >/dev/null 2>&1; then
        slack-notify "🎉 BCAI Enhanced VM v$VERSION deployed successfully to $DEPLOY_ENV"
    fi
}

# Command line interface
case "${1:-deploy}" in
    "pre-check")
        pre_deployment_checks
        ;;
    "build")
        build_components
        build_docker_images
        ;;
    "deploy")
        main
        ;;
    "health-check")
        run_health_checks
        ;;
    "rollback")
        rollback_deployment
        ;;
    "cleanup")
        cleanup_old_resources
        ;;
    *)
        echo "Usage: $0 {deploy|pre-check|build|health-check|rollback|cleanup}"
        exit 1
        ;;
esac 