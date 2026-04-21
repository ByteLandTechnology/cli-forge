---
name: '{{SKILL_NAME}}'
description: '{{DESCRIPTION}}'
---

# {{SKILL_NAME}}

## Description

{{DESCRIPTION}}

This generated Skill reuses the approved description contract across Cargo
metadata, `SKILL.md`, README, and help text.

## Prerequisites

- A working Rust toolchain (`rustup`, `cargo`) to compile and test the binary.
- No additional system dependencies are required for the default scaffold.
- The optional REPL feature adds interactive history and completion using the
  Rust crate ecosystem only.

## Agent Integration

### Installation

The CLI is distributed as an npm package. Agents should install it globally:

```bash
npm install -g {{PKG_NAME}}
```

The matching native binary ships in a per-platform npm package selected
automatically via `optionalDependencies`. No postinstall download required.

### Supported Platforms

| Platform    | Architecture |
| ----------- | ------------ |
| darwin      | arm64, x64   |
| linux       | arm64, x64   |
| win32       | arm64, x64   |

### Invocation

Agents invoke the CLI using the `Bash` tool with the bare command name.
Do not use `cargo run`, `./target/release/{{SKILL_NAME}}`, or other
developer-local forms in agent workflows.

```bash
{{SKILL_NAME}} [OPTIONS] <COMMAND>
```

### Version Alignment

The SKILL version and the CLI version are synchronized. Before invoking,
verify version compatibility:

```bash
{{SKILL_NAME}} --version
```

If the installed CLI version does not match the SKILL version, reinstall:

```bash
npm install -g {{PKG_NAME}}@{{VERSION}}
```

### Availability Check

Before the first invocation in a session, agents should verify the CLI is
on `PATH`:

```bash
which {{SKILL_NAME}} || npm install -g {{PKG_NAME}}
```

## Invocation

```text
{{SKILL_NAME}} [OPTIONS] <COMMAND>
{{SKILL_NAME}} help [COMMAND_PATH ...] [--format yaml|json|toml]
{{SKILL_NAME}} run [OPTIONS] <INPUT>
{{SKILL_NAME}} paths [OPTIONS]
{{SKILL_NAME}} context <show|use> [OPTIONS]
```

The canonical agent-facing contract uses the bare command name shown above.
`cargo run -- ...` and `./target/release/{{SKILL_NAME}} ...` are local
developer execution forms and should be documented in `README.md`, not treated
as the final installed skill interface.

### Global Options

| Flag              | Type                       | Default                   | Description                                                        |
| ----------------- | -------------------------- | ------------------------- | ------------------------------------------------------------------ |
| `--format`, `-f`  | `yaml` \| `json` \| `toml` | `yaml`                    | Structured output format for one-shot commands and structured help |
| `--help`, `-h`    | —                          | —                         | Man-like human-readable help only; never emits YAML/JSON/TOML      |
| `--config-dir`    | `PATH`                     | platform default          | Override the configuration directory                               |
| `--data-dir`      | `PATH`                     | platform default          | Override the durable data directory                                |
| `--state-dir`     | `PATH`                     | derived from data         | Override the runtime state directory                               |
| `--cache-dir`     | `PATH`                     | platform default          | Override the cache directory                                       |
| `--log-dir`       | `PATH`                     | `state/logs` when enabled | Override the optional log directory                                |
| `--version`, `-V` | —                          | —                         | Print version and exit                                             |

### Commands

| Command        | Kind | Purpose                                                      |
| -------------- | ---- | ------------------------------------------------------------ |
| `help`         | leaf | Return structured help for the requested command path        |
| `run`          | leaf | Execute the generated leaf command                           |
| `paths`        | leaf | Inspect config/data/state/cache and optional log directories |
| `context show` | leaf | Display the current Active Context and effective values      |
| `context use`  | leaf | Persist selectors or ambient cues as the Active Context      |

## Input

- The scaffolded CLI does not read default-mode input from `stdin`.
- `run` requires one positional `<INPUT>` argument.
- `context use` accepts one or more `--selector KEY=VALUE` flags and may also
  accept `--cwd PATH` as an ambient cue.
- `help` accepts an optional command path such as `run` or `context use`.

## Output

Standard command results are written to `stdout`. Errors and diagnostics are
written to `stderr`.

### Help Channels

- Leaf commands never auto-display help. Missing required input stays a
  structured validation failure in the selected output format.
- Top-level invocation and non-leaf invocation (for example `context`) display
  man-like human-readable help automatically and exit `0`.
- `--help` is the man-like human-readable help channel. It always prints text
  and exits `0`, regardless of `--format`.
- `help` is the structured help channel. It supports `yaml`, `json`, and
  `toml`, with YAML as the default.
- Human-readable help is a required man-like surface with these sections in
  order: `NAME`, `SYNOPSIS`, `DESCRIPTION`, `OPTIONS`, `FORMATS`, `EXAMPLES`,
  `EXIT CODES`.

### Structured Results

The default one-shot result format is YAML.

Example `run` result:

```yaml
status: ok
message: Hello from {{SKILL_NAME}}
input: demo-input
effective_context:
  workspace: demo
```

Example `paths` result:

```yaml
config_dir: /home/user/.config/{{SKILL_NAME}}
data_dir: /home/user/.local/share/{{SKILL_NAME}}
state_dir: /home/user/.local/share/{{SKILL_NAME}}/state
cache_dir: /home/user/.cache/{{SKILL_NAME}}
scope: user_scoped_default
override_mechanisms:
  - --config-dir
  - --data-dir
  - --state-dir
  - --cache-dir
  - --log-dir
```

### Runtime Directories and Active Context

- `paths` exposes the runtime directory family: `config`, `data`, `state`,
  `cache`, and optional `logs`.
- Defaults are user-scoped unless explicitly overridden.
- `context show` exposes the persisted and effective Active Context.
- Explicit per-invocation selectors on `run` override the persisted Active
  Context for that invocation only.

### Optional Features

- Streaming may be added later with `--stream`.
- REPL mode may be added later with `--repl`. When enabled, REPL help remains
  plain text only and the default REPL presentation is human-oriented.
- Package-local packaging-ready metadata or support fixtures may be added by a
  supported capability later, but repository-owned CI workflows and release
  automation are not copied into generated skill packages by default. If the
  target project later adopts the `cli-forge-publish` release asset pack,
  those files belong at repository root rather than inside the shipped skill
  package.

## Errors

| Exit Code | Meaning                              |
| --------- | ------------------------------------ |
| `0`       | Success or human-readable help       |
| `1`       | Unexpected runtime failure           |
| `2`       | Structured usage or validation error |

Structured errors preserve the selected output format and include at least
stable `code` and `message` fields.

Example structured error (`--format json`):

```json
{
  "code": "run.missing_input",
  "message": "the run command requires <INPUT>; use --help for man-like human-readable help",
  "source": "leaf_validation",
  "format": "json"
}
```

## Examples

Human-readable discovery:

```text
$ {{SKILL_NAME}}
NAME
  {{SKILL_NAME}} - {{DESCRIPTION}}
```

`--help` discovery:

```text
$ {{SKILL_NAME}} run --help
NAME
  {{SKILL_NAME}} run - Execute the generated leaf command
```

Structured help:

```text
$ {{SKILL_NAME}} help run --format yaml
```

Persist Active Context:

```text
$ {{SKILL_NAME}} context use --selector workspace=demo --selector provider=staging
```

Run with one explicit override:

```text
$ {{SKILL_NAME}} run demo-input --selector provider=preview
```

---

_Created: {{CURRENT_DATE}}_
