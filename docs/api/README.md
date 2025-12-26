# UAIP Hub API Documentation

Comprehensive API documentation for the Universal AI Integration Protocol Hub.

## 📚 Documentation Formats

This directory contains the OpenAPI specification and tools to generate beautiful, interactive API documentation in multiple formats:

- **Redoc** - Three-panel, responsive documentation (reading-focused)
- **Swagger UI** - Interactive documentation with "Try it out" functionality
- **OpenAPI YAML** - Machine-readable specification for code generation

## 🚀 Quick Start

### Generate Documentation

```bash
# From project root
./scripts/generate-docs.sh
```

This will:
1. Validate the OpenAPI specification
2. Generate Redoc HTML
3. Generate Swagger UI HTML
4. Create an index page
5. Offer to serve documentation locally

### View Documentation Locally

```bash
# Option 1: Auto-serve after generation
./scripts/generate-docs.sh
# Choose 'y' when prompted to serve

# Option 2: Manual serve
cd docs/api/generated
python3 -m http.server 8080
```

Then open: http://localhost:8080

### Using Makefile

```bash
# Generate documentation
make docs

# Generate and serve documentation
make docs-serve
```

## 📖 Documentation Structure

```
docs/api/
├── openapi.yaml          # OpenAPI 3.0 specification (source of truth)
├── README.md             # This file
└── generated/            # Generated documentation (git-ignored)
    ├── index.html        # Documentation hub/landing page
    ├── redoc.html        # Redoc documentation
    ├── swagger-ui.html   # Swagger UI documentation
    └── openapi.yaml      # Copy of spec for serving
```

## 🔧 OpenAPI Specification

The `openapi.yaml` file is the **single source of truth** for the UAIP Hub API.

### Key Information

- **Version:** OpenAPI 3.0.3
- **API Version:** 0.1.0
- **Base URL (dev):** `http://localhost:8443`
- **Base URL (prod):** `https://api.uaip.io`

### Tags

- **Authentication** - User and device authentication
- **Devices** - Device registration and management
- **Messages** - Message routing and delivery
- **System** - Health checks and monitoring
- **WebSocket** - Real-time communication

### Authentication Methods

1. **Bearer Token (JWT)**
   ```
   Authorization: Bearer <jwt-token>
   ```
   Get token from `/api/v1/auth/login`

2. **Mutual TLS (X.509 Certificates)**
   For device-to-hub authentication

## 📋 API Endpoints Overview

### Authentication

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/login` | User login, get JWT |
| POST | `/api/v1/auth/refresh` | Refresh JWT token |

### Devices

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/devices` | List all devices |
| POST | `/api/v1/devices` | Register new device |
| GET | `/api/v1/devices/{deviceId}` | Get device details |
| PUT | `/api/v1/devices/{deviceId}` | Update device |
| DELETE | `/api/v1/devices/{deviceId}` | Unregister device |
| POST | `/api/v1/devices/{deviceId}/command` | Send command to device |

### Messages

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/messages` | Send a message |
| GET | `/api/v1/messages/{messageId}` | Get message status |

### System

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/system/health` | Full health check |
| GET | `/api/v1/system/health/liveness` | Liveness probe (K8s) |
| GET | `/api/v1/system/health/readiness` | Readiness probe (K8s) |
| GET | `/metrics` | Prometheus metrics |

### WebSocket

| Protocol | Endpoint | Description |
|----------|----------|-------------|
| WS | `/ws/devices` | Real-time device communication |

## 🔐 Authentication Examples

### Get JWT Token

```bash
curl -X POST http://localhost:8443/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password123"
  }'
```

Response:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "username": "admin",
    "roles": ["admin"]
  }
}
```

### Use JWT Token

```bash
curl http://localhost:8443/api/v1/devices \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## 📝 Common API Patterns

### Quality of Service (QoS)

All message-related operations support QoS levels:
- **QoS 0:** At most once (fire-and-forget)
- **QoS 1:** At least once (acknowledged)
- **QoS 2:** Exactly once (guaranteed)

### Priority Levels

Messages support priority levels 0-10:
- **0-3:** Low priority
- **4-6:** Normal priority
- **7-10:** High priority

### Pagination

List endpoints support pagination:
```
GET /api/v1/devices?limit=50&offset=100
```

Response includes:
```json
{
  "devices": [...],
  "total": 1523,
  "limit": 50,
  "offset": 100
}
```

## 🔄 WebSocket Communication

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8443/ws/devices');

