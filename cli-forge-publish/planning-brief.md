# cli-forge Publish Planning Brief

This document contains planning constraints specific to the `cli-forge-publish`
stage. It must be read in addition to the root `planning-brief.md`.

## Publish Constraints

- **One Source of Truth**: The GitHub Release is the primary, authoritative
  publication event for any `cli-forge` skill.
- **Evidence Trail**: Every release must produce an `evidence.json` file inside
  the published asset pack that maps the exact git commit, built binary checksums,
  and the semantic version string.
- **Target Independence**: The generated skill repository must contain its own
  copy of the release automation (`package.json`, `.github/` workflows,
  `scripts/`). `cli-forge` does not deploy skills for the user; it outfits the
  user's skill repository to deploy itself.
- **Npm Separation**: Any npm-based package distribution must consume the exact
  evidence from the GitHub Release. This stage (`publish`) creates the release;
  npm publication belongs exclusively to `distribute`.

## Mode Requirements

- `report_only`: MUST NOT mutate local files or remote state.
- `dry_run`: MUST mutate local files (like package version bumps or generated
  changelogs) for inspection, but MUST NOT push to remotes.
- `rehearsal`: MAY push changes to a dedicated staging branch, but MUST NOT
  mutate production branch tags.
- `live_release`: MAY push production branch tags and trigger public release
  events.

If a requested action violates the constraints of its assigned mode, the release
process must halt with an error.
