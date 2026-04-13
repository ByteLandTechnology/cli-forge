# Plan CLI Instructions

Use this document as the operational source of truth for producing a detailed
CLI plan during the Plan stage.

## Pre-Checks

1. Confirm that `.cli-forge/design-contract.yml` exists and is approved.
2. Load the design contract to inherit the skill name, purpose, and
   positioning.
3. Load `planning-brief.md` from the repository root to ensure all required
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

If the skill has subcommands, list each one. If daemon is in scope, include
the full `daemon` subcommand group: `daemon start`, `daemon stop`,
`daemon restart`, `daemon status`, `daemon run`.

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

Define two distinct help surfaces:

1. **Plain-text help**: `<skill-name> --help` or `<skill-name> <command> --help`
   produces human-readable text on stdout.
2. **Structured help**: `<skill-name> help` or `<skill-name> help <command>`
   produces machine-readable output in the format specified by `--format`.

## Step 5: Lock Capability Scope

For each optional capability, explicitly state whether it is in scope:

| Capability | Status                  | Justification |
| ---------- | ----------------------- | ------------- |
| `stream`   | in_scope / out_of_scope | ...           |
| `repl`     | in_scope / out_of_scope | ...           |
| `daemon`   | in_scope / out_of_scope | ...           |

Do not leave any capability undefined. Explicitly marking something as
`out_of_scope` prevents ambiguity during scaffold and extend.

## Step 6: Lock Daemon Contract (If In Scope)

When daemon is in scope, the following must be defined:

- **Mode**: managed background daemon only (attached foreground is out of
  scope)
- **Instance model**: single instance by default
- **Lifecycle commands**: `daemon start|stop|restart|status`
- **Return behavior**: commands return only after reaching `running`,
  `stopped`, `failed`, or an explicit timeout
- **Recovery**: all recovery stays within the four lifecycle commands
- **Transport modes**: `--transport stdio|tcp|unix`
- **WebSocket**: TCP and Unix Socket transports use WebSocket framing
- **TLS**: `wss://` support with `--cert-file` and `--key-file`
- **Authentication**: `capability-token` and/or `signed-bearer-token`

## Step 7: Lock Runtime Behavior

Define:

- **Runtime directory**: where the skill stores persistent state
  (e.g., `~/.skill-name/`)
- **Active Context**: whether the skill supports Active Context, and if so,
  how context is discovered, persisted, and overridden via `--context`

## Step 8: Generate cli-plan.yml

Use the template at `contracts/cli-plan.yml.tpl` to generate the final plan.
Write the file to `.cli-forge/cli-plan.yml`.

## Step 9: User Approval

Present the complete CLI plan to the user using a dialog-based chooser when
the platform supports it. The plan must be approved before the Scaffold stage
can proceed, but the user must not be required to type an exact phrase or the
literal word `approved`.

## Done Condition

This instruction is complete only when:

- Every command and its flags are fully defined
- Output format strategy is locked with no ambiguity
- Every capability is explicitly scoped
- Daemon contract is locked (if applicable)
- `cli-plan.yml` is written and approved
- The Scaffold stage can proceed using the plan as its sole contract
