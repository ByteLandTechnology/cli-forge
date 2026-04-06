# cli-forge

`cli-forge` is a staged skill family for creating, evolving, validating, and
publishing Rust CLI Skill projects.

This repository is the workflow source: it contains the parent router skill,
the stage-specific skills, and the templates and runbooks those stages depend
on. It is not itself the generated CLI project, and it is not a single
`cargo run`-style application.

## What This Repository Is

- A parent entrypoint, `cli-forge`, that routes work to the earliest safe
  stage.
- Six workflow stages: `intake`, `description`, `scaffold`, `extend`,
  `validate`, and `publish`.
- The source of truth for stage instructions, template files, validation
  rules, and release-automation guidance.

## What This Repository Is Not

- Not the generated Rust CLI Skill project that end users install or run.
- Not a standalone Cargo workspace with one root `Cargo.toml`.
- Not the place where target-project release commands are executed after the
  publish asset pack is adopted.

## Stage Map

### `cli-forge`

Use the parent skill when you want one stable entrypoint. It inspects the
request and current filesystem state, then resumes from the earliest
incomplete phase.

### `cli-forge-intake`

Classifies the request and assembles required inputs for the next phase.

Use it when:

- the request is ambiguous
- the target path or feature is missing
- you want to resume an interrupted workflow safely

### `cli-forge-description`

Defines or refreshes the generated skill's description contract before
implementation work changes files downstream.

Use it when:

- creating a new skill
- changing an existing generated skill's purpose or positioning

### `cli-forge-scaffold`

Creates a new Rust CLI Skill project from packaged templates.

Use it when:

- the description contract is already approved
- you need a brand-new generated project directory

### `cli-forge-extend`

Adds one supported optional capability to an existing scaffolded project while
preserving the shared runtime contract.

Use it when:

- the project already has the scaffold baseline
- the requested feature is exactly `stream` or `repl`

### `cli-forge-validate`

Audits an existing generated CLI Skill project against structure, metadata,
build, help-surface, runtime-directory, Active Context, error, and optional
REPL requirements.

Use it when:

- you want a compliance report
- scaffold or extend work just finished
- release work needs current validation first

### `cli-forge-publish`

Closes the workflow with report-only readiness checks, dry runs, rehearsals,
or the documented live release path for a target generated repository.

Use it when:

- validation is current
- you need release automation guidance or asset-pack adoption
- you want `report_only`, `dry_run`, `rehearsal`, or `live_release`

## Main Workflows

### New Project

`intake -> description -> scaffold -> validate -> publish`

Typical result:

- a new Rust CLI Skill project directory
- baseline files expanded from templates
- validation status
- release-next-step guidance

### Extend an Existing Project

`intake -> extend -> validate -> publish`

Supported extensions:

- `stream` - Streaming output support for long-running commands
- `repl` - Interactive REPL mode
- `daemon` - Daemon app server mode with JSON-RPC over stdio, TCP, or Unix socket

### Description-Led Change

`intake -> description -> extend -> validate -> publish`

Use this path when a feature change also updates the generated skill's
user-facing contract.

### Validate Only

`intake -> validate -> publish`

This ends in `publish` so the workflow closes with readiness guidance even
when no release side effect is requested.

### Publish / Release Follow-Through

If validation is already current, the workflow can continue directly into the
publish stage. Supported publish modes are:

- `report_only`
- `dry_run`
- `rehearsal`
- `live_release`

## Repository Layout

```text
.
├── cli-forge/
├── cli-forge-intake/
├── cli-forge-description/
├── cli-forge-scaffold/
├── cli-forge-extend/
├── cli-forge-validate/
└── cli-forge-publish/
```

Each stage directory typically contains:

- `SKILL.md`: stage behavior and routing contract
- `planning-brief.md`: constraints and design principles
- `instructions/*.md`: operational steps for that phase
- `templates/*`: files expanded into generated projects or copied as asset-pack
  support files

## Generated Outputs and Boundaries

The scaffold stage generates the baseline project package. At minimum, that
includes:

- `Cargo.toml`
- `src/main.rs`
- `src/lib.rs`
- `src/help.rs`
- `src/context.rs`
- `SKILL.md`
- `tests/cli_test.rs`
- `README.md`

The extend stage adds package-local capability files when needed:

- `src/stream.rs` for `stream`
- `src/repl.rs` for `repl`

The generated package is intentionally narrower than a repository-level release
setup. Repository-owned CI and release automation are not copied into the
generated skill by default.

## Shared Daemon Contract

When a generated CLI includes daemon behavior, cli-forge now treats that
surface as one strict shared contract:

- managed background daemon mode only
- attached foreground execution is out of scope
- one managed daemon instance by default unless the CLI explicitly declares a
  bounded multi-instance model
- recovery stays inside `daemon start`, `daemon stop`, `daemon restart`, and
  `daemon status`
- lifecycle commands return only after `running`, `stopped`, `failed`, or an
  explicit timeout
- runtimes that cannot support managed background daemon mode are out of scope

### Daemon Transport Modes

The daemon supports multiple transport modes for different use cases:

| Mode | Flag | Description |
|------|------|-------------|
| Stdio | `--transport stdio` | JSON-RPC over stdin/stdout for subprocess integration |
| TCP | `--transport tcp --port N` | TCP socket for remote connections |
| Unix Socket | `--transport unix --socket PATH` | Unix domain socket for local connections |

### Daemon App Server Mode

The daemon can operate as an app server with the following capabilities:

