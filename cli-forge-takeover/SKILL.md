---
name: cli-forge-takeover
description: "Recovery stage for the cli-forge skill family: adopt an existing Rust CLI Skill project that lacks cli-forge contracts by reconstructing design/plan artifacts from the current implementation, enforcing gates, and stopping for user decisions on ambiguity."
---

# cli-forge Takeover

Use this stage when a pre-existing Rust CLI Skill project needs to join the
`cli-forge` pipeline but does not yet carry the required `.cli-forge/`
contracts.

## Purpose

Adopt an existing project into `cli-forge` without pretending it was scaffolded
by `cli-forge`.

This stage reconstructs the missing pipeline artifacts from the observed
implementation, documentation, tests, and help surface, then records the
adoption with a dedicated receipt that downstream stages can trust. The exit
path depends on the user's recorded post-adoption objective, on whether
Takeover preserved or rewrote the approved contracts, and on whether the
adopted project already satisfies downstream layout assumptions.

## Canonical References

- [`./instructions/takeover.md`](./instructions/takeover.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./contracts/design-contract.yml.tpl`](./contracts/design-contract.yml.tpl)
- [`./contracts/cli-plan.yml.tpl`](./contracts/cli-plan.yml.tpl)
- [`./contracts/takeover-receipt.yml.tpl`](./contracts/takeover-receipt.yml.tpl)
- [`../cli-forge-plan/instructions/daemon-app-server.md`](../cli-forge-plan/instructions/daemon-app-server.md)

## Entry Gate

| #   | Check                                                                                       | Source          |
| --- | ------------------------------------------------------------------------------------------- | --------------- |
| 1   | `project_path` is known and exists                                                          | Router/User     |
| 2   | `Cargo.toml` is present in the target directory                                             | Filesystem      |
| 3   | The project lacks an approved `cli-forge` contract baseline, or the user requested refresh | Filesystem/User |
| 4   | The user accepts that ambiguities must be resolved before adoption can proceed              | User            |

## Required Inputs

- `project_path`
- The user's current objective after adoption (`validate`, `extend`,
  `publish`, or general pipeline adoption)
- The takeover mode (`first_adoption`, `baseline_establishment`, or
  `contract_refresh`)
- Permission to inspect the implementation, docs, tests, and help output as
  evidence

## Workflow

1. Read [`./planning-brief.md`](./planning-brief.md) and
   [`./instructions/takeover.md`](./instructions/takeover.md).
2. Inspect the existing project to inventory:
   - `Cargo.toml`
   - `SKILL.md`
   - `README.md`
   - `src/`
   - `tests/`
   - observable help and runtime behavior when available
   - optional release/publish assets when present
3. Run the **Discovery Gate**:
   - confirm the purpose, invocation contract, command tree, flag set, output
     formats, help behavior, runtime directories, Active Context surface, and
     optional capabilities are either supported by evidence or reduced to a
     short list of targeted user questions
   - if evidence is missing or contradictory, stop and ask the user before
     generating contracts
4. Ensure `.cli-forge/` exists before any recovered contract is written.
5. Backfill `.cli-forge/design-contract.yml` from the observed project
   surfaces when takeover mode requires missing-contract recovery or explicit
   refresh. If an approved design contract already exists and the current
   takeover mode is baseline establishment only, preserve that approved
   wording instead of regenerating it.
6. Run the **Design Gate**:
   - if purpose, positioning, or shared wording drift across code, docs, and
     help output, present the drift and ask the user which wording is
     authoritative
   - write the contract only after the user approves the reconstructed wording
7. Backfill `.cli-forge/cli-plan.yml` from the observed CLI behavior.
   If an approved plan already exists and the current takeover mode is
   baseline establishment only, preserve the approved plan instead of
   regenerating it.
8. Run the **Plan Gate**:
   - if commands, flags, defaults, capability scope, daemon behavior, or
     runtime conventions are unclear, ask the user before locking the plan
   - preserve actual observed behavior in the reconstructed plan rather than
     silently upgrading the project to a more compliant shape
   - write the plan only after the user approves the reconstructed contract
9. Run the **Baseline Gate**:
   - confirm `.cli-forge/` still exists
   - ensure `.gitignore` ignores `.cli-forge/`
   - generate `.cli-forge/takeover-receipt.yml`
   - do **not** synthesize `scaffold-receipt.yml` or `extend-receipt.yml`
   - record whether `.gitignore` or any other validation-covered baseline
     surface changed during takeover
10. Resolve the next stage from the recorded post-adoption objective:
   - `validate` or general adoption -> hand off to
     [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
   - `publish` -> hand off to Validate unless all of the following are true:
     takeover ran as `baseline_establishment`, it preserved the approved
     `design-contract.yml` and `cli-plan.yml` without rewriting them, it
     preserved validation-covered baseline surfaces such as `.gitignore`
     without changing them, and a fresh `validation-report.yml` already exists
     with aggregate result `compliant` or `warning`; only then continue to
     [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md)
   - `extend` -> hand off to
     [`../cli-forge-extend/SKILL.md`](../cli-forge-extend/SKILL.md) only if the
     adopted project already has the scaffold-compatible files that Extend
     patches directly; otherwise stop and tell the user normalization to that
     layout, or manual feature implementation, is required before template
     expansion can proceed

## Outputs

- `.cli-forge/design-contract.yml`
- `.cli-forge/cli-plan.yml`
- `.cli-forge/takeover-receipt.yml`

## Exit Gate

| #   | Check                                            |
| --- | ------------------------------------------------ |
| 1   | Evidence inventory completed                     |
| 2   | `design-contract.yml` reconstructed and approved |
| 3   | `cli-plan.yml` reconstructed and approved        |
| 4   | `takeover-receipt.yml` written                   |
| 5   | Next stage resolved from the recorded objective  |

## Guardrails

- Never invent behavior that cannot be traced to the implementation or to an
  explicit user decision.
- Reconstructed contracts must record observed current behavior, including
  per-command output formats, error shape, help semantics, runtime overrides,
  and Active Context support. Do not silently substitute cli-forge defaults.
- When code, docs, tests, and help output disagree, stop and ask the user
  instead of choosing silently.
- Use `takeover-receipt.yml` to mark the adopted baseline. Do not fake a
  scaffold origin for a pre-existing project.
- Do not reject a repository solely because `design-contract.yml` and
  `cli-plan.yml` already exist. When the user explicitly requested contract
  refresh, or when a downstream stage needs takeover baseline establishment,
  takeover remains valid and must preserve or refresh the contracts according
  to the recorded takeover mode.
- Do not claim takeover alone makes a project Extend-compatible. Template-based
  feature expansion is allowed only when the adopted repository already matches
  Extend's scaffold-compatible file layout.
- Limit edits to pipeline artifacts plus the minimal repository hygiene needed
  to host them, such as adding `.cli-forge/` to `.gitignore`, unless the user
  explicitly asks for behavioral fixes.
- If takeover rewrites `design-contract.yml`, `cli-plan.yml`, `.gitignore`, or
  any other validation-covered baseline surface, treat any pre-existing
  `validation-report.yml` as stale for direct Publish gating until Validate
  runs again against the updated baseline.
- If daemon behavior is present, compare it against the app-server planning
  contract and call out mismatches explicitly instead of silently normalizing
  the project.
- Do not publish automatically. Takeover only reconstructs contracts and the
  adoption baseline.

## Next Step

Continue based on the recorded post-adoption objective:

- Validate or general adoption ->
  [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
- Publish ->
  [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md) first,
  unless takeover was `baseline_establishment`, the approved contracts were
  preserved without rewriting, validation-covered baseline surfaces were
  preserved without changes, and a fresh compliant/warning validation report
  already exists; only then
  [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md)
- Extend ->
  [`../cli-forge-extend/SKILL.md`](../cli-forge-extend/SKILL.md) only when the
  adopted repository already matches Extend's scaffold-compatible layout;
  otherwise stop and tell the user normalization or manual implementation is
  required
