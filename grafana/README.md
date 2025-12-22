# Grafana Dashboards for UAIP Hub

This directory contains pre-configured Grafana dashboards for monitoring the UAIP Hub and its infrastructure.

## Overview

Two comprehensive dashboards are provided:

1. **UAIP Hub Overview** (`uaip-overview`) - Application-level metrics
2. **UAIP Infrastructure Monitoring** (`uaip-infrastructure`) - Infrastructure metrics

## Quick Start

### Using Docker Compose

The easiest way to get started is using the provided `docker-compose.dev.yml`:

```bash
# Start all services including Grafana
make up

# Or manually:
docker-compose -f docker-compose.dev.yml up -d
```

### Accessing Grafana

1. Open your browser to: http://localhost:3000
2. Login credentials:
   - **Username**: `admin`
   - **Password**: `admin` (you'll be prompted to change this)
3. Navigate to **Dashboards** → **Browse**
4. You'll see two dashboards:
   - UAIP Hub Overview
   - UAIP Infrastructure Monitoring

## Dashboard Details

### 1. UAIP Hub Overview

**Panels:**

**HTTP Request Metrics:**
- HTTP Request Rate - Requests per second by endpoint and status
- HTTP Request Latency - p50, p95, p99 latency percentiles

**Device Metrics:**
- Total Devices Registered - Gauge showing device count
- Device Status Distribution - Pie chart of device statuses
- Device Command Rate - Commands per second by type and status

**WebSocket & Messaging:**
- WebSocket Connections - Active WebSocket connections over time
- Message Queue Depth - Queue depth by priority
- Message Processing Rate - Messages processed per second

**Database & Cache:**
- Database Query Latency - p50, p95, p99 query latency
- Cache Hit Rate - Percentage of cache hits vs misses
- Cache Operations - Cache hits and misses per second

**System Resources:**
- Memory Usage - Application memory consumption
- CPU Usage - CPU utilization percentage

**Metrics Used:**
- `uaip_http_requests_total`
- `uaip_http_request_duration_seconds`
- `uaip_devices_registered_total`
- `uaip_device_status_count`
- `uaip_device_commands_total`
- `uaip_websocket_connections_total`
- `uaip_message_queue_depth`
- `uaip_messages_processed_total`
- `uaip_db_query_duration_seconds`
- `uaip_cache_hits_total`
- `uaip_cache_misses_total`
- `uaip_memory_usage_bytes`
- `process_cpu_seconds_total`

### 2. UAIP Infrastructure Monitoring

**Panels:**

**PostgreSQL Metrics:**
- PostgreSQL Status - Up/Down indicator
- PostgreSQL Connections - Active vs max connections
- PostgreSQL Transaction Rate - Commits and rollbacks per second
- Active Backends - Number of active database connections

**Redis Metrics:**
- Redis Status - Up/Down indicator
- Redis Memory Usage - Used vs max memory
- Redis Command Rate - Commands processed per second
- Connected Clients - Number of connected Redis clients

**NATS Metrics:**
- NATS Status - Up/Down indicator
- NATS Connections - Total connections over time
- NATS Throughput - Inbound and outbound bytes per second
- Total Subscriptions - Number of active NATS subscriptions

**Metrics Used:**
- `up{job="postgresql|redis|nats"}`
- `pg_stat_activity_count`
- `pg_stat_database_xact_commit`
- `pg_stat_database_xact_rollback`
- `redis_memory_used_bytes`
- `redis_commands_processed_total`
- `redis_connected_clients`
- `nats_core_total_connections`
- `nats_core_in_bytes`
- `nats_core_out_bytes`

## Directory Structure

```
grafana/
├── README.md                           # This file
├── provisioning/
│   ├── datasources/
│   │   └── prometheus.yml              # Prometheus datasource config
│   └── dashboards/
│       └── dashboard.yml               # Dashboard auto-provisioning config
└── dashboards/
    ├── uaip-overview.json              # Main application dashboard
    └── infrastructure-monitoring.json  # Infrastructure dashboard
```

## Configuration

### Datasource Configuration

The Prometheus datasource is automatically configured via `provisioning/datasources/prometheus.yml`:

```yaml
datasources:
  - name: Prometheus
    type: prometheus
    url: http://prometheus:9090
    isDefault: true
```

**Note:** This assumes Prometheus is running on `http://prometheus:9090`, which is the case when using the provided `docker-compose.dev.yml`.

### Dashboard Auto-Loading

Dashboards are automatically loaded from `dashboards/` via `provisioning/dashboards/dashboard.yml`:

```yaml
providers:
  - name: 'UAIP Dashboards'
    folder: ''
    type: file
    options:
      path: /var/lib/grafana/dashboards
```

## Customization

### Modifying Dashboards

1. **In Grafana UI:**
   - Make changes in the Grafana web interface
   - Click **Save dashboard** → **Save JSON to file**
   - Replace the corresponding file in `grafana/dashboards/`
   - Restart Grafana to reload: `docker-compose -f docker-compose.dev.yml restart grafana`

2. **Editing JSON Directly:**
   - Edit the JSON files in `grafana/dashboards/`
   - Restart Grafana to reload changes

### Adding New Panels

To add custom panels:

1. Open the dashboard in Grafana
2. Click **Add panel** → **Add a new panel**
3. Configure your query and visualization
4. Save the dashboard and export to JSON

### Changing Refresh Intervals

Dashboards auto-refresh every 5 seconds. To change this:

1. Open dashboard → **Dashboard settings** (gear icon)
2. Under **General** → **Auto refresh**, select a different interval
3. Save the dashboard

## Alerting

### Setting Up Alerts

Grafana supports alerting on dashboard panels:

1. Edit a panel
2. Go to the **Alert** tab
3. Configure alert conditions
4. Set notification channels (email, Slack, PagerDuty, etc.)

### Recommended Alerts

**High Priority:**
- HTTP 5xx error rate > 5%
- Database connection pool exhausted
- Redis memory usage > 90%
- WebSocket connections > 1000

**Medium Priority:**
- HTTP request latency p95 > 500ms
- Cache hit rate < 70%
- Message queue depth > 1000

**Low Priority:**
- Device registration rate drop > 50%
- CPU usage > 80%
- Memory usage > 80%

## Troubleshooting

### Dashboards Not Loading

**Issue:** Dashboards don't appear in Grafana

**Solutions:**
1. Check Grafana logs: `docker-compose -f docker-compose.dev.yml logs grafana`
2. Verify file permissions: `ls -la grafana/dashboards/`
3. Ensure volume mounts are correct in `docker-compose.dev.yml`
4. Restart Grafana: `docker-compose -f docker-compose.dev.yml restart grafana`

### No Data in Panels

**Issue:** Panels show "No data"

**Solutions:**
1. Verify Prometheus is running: `curl http://localhost:9090/api/v1/query?query=up`
2. Check if UAIP Hub is exposing metrics: `curl http://localhost:8443/metrics`
3. Verify datasource connection: **Configuration** → **Data sources** → **Prometheus** → **Save & test**
4. Check metric names match those in your application

### Slow Dashboard Performance

**Issue:** Dashboards load slowly

**Solutions:**
1. Reduce time range (e.g., from 1h to 15m)
2. Increase refresh interval (from 5s to 30s)
3. Reduce number of series in queries using label filters
4. Enable query caching in Prometheus

## Best Practices

### Dashboard Organization

- **Overview Dashboard**: High-level metrics for daily monitoring
- **Infrastructure Dashboard**: Deep dive into infrastructure health
- Create additional dashboards for specific use cases (e.g., "Security", "Performance")

### Panel Configuration

- Use appropriate visualization types:
  - **Time series**: Trends over time (latency, throughput)
  - **Gauge**: Current values with thresholds (CPU, memory)
  - **Stat**: Single values (total devices, connections)
  - **Pie chart**: Distribution (device status, error types)

### Query Optimization

- Use `rate()` for counters: `rate(metric[1m])`
- Use `histogram_quantile()` for latencies: `histogram_quantile(0.95, rate(metric_bucket[1m]))`
- Apply label filters to reduce series: `metric{endpoint="/api/v1/devices"}`

### Threshold Configuration

Set appropriate thresholds:
- **Green**: Normal operation
- **Yellow**: Warning (needs attention)
- **Red**: Critical (immediate action required)

## Production Considerations

### High Availability

For production deployments:

1. **Use external Grafana instance** instead of docker-compose
2. **Configure persistent storage** for dashboards and settings
3. **Set up backup** of Grafana database (SQLite by default)
4. **Enable authentication** (LDAP, OAuth, SAML)
5. **Set up HTTPS** with valid certificates

### Multi-Environment Setup

Use Grafana variables for multi-environment dashboards:

```json
"templating": {
  "list": [
    {
      "name": "environment",
      "type": "custom",
      "options": ["dev", "staging", "prod"]
    }
  ]
}
```

### Monitoring Grafana Itself

Monitor Grafana performance:
- Enable Grafana metrics endpoint
- Create dashboard for Grafana self-monitoring
- Alert on Grafana downtime or errors

## Resources

- [Grafana Documentation](https://grafana.com/docs/grafana/latest/)
- [Prometheus Query Examples](https://prometheus.io/docs/prometheus/latest/querying/examples/)
- [Grafana Dashboard Best Practices](https://grafana.com/docs/grafana/latest/best-practices/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)

## Support

For issues or questions:
- Check Grafana logs: `docker-compose -f docker-compose.dev.yml logs grafana`
- Review Prometheus targets: http://localhost:9090/targets
- GitHub Issues: https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
