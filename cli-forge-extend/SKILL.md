---
name: cli-forge-extend
description: "Extend stage for the cli-forge skill family: add stream or repl features to an existing scaffolded skill project or to a takeover-adopted project that already matches the scaffold-compatible layout."
---

# cli-forge Extend

Use this stage when a generated Rust CLI Skill project, or a takeover-adopted
project that already matches the scaffold-compatible file layout, requires
optional capabilities like JSON streaming (`stream`) or interactive terminal
sessions (`repl`).

## Purpose

Add optional feature modules to an existing project.

This stage expands feature-specific templates, injects the necessary hooks into
the main program flow, updates tests, and synchronizes the capability state
back to the `cli-plan.yml` contract.

## Canonical References

- [`./instructions/add-feature.md`](./instructions/add-feature.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./contracts/extend-receipt.yml.tpl`](./contracts/extend-receipt.yml.tpl)
- [`./templates/`](./templates/)

## Entry Gate

| #   | Check                                                     | Source      |
| --- | --------------------------------------------------------- | ----------- |
| 1   | Target project directory exists                           | Filesystem  |
| 2   | Scaffold baseline exists, or takeover baseline is scaffold-compatible | Filesystem  |
| 3   | Requested feature is `stream` or `repl`                   | User/Router |
| 4   | Feature is explicitly marked `in_scope` in `cli-plan.yml` | Plan        |
| 5   | Feature is not already added (idempotency check)          | Filesystem  |

## Required Inputs

- `project_path`
- `feature` (`stream` or `repl`)
- Details for updating the `cli-plan.yml`

## Workflow

1. If the project lacks both `scaffold-receipt.yml` and
   `takeover-receipt.yml`, stop and route back to Takeover for baseline
   establishment or refresh. Treat that path as the supported way to create a
   takeover baseline for an existing project whose contracts may already
   exist, not as a first-time-only adoption path.
2. If the project only has `takeover-receipt.yml`, continue only when the
   adopted repository already contains the scaffold-compatible files this stage
   patches directly and the overlay-required API/dependency surface:
   `Cargo.toml`, `SKILL.md`, `src/main.rs`, `src/lib.rs`, `src/help.rs`, and
   `src/context.rs`; `tests/cli_test.rs` should also be present when the
   generated feature workflow is expected to patch the integration tests
   directly. `src/context.rs` must expose the scaffold-style context/runtime
   helpers the overlays call, including `resolve_runtime_locations`. For REPL
   compatibility, the adopted dependency surface must also already include
   `rustyline` plus the context API consumed by `repl.rs.tpl`. If those files
   or required surfaces are missing, stop and tell the user normalization or
   manual implementation is required; do not route back to Takeover.
3. If the feature is missing or listed as `out_of_scope` in `cli-plan.yml`,
   route back to the Plan stage to update the CLI contract first. Do not add
   code before the plan allows it.
4. Follow [`./instructions/add-feature.md`](./instructions/add-feature.md).
5. Expand exactly the requested templates from `./templates/`
   (e.g., `stream.rs.tpl` or `repl.rs.tpl`).
6. Run integration updates safely:
   - add the relevant flags to the args struct
   - wire the subsystem branch into the `match` loop
   - add matching struct format assertions in tests
   - update standard `SKILL.md` boundaries, help text, and README wording
7. Require explicit compiler safety at every step using `cargo clippy`.
8. Run the required verification boundaries from the generated project root:
   - `cargo build`
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
9. Ensure `cli-plan.yml` correctly reflects the added capability as `in_scope`.
10. Generate `.cli-forge/extend-receipt.yml` using the template at
   [`./contracts/extend-receipt.yml.tpl`](./contracts/extend-receipt.yml.tpl).

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

- Use templates EXCLUSIVELY from the bundled `./templates/` directory for this
  stage. Do not reach outside this skill package for extension templates.
- If the feature being added contradicts the `cli-plan.yml`, update the plan
  first. Scaffolded code and the CLI plan must remain perfectly aligned.
- A takeover-adopted project remains takeover-adopted even after extension.
  Keep `takeover-receipt.yml` as the baseline marker and add
  `extend-receipt.yml` as the feature-change record, but only after confirming
  the repository already matches the scaffold-compatible file layout and
  overlay-required API/dependency surface that this stage patches directly.
- If a takeover-adopted project does not have the scaffold-compatible files or
  overlay-required API/dependency surface this stage expects, stop and tell the
  user normalization or manual feature implementation is required. Do not route
  that case back through Takeover.
- If a project already has `design-contract.yml` and `cli-plan.yml` but lacks
  both baseline receipts, route to Takeover specifically for baseline
  establishment or contract refresh so the missing adopted baseline can be
  recorded without pretending the project was scaffolded.
- If the target project already declares daemon behavior in `cli-plan.yml`,
  preserve that documented daemon surface. `stream` and `repl` overlays must
  not silently rewrite daemon commands, routing flags, transport choices, or
  recovery semantics.
- Refuse to add unsupported features through this stage. This stage only
  handles `stream` and `repl`.
- The planned daemon app-server capability is documented in
  [`../cli-forge-plan/instructions/daemon-app-server.md`](../cli-forge-plan/instructions/daemon-app-server.md)
  but is not yet implemented by this stage.

## Next Step

Continue with [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
to run the full compliance rule set.
