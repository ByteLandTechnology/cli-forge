# Daemon App-Server Design

Status: planning reference only. This document defines the intended `daemon`
capability redesign for `cli-forge`, but the current scaffold/extend/validate
pipeline does not implement this contract yet.

## Purpose

The daemon capability exists so a generated CLI can:

- run a long-lived app-server in the background
- reuse warm state across invocations
- execute selected leaf commands through that background server
- expose one binary that can act as both daemon server and daemon client

The daemon is not meant to be a cosmetic lifecycle wrapper around a state file.
Its value is persistent execution, shared resources, and command routing.

## Goals

- Keep one CLI binary as the single shipped interface.
- Let users start, stop, inspect, and restart the daemon explicitly.
- Let users route selected leaf commands through the daemon without changing
  the command's core argument shape.
- Preserve the existing human-facing help and structured output conventions.
- Keep local-only transports safe by default.
- Make daemon execution observable, testable, and recoverable.

## Non-Goals

- Remote multi-tenant daemon hosting.
- Automatic backgrounding for every command.
- Replacing plain local execution for simple one-shot commands.
- Making `help`, `paths`, `context`, or `daemon` themselves remote commands.

## Conceptual Model

- `daemon` becomes an optional capability: `in_scope` or `out_of_scope`.
- The same binary has two roles:
  - server role: runs the background app-server
  - client role: sends execution requests to the app-server
- Only daemonizable leaf commands may run through the daemon.
- The CLI remains command-first. Users still type the same leaf commands; they
  choose the execution path with routing flags.

## Command Surface

When daemon capability is enabled, the CLI should expose:

```text
<skill> daemon run [OPTIONS]
<skill> daemon start [OPTIONS]
<skill> daemon stop [OPTIONS]
<skill> daemon restart [OPTIONS]
<skill> daemon status [OPTIONS]

<skill> <leaf-command> ... [--via local|daemon] [--ensure-daemon]
```

### Lifecycle Commands

- `daemon run`
  - foreground app-server mode
  - blocks the current terminal
  - primary entrypoint for development, debugging, and supervised execution
- `daemon start`
  - launches the app-server in background mode
  - returns only after `running`, `failed`, or `timeout`
- `daemon stop`
  - requests graceful shutdown
  - may support `--force` in later phases
- `daemon restart`
  - performs stop/start using the same runtime profile
- `daemon status`
  - reports health, endpoint, pid, uptime, and next action

### Command Routing Flags

- `--via local|daemon`
  - selects where a daemonizable leaf command executes
  - default: `local`
- `--ensure-daemon`
  - valid only with `--via daemon`
  - if the daemon is not running, start it first and then execute

Commands that are not daemonizable must reject `--via daemon` with a
structured error.

## Interaction Model

### Server Role

The server process is responsible for:

- holding long-lived resources such as caches, indexes, sessions, or watchers
- accepting client requests over a local transport
- executing daemonized leaf commands
- streaming progress or incremental results when supported
- reporting health and lifecycle state

### Client Role

The CLI client is responsible for:

- parsing the user's command line
- deciding whether execution stays local or is routed to the daemon
- resolving the effective invocation context before sending the request
- serializing daemon responses back into the selected user-facing output format

### Context Rules

The client should resolve the effective Active Context first, then send the
fully normalized execution context to the daemon. The daemon should not infer
the client's ambient shell state on its own.

This keeps local execution and daemon execution aligned:

- same command path
- same effective selectors
- same explicit cwd override
- same output format request

## Transport Model

### Default Local Transport

The default transport should be local IPC:

- macOS/Linux: Unix domain socket
- Windows: named pipe

This should be the only required transport in the first supported version.

### Optional TCP Transport

TCP should be opt-in only.

- default bind must be loopback only: `127.0.0.1`
- non-loopback bind requires explicit config and stronger warnings
- TCP mode must require authentication

## RPC Contract

The internal daemon protocol should use structured JSON messages. JSON-RPC 2.0
is a good fit because it gives request ids, method names, notifications, and a
familiar error model.

Recommended method set:

- `daemon.health`
- `daemon.status`
- `daemon.shutdown`
- `command.execute`
- `command.execute_stream`

