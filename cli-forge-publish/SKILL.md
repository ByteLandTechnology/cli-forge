---
name: cli-forge-publish
description: "Publish stage for the cli-forge skill family: manage the GitHub Release and multi-package npm publication automation for a generated skill."
---

# cli-forge Publish

Use this stage when a compliant generated CLI skill needs to adopt standard
release automation, prepare a new version, or execute the paired GitHub
Release + multi-package npm publication flow.

## Purpose

Manage the release pipeline for a generated `cli-forge` skill.

This stage owns the paired publication flow: **repo-native GitHub Release plus
platform-split npm publication** from the same semantic-release event. One
release produces:

- git tag `v<version>` + GitHub Release (archive + sha256 per target)
- 6 per-platform npm packages (`<pkg>-darwin-arm64` / `-darwin-x64` /
  `-linux-arm64` / `-linux-x64` / `-win32-arm64` / `-win32-x64`) carrying the
  native binaries
- 1 main npm package (JS shim, `optionalDependencies` pinned to the same
  version)
- `CHANGELOG.md` entry + `chore(release): <version> [skip ci]` commit

All npm packages publish in one semantic-release run and share the
version with the git tag + GitHub Release. The total count is
`1 + config.targets.length` (default: 7).

## Canonical References

- [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./shared-planning-brief.md`](./shared-planning-brief.md)
- [`./contracts/release-receipt.yml.tpl`](./contracts/release-receipt.yml.tpl)
- [`./templates/`](./templates/)

## Entry Gate

| #   | Check                                                                 | Source        |
| --- | --------------------------------------------------------------------- | ------------- |
| 1   | `validation-report.yml` exists, is fresh, and `result == compliant` or `warning` | Validate      |
| 2   | Target repository path is explicitly known                            | Router / User |
| 3   | Requested publish mode is explicit                                    | Router / User |
| 4   | npm package name, CLI name, and scope decision are explicit           | User          |

Scope decision is a required input at this stage. Ask the user whether the
npm package should publish as **unscoped** (`<name>`) or **scoped**
(`@<scope>/<name>`), and collect the scope string in the latter case. The same
decision applies uniformly to the main package and all six platform packages.

## Required Inputs

- Validation outcome plus provenance snapshot (refusal to publish if `non_compliant`, stale, or provenance-mismatched)
- Publish mode: `report_only`, `dry_run`, or `live_release`
- Source project path
- `cliName` — the CLI binary name
- `packageName` — the main npm package name (with scope prefix if scoped)
- `npmScope` — `null` for unscoped, or the scope string
- `sourceRepository` — `owner/repo` on GitHub

## Workflow

1. Read [`./planning-brief.md`](./planning-brief.md) for publish-specific
   planning constraints.
2. Accept the inbound `validation-report.yml`. Refuse to proceed if validation
   failed; do not re-run validation here.
3. Verify validation freshness before any publish-mode action:
   - compare the current `.cli-forge/design-contract.yml`,
     `.cli-forge/cli-plan.yml`, baseline receipt (`scaffold-receipt.yml` or
     `takeover-receipt.yml`), and optional `extend-receipt.yml` against the
     provenance snapshot recorded in `validation-report.yml`
   - for takeover-adopted baselines, require the report provenance to match the
     current adopted contract + receipt set exactly; do not rely only on the
     aggregate validation result
   - if any required baseline artifact is missing, has a different receipt, or
     takeover recorded contract/baseline rewrites after validation, stop and
     route back to Validate
4. Ask the user for the inputs listed above if not already collected. If the
   mode is `report_only`, skip to step 7.
5. If the skill project lacks the release automation assets, adopt them by
   copying the contents of `./templates/` to the project root. The adopted
   asset pack does NOT contain Scaffold-stage source templates (`*.tpl`).
6. Fill `release/config.json` and `npm/main/package.json` with the collected
   inputs.
7. Verify the target repository has:
   - GitHub Actions enabled on a GitHub-hosted runner with `id-token: write`
   - npm trusted publishing configured on npmjs.com for the main package and
     for each of the 6 platform packages, every entry pointing at `release.yml`
8. Follow the mode path defined in
   [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md):
   - `report_only`: audit readiness only. Read config and repository state,
     report blockers and next actions. Do NOT copy files, fill placeholders,
     or alter the repository in any way.
   - `dry_run`: run `npm run release:rehearse` locally. This builds every
     target, syncs platform packages, and runs `npm publish --dry-run` for
     each. No tag, no GitHub Release, no real npm publication.
   - `live_release`: push to `main` (or `workflow_dispatch`) so
     `.github/workflows/release.yml` drives the end-to-end run.
9. Generate `.cli-forge/release-receipt.yml` from the template at
   [`./contracts/release-receipt.yml.tpl`](./contracts/release-receipt.yml.tpl).

## Outputs

- A verified GitHub Release with archives + checksums (if `live_release`)
- 1 main + N platform npm packages published at the same version (if `live_release`)
- Updated `CHANGELOG.md` and release commit on `main`
- `.cli-forge/release-receipt.yml`

## Exit Gate

| #   | Check                                                                   |
| --- | ----------------------------------------------------------------------- |
| 1   | The requested mode was executed                                         |
| 2   | For `report_only`: no files were written to the target repository       |
| 3   | For `dry_run` / `live_release`: asset pack adopted and filled           |
| 4   | For `live_release`: version, tag, main package, platform packages agree |
| 5   | `release-receipt.yml` generated                                         |

## Guardrails

- All npm packages (main + platforms) MUST share the version chosen by
  semantic-release. The two release scripts enforce this by consuming
  `${nextRelease.version}` from semantic-release hooks.
- Do not bypass stale validation. If the codebase changed since the last
  validation report, route back to Validate.
- For takeover-adopted repositories, freshness checks MUST verify provenance
  against the current contract + receipt set, not just `validation-report.yml`
  aggregate status or timestamp.
- Do not hand-bump `npm/main/package.json#version`, hand-edit `CHANGELOG.md`
  release entries, or create tags outside semantic-release.
- The asset pack's `npm/platforms/` directory is generated at release time.
  Do not check generated platform package directories into source control
  beyond the `.gitkeep` placeholder.

## Next Step

This is a terminal stage.
