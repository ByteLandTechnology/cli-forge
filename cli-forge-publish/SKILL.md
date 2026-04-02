---
name: cli-forge-publish
description: "Publish stage for the cli-forge skill family: prepare, apply, and verify the target-project release automation asset pack for generated CLI skill repositories."
---

# cli-forge Publish

Use this stage when a generated CLI skill project needs a normalized release
workflow, release dry-run guidance, or the documented live publication path.

This stage does not publish the current `cli-forge` skill-family repository.
Instead, it provides a release automation asset pack that should be adopted by
the target CLI skill project's own repository.

## Stage Goal

Finish this stage with one of these outcomes:

- `report_only`: release-readiness guidance without side effects
- `dry_run`: a local semantic-release review from the target project root
- `rehearsal`: a local release rehearsal that prepares repo-native evidence
- `live_release`: the documented target-project GitHub Actions release path

The default release path is always repo-native GitHub Release publication:
version, tag, release page, CLI binaries, and release evidence must all align.
Shared-destination publication may be discussed only as an optional secondary
follow-up after the repo-native contract is healthy.

## Canonical References

- [`./planning-brief.md`](./planning-brief.md)
- [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md)
- [`./templates/README.md`](./templates/README.md)
- [`./templates/.github/workflows/release.yml`](./templates/.github/workflows/release.yml)
- [`./templates/release/skill-release.config.json`](./templates/release/skill-release.config.json)
- [`./templates/package.json`](./templates/package.json)

Read `planning-brief.md` first as the publish-stage contract, then use the
runbook as the operational source of truth for the selected release mode.
Treat `templates/` as the asset pack that will be copied into the target CLI
skill repository root.

## Required Inputs

- the intended release mode:
  - `report_only`
  - `dry_run`
  - `rehearsal`
  - `live_release`
- the current validation status for the target CLI skill project
- the target repository path or repository identity
- confirmation that the target repo owns its own GitHub Release publication
- credentials appropriate to the chosen mode
- optional secondary publication settings only if that follow-up is explicitly
  requested

## Prerequisites

- The target CLI skill project is already scaffolded and validated.
- The target repository has adopted the contents of `templates/` into its
  repository root, or the user explicitly wants guidance for doing so now.
- The target project's final CLI invocation contract is settled:
  bare command name for the shipped skill, `cargo run -- ...` for local
  development, and `./target/release/<skill-name> ...` for a built release
  binary.
- The target repository exposes `scripts/install-current-release.sh` so cloned
  released checkouts can install the matching binary for their checked out tag.

## Workflow

1. Read [`./planning-brief.md`](./planning-brief.md) to lock the publish-stage
   contract, then read
   [`./instructions/release/skill-release-runbook.md`](./instructions/release/skill-release-runbook.md)
   and confirm which release mode the user wants.
2. Confirm whether the target repository has already adopted the `templates/`
   asset pack. If not, provide or apply that adoption step first.
3. Verify the target project's placeholders and release config are set
   correctly, especially skill id, source repository, install-helper path,
   required artifact targets, and release-evidence settings.
4. If no explicit release action was requested, default to `report_only`:
   summarize readiness, blockers, next actions, and which commands should be
   run from the target repository root.
5. For `dry_run`, run the documented package script from the target repository
   root and review the semantic-release outcome without claiming a production
   publish occurred.
6. For `rehearsal`, run the documented rehearsal commands from the target
   repository root and inspect the staged GitHub Release assets,
   `release-evidence.json`, and the publication receipt.
7. For `live_release`, follow the target project's GitHub Actions workflow
   after confirming validation, config, and credentials are current.
8. Keep the final skill contract clean:
   - `SKILL.md` documents the bare command-name invocation
   - README distinguishes local development from the shipped skill surface
   - README and release docs explain clone `->` checkout tag `->`
     `scripts/install-current-release.sh`
   - release evidence points back to the same repo version and GitHub Release
9. If a user explicitly wants secondary shared-destination publication, treat
   it as an optional post-release mirror and keep its wording subordinate to
   the repo-native release path.

## Guardrails

- Do not describe the current skill-family repository as the thing being
  published.
- Do not tell users to run release commands from `cli-forge-publish/`; the
  commands run from the target CLI skill repository root after the asset pack
  is adopted there.
- Do not describe `cargo run -- ...` as the final agent-facing invocation
  contract.
- Do not skip validation when the latest substantive change in the target
  project has not been checked yet.
- Do not imply that local dry-runs or rehearsals created a production release.
- Do not reintroduce shared-destination publication as the default release
  model.

## Done Condition

This stage is complete only when the requested release activity has been
performed or reviewed and the target project has:

- the correct release mode guidance or outcome
- a repo-native version/tag/release/binary/evidence chain
- a documented install helper for cloned released checkouts
- a clear target-repository command surface
- a normalized CLI invocation contract for the shipped skill

## Next Step

- Stop after `report_only`, `dry_run`, or `rehearsal` if the user asked only
  for readiness or review.
- After `live_release`, monitor the resulting GitHub Release assets and confirm
  the published release evidence matches the same repo version and install path.
