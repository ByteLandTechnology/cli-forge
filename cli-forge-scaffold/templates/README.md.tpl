# {{SKILL_NAME}}

{{DESCRIPTION}}

This scaffold reuses the approved description contract across Cargo metadata,
`SKILL.md`, README, and help text.

## Build

```sh
cargo build --release
```

The compiled binary will be at `./target/release/{{SKILL_NAME}}`.

## Agent Installation

For agent use, install the CLI via npm:

```sh
npm install -g {{PKG_NAME}}
```

Users on Apple Silicon Macs should ensure Rosetta 2 is installed if using
darwin-x64 packages:

```sh
softwareupdate --install-rosetta
```

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

- leaf commands never auto-render help; validation failures stay structured
- top-level and non-leaf command paths auto-render human-readable help
- `--help` stays human-readable only
- `help` returns structured help in YAML, JSON, or TOML
- human-readable help is man-like and must keep the section order `NAME`,
  `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`, `FORMATS`, `EXAMPLES`, `EXIT CODES`
- runtime directories are separated into `config`, `data`, `state`, `cache`,
  and optional `logs`
- `Active Context` is inspectable and can be persisted or overridden per
  invocation

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

### Human-Readable Help

Top-level invocation and non-leaf invocation automatically print man-like
human-readable help and exit `0`:

```sh
{{SKILL_NAME}}
{{SKILL_NAME}} context
```

### Structured Help

```sh
{{SKILL_NAME}} help run
{{SKILL_NAME}} help context use --format json
```

### `--help`

`--help` always renders the same man-like human-readable help surface, even if
`--format json` or `--format toml` is present.

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

Missing required leaf input stays a structured failure on `stderr`; it does
not auto-render help.

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
prioritizes readability over raw YAML.

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
