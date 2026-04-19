---
name: cli-forge-plan
description: "Plan stage for the cli-forge skill family: define the detailed CLI contract including commands, flags, output formats, stream/repl/daemon capability scope, the daemon app-server contract, and runtime behavior before scaffold or extend stages proceed."
---

# cli-forge Plan

Use this stage to translate the approved design contract into a detailed,
actionable CLI contract that Scaffold, Extend, and Validate will consume as
their authoritative reference.

## Purpose

Define the detailed CLI contract: every command, every flag, every output
format, every behavioral rule. This stage answers the question **"How does the
CLI work?"** while the Design stage upstream answered **"What is this skill?"**

The `cli-plan.yml` produced by this stage is the single source of truth that
Scaffold uses for template expansion, Extend uses for capability pre-checks,
and Validate uses as the compliance baseline.

## Canonical References

- [`./instructions/plan-cli.md`](./instructions/plan-cli.md)
- [`./instructions/daemon-app-server.md`](./instructions/daemon-app-server.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./contracts/cli-plan.yml.tpl`](./contracts/cli-plan.yml.tpl)
- [`./contracts/design-contract.yml.tpl`](./contracts/design-contract.yml.tpl)

## Entry Gate

| #   | Check                                         | Source       |
| --- | --------------------------------------------- | ------------ |
| 1   | `design-contract.yml` exists and is approved  | Design stage |
| 2   | Skill scope is clear from the design contract | Design stage |

## Required Inputs

- Approved `design-contract.yml` from the Design stage
- User requirements for CLI behavior (commands, flags, formats)
- Capability scope decisions (`stream`, `repl`, and `daemon`: in-scope or out-of-scope)

## Workflow

1. Load the approved `design-contract.yml` from `.cli-forge/`.
2. Read [`./planning-brief.md`](./planning-brief.md) to load the shared
   constraints.
3. Follow the detailed steps in
   [`./instructions/plan-cli.md`](./instructions/plan-cli.md).
4. Define the command tree:
   - Primary CLI entrypoint
   - Subcommands (if any)
   - The `help` subcommand (always present)
   - The `daemon` subcommand group when daemon capability is in scope
   - Every command path must be classified as either a runnable leaf command or
     a non-leaf container for child subcommands, never both
5. For each command, lock:
   - Required flags with types, defaults, and descriptions
   - Optional flags with types, defaults, and descriptions
   - Supported output formats
   - Error format (always `structured_stderr`)
6. Lock the invocation hierarchy:
   - Shipped: `<skill-name> ...`
   - Dev: `cargo run -- ...`
   - Release binary: `./target/release/<skill-name> ...`
7. Lock the help behavior as four explicit scenarios:
   - Leaf default: missing required leaf input stays a structured failure, not auto-help
   - Non-leaf default: top-level and non-leaf command paths auto-render human-readable help
   - Flag-driven: `--help` on any command path renders human-readable help
   - Structured: `help` subcommand with `--format`
   - Human-readable help must be man-like, with `NAME`, `SYNOPSIS`,
     `DESCRIPTION`, `OPTIONS`, `FORMATS`, `EXAMPLES`, and `EXIT CODES`
     in that order
8. Lock capability scope — for each of `stream`, `repl`, and `daemon`,
   explicitly state `in_scope` or `out_of_scope`.
9. If daemon is in scope, lock the daemon app-server contract:
   - App-server mode
   - Single-instance default
   - Lifecycle commands: run, start, stop, restart, status
   - Client routing flags and daemonizable commands
   - Local IPC default, optional TCP, and auth expectations
10. Lock runtime directory and Active Context behavior.
11. Generate `.cli-forge/cli-plan.yml` using the format defined in
    [`./contracts/cli-plan.yml.tpl`](./contracts/cli-plan.yml.tpl).
12. Present the CLI plan to the user for approval. Call the runtime's
    dialog-based chooser (e.g., `AskUserQuestion`) with the options `approve
    and continue`, `request changes`, or `stop for now` when it is available.
    If no dialog-based chooser is available, use a numbered text menu with the
    same three options and add `Other:
    <custom response>` as the fallback escape hatch. Accept exact replies `1`,
    `2`, or `3`, or `Other: ...`. If a numeric reply includes additional text,
    ask for clarification before proceeding. Do not require the user to type
    the full option label.

## Outputs

- `.cli-forge/cli-plan.yml` — approved detailed CLI contract

## Exit Gate

| #   | Check                                                                          |
| --- | ------------------------------------------------------------------------------ |
| 1   | Command tree is fully defined                                                  |
| 2   | No command path is both a leaf command and a container for subcommands         |
| 3   | Every command has its flags listed with types and defaults                     |
| 4   | Output format strategy is locked                                               |
| 5   | Help behavior (leaf/non-leaf/`--help`/`help`) is defined with man-like output  |
| 6   | Each optional feature capability is explicitly marked in_scope or out_of_scope |
| 7   | The daemon capability contract is locked or explicitly out of scope            |
| 8   | Runtime directory and Active Context behavior are defined                      |
| 9   | `cli-plan.yml` is generated and approved                                       |

## Guardrails

- **CRITICAL DIRECTIVE TO THE ASSISTANT**: You MUST STOP execution and ask for the user's explicit approval after generating `cli-plan.yml`. Do NOT proceed to the Scaffold stage autonomously. Call the runtime's dialog-based chooser (e.g., `AskUserQuestion`) for approval when it is available, or use the standardized numbered text fallback with `1. approve and continue`, `2. request changes`, `3. stop for now`, and `Other: <custom response>`. Never require the user to type the literal word `approved`.
- Do not change the skill's purpose or positioning here. That work was done in
  the Design stage and is locked in `design-contract.yml`.
- Do not begin implementing code. This stage produces a plan document, not
  source files.
- Every decision in `cli-plan.yml` must be traceable to the planning brief
  constraints.
- Do not define a hybrid command path that is both a runnable leaf and a
  parent for subcommands. Every command path must be exactly one of those
  shapes.
- If `stream`, `repl`, or `daemon` is out of scope, say so explicitly. Do not
  leave the optional feature set undefined.
- The daemon app-server design is documented in
  [`./instructions/daemon-app-server.md`](./instructions/daemon-app-server.md)
  and is the planning source of truth for daemon behavior. If scaffold,
  extend, or validate have not adopted that contract yet, call the mismatch out
  explicitly instead of claiming end-to-end implementation parity.
- When the Extend stage later adds a feature, it must update `cli-plan.yml` to
  reflect the new capability. Plan is the living contract.

## Next Step

Continue with
[`../cli-forge-scaffold/SKILL.md`](../cli-forge-scaffold/SKILL.md) to create
the project from the approved plan.
