# {{SKILL_NAME}}

{{DESCRIPTION}}

This scaffold reuses the approved description contract across Cargo metadata,
`SKILL.md`, README, and help text.

## Build

```sh
cargo build --release
```

The compiled binary will be at `./target/release/{{SKILL_NAME}}`.

## Invocation Layers

The generated CLI uses three different invocation contexts. Keep them distinct:

- Final installed skill contract: `{{SKILL_NAME}} ...`
- Local development from repo root: `cargo run -- ...`
- Built release binary from repo root: `./target/release/{{SKILL_NAME}} ...`

`SKILL.md` documents the final installed contract with the bare command name.
This README may also show the development and release-binary forms for local
verification.

## Runtime Conventions

This scaffold follows the shared cli-forge runtime contract:

- `--help` stays plain text only
- `help` returns structured help in YAML, JSON, or TOML
- runtime directories are separated into `config`, `data`, `state`, `cache`,
  and optional `logs`
- `Active Context` is inspectable and can be persisted or overridden per
  invocation
- daemon-capable skills expose managed background control through
  `daemon start`, `daemon stop`, `daemon restart`, and `daemon status`
- attached foreground execution is out of scope for the daemon contract
- the default daemon model is one managed instance unless the generated skill
  explicitly documents a bounded multi-instance model

## Package Boundary

The generated package includes the baseline skill files plus any package-local
support files required by enabled capabilities. Repository-owned CI workflows,
release scripts, and release automation are not scaffolded into the generated
project by default. If a target repository later adopts the `cli-forge-publish`
stage's bundled release asset pack, those files live at the target repository
root rather than inside the shipped CLI skill package.

Package-local packaging-ready metadata or support fixtures should appear only
when a supported capability or packaging path explicitly requires them.

## Commands

### Plain-text Help

Top-level invocation and non-leaf invocation automatically print plain-text
help and exit `0`:

```sh
{{SKILL_NAME}}
{{SKILL_NAME}} context
```

### Structured Help

```sh
{{SKILL_NAME}} help run
{{SKILL_NAME}} help context use --format json
```

### Runtime Directories

```sh
{{SKILL_NAME}} paths
{{SKILL_NAME}} paths --log-enabled
```

### Active Context

```sh
{{SKILL_NAME}} context show
{{SKILL_NAME}} context use --selector workspace=demo --selector provider=staging
```

### Run Command

Default YAML output:

```sh
{{SKILL_NAME}} run demo-input
```

JSON output:

```sh
{{SKILL_NAME}} run demo-input --format json
```

Per-invocation context override:

```sh
{{SKILL_NAME}} run demo-input --selector provider=preview
```

### Managed Daemon Lifecycle

Daemon-capable generated skills use one shared CLI-first contract:

- only managed background daemon mode is standardized
- recovery stays inside `daemon start|stop|restart|status`
- lifecycle commands return after `running`, `stopped`, `failed`, or an
  explicit timeout
- unsupported runtimes are out of scope rather than treated as fallback
  foreground modes

Examples:

```sh
{{SKILL_NAME}} daemon start
{{SKILL_NAME}} daemon status --format json
{{SKILL_NAME}} daemon restart
{{SKILL_NAME}} daemon stop
```

If a lifecycle command times out or reports failure, use `daemon status` to
inspect the observable state and then continue with `daemon start`,
`daemon stop`, or `daemon restart`.

### Local Development

Use `cargo run -- ...` while iterating without a release build:

```sh
cargo run -- help run
cargo run -- run demo-input --format json
```

### Built Release Binary

After `cargo build --release`, you can verify the compiled binary directly:

```sh
./target/release/{{SKILL_NAME}} help run
./target/release/{{SKILL_NAME}} run demo-input --selector provider=preview
```

### Optional Features

Streaming and REPL support are not enabled in the default scaffold, but they
can be added later with this Skill package's `add-feature` workflow. When REPL
is enabled, REPL help remains plain text only and the default session view
prioritizes readability over raw YAML. These optional features do not replace
the daemon contract: daemon control still belongs to the dedicated `daemon`
subcommands.

## Development

Run tests:

```sh
cargo test
```

Lint:

```sh
cargo clippy -- -D warnings
```

Format check:

```sh
cargo fmt --check
```

## Author

{{AUTHOR}}

---

*Generated: {{CURRENT_DATE}}*
