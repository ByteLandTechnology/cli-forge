---
name: cli-forge-validate
description: "Validate stage for the cli-forge skill family: run the 47-check compliance ruleset against a scaffolded, takeover-adopted, or extended skill project and compare any daemon surface against the declared CLI plan."
---

# cli-forge Validate

Use this stage to verify that a generated or takeover-adopted Rust CLI Skill
project complies with all structural, content, and code constraints before it
proceeds to publication or whenever a fresh audit is needed.

## Purpose

Run the authoritative 47-check compliance ruleset against a scaffolded,
takeover-adopted, or extended project.

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
   includes 47 checks across these categories:
   - `STRUCT-*` (structure and pipeline-artifact ignore policy)
   - `NAME-*` (directory and package naming)
   - `DEPS-*` and `META-*` (dependencies and Cargo metadata)
   - `SKILL-*` (the `SKILL.md` contract)
   - `BUILD-*` (build, clippy, and format verification)
   - `TREE-*` (command-tree contract integrity)
   - `HELP-*` (leaf/non-leaf/`--help`/`help` behavior plus man-like formatting)
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
   The report must snapshot the contract/receipt provenance Publish will later
   compare against the current `.cli-forge/` baseline set.
8. Present the validation outcome and any next-stage options. When there are
   2 or 3 legal next-stage options, call the runtime's dialog-based chooser
   (e.g., `AskUserQuestion`) with those options when it is available. If no
   dialog-based chooser is available, or if only one legal follow-up path remains, present a numbered
   text menu with 1 to 3 next-stage
   options using the smallest valid set. When only one follow-up path remains
   legal, show a single `1.` option for that path. Put the recommended path
   first and add `Other: explain a different follow-up request` as an escape
   hatch. Accept only the exact digits that correspond to the numbered options
   actually shown, or `Other: ...`. Do not auto-map plain-language replies
   onto numbered options. If a numeric reply includes additional text, or if an
   `Other:` reply conflicts with the validation guardrails, ask for
   clarification before proceeding. Do not require the user to type an exact
   phrase to continue to Publish or a fix-up stage.

## Outputs

- `.cli-forge/validation-report.yml`

## Exit Gate

| #   | Check                                                     |
| --- | --------------------------------------------------------- |
| 1   | All 47 checks executed                                    |
| 2   | Final result aggregated (compliant/warning/non_compliant) |
| 3   | `validation-report.yml` written                           |

## Guardrails

- `WARN` results do not block publication; they highlight where the `cli-forge`
  standard advises a better path but the skill operates safely.
- `FAIL` results MUST block any publication or npm distribution attempt. The
  workflow must return to Takeover, Scaffold, Extend, or Design to correct the
  issue.
- If `cli-plan.yml` marks daemon behavior `in_scope`, validate the declared
  daemon contract rather than silently downgrading to older managed-daemon
  assumptions. If the project still implements an older surface, call that
  mismatch out explicitly.
- If a pre-existing project lacks `.cli-forge/cli-plan.yml`, do not improvise a
  plan inside Validate. If `.cli-forge/design-contract.yml` already exists,
  route to Plan so the approved design wording stays authoritative. Otherwise,
  route to Takeover to reconstruct the missing baseline.
- **Do not publish automatically.** Ensure the user has the chance to review the
  validation report before handing off to the Publish stage. Use the runtime's dialog-based chooser
  (e.g., `AskUserQuestion`) for the handoff when there are 2 or 3 legal
  next-stage options. If no dialog-based chooser is available, or if only one
  legal follow-up path
  remains, use
  the standardized numbered text fallback with `Other: explain a different
  follow-up request`.

## Next Step

- If `compliant` or `warning`, proceed to
  [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md).
- If `non_compliant`, return to Takeover, Scaffold, Extend, or Design to fix the
  underlying issues.
