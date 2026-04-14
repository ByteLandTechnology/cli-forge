# Release Automation Asset Pack

Repository-owned release automation for target CLI skill repositories generated
with the `cli-forge` skill family.

## Release Model

One semantic-release run publishes:

1. git tag `v<version>` + GitHub Release page (archive + sha256 per target)
2. **N platform npm packages** (`<name>-darwin-arm64`, `<name>-linux-x64`, ...) each
   carrying only the matching native binary, gated by `os` / `cpu`
3. **1 main npm package** (`<name>`) — a tiny JS shim whose `optionalDependencies`
   pin every platform package to the same version
4. `CHANGELOG.md` entry
5. `chore(release): <version> [skip ci]` commit (`CHANGELOG.md`,
   `Cargo.toml`, `Cargo.lock`, `npm/main/package.json`)

Users install with plain `npm install -g <name>`; npm picks the right platform
package automatically. No postinstall download.

## How To Use It

1. Copy the contents of this directory into the root of the target CLI skill
   repository.
2. Fill `release/config.json`:
   - `cliName` — the shipped CLI binary name
   - `packageName` — the npm main package name (with scope if used,
     e.g. `@acme/foo`)
   - `npmScope` — `null` for unscoped, or the scope string (e.g. `acme`)
     for scoped publication
   - `sourceRepository` — `owner/repo` on GitHub
3. `npm/main/package.json` is derived at release time from
   `release/config.json`. You may pre-fill it for local testing, but
   `sync-platform-packages.mjs` will overwrite `name`, `version`, `bin`, and
   `optionalDependencies` from the authoritative config during `prepare`.
4. Configure npm trusted publishing on npmjs.com for this repository's
   `release.yml`:
   - configure a publisher entry for the main package and for **each** platform
     package: `<name>-darwin-arm64` / `-darwin-x64` / `-linux-arm64` /
     `-linux-x64` / `-win32-arm64` / `-win32-x64`
   - each entry points at the same workflow file (`release.yml`)
   - keep the job's `id-token: write` permission and do not inject `NPM_TOKEN`
5. Install the release harness locally once for a dry-run:

   ```bash
   npm ci
   npm run release:rehearse
   ```

   This builds every target, generates platform packages, and runs
   `npm publish --dry-run` for each — validating the full pipeline without
   pushing tags or publishing to npm.

   To see what version semantic-release _would_ choose without exercising the
   custom hooks:

   ```bash
   npm run release:dry-run
   ```

6. Push to `main`; the workflow drives the live release.

## Targets

Defined in `release/config.json#targets`. Defaults:

| rustTarget                   | npm package suffix | os     | cpu   |
| ---------------------------- | ------------------ | ------ | ----- |
| `aarch64-apple-darwin`       | `darwin-arm64`     | darwin | arm64 |
| `x86_64-apple-darwin`        | `darwin-x64`       | darwin | x64   |
| `aarch64-unknown-linux-musl` | `linux-arm64`      | linux  | arm64 |
| `x86_64-unknown-linux-musl`  | `linux-x64`        | linux  | x64   |
| `aarch64-pc-windows-gnullvm` | `win32-arm64`      | win32  | arm64 |
| `x86_64-pc-windows-gnullvm`  | `win32-x64`        | win32  | x64   |

All six are built on a single `macos-14` runner using `cargo` + `cargo zigbuild`

- `llvm-mingw` (set up by `.github/actions/setup-build-env`). Linux targets use
  musl for fully static binaries that run on both glibc and musl (Alpine) systems.

## Clone-First Install (optional)

When users already have a checkout and want to install the binary directly from
the tagged GitHub Release archive, the harness attaches `tar.gz` + `sha256` per
target:

```bash
git clone https://github.com/<owner>/<repo>.git
cd <repo>
git checkout v<version>
./scripts/install-current-release.sh
```

The helper requires Node.js to read `release/config.json`. Prefer
`npm install -g <name>` for everything else.

## Files

- `.releaserc.json` — semantic-release plugin chain
- `.github/workflows/release.yml` — single release job
- `.github/actions/setup-build-env/action.yml` — macOS cross-build toolchain
- `release/config.json` — CLI name, package name, target list
- `npm/main/` — JS shim + published main package template
- `npm/platforms/` — generated at release time by
  `scripts/release/sync-platform-packages.mjs`
- `scripts/release/build-binaries.mjs` — semantic-release `prepare` hook that bumps
  `Cargo.toml#version`, builds all targets, and creates dist archives + provenance
- `scripts/release/validate-config.mjs` — shared config validation (fields, scope
  consistency, repository match, placeholder check); called by release.yml and
  sync-platform-packages.mjs
- `scripts/release/sync-platform-packages.mjs` — semantic-release `prepare` hook
- `scripts/release/publish-npm-packages.mjs` — semantic-release `publish` hook
  that publishes every platform package and the main package, each guarded by
  an `npm view` existence check for idempotent reruns
- `scripts/release/rehearse.mjs` — local rehearsal: build + sync +
  `npm publish --dry-run` for every package (no tag, no real publish)
- `scripts/install-current-release.sh` — clone-first install helper
- `package.json` — devDependencies (semantic-release + plugins) only
- `CHANGELOG.md` — maintained by semantic-release
