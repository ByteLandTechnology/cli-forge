---
name: cli-forge-validate
description: "Validate stage for the cli-forge skill family: run the 46-check compliance ruleset against an existing skill project and compare any daemon surface against the declared CLI plan."
---

# cli-forge Validate

Use this stage to verify that a generated Rust CLI Skill project complies with
all structural, content, and code constraints before it proceeds to publication
or whenever a fresh audit is needed.

## Purpose

Run the authoritative 46-check compliance ruleset against a scaffolded or
extended project.

This stage acts as the final gatekeeper before release. It consumes the
`cli-plan.yml` to define _what_ should be checked, and produces a structured
`validation-report.yml` detailing the results.

## Canonical References

- [`./instructions/validate.md`](./instructions/validate.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./contracts/validation-report.yml.tpl`](./contracts/validation-report.yml.tpl)
- [`../cli-forge-plan/instructions/daemon-app-server.md`](../cli-forge-plan/instructions/daemon-app-server.md)

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

1. Read [`./planning-brief.md`](./planning-brief.md) to understand the
   baseline planning limits.
2. Execute the rule set defined in
   [`./instructions/validate.md`](./instructions/validate.md). The baseline
   includes 46 checks across these categories:
   - `STRUCT-*` (structure and pipeline-artifact ignore policy)
   - `NAME-*` (directory and package naming)
   - `DEPS-*` and `META-*` (dependencies and Cargo metadata)
   - `SKILL-*` (the `SKILL.md` contract)
   - `BUILD-*` (build, clippy, and format verification)
   - `HELP-*` (plain-text and structured help behavior)
   - `DIR-*` (runtime directory documentation)
   - `CTX-*` (Active Context behavior and precedence)
   - `ERR-*` (structured error behavior)
   - `REPL-*` (REPL behavior when present)
   Daemon expectations are currently validated through plan-aligned help,
   runtime, and error checks plus a daemon-specific narrative overlay whenever
   daemon behavior is present. A dedicated `DAEMON-*` ruleset is future work.
3. For each check, look at the filesystem, code, and test output.
4. If the check requires knowledge of expected CLI behaviors (for example,
   flags, optional feature scope, or runtime conventions), source those
   expectations from `cli-plan.yml`.
5. Tabulate the results. A check is either `PASS`, `WARN` (missing best
   practice but non-blocking), or `FAIL` (blocks release).
6. Determine the final aggregate result:
   - `compliant` (0 fails)
   - `warning` (0 fails, >= 1 warn)
   - `non_compliant` (>= 1 fail)
7. Generate `.cli-forge/validation-report.yml` using the template at
   [`./contracts/validation-report.yml.tpl`](./contracts/validation-report.yml.tpl).
8. Present the validation outcome and any next-stage options using a
   dialog-based chooser. Do not require the user to type an exact phrase to
   continue to Publish, Distribute, or a fix-up stage, and do not present
   numbered options that expect typed input.

## Outputs

- `.cli-forge/validation-report.yml`

## Exit Gate

| #   | Check                                                     |
| --- | --------------------------------------------------------- |
| 1   | All 46 checks executed                                    |
| 2   | Final result aggregated (compliant/warning/non_compliant) |
| 3   | `validation-report.yml` written                           |

## Guardrails

- `WARN` results do not block publication; they highlight where the `cli-forge`
  standard advises a better path but the skill operates safely.
- `FAIL` results MUST block any publication or npm distribution attempt. The
  workflow must return to Scaffold or Extend to correct the issue.
- If `cli-plan.yml` marks daemon behavior `in_scope`, validate the declared
  daemon contract rather than silently downgrading to older managed-daemon
  assumptions. If the project still implements an older surface, call that
  mismatch out explicitly.
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
