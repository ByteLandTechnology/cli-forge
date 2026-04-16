# cli-forge Publish Planning Brief

This document contains planning constraints specific to the `cli-forge-publish`
stage. It must be read in addition to the root `planning-brief.md`.

## Publish Constraints

- **One Source of Truth**: The GitHub Release is the primary, authoritative
  production publication event for any `cli-forge` skill. A pre-CI npm
  prepublish step may happen first, but it must use a dedicated prerelease version and
  does not replace the production release event.
- **Evidence Trail**: The git tag, GitHub Release page (archives + sha256), npm
  registry entries, and workflow run logs constitute the authoritative
  production evidence trail. Prepublish bootstrap should be recorded in the
  publish receipt, but no custom audit JSON is produced.
- **Target Independence**: The generated skill repository must contain its own
  copy of the release automation (`package.json`, `.github/` workflows,
  `scripts/`). `cli-forge` does not deploy skills for the user; it outfits the
  user's skill repository to deploy itself.
- **Unified Publication**: npm publication is part of `publish`, not a later
  child stage. A successful live release means the GitHub Release, binary
  assets, and npm packages all align on one semantic version.
- **Split Scope, Shared Base Name**: The main package and platform packages may
  use different npm scopes, but platform package names must still derive from
  the main package body plus `-<target-suffix>`.

## Mode Requirements

- `report_only`: MUST NOT mutate local files or remote state.
- `dry_run`: MAY mutate local files (like package version bumps, generated
  changelogs, or built artifacts) for inspection and local rehearsal, but MUST
  NOT push to remotes. Local mutations MUST be restored before exit so the
  working tree is left clean.
- `prepublish`: MAY perform real public npm publishes for the bootstrap
  prerelease only. It MUST NOT create tags, changelog entries, or GitHub
  Releases, and it MUST pause for interactive `npm login` when local auth is
  required.
- `live_release`: MAY push production branch tags and trigger public release
  events, but only after prepublish bootstrap has completed for the configured
  package set.

If a requested action violates the constraints of its assigned mode, the release
process must halt with an error.
