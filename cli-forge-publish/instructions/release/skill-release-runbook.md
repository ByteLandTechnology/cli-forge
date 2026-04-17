# Skill Release Runbook

## Purpose

Defines the supported release behavior for target CLI skill repositories that
adopt the `cli-forge-publish` asset pack.

Before the first production release, perform one real npm prepublish step to
bootstrap package visibility and local auth. After that, one semantic-release
run produces all production release surfaces:

- git tag `v<version>` + GitHub Release page (tar.gz archive + sha256 per target)
- `N` per-platform npm packages carrying the native binaries
- `1` main npm package (tiny JS shim, `optionalDependencies` pin all platform
  packages to the same version)
- `CHANGELOG.md` entry
- `chore(release): <version> [skip ci]` commit

## Asset Pack Model

`cli-forge-publish/templates/` is a portable asset pack. Dot-prefixed repo
assets are stored inside this skill package under install-safe non-dot aliases
so `.agents` installers preserve them. When adopting the asset pack, restore
those aliases to the real target paths:

- `dot-releaserc.json` -> `.releaserc.json`
- `dot-github/actions/setup-build-env/action.yml` ->
  `.github/actions/setup-build-env/action.yml`
- `dot-github/workflows/release.yml` -> `.github/workflows/release.yml`
- `npm/platforms/dot-gitkeep` -> `npm/platforms/.gitkeep`

After restoring those alias-backed paths, the adopted target CLI skill
repository contains:

- `package.json` — devDependencies only (semantic-release + plugins)
- `CHANGELOG.md`
- `.releaserc.json`
- `.github/actions/setup-build-env/action.yml`
- `.github/workflows/release.yml`
- `release/config.json`
- `npm/main/` (package.json, bin/cli.js, README.md)
- `npm/platforms/` (generated at release time)
- `scripts/release/build-binaries.mjs`
- `scripts/release/validate-config.mjs`
- `scripts/release/sync-platform-packages.mjs`
- `scripts/release/publish-npm-packages.mjs`
- `scripts/release/rehearse.mjs`
- `scripts/release/prepublish.mjs`
- `scripts/release/ensure-npm-login.mjs`
- `scripts/install-current-release.sh`

The root `package.json` is not published. The main published wrapper is
`npm/main/package.json`.

## Command Execution Context

All commands run from the target CLI skill repository root after the asset
pack has been copied. Do not run them from inside this skill-family repository.

## Required Configuration

After copying the asset pack:

1. Edit `release/config.json`:
   - `cliName`
   - `mainPackageName` (with scope prefix if used)
   - `mainNpmScope` (`null` for unscoped, string otherwise)
   - `platformNpmScope` (`null` for unscoped platform packages, string otherwise)
   - `sourceRepository` (`owner/repo`)
2. Edit `npm/main/package.json`:
   - `name` — must match `release/config.json#mainPackageName`
   - `bin` — must use `cliName`
   - `description`, `license`
3. Platform package names are derived automatically from the main package body
   plus `-<target-suffix>`. The main package and platform packages may use
   different scopes, but only the scope may differ.
4. On npmjs.com, configure **GitHub Actions trusted publishing** for the main
   package and for each of the six platform packages. Every entry points at
   `release.yml` in this repository.
5. Keep `id-token: write` on the publishing job. Do not set `NPM_TOKEN`.

Never leave the install-safe `dot-*` aliases in the target repository. They
are packaging-only names, not part of the target repo contract.

## Publish Modes

### `report_only`

Audit-only. Read the repository state and report readiness, blockers, and next
actions. Do NOT adopt the asset pack, fill placeholders, or write any files.

### `dry_run`

The recommended local validation path. From the target repository root:

```bash
npm ci
npm run release:rehearse
```

This drives `scripts/release/rehearse.mjs`, which:

1. Builds every configured target into `dist/<rustTarget>/`.
2. Runs `sync-platform-packages.mjs` with a rehearsal version `0.0.0-rehearsal`,
   generating `npm/platforms/*` and stamping `npm/main/package.json`.
3. Runs `npm publish --dry-run --access=public` for every platform package and
   the main package.
4. Cleans up generated `npm/platforms/*` and restores `npm/main/package.json`.

Nothing is pushed to GitHub or npm. This validates the full build → sync →
publish path.

`npm run release:dry-run` is also available and drives `semantic-release
--dry-run --no-ci`, which reports the version that semantic-release _would_
choose and the release notes it _would_ generate. It does NOT exercise the
custom prepare/publish hooks (semantic-release skips those phases in dry-run).

### `prepublish`

The required local bootstrap step before the first production CI release. From
the target repository root:

```bash
npm ci
npm run release:prepublish
```

This drives `scripts/release/prepublish.mjs`, which:

1. Verifies `release/config.json` and derives all package names with the split
   scope policy.
2. Ensures local npm auth is ready. If not, it runs `npm login` and relays the
   verification URL so the operator can open it in a browser and finish
   authentication.
3. Selects or accepts a bootstrap prerelease version in the form
   `0.0.0-prepublish.N`.
4. Builds every configured target into `dist/<rustTarget>/`.
5. Runs `sync-platform-packages.mjs` with that prepublish version.
6. Temporarily removes npm provenance from the local package manifests so the
   prepublish step does not pretend to be a CI-backed provenance build.
