# cli-forge-publish Planning Brief

Use this brief as the publish-stage contract when planning or reviewing release
work for a generated CLI skill repository.

## Purpose And Scope

- Treat `cli-forge-publish` as a publish-stage skill, not as the repository
  that is itself being released.
- Plan how a target CLI skill repository adopts and uses the
  `templates/` asset pack.
- Keep release work anchored to the target repository's shipped CLI contract.

## Publish-Stage Contract

Every publish-stage plan must lock these decisions:

- which release mode is in scope: `report_only`, `dry_run`, `rehearsal`, or
  `live_release`
- whether the target repository has already adopted the `templates/`
  asset pack or must do so first
- which target repository path or repository identity is being prepared
- whether validation is current or must be re-run before publish work
- how destination repository configuration and credentials will be supplied

If a plan cannot answer those items, it should route back through
`cli-forge-intake` before continuing.

## Invocation Contract Requirements

Publish planning must preserve one consistent invocation hierarchy for the
target skill:

1. final shipped skill contract: `<skill-name> ...`
2. local development from repository root: `cargo run -- ...`
3. built release binary: `./target/release/<skill-name> ...`

`SKILL.md` must treat the bare command name as the canonical agent-facing
surface. README may document development and built-binary forms, but it must
label them clearly. Validation and release readiness should be based on the
compiled CLI surface, not an ad hoc wrapper.

## Release Automation Boundary

Plans must keep this boundary explicit:

- target skill deliverable:
  - the compiled CLI binary
  - shipped skill docs and runtime behavior
- repository-owned release automation:
  - `package.json`
  - `.releaserc.json`
  - `.github/workflows/release.yml`
  - `release/`
  - `scripts/release/`
  - `templates/` used by release quality gates and rehearsal flows

The asset pack supports the target repository. It is not the runtime surface of
the shipped CLI skill itself.

## Risks To Check Explicitly

- release config placeholders were not replaced in the target repository
- validation is stale but publish work is being attempted anyway
- semantic-release dry-run output is being mistaken for a real release
- destination repository settings or tokens are missing
- target artifact declarations are incomplete
- `SKILL.md`, README, tests, and actual binary invocation drift from the final
  shipped command surface
