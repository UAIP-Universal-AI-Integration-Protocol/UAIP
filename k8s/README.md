# Kubernetes Deployment Guide for UAIP Hub

This directory contains Kubernetes manifests for deploying the UAIP Hub in a production environment.

## Prerequisites

- Kubernetes cluster (v1.25+)
- `kubectl` configured
- PostgreSQL database
- Redis cache
- NATS message queue

## Quick Start

### 1. Create Namespace

```bash
kubectl apply -f namespace.yaml
```

### 2. Create Secrets

**⚠️ Important:** Never commit actual secrets to version control!

```bash
# Generate JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# Create secrets
kubectl create secret generic uaip-secrets \
  --from-literal=database-url="postgresql://uaip:YOUR_PASSWORD@postgres:5432/uaip" \
  --from-literal=redis-url="redis://:YOUR_PASSWORD@redis:6379" \
  --from-literal=nats-url="nats://nats:4222" \
  --from-literal=jwt-secret="$JWT_SECRET" \
  -n uaip
```

### 3. Apply Configuration

```bash
kubectl apply -f configmap.yaml
kubectl apply -f serviceaccount.yaml
```

### 4. Deploy Application

```bash
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f hpa.yaml
```

### 5. Verify Deployment

```bash
# Check pods
kubectl get pods -n uaip

# Check services
kubectl get svc -n uaip

# Check logs
kubectl logs -n uaip -l app=uaip-hub --tail=100

# Check health
kubectl get pods -n uaip -o jsonpath='{.items[*].status.conditions[?(@.type=="Ready")].status}'
```

## Architecture

```
┌─────────────────────────────────────────┐
│          Load Balancer (Service)        │
│            uaip-hub:443                  │
└─────────────────┬───────────────────────┘
                  │
      ┌───────────┴───────────┐
      │  HorizontalPodAutoscaler│
      │    (3-10 replicas)      │
      └───────────┬───────────┘
                  │
      ┌───────────┴───────────┐
      │   uaip-hub Deployment  │
      │  (Rolling Update)      │
      └───────────┬───────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐    ┌───▼───┐    ┌───▼───┐
│ Pod 1 │    │ Pod 2 │    │ Pod 3 │
└───┬───┘    └───┬───┘    └───┬───┘
    │            │            │
    └────────────┴────────────┘
                 │
      ┌──────────┼──────────┐
      │          │          │
  ┌───▼───┐  ┌──▼──┐  ┌───▼───┐
  │Postgres│ │Redis│  │ NATS  │
  └────────┘ └─────┘  └───────┘
```

## Configuration

### Resource Limits

Default resource limits per pod:
- **Requests:** 200m CPU, 256Mi Memory
- **Limits:** 500m CPU, 512Mi Memory

Adjust in `deployment.yaml` based on your workload.

### Auto-Scaling

HPA is configured to scale between 3-10 replicas based on:
- CPU utilization > 70%
- Memory utilization > 80%

### High Availability

- **3 replicas minimum** for fault tolerance
- **Pod anti-affinity** spreads pods across nodes
- **Rolling updates** with zero downtime
- **Graceful shutdown** (30s grace period)

## Monitoring

### Health Checks

```bash
# Liveness probe
curl http://POD_IP:8443/api/v1/system/health

# Readiness probe
curl http://POD_IP:8443/api/v1/system/health
```

### Prometheus Metrics

Metrics are exposed at `/metrics` endpoint:

```bash
kubectl port-forward -n uaip svc/uaip-hub 8443:443
curl http://localhost:8443/metrics
```

### Logs

```bash
# Stream logs from all pods
kubectl logs -n uaip -l app=uaip-hub -f --max-log-requests=10

# View logs from specific pod
kubectl logs -n uaip POD_NAME -f

# View logs from previous pod (after crash)
kubectl logs -n uaip POD_NAME --previous
```

## Troubleshooting

### Pods not starting

```bash
kubectl describe pod -n uaip POD_NAME
kubectl logs -n uaip POD_NAME
```

### Database connection issues

```bash
# Check secret
kubectl get secret uaip-secrets -n uaip -o yaml

# Test from pod
kubectl exec -it -n uaip POD_NAME -- sh
# Then try: nc -zv postgres 5432
```

### High memory usage

```bash
# Check metrics
kubectl top pods -n uaip

# Increase memory limits in deployment.yaml
```

## Security Best Practices

✅ **Implemented:**
- Non-root user (UID 1000)
- Read-only root filesystem
- No privilege escalation
- Dropped all capabilities
- Network policies ready
- Secret management via Kubernetes Secrets

🔒 **Recommended:**
- Use external secret manager (AWS Secrets Manager, HashiCorp Vault)
- Enable Pod Security Policies/Admission Controllers
- Implement NetworkPolicies for traffic control
- Use private container registry
- Enable audit logging
- Rotate secrets regularly

## Updating

```bash
# Update image tag in deployment.yaml
# Then apply:
kubectl apply -f deployment.yaml

# Watch rollout
kubectl rollout status deployment/uaip-hub -n uaip

# Rollback if needed
kubectl rollout undo deployment/uaip-hub -n uaip
```

## Cleanup

```bash
kubectl delete -f hpa.yaml
kubectl delete -f service.yaml
kubectl delete -f deployment.yaml
kubectl delete -f serviceaccount.yaml
kubectl delete -f configmap.yaml
kubectl delete secret uaip-secrets -n uaip
kubectl delete -f namespace.yaml
```

## Production Checklist

Before deploying to production:

- [ ] Update all secrets with secure random values
- [ ] Configure TLS certificates
- [ ] Set up ingress controller with SSL termination
- [ ] Configure monitoring and alerting
- [ ] Set up log aggregation
- [ ] Configure backup strategy
- [ ] Test disaster recovery procedures
- [ ] Configure autoscaling parameters
- [ ] Set up CI/CD pipeline
- [ ] Document runbooks
- [ ] Perform load testing
- [ ] Security audit and penetration testing

## Support

For issues and questions:
- GitHub Issues: https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
- Documentation: https://docs.uaip.io
