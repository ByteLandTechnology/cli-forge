# Daemon Contract Specification

**Version**: 1.0.0
**Feature**: 003-daemon-app-server-mode

## Overview

This document describes the daemon protocol contract for CLI Forge daemon instances, enabling JSON-RPC 2.0 communication over stdio, WebSocket, and IPC transports.

## Transport Bindings

### Stdio Transport (Subprocess Mode)

- **Use case**: Local subprocess integration
- **Protocol**: JSON-Lines (one JSON object per newline)
- **Connection**: Daemon spawned as child process, communicates via stdin/stdout

### WebSocket Transport (Remote Mode)

- **Use case**: Remote client connections
- **Protocol**: JSON-RPC 2.0 over WebSocket frames
- **Connection**: `ws://host:port/path` or `wss://host:port/path` (TLS)
- **Authentication**: Capability token or JWT bearer token

### IPC Transport (CLI-to-Daemon)

- **TCP Socket**: `tcp:host:port`
- **Unix Domain Socket**: `unix:/path/to/socket`
- **Protocol**: JSON-RPC 2.0 over byte streams

## JSON-RPC 2.0 Protocol

### Message Types

```json
// Request
{"jsonrpc": "2.0", "id": "1", "method": "method.name", "params": {}}

// Success Response
{"jsonrpc": "2.0", "id": "1", "result": {}}

// Error Response
{"jsonrpc": "2.0", "id": "1", "error": {"code": -32601, "message": "Method not found"}}

// Notification (no id)
{"jsonrpc": "2.0", "method": "event.name", "params": {}}
```

### Standard Error Codes

| Code   | Constant          | Description                    |
| ------ | ----------------- | ------------------------------ |
| -32001 | SERVER_OVERLOADED | Server overloaded, retry later |
| -32600 | INVALID_REQUEST   | Invalid JSON-RPC request       |
| -32601 | METHOD_NOT_FOUND  | Method does not exist          |
| -32603 | INTERNAL_ERROR    | Internal server error          |

## Daemon Methods

### Core Methods

| Method       | Description               | Params                |
| ------------ | ------------------------- | --------------------- |
| `initialize` | Initialize daemon session | `{ version: string }` |
| `ping`       | Health check              | `{}`                  |
| `start`      | Start daemon (if stopped) | `{}`                  |
| `stop`       | Stop daemon gracefully    | `{}`                  |
| `restart`    | Restart daemon            | `{}`                  |
| `status`     | Get daemon status         | `{}`                  |

### Notifications

| Notification    | Description             | Payload                               |
| --------------- | ----------------------- | ------------------------------------- |
| `stateChanged`  | Lifecycle state changed | `{ state: string, health: string }`   |
| `healthChanged` | Health status changed   | `{ health: string, reason?: string }` |
| `error`         | Error occurred          | `{ code: number, message: string }`   |

## Lifecycle States

```
Stopped → Starting → Running → Stopping → Stopped
                  ↓
               Failed
```

### State Transitions

| From     | To       | Trigger                 |
| -------- | -------- | ----------------------- |
| Stopped  | Starting | `start()` called        |
| Starting | Running  | Initialization complete |
| Starting | Failed   | Initialization error    |
| Running  | Stopping | `stop()` called         |
| Running  | Failed   | Unrecoverable error     |
| Stopping | Stopped  | Shutdown complete       |
| Failed   | Starting | `restart()` called      |

## Health Status

| Status         | Description                      |
| -------------- | -------------------------------- |
| `Initializing` | Daemon starting up               |
| `Ready`        | Daemon running and healthy       |
| `Degraded`     | Daemon running but with warnings |
| `Unhealthy`    | Daemon not fully functional      |

## Authentication

### Capability Token

- Single-use or multi-use tokens
- Stored in file, read at startup
- Passed via `Authorization: Bearer <token>` header

### JWT Bearer Token

- Signed tokens with claims
- Configuration: issuer, audience, shared secret
- Standard JWT validation

## Session Management

### Session Lifecycle

1. Client connects
2. Client sends `initialize` request
3. Server creates `ProtocolSession`
4. Server responds with session confirmation
5. Client sends requests
6. Server sends notifications
7. Client disconnects
8. Server cleans up session

### Concurrent Sessions

- Support 100+ simultaneous WebSocket connections
- Session ID unique per connection
- Sessions cleaned up on disconnect

## Status Response Format

```json
{
  "lifecycleState": "Running",
  "health": "Ready",
  "instanceId": "daemon-abc123",
  "reason": null,
  "nextAction": null
}
```

## Multi-Instance Support

### Port-Based Identification

```
daemon start --port 9090
daemon status --port 9090
```

### Socket Path Identification

```
daemon start --socket /tmp/my-daemon.sock
daemon status --socket /tmp/my-daemon.sock
```

## Usage Examples

### Start Daemon (Stdio Mode)

```bash
daemon run --transport stdio
```

### Start Daemon (TCP Mode)

```bash
daemon start --host 127.0.0.1 --port 9090
```

### Start Daemon (Unix Socket Mode)

```bash
daemon start --socket /tmp/daemon.sock
```

### Check Status

```bash
daemon status --port 9090
```

### Stop Daemon

```bash
daemon stop --port 9090
```

## Error Handling

### Client Errors

- Invalid JSON: Parse error
- Invalid request structure: -32600
- Unknown method: -32601

### Server Errors

- Internal errors: -32603
- Overload: -32001
- Auth failure: 401 Unauthorized

## Implementation Notes

- All transport implementations must be thread-safe
- JSON-RPC handlers must not block
- Use async/await for I/O operations
- Validate all incoming messages before processing