### Example Request

```json
{
  "jsonrpc": "2.0",
  "id": "req-123",
  "method": "command.execute",
  "params": {
    "command_path": ["run"],
    "arguments": {
      "input": "demo-input"
    },
    "context": {
      "selectors": {
        "provider": "preview"
      },
      "cwd": "/tmp/project"
    },
    "client": {
      "version": "0.1.0"
    }
  }
}
```

### Example Success Response

```json
{
  "jsonrpc": "2.0",
  "id": "req-123",
  "result": {
    "execution": {
      "mode": "daemon",
      "instance_id": "default"
    },
    "payload": {
      "status": "ok",
      "message": "Hello from daemonized command"
    }
  }
}
```

The daemon transport itself should stay JSON-based even when the user asked
for YAML or TOML. The client should convert the structured result into the
final requested output format at the CLI boundary.

## Streaming Model

Daemonized streaming commands should use `command.execute_stream`.

Suggested event types:

- `start`
- `progress`
- `item`
- `stderr`
- `result`
- `error`
- `end`

The daemon sends structured stream events; the client maps those into the
selected user-facing stream format.

Examples:

- `--format json`: NDJSON stream
- `--format yaml`: multi-document YAML stream
- `--format toml`: unsupported unless the plan explicitly defines a safe
  framing strategy

## Runtime Files

Daemon runtime artifacts should live beneath `state/daemon/`.

Suggested layout:

- `state/daemon/daemon.pid`
- `state/daemon/daemon.sock` or platform pipe name metadata
- `state/daemon/daemon-state.json`
- `state/daemon/daemon.log`
- `state/daemon/daemon.lock`
- `state/daemon/auth.token` when TCP mode is enabled

## Lifecycle And Health

Recommended daemon states:

- `stopped`
- `starting`
- `running`
- `degraded`
- `stopping`
- `failed`

`daemon status` should report at least:

- `state`
- `readiness`
- `instance_id`
- `pid`
- `transport`
- `endpoint`
- `uptime_sec`
- `active_requests`
- `queue_depth`
- `last_error`
- `recommended_next_action`

## Execution Semantics

Daemon execution should preserve command-level behavior:

- the command path stays the same
- command arguments stay the same
- validation failures remain structured
- successful payload shape matches local execution as closely as possible

The daemon response may add execution metadata, but the business payload should
not fork into a separate schema just because it ran remotely.

## Security Model

- Local IPC transport relies on OS-level filesystem or named-pipe permissions.
- TCP is disabled by default.
- When TCP is enabled, authentication is mandatory.
- Daemon requests must reject unknown methods and malformed payloads with
  stable structured errors.

## Help And Documentation Rules

When daemon capability is enabled:

- `--help` remains plain text only
- `help daemon ...` remains structured
- help must describe both server mode and client routing flags
- help must explain which commands are daemonizable and which always remain
  local

## Validation Requirements

When daemon capability is enabled, validate at least:

- `daemon start` reaches `running` or returns a structured failure
- `daemon status` reports endpoint, state, and readiness
- a daemonized leaf command succeeds through `--via daemon`
- local and daemon execution preserve the same payload shape
- `--ensure-daemon` starts the daemon when needed
- `daemon stop` shuts the server down cleanly
- timeout and failure cases return stable structured errors

## Suggested Plan Schema Additions

When this redesign is implemented, `cli-plan.yml` should grow fields such as:

- `capabilities.daemon`
- `daemon_contract.mode: app_server`
- `daemon_contract.transports`
- `daemon_contract.auth`
- `daemon_contract.daemonizable_commands`
- `daemon_contract.client_routing`
- `daemon_contract.streaming`

## Rollout Plan

### Phase 1

- Accept this app-server daemon design as the planning source of truth.
- Restore daemon as an optional capability in the plan stage.

### Phase 2

- Teach extend/scaffold to generate daemon server and client code when daemon
  is in scope.
- Remove the current baseline managed-daemon placeholder from default scaffold.

### Phase 3

- Update validate with daemon-specific app-server checks.
- Add smoke tests for `start`, `status`, routed execution, and `stop`.

### Phase 4

- Consider optional TCP mode, auth hardening, and named instances.
