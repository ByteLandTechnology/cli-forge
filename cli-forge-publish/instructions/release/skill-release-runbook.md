# Skill Release Runbook

## Purpose

This runbook defines the supported release-facing behavior for target CLI skill
repositories that adopt the `cli-forge-publish` release automation asset pack.

The current `cli-forge` skill-family repository is the source of this release
pattern. It is not the thing being published by these instructions.

## Asset Pack Model

`cli-forge-publish/templates/` is a portable asset pack. Its contents are meant
to be copied into the root of a target CLI skill repository.

The target repository should receive these files at its own root:

- `package.json`
- `package-lock.json`
- `.releaserc.json`
- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/*`
- `scripts/install-current-release.sh`
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

## Release Contract

Every supported release should align these surfaces to the same version:

1. repository version chosen by semantic-release
2. git tag `v<version>`
3. GitHub Release page for that tag
4. CLI binary archives attached to the GitHub Release
5. `release-evidence.json` and `.release-manifest.json`
6. `scripts/install-current-release.sh` for clone-first installation

Shared-destination publication remains optional secondary follow-up only.

## Required Configuration

After copying the asset pack into the target repository, update
`release/skill-release.config.json`:

- replace `REPLACE_WITH_SKILL_ID`
- replace `REPLACE_WITH_OWNER/REPO`
- replace `REPLACE_WITH_DESCRIPTION`
- replace `REPLACE_WITH_AUTHOR_OR_TEAM`
- confirm `githubRelease.installScriptPath`
- confirm required artifact targets
- leave `optionalSecondaryPublication.enabled` as `false` unless that mirror is
  explicitly required

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

Run from the target repository root:

```bash
npm run release:verify-config
npm run release:quality-gates
npm run release:build-artifact -- x86_64-unknown-linux-gnu
npm run release:build-artifact -- aarch64-apple-darwin
node scripts/release/publish-skill-to-target-repo.mjs \
  0.0.0-local \
  v0.0.0-local \
  "$(git rev-parse HEAD)"
```

Then inspect:

- `.work/release/github-release/`
- `.work/release/github-release/release-evidence.json`
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
5. attach version-matched archives and evidence to the repo's GitHub Release

## Clone-First Install Flow

The supported user-facing install path is:

```bash
git clone https://github.com/<owner>/<repo>.git
cd <repo>
git checkout v<version>
./scripts/install-current-release.sh <version>
```

This helper must resolve the archive for the checked out release version instead
of downloading an arbitrary latest asset.

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
- `scripts/install-current-release.sh` exists and the generated docs mention it
- the generated docs mention `release-evidence.json`

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
  - `scripts/install-current-release.sh`
  - `templates/` used by release support flows

Repository-owned automation supports the target project repository. It is not
part of the final shipped CLI binary interface.

## Failure Recovery

When release work fails, check these categories first:

- placeholders in `release/skill-release.config.json` were not replaced
- required build outputs are absent
- repo version, tag, release page, and release evidence disagree
- the install helper points at the wrong version or wrong target
- semantic-release found no releasable changes
- the target project's CLI contract or docs drifted from the expected release
  surface
- optional secondary publication assumptions leaked into the default workflow
