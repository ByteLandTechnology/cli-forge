---
name: cli-forge-publish-npm
description: "Publish stage for the cli-forge skill family: review, prepare, and guide optional npm publication for the shipped CLI command without replacing the target repository's repo-native GitHub Release flow."
---

# cli-forge Publish npm

Use this stage when a generated CLI skill repository wants an optional npm
distribution path for the shipped CLI command.

This stage does not publish the current `cli-forge` skill-family repository,
and it does not replace the target repository's repo-native GitHub Release
workflow. Its job is to guide or execute npm publication for the shipped CLI
command after the target repository's release contract is clear.

## Stage Goal

Finish this stage with one of these outcomes:

- `report_only`: npm readiness guidance without side effects
- `dry_run`: package-set review and packaging dry-run guidance without claiming
  a production npm publish occurred
- `live_publish`: the documented npm publication path for the coordinating
  package plus every required platform package

Repo-native GitHub Release remains the default distribution model:
repository version, tag, GitHub Release page, CLI binaries, and release
evidence stay primary. npm publication is an explicit optional follow-up
channel for the same shipped CLI command.

## Canonical References

- [`./planning-brief.md`](./planning-brief.md)
- [`./instructions/release/npm-publish-runbook.md`](./instructions/release/npm-publish-runbook.md)
- [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md)
- [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)

Read `planning-brief.md` first as the npm publish-stage contract, then use the
runbook as the operational source of truth for the selected npm publication
path.

## Required Inputs

- the intended npm publication mode:
  - `report_only`
  - `dry_run`
  - `live_publish`
- the current validation status for the target CLI skill repository
- the target repository path or repository identity
- confirmation that the repo-native release path remains the default release
  model for this target repository
- the authoritative released skill version sourced from the target
  repository's released tag and matching release evidence such as
  `release-evidence.json`
- the coordinating package name for the user-facing npm install surface
- the full set of required platform package names and supported targets
- package ownership or registry access context for the coordinating package and
  every required platform package
- the user-facing npm install surface and how it resolves the correct
  platform-specific package

## Prerequisites

- The target CLI skill repository is already scaffolded and validation is
  current.
- The target repository's repo-native release posture is explicit enough to
  identify the released tag and matching release evidence record for the npm
  package set.
- The shipped CLI invocation contract is settled:
  bare command name for the shipped skill, `cargo run -- ...` for local
  development, and `./target/release/<skill-name> ...` for a built release
  binary.
- One coordinating npm package and one platform-specific package per supported
  target are named clearly.
- The coordinating package version and every required platform-package version
  are expected to match the same authoritative released skill version.
- The user-facing npm install surface is documented clearly enough that users
  install the coordinating package, while the package set resolves the correct
  platform package for the supported target.

## Workflow

1. Read [`./planning-brief.md`](./planning-brief.md) to lock the npm
   publication contract, then read
   [`./instructions/release/npm-publish-runbook.md`](./instructions/release/npm-publish-runbook.md)
   and confirm whether the user wants `report_only`, `dry_run`, or
   `live_publish`.
2. Confirm the request is actually about optional npm publication for the
   shipped CLI command. If the user wants only the target repository's
   repo-native release path, route back to
   [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md).
3. Verify that validation is current and that the target repository context is
   explicit. If not, route back through
   [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md) or
   `cli-forge-intake`.
4. Confirm the authoritative released skill version using the target
   repository's released tag and matching release evidence. Treat that version
   as the source of truth for the npm package set.
5. Review the coordinating package, required platform packages, supported
   target matrix, ownership, and user-facing install path:
   - the coordinating package is the user-facing npm entry surface
   - each required platform target maps to one platform-specific package
   - the user-facing install story explains how the correct platform package is
     resolved
6. Check the version chain. npm publication is blocked if any of these differ:
   - released skill version from the repo-native released tag
   - matching release-evidence version
   - coordinating package version
   - any required platform-package version
7. If no explicit npm side effect was requested, default to `report_only`:
   summarize readiness, blockers, required next actions, and whether the
   repository should stay repo-native only for this release.
8. For `dry_run`, review the target repository's documented npm packaging or
   publish-dry-run commands without claiming a production npm publish occurred.
   Keep orchestration anchored to the target repository root even if individual
   package manifests live in workspaces or subdirectories.
9. For `live_publish`, follow the target repository's documented npm release
   path only after the package set is complete, ownership is confirmed, and the
   authoritative version chain is aligned. Publish required platform packages
   before the coordinating package when the packaging model depends on them.
10. Keep the final boundary explicit:
   - repo-native GitHub Release remains the default release contract
   - npm publication is optional CLI distribution only
   - the coordinating package and platform packages belong to the same released
     CLI version

## Guardrails

- Do not describe npm publication as publishing the target repository itself.
- Do not use this stage to replace or weaken the target repository's
  repo-native GitHub Release contract.
- Do not bypass validation or target-repository context checks before npm
  publication work begins.
- Do not allow the coordinating package or any required platform package to
  drift from the authoritative released skill version.
- Do not describe one cross-platform binary npm package as the required model;
  use one coordinating package plus separate platform-specific packages.
- Do not treat unclear package ownership, missing platform coverage, or unclear
  install guidance as ready-to-publish states.

## Done Condition

This stage is complete only when the requested npm publication review or
publish activity has been performed and the target project has:

- a clear npm readiness or publication outcome
- one coordinating package plus a complete required platform-package set
- a user-facing npm install story that stays distinct from the repo-native
  clone-first install flow
- an authoritative version chain anchored to the released tag and matching
  release evidence
- explicit wording that repo-native GitHub Release remains the default release
  model

## Next Step

- Stop after `report_only` or `dry_run` if the user asked only for readiness
  or review.
- If the user wants both repo-native release publication and npm publication,
  keep repo-native release primary and use this stage as the explicit follow-up
  path for the same shipped CLI version.
