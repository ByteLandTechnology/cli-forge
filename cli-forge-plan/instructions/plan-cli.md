# Plan CLI Instructions

Use this document as the operational source of truth for producing a detailed
CLI plan during the Plan stage.

## Pre-Checks

1. Confirm that `.cli-forge/design-contract.yml` exists and is approved.
2. Load the design contract to inherit the skill name, purpose, and
   positioning.
3. Load `planning-brief.md` from this Skill package root to ensure all required
   planning decisions are addressed.

## Step 1: Define the Command Tree

Start by listing every command that the CLI will expose:

```yaml
commands:
  - name: "<primary-command>"
    description: "..."
  - name: "help"
    description: "Structured help output"
```

If the skill has subcommands, list each one. If daemon capability is in scope,
also list the `daemon` subcommand group: `daemon run`, `daemon start`,
`daemon stop`, `daemon restart`, and `daemon status`. Plans that enable daemon
should also identify which leaf commands are daemonizable and therefore accept
client-routing flags such as `--via daemon` and `--ensure-daemon`.
Do not define a hybrid path such as a command that both performs its own leaf
action and also owns child subcommands. Each command path must be either a
leaf or a container, never both.

## Step 2: Define Flags Per Command

For each command, list every flag:

| Flag       | Type | Required | Default | Enum Values      | Description      |
| ---------- | ---- | -------- | ------- | ---------------- | ---------------- |
| `--input`  | path | yes      | —       | —                | Input file path  |
| `--format` | enum | no       | yaml    | yaml, json, toml | Output format    |
| `--stream` | bool | no       | false   | —                | Enable streaming |

Flag types: `string`, `path`, `bool`, `enum`, `int`.

## Step 3: Lock Output Format Strategy

Every command that produces structured output must support:

- Default format: YAML (unless the plan justifies otherwise)
- `--format yaml|json|toml` explicit switching
- Structured errors on `stderr` with stable machine-readable fields

Document any exceptions. For example, `--stream --format toml` may be
explicitly unsupported if streaming TOML is impractical.

## Step 4: Lock Help Behavior

Define four explicit help scenarios and record all of them in `cli-plan.yml`:

1. **Leaf default behavior**: invoking a leaf command without required input
   must fail with a structured error in the selected `--format`; it must not
   auto-render help text.
2. **Non-leaf default behavior**: invoking the top level or any non-leaf
   command path without selecting a leaf command must render human-readable
   help on stdout and exit `0`.
3. **`--help` flag behavior**: `<skill-name> --help` or
   `<skill-name> <command> --help` must render human-readable help on stdout
   and exit `0`, regardless of `--format`.
4. **`help` subcommand behavior**: `<skill-name> help` or
   `<skill-name> help <command>` must return structured help in the format
   selected by `--format`.

Human-readable help is not free-form prose. It must be locked as a man-like
surface with these required sections in this exact order:

- `NAME`
- `SYNOPSIS`
- `DESCRIPTION`
- `OPTIONS`
- `FORMATS`
- `EXAMPLES`
- `EXIT CODES`

## Step 5: Lock Capability Scope

For each optional capability, explicitly state whether it is in scope:

| Capability | Status                  | Justification |
| ---------- | ----------------------- | ------------- |
| `stream`   | in_scope / out_of_scope | ...           |
| `repl`     | in_scope / out_of_scope | ...           |
| `daemon`   | in_scope / out_of_scope | ...           |

Do not leave any capability undefined. Explicitly marking something as
`out_of_scope` prevents ambiguity during scaffold and extend.

## Step 6: Lock The Daemon Capability Contract

If daemon capability is in scope, the following must be defined:

- **Enabled flag**: `daemon_contract.enabled: true`
- **Mode**: `app_server`
- **Instance model**: single instance by default
- **Lifecycle commands**: `daemon run|start|stop|restart|status`
- **Daemonizable commands**: which leaf commands may execute through the daemon
- **Client routing**: `--via local|daemon` and `--ensure-daemon`
- **Transports**: local IPC required, TCP opt-in only
- **Auth**: OS permissions for local IPC, mandatory auth for TCP mode
- **RPC**: structured JSON transport, preferably JSON-RPC 2.0
- **Streaming**: how daemonized streaming maps to JSON, YAML, and TOML
- **Runtime artifacts**: the files written beneath `state/daemon/`
- **Return behavior**:
  - `daemon start` returns after `running`, `failed`, or `timeout`
  - `daemon stop` returns after `stopped`, `failed`, or `timeout`
  - `daemon restart` returns after `running`, `failed`, or `timeout`
  - `daemon run` stays attached until stopped or failed
- **Recovery**: client recovery stays within structured daemon lifecycle and
  routed-command errors

If daemon capability is out of scope, set `capabilities.daemon:
out_of_scope`, keep `daemon_contract.enabled: false`, and do not add daemon
commands or daemon-routing flags to the command plan. In `cli-plan.yml`, keep
the daemon section collapsed to the minimal `daemon_contract.enabled: false`
block rather than emitting daemon-only routing or transport fields.

Use the supplemental design note at `instructions/daemon-app-server.md` as the
source of truth for daemon app-server planning details.

## Step 7: Lock Runtime Behavior

Define:

- **Runtime directory family**: document `config`, `data`, `state`, `cache`,
  and optional `logs` separately, along with their default user-scoped
  behavior and override flags
- **Active Context**: whether the skill supports Active Context, and if so,
  how it is inspected (`context show`), persisted (`context use`), and
  overridden per invocation (`--selector`, `--cwd`)

## Step 8: Generate cli-plan.yml

Use the template at `contracts/cli-plan.yml.tpl` to generate the final plan.
Write the file to `.cli-forge/cli-plan.yml`.

## Step 9: User Approval

Present the complete CLI plan to the user using a dialog-based chooser when
a dialog-capable surface is available. The plan must be approved before the
Scaffold stage can proceed, but the user must not be required to type an exact
phrase, the literal word `approved`, or any numbered menu response. If dialog
tooling is unavailable, stop and report the blocker instead of falling back to
text entry.

## Done Condition

This instruction is complete only when:

- Every command and its flags are fully defined
- No command path is both a leaf and a container
- Output format strategy is locked with no ambiguity
- Every optional feature capability is explicitly scoped
- The daemon capability contract is locked or explicitly out of scope
- `cli-plan.yml` is written and approved
- The Scaffold stage can proceed using the plan as its sole contract
