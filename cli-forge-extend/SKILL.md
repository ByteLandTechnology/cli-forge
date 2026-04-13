---
name: cli-forge-extend
description: "Extend stage for the cli-forge skill family: add stream, repl, or daemon features to an existing skill project."
---

# cli-forge Extend

Use this stage when a generated Rust CLI Skill project already exists, but
requires optional capabilities like JSON streaming (`stream`), interactive
terminal sessions (`repl`), or managed background service behavior (`daemon`).

## Purpose

Add optional feature modules to an existing project.

This stage expands feature-specific templates, injects the necessary hooks into
the main program flow, updates tests, and synchronizes the capability state
back to the `cli-plan.yml` contract.

## Canonical References

- [`./instructions/add-feature.md`](./instructions/add-feature.md)
- [`../planning-brief.md`](../planning-brief.md)
- [`../contracts/extend-receipt.yml.tpl`](../contracts/extend-receipt.yml.tpl)
- [`../templates/extensions/`](../templates/extensions/)

## Entry Gate

| #   | Check                                                     | Source      |
| --- | --------------------------------------------------------- | ----------- |
| 1   | Target project directory exists                           | Filesystem  |
| 2   | Scaffold baseline is complete                             | Filesystem  |
| 3   | Requested feature is `stream`, `repl`, or `daemon`        | User/Router |
| 4   | Feature is explicitly marked `in_scope` in `cli-plan.yml` | Plan        |
| 5   | Feature is not already added (idempotency check)          | Filesystem  |

## Required Inputs

- `project_path`
- `feature` (`stream`, `repl`, or `daemon`)
- Details for updating the `cli-plan.yml`

## Workflow

1. If the feature is missing or listed as `out_of_scope` in `cli-plan.yml`,
   route back to the Plan stage to update the CLI contract first. Do not add
   code before the plan allows it.
2. Follow [`./instructions/add-feature.md`](./instructions/add-feature.md).
3. Expand exactly the requested templates from `../templates/extensions/`
   (e.g., `stream.rs.tpl` or `repl.rs.tpl`). For the `daemon` feature, follow
   the specific integration contract documented in the reference implementation
   at `../templates/extensions/daemon/`.
4. Run integration updates safely:
   - add the relevant flags to the args struct
   - wire the subsystem branch into the `match` loop
   - add matching struct format assertions in tests
   - update standard `SKILL.md` boundaries, help text, and README wording
5. Require explicit compiler safety at every step using `cargo clippy`.
6. Run the required verification boundaries from the generated project root:
   - `cargo build`
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
7. Ensure `cli-plan.yml` correctly reflects the added capability as `in_scope`.
8. Generate `.cli-forge/extend-receipt.yml` using the template at
   [`../contracts/extend-receipt.yml.tpl`](../contracts/extend-receipt.yml.tpl).

## Outputs

- Extended source code and documentation
- Updated `.cli-forge/cli-plan.yml`
- `.cli-forge/extend-receipt.yml`

## Exit Gate

| #   | Check                                           |
| --- | ----------------------------------------------- |
| 1   | Feature files generated from template           |
| 2   | Source, README, help, and tests correctly wired |
| 3   | `cli-plan.yml` updated and synced               |
| 4   | `cargo build` passes                            |
| 5   | `cargo clippy -- -D warnings` passes            |
| 6   | `cargo fmt --check` passes                      |
| 7   | `cargo test` passes                             |
| 8   | `extend-receipt.yml` generated                  |

## Guardrails

- Use templates EXCLUSIVELY from `../templates/extensions/`. Do not maintain
  local copies of templates here.
- If the feature being added contradicts the `cli-plan.yml`, update the plan
  first. Scaffolded code and the CLI plan must remain perfectly aligned.
- Refuse to add unsupported features through this stage. This stage only handles
  `stream`, `repl`, and `daemon`.

## Next Step

Continue with [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
to run the full compliance rule set.
