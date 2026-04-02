# Release Automation Asset Pack

This directory contains a repository-owned, repo-native release automation
asset pack for target CLI skill repositories generated with the `cli-forge`
skill family.

## Default Release Model

The default distribution path is the target repository's own GitHub Release:

1. semantic-release computes one version
2. the repo creates tag `v<version>`
3. GitHub Release assets publish version-matched CLI archives
4. `release-evidence.json` and `.release-manifest.json` record the same version
5. users clone the repo and run `scripts/install-current-release.sh`

Shared-destination publication is optional secondary follow-up only. It is not
the default user-facing distribution path.

## How To Use It

1. Copy the contents of this directory into the root of the target CLI skill
   repository that will own the release workflow.
2. Run `npm ci` from that target repository root.
3. Update `release/skill-release.config.json` and replace the placeholder
   values with the target project's real skill id, repository identity,
   description, author/team, and any optional secondary publication settings.
4. Keep `scripts/install-current-release.sh` in the target repository root so
   cloned checkouts can install the matching released binary for the checked
   out tag.
5. Run release commands from the target repository root, not from this
   skill-family repository.

## Root-Level Files Expected In The Target Repository

- `package.json`
- `package-lock.json`
- `.releaserc.json`
- `.github/workflows/release.yml`
- `release/skill-release.config.json`
- `scripts/release/*`
- `scripts/install-current-release.sh`
- `templates/*`

## Required Verification

From the target repository root:

```bash
npm run release:verify-config
npm run release:quality-gates
npm run release:dry-run
```

The quickstart install path should also be verifiable from a released checkout:

```bash
git checkout v<version>
./scripts/install-current-release.sh <version>
```

The `templates/` directory is part of the release support bundle. It exists for
fixture generation and validation support; it is not the final shipped CLI
binary interface.
