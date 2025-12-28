# Protocol Adapter REST APIs

This document describes the REST API endpoints for interacting with protocol adapters in the UAIP Hub.

## Base URL

All API endpoints are prefixed with: `/api/v1/adapters`

## Available Adapters

UAIP Hub supports the following protocol adapters:

- **HTTP/REST**: HTTP client for REST API communication
- **WebSocket**: Real-time bidirectional WebSocket communication
- **MQTT**: MQTT publish/subscribe messaging
- **Modbus**: Industrial Modbus TCP protocol client
- **OPC UA**: Industrial automation OPC UA protocol client
- **WebRTC**: Real-time peer-to-peer communication with data channels

## API Endpoints

### List All Adapters

Get information about all available protocol adapters.

**Endpoint**: `GET /api/v1/adapters`

**Response**:
```json
{
  "adapters": [
    {
      "adapter_type": "modbus",
      "name": "Modbus TCP Client",
      "description": "Industrial Modbus TCP protocol client",
      "supported_operations": [
        "read_coils",
        "read_holding_registers",
        "write_single_coil",
        "write_single_register"
      ],
      "status": "available"
    }
  ],
  "total": 6
}
```

---

### HTTP Adapter

#### Test HTTP Connection

Test connectivity to an HTTP endpoint.

**Endpoint**: `POST /api/v1/adapters/http/test`

**Request Body**:
```json
{
  "base_url": "https://api.example.com",
  "timeout_seconds": 10,
  "headers": {
    "X-Custom-Header": "value"
  },
  "auth": {
    "type": "bearer",
    "token": "your-token-here"
  },
  "verify_tls": true
}
```

**Authentication Types**:
- `basic`: `{"type": "basic", "username": "user", "password": "pass"}`
- `bearer`: `{"type": "bearer", "token": "your-token"}`
- `apikey`: `{"type": "apikey", "header_name": "X-API-Key", "api_key": "key123"}`

**Response**:
```json
{
  "success": true,
  "message": "Successfully connected to https://api.example.com",
  "details": null
}
```

---

### Modbus Adapter

#### Test Modbus Connection

Test connectivity to a Modbus TCP server.

**Endpoint**: `POST /api/v1/adapters/modbus/test`

**Request Body**:
```json
{
  "server_address": "192.168.1.100:502",
  "unit_id": 1,
  "connection_timeout": 10
}
```

**Response**:
```json
{
  "success": true,
  "message": "Successfully connected to Modbus server 192.168.1.100:502",
  "details": null
}
```

#### Read Modbus Registers

Read holding registers from a Modbus device.

**Endpoint**: `POST /api/v1/adapters/modbus/read`

**Request Body**:
```json
{
  "server_address": "192.168.1.100:502",
  "unit_id": 1,
  "address": 100,
  "count": 10
}
```

**Parameters**:
- `server_address`: Modbus server address (host:port)
- `unit_id`: Modbus unit/slave ID (default: 1)
- `address`: Starting register address (0-65535)
- `count`: Number of registers to read (1-125)

**Response**:
```json
{
  "values": [1234, 5678, 9012, 3456, 7890, 1234, 5678, 9012, 3456, 7890]
}
```

---

### OPC UA Adapter

#### Test OPC UA Connection

Test connectivity to an OPC UA server.

**Endpoint**: `POST /api/v1/adapters/opcua/test`

**Request Body**:
```json
{
  "endpoint_url": "opc.tcp://192.168.1.100:4840",
  "security_mode": "none",
  "security_policy": "none",
  "username": null,
  "password": null
}
```

**Security Modes**:
- `none`: No security
- `sign`: Message signing
- `signandencrypt`: Message signing and encryption

**Security Policies**:
- `none`: No security policy
- `basic128rsa15`: Basic128Rsa15
- `basic256`: Basic256
- `basic256sha256`: Basic256Sha256
- `aes128sha256rsaoaep`: Aes128Sha256RsaOaep
- `aes256sha256rsapss`: Aes256Sha256RsaPss

**Response**:
```json
{
  "success": true,
  "message": "Successfully connected to OPC UA server opc.tcp://192.168.1.100:4840",
  "details": "session-abc123-def456"
}
```

#### Read OPC UA Node

Read a value from an OPC UA node.

**Endpoint**: `POST /api/v1/adapters/opcua/read`

**Request Body**:
```json
{
  "endpoint_url": "opc.tcp://192.168.1.100:4840",
  "node_id": "ns=2;s=Temperature",
  "username": null,
  "password": null
}
```

**Node ID Formats**:
- Numeric: `ns=0;i=2253` (namespace=0, identifier=2253)
- String: `ns=2;s=Temperature` (namespace=2, identifier="Temperature")

