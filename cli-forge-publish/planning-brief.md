# cli-forge-publish Planning Brief

Use this brief as the publish-stage contract when planning or reviewing release
work for a generated CLI skill repository.

## Purpose And Scope

- Treat `cli-forge-publish` as a publish-stage skill, not as the repository
  that is itself being released.
- Plan how a target CLI skill repository adopts and uses the `templates/`
  asset pack.
- Keep release work anchored to the target repository's shipped CLI contract.
- Treat repo-native GitHub Release publication as the primary distribution
  surface.
- Keep optional npm publication out of this stage's default flow; the
  `cli-forge-publish-npm` child skill owns the npm package-set boundary.
- Include the generated install helper template
  `templates/templates/install-current-release.sh.tpl` and any repo-facing
  guidance that downstream stages need, including
  `cli-forge-description/instructions/add-feature.md`, when those surfaces are
  touched by release expectations.

## Publish-Stage Contract

Every publish-stage plan must lock these decisions:

- which release mode is in scope: `report_only`, `dry_run`, `rehearsal`, or
  `live_release`
- whether the target repository has already adopted the `templates/` asset pack
  or must do so first
- which target repository path or repository identity is being prepared
- whether validation is current or must be re-run before publish work
- how the repo-native GitHub Release, evidence file, and install helper are
  configured
- whether optional secondary distribution is in scope after the repo-native
  release path and, if it is npm publication, that it is routed to the
  dedicated npm child skill

If a plan cannot answer those items, it should route back through
`cli-forge-intake` before continuing.

## Invocation Contract Requirements

Publish planning must preserve one consistent invocation hierarchy for the
target skill:

1. final shipped skill contract: `<skill-name> ...`
2. local development from repository root: `cargo run -- ...`
3. built release binary: `./target/release/<skill-name> ...`
4. cloned release install path: `./scripts/install-current-release.sh <version>`

`SKILL.md` must treat the bare command name as the canonical agent-facing
surface. README may document development and built-binary forms, but it must
label them clearly. Validation and release readiness should be based on the
compiled CLI surface, not an ad hoc wrapper.

## Release Automation Boundary

Plans must keep this boundary explicit:

- target skill deliverable:
  - the compiled CLI binary
  - shipped skill docs and runtime behavior
  - repo-native GitHub Release assets and release evidence
- repository-owned release automation:
  - `package.json`
  - `.releaserc.json`
  - `.github/workflows/release.yml`
  - `release/`
  - `scripts/release/`
  - `scripts/install-current-release.sh`
  - `templates/` used by release quality gates and rehearsal flows

The asset pack supports the target repository. It is not the runtime surface of
the shipped CLI skill itself.

Optional npm package governance is not part of this child skill's automation
boundary. If the same shipped CLI version will also be distributed through npm,
that package-set work belongs in `cli-forge-publish-npm`.

## Risks To Check Explicitly

- release config placeholders were not replaced in the target repository
- validation is stale but publish work is being attempted anyway
- semantic-release dry-run output is being mistaken for a real release
- repo version, tag, release page, release assets, and release evidence drift
  apart
- cloned released checkouts cannot install the matching binary
- optional secondary publication is being treated like the default path
- npm publication is being mixed into repo-native release guidance without a
  separate child-skill handoff
- `SKILL.md`, README, tests, and actual binary invocation drift from the final
  shipped command surface