- **JSON-RPC 2.0 Protocol**: All communication uses JSON-RPC 2.0 over the selected transport
- **WebSocket Support**: TCP and Unix socket transports use WebSocket framing
- **TLS Encryption**: wss:// support with `--cert-file` and `--key-file` flags
- **Authentication**: Two auth modes supported:
  - `capability-token`: File-based secret token authentication
  - `signed-bearer-token`: JWT bearer token with issuer/audience validation

### CLI Commands for Daemon

```bash
# Start daemon (uses default transport from generated CLI)
<skill-name> daemon start

# Run daemon in specific mode
<skill-name> daemon run --transport tcp --port 9000
<skill-name> daemon run --transport unix --socket /tmp/daemon.sock
<skill-name> daemon run --transport stdio

# With TLS and authentication
<skill-name> daemon run --transport tcp --port 9000 \
    --cert-file /path/to/cert.pem \
    --key-file /path/to/key.pem \
    --ws-auth capability-token \
    --ws-token-file /path/to/token.txt

# Status and control
<skill-name> daemon status
<skill-name> daemon stop
<skill-name> daemon restart
```

### WebSocket Client Integration

Remote clients can connect via WebSocket:

```python
import websocket

ws = websocket.WebSocket()
ws.connect("ws://localhost:9000")
ws.send('{"id": "1", "method": "initialize", "params": {}}')
resp = ws.recv()
ws.close()
```

This contract must stay aligned across templates, README examples, `SKILL.md`,
help text, extension guidance, validation instructions, and generated tests.

## Validation Requirements

The validate stage checks the generated project against the cli-forge planning
brief and runtime conventions. The ruleset covers:

- project structure and naming
- Cargo metadata and required dependencies
- `SKILL.md` contract sections and ordering
- `cargo build`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- plain-text `--help` vs structured `help`
- runtime directory documentation
- Active Context visibility and override behavior
- structured error output
- REPL-specific behavior when REPL is enabled

The current validation instructions define 28 concrete checks and return a
markdown report table with `PASS` / `FAIL` status.

Daemon-capable projects are also expected to document and validate the shared
managed-background daemon contract, including the single-instance default and
the four-command recovery path.

For daemon app server mode, additional validation includes:

- Transport mode selection via `--transport` flag
- TLS certificate and key file handling
- Authentication mode selection via `--ws-auth` flag
- JWT validation with issuer and audience claims when using signed-bearer-token
- WebSocket connection acceptance and message framing
- JSON-RPC request/response cycle over WebSocket

## Publish Modes and Asset Pack

The publish stage is for the target generated repository, not this workflow
repository itself.

`cli-forge-publish/templates/` contains a portable release automation asset
pack that is meant to be copied into the root of a target generated CLI skill
repository. The expected root-level files there include:

- `package.json`
- `package-lock.json`
- `.releaserc.json`
- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/*`
- `templates/*`

Supported modes:

- `report_only`: summarize readiness, blockers, and next actions
- `dry_run`: run semantic-release in dry-run mode
- `rehearsal`: publish into a local rehearsal destination tree
- `live_release`: use the target repository's release workflow

## Quick Start

### 1. Use `cli-forge` as the stable entrypoint

Start with the parent skill when you want automatic routing instead of naming
the stage yourself.

Examples of requests that should enter through `cli-forge`:

- "Create a new cli-forge skill"
- "Add repl to this generated skill"
- "Check whether this skill is compliant"
- "Do a release dry run"

### 2. Verify a newly scaffolded target project

Run these commands from the generated skill project root:

```bash
cargo build
cargo clippy -- -D warnings
cargo fmt --check
cargo test
```

### 2.5 Validate the daemon contract in a generated project

For daemon-capable generated skills, walk the shared contract in this order:

```bash
<skill-name> help daemon --format yaml
<skill-name> daemon start
<skill-name> daemon status --format json
<skill-name> daemon start
<skill-name> daemon restart
<skill-name> daemon stop
```

Expected results:

- help documents only managed background daemon control
- the default instance model is one managed daemon
- `start`, `stop`, and `restart` return only after a terminal outcome or
  explicit timeout
- no-op, failure, and timeout cases keep recovery inside `daemon start`,
  `daemon stop`, `daemon restart`, and `daemon status`

### 3. Run publish-stage quality gates in a target repository

After copying the publish asset pack into the target generated repository root:

```bash
npm ci
npm run release:verify-config
npm run release:quality-gates
GITHUB_TOKEN=<valid token> npm run release:dry-run
```

Do not run those release commands from `cli-forge-publish/` inside this
repository.

### 4. Rehearse publication to a local destination

From the target generated repository root:

```bash
mkdir -p .work/release/destination-rehearsal/skills/other-skill
printf '%s\n' '{"entries":[],"entry_count":0,"format":"json","path":"catalog.json","updated_at":null}' \
  > .work/release/destination-rehearsal/catalog.json
SKILL_RELEASE_DESTINATION_REPOSITORY=.work/release/destination-rehearsal \
npm run release:verify-config
SKILL_RELEASE_DESTINATION_REPOSITORY=.work/release/destination-rehearsal \
node scripts/release/publish-skill-to-target-repo.mjs 0.0.0-local v0.0.0-local "$(git rev-parse HEAD)"
```

## Key References

- `cli-forge/SKILL.md`
- `cli-forge-intake/SKILL.md`
- `cli-forge-description/SKILL.md`
- `cli-forge-scaffold/SKILL.md`
- `cli-forge-extend/SKILL.md`
- `cli-forge-validate/SKILL.md`
- `cli-forge-publish/SKILL.md`
- `cli-forge-scaffold/instructions/new.md`
- `cli-forge-validate/instructions/validate.md`
- `cli-forge-publish/instructions/release/skill-release-runbook.md`
- `cli-forge-publish/templates/README.md`