**Well-known Node IDs**:
- Root folder: `ns=0;i=84`
- Objects folder: `ns=0;i=85`
- Server: `ns=0;i=2253`
- Server status: `ns=0;i=2256`

**Response**:
```json
{
  "node_id": "ns=2;s=Temperature",
  "value": {
    "type": "Double",
    "value": 42.5
  },
  "source_timestamp": "2024-01-15T10:30:00Z",
  "server_timestamp": "2024-01-15T10:30:00Z",
  "status_code": 0
}
```

**Value Types**:
- `Boolean`: `{"type": "Boolean", "value": true}`
- `Int32`: `{"type": "Int32", "value": 42}`
- `Double`: `{"type": "Double", "value": 42.5}`
- `String`: `{"type": "String", "value": "text"}`
- `ByteString`: `{"type": "ByteString", "value": [1, 2, 3, 4]}`

---

### WebRTC Adapter

#### Create WebRTC Offer

Create a WebRTC offer for establishing a peer-to-peer connection.

**Endpoint**: `POST /api/v1/adapters/webrtc/offer`

**Request Body**:
```json
{
  "ice_servers": [
    {
      "urls": ["stun:stun.l.google.com:19302"]
    },
    {
      "urls": ["turn:turn.example.com:3478"],
      "username": "user",
      "credential": "pass"
    }
  ],
  "enable_audio": false,
  "enable_video": false,
  "enable_data_channels": true,
  "data_channels": [
    {
      "label": "data",
      "ordered": true,
      "max_packet_life_time": null,
      "max_retransmits": null,
      "protocol": null,
      "negotiated": false,
      "id": null
    }
  ]
}
```

**Parameters**:
- `ice_servers`: Array of ICE servers (STUN/TURN) for NAT traversal
- `enable_audio`: Enable audio streams (default: false)
- `enable_video`: Enable video streams (default: false)
- `enable_data_channels`: Enable data channels (default: true)
- `data_channels`: Array of data channel configurations

**Default ICE Servers**:
If not provided, Google STUN servers are used by default:
- `stun:stun.l.google.com:19302`
- `stun:stun1.l.google.com:19302`

**Response**:
```json
{
  "sdp_type": "Offer",
  "sdp": "v=0\r\no=- 1234567890 2 IN IP4 127.0.0.1\r\ns=-\r\n..."
}
```

---

## Error Responses

All endpoints return consistent error responses:

```json
{
  "code": "InvalidParameter",
  "message": "Connection failed: timeout",
  "details": null
}
```

**HTTP Status Codes**:
- `200 OK`: Request successful
- `400 Bad Request`: Invalid parameters
- `401 Unauthorized`: Authentication failed
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

**Error Codes**:
- `InvalidParameter`: Invalid request parameters
- `ConnectionError`: Failed to connect to adapter
- `Timeout`: Operation timed out
- `DeviceNotFound`: Device not found
- `AuthenticationFailed`: Authentication failed
- `AuthorizationFailed`: Insufficient permissions
- `RateLimitExceeded`: Too many requests
- `InternalError`: Internal server error

---

## Examples

### Read Temperature from Modbus Device

```bash
curl -X POST http://localhost:3000/api/v1/adapters/modbus/read \
  -H "Content-Type: application/json" \
  -d '{
    "server_address": "192.168.1.100:502",
    "unit_id": 1,
    "address": 0,
    "count": 1
  }'
```

### Read OPC UA Node Value

```bash
curl -X POST http://localhost:3000/api/v1/adapters/opcua/read \
  -H "Content-Type: application/json" \
  -d '{
    "endpoint_url": "opc.tcp://localhost:4840",
    "node_id": "ns=2;s=Temperature"
  }'
```

### Test HTTP API Connection

```bash
curl -X POST http://localhost:3000/api/v1/adapters/http/test \
  -H "Content-Type: application/json" \
  -d '{
    "base_url": "https://api.example.com",
    "auth": {
      "type": "bearer",
      "token": "your-api-token"
    }
  }'
```

### Create WebRTC Offer

```bash
curl -X POST http://localhost:3000/api/v1/adapters/webrtc/offer \
  -H "Content-Type: application/json" \
  -d '{
    "enable_data_channels": true,
    "data_channels": [
      {
        "label": "control",
        "ordered": true
      }
    ]
  }'
```

---

## Notes

- All timestamps are in ISO 8601 format (UTC)
- Modbus addresses are 0-indexed
- OPC UA node IDs use the standard format: `ns=<namespace>;i=<id>` or `ns=<namespace>;s=<string>`
- WebRTC SDP offers should be exchanged with the peer via your signaling mechanism
- Connection timeouts can be configured per request
- All adapters support connection pooling and retry logic

For more information, see the [UAIP Core Protocol Documentation](../README.md).
