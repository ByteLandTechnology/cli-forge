# cli-forge Publish Planning Brief

This document contains planning constraints specific to the `cli-forge-publish`
stage. It must be read in addition to the root `planning-brief.md`.

## Publish Constraints

- **One Source of Truth**: The GitHub Release is the primary, authoritative
  publication event for any `cli-forge` skill, and npm publication must use
  that same released version in the same publish stage.
- **Evidence Trail**: The git tag, GitHub Release page (archives + sha256), npm
  registry entries, and workflow run logs constitute the authoritative evidence
  trail. No custom audit JSON is produced.
- **Target Independence**: The generated skill repository must contain its own
  copy of the release automation (`package.json`, `.github/` workflows,
  `scripts/`). `cli-forge` does not deploy skills for the user; it outfits the
  user's skill repository to deploy itself.
- **Unified Publication**: npm publication is part of `publish`, not a later
  child stage. A successful live release means the GitHub Release, binary
  assets, and npm packages all align on one semantic version.

## Mode Requirements

- `report_only`: MUST NOT mutate local files or remote state.
- `dry_run`: MAY mutate local files (like package version bumps, generated
  changelogs, or built artifacts) for inspection and local rehearsal, but MUST
  NOT push to remotes. Local mutations MUST be restored before exit so the
  working tree is left clean.
- `live_release`: MAY push production branch tags and trigger public release
  events.

If a requested action violates the constraints of its assigned mode, the release
process must halt with an error.
