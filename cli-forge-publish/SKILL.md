---
name: cli-forge-publish
description: "Publish stage for the cli-forge skill family: manage the repository-native GitHub Release automation for a generated skill."
---

# cli-forge Publish

Use this stage when a compliant generated CLI skill needs to adopt standard
release automation, prepare a new version, or execute its repository-native
GitHub Release.

## Purpose

Manage the repository-native release pipeline for a generated `cli-forge` skill.

This stage is strictly focused on **repository-native GitHub Release**.
It executes semantic versioning, builds cross-platform binaries, generates
checksums and release evidence, and creates the GitHub Release payload. It does
not publish to npm; that is the distinct responsibility of the Distribute stage.

## Canonical References

- [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`../planning-brief.md`](../planning-brief.md)
- [`../contracts/release-receipt.yml.tpl`](../contracts/release-receipt.yml.tpl)
- [`../templates/publish/`](../templates/publish/)

## Entry Gate

| #   | Check                                                                 | Source        |
| --- | --------------------------------------------------------------------- | ------------- |
| 1   | `validation-report.yml` exists and `result == compliant` or `warning` | Validate      |
| 2   | Target repository path is explicitly known                            | Router / User |
| 3   | Requested publish mode is explicit                                    | Router / User |
| 4   | The command hierarchy `caller -> agent -> skill -> package` is clear  | User          |

## Required Inputs

- Validation outcome (refusal to publish if `non_compliant` or stale)
- Publish mode: `report_only`, `dry_run`, `rehearsal`, or `live_release`
- Source project path

## Workflow

1. Read [`./planning-brief.md`](./planning-brief.md) to load the
   publish-specific planning and execution constraints.
2. Accept the inbound `validation-report.yml`. Do not re-run validation here.
   If validation failed, refuse to proceed.
3. If the skill project lacks the release automation assets, adopt them by
   copying the contents of `../templates/publish/` to the project root. Note:
   this template pack DOES NOT contain project source templates (like
   `main.rs.tpl`); those are consumed during Scaffold.
4. Verify the required target repository configuration (`GITHUB_TOKEN`, write
   access, GitHub Actions enabled).
5. Follow the mode-specific execution path defined in
   [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md):
   - `report_only`: Validate environment, report pending version changes, do
     not alter the repository.
   - `dry_run`: Execute local packaging and test hooks but do not push tags or
     communicate with external registries.
   - `rehearsal`: Verify release workflow execution against a staging
     environment or a release branch without affecting the production tag
     line.
   - `live_release`: Execute the permanent semantic release hook, tagging the
     repository, creating the GitHub Release, and uploading the binary asset
     pack.
6. Verify the outcome of the requested mode (e.g., verifying the tag and assets
   were created for `live_release`).
7. Generate `.cli-forge/release-receipt.yml` using the template at
   [`../contracts/release-receipt.yml.tpl`](../contracts/release-receipt.yml.tpl).

## Outputs

- A verified GitHub Release with metadata and binaries (if `live_release`)
- `scripts/install-current-release.sh` generated in target
- `.cli-forge/release-receipt.yml`

## Exit Gate

| #   | Check                                                              |
| --- | ------------------------------------------------------------------ |
| 1   | The requested mode (report, dry_run, rehearsal, live) was executed |
| 2   | Code base and automation assets are correctly aligned              |
| 3   | Asset pack was adopted if it was missing                           |
| 4   | Version tag and evidence match                                     |
| 5   | `release-receipt.yml` generated                                    |

## Guardrails

- This stage is exclusively for the repository-native GitHub Release. It must
  explicitly enforce that boundary by refusing to implement npm publication
  directives.
- The `publish/` asset pack copied to the target root must not contain `.tpl`
  source files or `package-lock.json`. Scaffold assets live in
  `templates/scaffold`.
- Do not bypass stale validation. If the codebase changed since the last
  validation report, route back to Validate.

## Next Step

- If the design contract specified optional npm distribution, continue to
  [`../cli-forge-distribute/SKILL.md`](../cli-forge-distribute/SKILL.md).
- Otherwise, this is a terminal stage.