ws.onopen = () => {
  // Send registration
  ws.send(JSON.stringify({
    type: 'register',
    device_id: 'uuid',
    device_type: 'thermostat',
    name: 'Living Room Thermostat'
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Message Types

1. **Registration**
   ```json
   {
     "type": "register",
     "device_id": "uuid",
     "device_type": "thermostat",
     "name": "Living Room",
     "location": "Living Room"
   }
   ```

2. **Telemetry**
   ```json
   {
     "type": "telemetry",
     "device_id": "uuid",
     "timestamp": "2025-01-22T14:30:00Z",
     "data": {
       "temperature": 22.5,
       "humidity": 55.0
     }
   }
   ```

3. **Command**
   ```json
   {
     "type": "command",
     "device_id": "uuid",
     "command": "set_temperature",
     "params": {
       "temperature": 23.0
     }
   }
   ```

4. **Event**
   ```json
   {
     "type": "event",
     "device_id": "uuid",
     "event": "motion_detected",
     "data": {
       "confidence": 0.95
     }
   }
   ```

## 🛠️ Code Generation

Use the OpenAPI spec to generate client libraries:

### Generate TypeScript Client

```bash
npx @openapitools/openapi-generator-cli generate \
  -i docs/api/openapi.yaml \
  -g typescript-axios \
  -o clients/typescript
```

### Generate Python Client

```bash
openapi-generator generate \
  -i docs/api/openapi.yaml \
  -g python \
  -o clients/python
```

### Generate Go Client

```bash
openapi-generator generate \
  -i docs/api/openapi.yaml \
  -g go \
  -o clients/go
```

### Generate Rust Client

```bash
openapi-generator generate \
  -i docs/api/openapi.yaml \
  -g rust \
  -o clients/rust
```

## 📚 Additional Documentation

- [Main README](../../README.md) - Project overview
- [Technical Analysis](../TECHNICAL_ANALYSIS.md) - Architecture details
- [Phase 2 Improvements](../../PHASE2_IMPROVEMENTS.md) - Production features
- [Phase 3 Improvements](../../PHASE3_IMPROVEMENTS.md) - DevOps tooling

## 🧪 Testing the API

### Using Swagger UI

1. Generate and serve docs: `./scripts/generate-docs.sh`
2. Open Swagger UI: http://localhost:8080/swagger-ui.html
3. Click "Authorize" and enter your JWT token
4. Try out endpoints with "Try it out" button

### Using cURL

```bash
# Health check (no auth required)
curl http://localhost:8443/api/v1/system/health

# Login
TOKEN=$(curl -X POST http://localhost:8443/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password123"}' \
  | jq -r '.token')

# List devices
curl http://localhost:8443/api/v1/devices \
  -H "Authorization: Bearer $TOKEN"

# Register device
curl -X POST http://localhost:8443/api/v1/devices \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Kitchen Light",
    "device_type": "light",
    "capabilities": ["on_off", "brightness"],
    "location": "Kitchen"
  }'
```

### Using Device Simulator

```bash
# Start UAIP Hub
make dev

# Start device simulator (in another terminal)
cd tools/device-simulator
cargo run -- -c 10 -t "temp,humidity,motion"
```

## 📊 Monitoring API Usage

### Prometheus Metrics

```bash
# View all metrics
curl http://localhost:8443/metrics

# Filter HTTP metrics
curl http://localhost:8443/metrics | grep uaip_http

# Filter device metrics
curl http://localhost:8443/metrics | grep uaip_device
```

### Grafana Dashboards

1. Start services: `make up`
2. Open Grafana: http://localhost:3000 (admin/admin)
3. View "UAIP Hub Overview" dashboard
4. Monitor API request rates, latencies, and errors

## 🔧 Customizing Documentation

### Update OpenAPI Spec

1. Edit `docs/api/openapi.yaml`
2. Validate: `./scripts/generate-docs.sh` (validates automatically)
3. Regenerate docs: `./scripts/generate-docs.sh`

### Add New Endpoint

```yaml
paths:
  /api/v1/your-endpoint:
    get:
      tags:
        - YourTag
      summary: Your endpoint description
      operationId: yourOperation
      security:
        - bearerAuth: []
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/YourSchema'
```

### Add New Schema

```yaml
components:
  schemas:
    YourSchema:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
      required:
        - id
        - name
```

## 🚀 CI/CD Integration

### GitHub Actions

```yaml
- name: Generate API Documentation
  run: |
    ./scripts/generate-docs.sh
    # No serve, just generate

- name: Deploy Documentation
  run: |
    # Upload to S3, GitHub Pages, etc.
    aws s3 sync docs/api/generated s3://your-bucket/api-docs
```

### Make Targets

```bash
# Generate docs
make docs

# Serve docs locally
make docs-serve

# Validate OpenAPI spec only
make docs-validate
```

## 📖 Best Practices

### API Documentation

1. **Keep spec up-to-date** - Update `openapi.yaml` when API changes
2. **Add examples** - Include request/response examples for all endpoints
3. **Document errors** - Specify all possible error responses
4. **Use descriptions** - Add clear descriptions for all fields
5. **Version properly** - Increment API version for breaking changes

### Security

1. **Never commit tokens** - Don't include real tokens in examples
2. **Document auth** - Clearly document authentication requirements
3. **Rate limits** - Document rate limiting policies
4. **CORS** - Document CORS policies for web clients

## 🐛 Troubleshooting

### Documentation Generation Fails

**Problem:** `./scripts/generate-docs.sh` fails

**Solutions:**
1. Ensure OpenAPI spec exists: `ls -l docs/api/openapi.yaml`
2. Check YAML syntax: `yamllint docs/api/openapi.yaml`
3. Review error messages for specific issues

### Swagger UI Not Loading

**Problem:** Swagger UI shows blank page

**Solutions:**
1. Check browser console for errors
2. Ensure `openapi.yaml` is in the same directory
3. Try accessing `openapi.yaml` directly
4. Clear browser cache

### Documentation Server Won't Start

**Problem:** HTTP server won't start on port 8080

**Solutions:**
1. Check if port is in use: `lsof -i :8080`
2. Use different port: `SERVE_PORT=9000 ./scripts/generate-docs.sh`
3. Install Python if missing: `brew install python3` or `apt install python3`

## 📞 Support

- **Issues:** https://github.com/UAIP-Universal-AI-Integration-Protocol/UAIP/issues
- **Documentation:** Main project README
- **Examples:** `tools/device-simulator/`

## 📜 License

Apache 2.0 - See main project LICENSE

---

**Generated with ❤️ for UAIP Hub developers**
