# Release Automation Asset Pack

This directory contains a repository-owned release automation asset pack that
is meant to be copied into a target CLI skill repository generated with the
`cli-forge` skill family.

## How To Use It

1. Copy the contents of this directory into the root of the target CLI skill
   repository that will own the release automation workflow.
2. Run `npm ci` from that target repository root.
3. Update `release/skill-release.config.json` and replace the placeholder
   values with the target project's real skill id, description, author/team,
   and destination repository settings.
4. Run the release commands from the target repository root, not from this
   skill-family repository.

## Root-Level Files Expected In The Target Repository

- `package.json`
- `package-lock.json`
- `.releaserc.json`
- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/*`
- `templates/*`

The `templates/` directory is part of the release support bundle. It exists for
fixture generation and rehearsal support; it is not the final shipped CLI
binary interface.
