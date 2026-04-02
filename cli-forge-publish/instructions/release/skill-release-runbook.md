# Skill Release Runbook

## Purpose

This runbook defines the supported release-facing behavior for target CLI skill
repositories that adopt the `cli-forge-publish` release automation asset pack.

The current `cli-forge` skill-family repository is the source of this release
pattern. It is not the thing being published by these instructions.

## Asset Pack Model

`cli-forge-publish/templates/` is a portable asset pack. Its contents
are meant to be copied into the root of a target CLI skill repository.

The target repository should receive these files at its own root:

- `package.json`
- `package-lock.json`
- `.releaserc.json`
- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/*`
- `templates/*`

The `templates/` directory inside the asset pack is repository-owned support
data for release quality gates and rehearsal flows. It is not the target
project's runtime deliverable.

## Command Execution Context

Every command in this runbook is executed from the target CLI skill repository
root after the contents of `templates/` have been copied there.

Examples:

```bash
npm ci
npm run release:verify-config
npm run release:quality-gates
GITHUB_TOKEN=<valid token> npm run release:dry-run
```

Do not run these commands from `cli-forge-publish/` inside this skill-family
repository. They are only supported from the target CLI skill repository root
after the asset pack has been copied there.

## CLI Invocation Contract

Release readiness assumes the target project documents and tests one consistent
invocation hierarchy:

1. Final shipped skill contract: `<skill-name> ...`
2. Local development from repo root: `cargo run -- ...`
3. Built release binary: `./target/release/<skill-name> ...`

`SKILL.md` should describe the first form as the canonical agent-facing
contract. README may additionally describe the second and third forms as local
developer workflows.

## Required Configuration

After copying the asset pack into the target repository, update
`release/skill-release.config.json`:

- replace `REPLACE_WITH_SKILL_ID`
- replace `REPLACE_WITH_DESCRIPTION`
- replace `REPLACE_WITH_AUTHOR_OR_TEAM`
- confirm destination repository settings
- confirm required artifact targets and release metadata paths

The release config is intentionally generic until the target repository fills
those placeholders in.

## Publish Modes

### `report_only`

Use when the workflow has reached `publish`, but the user did not ask to run
release commands yet.

- summarize readiness, blockers, and next actions
- confirm the target repository has adopted the asset pack correctly
- point to `dry_run`, `rehearsal`, and `live_release` as explicit follow-ups

### `dry_run`

Run from the target repository root:

```bash
GITHUB_TOKEN=<valid token> npm run release:dry-run
```

Review whether semantic-release reports a real release or a no-release outcome.
Do not describe this as a production publish.

### `rehearsal`

Run from the target repository root and use a local destination path for the
shared publication tree:

```bash
mkdir -p .work/release/destination-rehearsal/skills/other-skill
printf '%s\n' '{"entries":[],"entry_count":0,"format":"json","path":"catalog.json","updated_at":null}' \
  > .work/release/destination-rehearsal/catalog.json
SKILL_RELEASE_DESTINATION_REPOSITORY=.work/release/destination-rehearsal \
npm run release:verify-config
SKILL_RELEASE_DESTINATION_REPOSITORY=.work/release/destination-rehearsal \
node scripts/release/publish-skill-to-target-repo.mjs \
  0.0.0-local \
  v0.0.0-local \
  "$(git rev-parse HEAD)"
```

Then inspect:

- `.work/release/publish/`
- `.work/release/last-publication-receipt.json`
- `.release-manifest.json`
- versioned archives and checksum files

### `live_release`

The supported production path is the target repository's
`.github/workflows/release.yml`.

That workflow should:

1. verify release config
2. run release quality gates
3. build required target artifacts
4. run semantic-release
5. assemble and publish the shared-repository tree

## Quality Gates

From the target repository root:

```bash
npm run release:verify-config
npm run release:quality-gates
```

These checks should confirm:

- release config placeholders were replaced
- required artifact targets are declared
- the generated fixture and support assets are structurally valid
- the target CLI invocation contract is coherent with the shipped skill docs

## Package Boundary

Keep this distinction explicit:

- generated skill outputs:
  - compiled CLI binary
  - skill docs and runtime behavior
- repository-owned release automation:
  - `package.json`
  - `.releaserc.json`
  - `.github/workflows/release.yml`
  - `release/`
  - `scripts/release/`
  - `templates/` used by release support flows

Repository-owned automation supports the target project repository. It is not
part of the final shipped CLI binary interface.

## Failure Recovery

When release work fails, check these categories first:

- placeholders in `release/skill-release.config.json` were not replaced
- destination repository or token configuration is missing
- required build outputs are absent
- semantic-release found no releasable changes
- the target project's CLI contract or docs drifted from the expected release
  surface
