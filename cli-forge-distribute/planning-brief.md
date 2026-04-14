# cli-forge Distribute Planning Brief (Archived)

Use this brief only as archived background for the old split-stage npm
publication model. New work should follow `cli-forge-publish/`.

## Purpose And Scope

- Preserve historical context for the earlier package-set design.
- Do not route new release work here.
- Treat npm publication as part of Publish for current workflows.

## Publish-Stage Contract

For current workflows, do not plan npm publication here. Route to
`cli-forge-publish` and keep GitHub Release plus npm publication in one stage.

## Install Surface Contract

Historical note: this archived brief assumed a coordinating npm package plus
platform packages. That is no longer the active `cli-forge` release contract.

## Version And Evidence Contract

The active version contract now lives in Publish: GitHub Release and npm
publication must align to the same semantic-release version.

## Package Boundary

Keep this boundary explicit when reading archived material: it is historical
reference only and must not override the active Publish-stage contract.

## Risks To Check Explicitly

- someone routes new work to the archived Distribute stage
- historical package-set guidance is mistaken for the active publish contract