7. Publishes every platform package with `npm publish --access=public`.
8. Publishes the main package last with `npm publish --access=public`.
9. Restores local generated files so the working tree returns to its pre-run
   state.

`prepublish` does **not** create a git tag, changelog entry, release commit,
or GitHub Release.

### `live_release`

Push a release-eligible commit to `main`; `.github/workflows/release.yml` runs
the release job. The job:

1. Checks out with full history
2. Sets up Node 24, Rust (with all configured targets), and the composite
   `setup-build-env` action (zig + llvm-mingw for macOS-led cross-build)
3. Installs devDependencies (`npm ci`)
4. Builds every target into `dist/<rustTarget>/<binary>` and produces
   `dist/<cli>-<rustTarget>.tar.gz` + `.sha256`
5. Runs `npx semantic-release`, which:
   - Selects the version from conventional commits
   - Writes `CHANGELOG.md`
   - **`prepare` → `build-binaries.mjs`**: bumps `Cargo.toml#version`, builds every
     target, produces `dist/<cli>-<rustTarget>.tar.gz` + `.sha256`, writes
     `dist/provenance.json`
   - **`prepare` → `sync-platform-packages.mjs`**: stamps `npm/main/package.json`
     `optionalDependencies` and materializes `npm/platforms/<suffix>/package.json`
     - copies binaries into each platform package
   - **`prepare` → `@semantic-release/npm`**: bumps `npm/main/package.json`
     `version`
   - **`prepare` → `@semantic-release/git`**: commit `CHANGELOG.md`,
     `Cargo.toml`, `Cargo.lock`, `npm/main/package.json` as
     `chore(release): <version> [skip ci]` and push
   - semantic-release core pushes the tag `v<version>`
   - **`publish` → `publish-npm-packages.mjs`**: `npm publish` every platform
     package and the main package (trusted publishing via OIDC); each call is
     guarded by `npm view` so a rerun at the same version skips anything that
     already made it to the registry
   - **`publish` → `@semantic-release/github`**: attach the archives + sha256
     to the GitHub Release for the just-pushed tag

`live_release` is the production path only. It assumes the prepublish bootstrap has
already completed and continues to rely on trusted publishing in CI rather than
interactive local `npm login`.

`@semantic-release/npm` runs with `npmPublish: false`; it only bumps
`npm/main/package.json#version` during `prepare`. All `npm publish` calls are
owned by `publish-npm-packages.mjs`.

## Install Paths

### npm (default)

```bash
npm install -g <package-name>
<cli-name> ...
```

npm resolves the matching platform package via `optionalDependencies` + `os` +
`cpu` metadata. No postinstall download.

### Clone-first (optional)

For users who already have the repository checked out and prefer installing a
binary directly from the tagged GitHub Release:

```bash
git clone https://github.com/<owner>/<repo>.git
cd <repo>
git checkout v<version>
./scripts/install-current-release.sh
```

Reads `release/config.json`, downloads the tagged `tar.gz` from the GitHub
Release, installs the binary to `INSTALL_DIR` (default `.local/bin`). Requires
Node.js to parse the config file. Supports darwin + linux; Windows users
should use npm.

## What This Pipeline Does Not Do

- A prepublish bootstrap step does not create a production release record.
- No custom audit JSON (`release-evidence.json`, `.release-manifest.json`,
  `last-publication-receipt.json`) is produced. GitHub Release + npm registry +
  workflow run logs are the authoritative record.
- No mirror-repository secondary publication.

## Failure Recovery

Common failure categories:

- `release/config.json` still has placeholders → fill and retry
- `release/config.json#mainPackageName` scope mismatches `mainNpmScope`, or the
  platform scope policy is malformed →
  `validate-config.mjs` hard-fails with a pointer to the offending
  field before any external write
- Local prepublish needs interactive auth → run `npm login`, open the
  verification URL it prints, and re-run `npm run release:prepublish`
- Platform package trusted publisher not configured on npmjs.com → the first
  release of that package fails; configure the publisher entry and re-run
- semantic-release reports "no releasable changes" → commits since last tag
  don't match any release rule; nothing to do

### Wedged mid-release (tag pushed, some `npm publish` failed)

If the release commit and tag reached the origin but some `npm publish` calls
failed, pushing a new commit to `main` will not re-drive the same version
(commit-analyzer sees no new commits). Use the **recover** path instead:

1. Go to **Actions → Release → Run workflow** on GitHub.
2. Set **recover-version** to the version string (e.g. `1.2.3`).
3. Run the workflow.

The recover path reuses the same job but skips semantic-release entirely. It:

1. Builds every target via `build-binaries.mjs` (producing `dist/` fresh), or
   downloads the original build artifacts if `recover-run-id` is provided.
2. Runs `sync-platform-packages.mjs` (generating `npm/platforms/*`).
3. Runs `publish-npm-packages.mjs`, which skips every package already on the
   registry and publishes only the missing ones.

This works because `publish-npm-packages.mjs` is idempotent: every `npm
publish` is preceded by `npm view <pkg>@<version>`, and an already-published
version is skipped cleanly.

If you prefer to recover locally (CI unavailable), the same three steps work
from a tag checkout:

```bash
git checkout v<version>
npm ci
# Build all targets first
node scripts/release/build-binaries.mjs <version>
node scripts/release/sync-platform-packages.mjs <version>
node scripts/release/publish-npm-packages.mjs <version>
```
