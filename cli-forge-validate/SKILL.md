---
name: cli-forge-validate
description: "Validate stage for the cli-forge skill family: run the 29-point compliance ruleset against an existing skill project."
---

# cli-forge Validate

Use this stage to verify that a generated Rust CLI Skill project complies with
all structural, content, and code constraints before it proceeds to publication
or whenever a fresh audit is needed.

## Purpose

Run the authoritative 29-point compliance checks against a scaffolded or
extended project.

This stage acts as the final gatekeeper before release. It consumes the
`cli-plan.yml` to define _what_ should be checked, and produces a structured
`validation-report.yml` detailing the results.

## Canonical References

- [`./instructions/validate.md`](./instructions/validate.md)
- [`../planning-brief.md`](../planning-brief.md)
- [`../contracts/validation-report.yml.tpl`](../contracts/validation-report.yml.tpl)

## Entry Gate

| #   | Check                                             | Source     |
| --- | ------------------------------------------------- | ---------- |
| 1   | `project_path` is known and exists                | Router     |
| 2   | `Cargo.toml` is present in the target directory   | Filesystem |
| 3   | `cli-plan.yml` is present (for behavioral checks) | Filesystem |

## Required Inputs

- Target `project_path`
- `.cli-forge/cli-plan.yml` for expected command/flag definitions

## Workflow

1. Read [`../planning-brief.md`](../planning-brief.md) to understand the
   baseline planning limits.
2. Execute the rule set defined in
   [`./instructions/validate.md`](./instructions/validate.md). The baseline
   includes 29 specific checks grouped into 6 categories:
   - `STRUCT-01` to `STRUCT-04` (Structure, License, Readme, Skill Contract)
   - `CARGO-01` to `CARGO-05` (Cargo Metadata, Rust Edition, Clippy Limits, Build Verification, Release Flags)
   - `HELP-01` to `HELP-04` (Help Output Strategy, Format Alignment, JSON Format Rule, Command Structure)
   - `ACTIVE-01` to `ACTIVE-04` (Active Context Behavior, Precedence, Local Testing Override, Clean Scope)
   - `DAEMON-01` to `DAEMON-08` (Managed Background Rule, Instance Model, Return Boundary, Transport Alignment, Terminal Timeout, Capability Parity, WebSocket Framing, TLS Binding) - Checked only if daemon is in_scope.
   - `RELEAS-01` to `RELEAS-03` (Release Channel Strategy, Default Mechanism, Target-Package Extrusion)
3. For each check, look at the filesystem, code, and test output.
4. If the check requires knowledge of expected CLI behaviors (e.g., flags or
   daemon scope), source those expectations from `cli-plan.yml`.
5. Tabulate the results. A check is either `PASS`, `WARN` (missing best
   practice but non-blocking), or `FAIL` (blocks release).
6. Determine the final aggregate result:
   - `compliant` (0 fails)
   - `warning` (0 fails, >= 1 warn)
   - `non_compliant` (>= 1 fail)
7. Generate `.cli-forge/validation-report.yml` using the template at
   [`../contracts/validation-report.yml.tpl`](../contracts/validation-report.yml.tpl).
8. Present the validation outcome and any next-stage options using a
   dialog-based chooser. Do not require the user to type an exact phrase to
   continue to Publish, Distribute, or a fix-up stage, and do not present
   numbered options that expect typed input.

## Outputs

- `.cli-forge/validation-report.yml`

## Exit Gate

| #   | Check                                                     |
| --- | --------------------------------------------------------- |
| 1   | All 29 checks executed                                    |
| 2   | Final result aggregated (compliant/warning/non_compliant) |
| 3   | `validation-report.yml` written                           |

## Guardrails

- `WARN` results do not block publication; they highlight where the `cli-forge`
  standard advises a better path but the skill operates safely.
- `FAIL` results MUST block any publication or npm distribution attempt. The
  workflow must return to Scaffold or Extend to correct the issue.
- **Do not publish automatically.** Ensure the user has the chance to review the
  validation report before handing off to the Publish stage. Use a dialog-based
  handoff, never replace it with numbered text input, and if dialog tooling is
  unavailable, stop and report the blocker instead of accepting a typed
  fallback response.

## Next Step

- If `compliant` or `warning`, proceed to
  [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md) (or
  `../cli-forge-distribute/SKILL.md` if npm only).
- If `non_compliant`, return to Scaffold, Extend, or Design to fix the
  underlying issues.
