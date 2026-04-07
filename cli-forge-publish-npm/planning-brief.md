# cli-forge-publish-npm Planning Brief

Use this brief as the npm publish-stage contract when planning or reviewing
optional npm publication work for a generated CLI skill repository.

## Purpose And Scope

- Treat `cli-forge-publish-npm` as the child skill for optional npm
  distribution of the shipped CLI command.
- Keep repo-native GitHub Release publication as the default release surface.
- Plan npm publication around one coordinating package plus one
  platform-specific package per supported target.
- Require the target repository's released tag and matching release evidence to
  identify the authoritative released skill version for the npm package set.
- Keep npm guidance reusable across generated target repositories instead of
  binding it to one downstream project.

## Publish-Stage Contract

Every npm publish-stage plan must lock these decisions:

- which npm publication mode is in scope: `report_only`, `dry_run`, or
  `live_publish`
- which target repository path or repository identity is being prepared
- whether validation is current or must be re-run before npm publication work
- whether the target repository already has a healthy repo-native release for
  the intended version, or whether npm work must wait for that release posture
- which coordinating package name is user-facing
- which platform-package names and supported targets are required for the same
  release
- how package ownership and registry access are confirmed for every package in
  the set
- how the user-facing npm install surface resolves the correct platform package

If a plan cannot answer those items, it should route back through
`cli-forge-intake` or `cli-forge-validate` before continuing.

## Install Surface Contract

npm planning must preserve one consistent hierarchy of user-facing release
surfaces:

1. default repo-native release path:
   clone `->` checkout released tag `->` `scripts/install-current-release.sh`
2. optional npm install path:
   install the coordinating package for the same released version
3. package-set resolution:
   the coordinating package resolves the required platform package for the
   user's supported target

The coordinating package is the only user-facing npm entry surface. Platform
packages are release-completeness and target-resolution surfaces, not the
primary user-facing install name.

## Version And Evidence Contract

Plans must keep this version chain explicit:

1. target repository released tag `v<version>`
2. matching repo-native release evidence for `<version>`
3. coordinating package version `<version>`
4. every required platform-package version `<version>`

If any link in that chain diverges, npm publication is blocked for that release
event.

## Package Boundary

Keep this boundary explicit:

- repo-native release surfaces:
  - repository version
  - git tag
  - GitHub Release assets
  - release evidence and manifests
  - clone-first install helper
- optional npm distribution surfaces:
  - coordinating package
  - platform packages
  - package-set ownership and install guidance

The npm package set distributes the shipped CLI command. It does not redefine
the repository's default release model.

## Risks To Check Explicitly

- validation is stale but npm publication work is being attempted anyway
- the target repository's released tag and release evidence do not identify one
  clear authoritative skill version
- the coordinating package or any required platform package is missing,
  misnamed, or not owned by the release team
- supported target coverage is incomplete but the package set is still treated
  as publish-ready
- user-facing npm install guidance does not explain how the coordinating
  package resolves the platform package
- npm publication wording is treated as the default release path instead of an
  explicit secondary channel
