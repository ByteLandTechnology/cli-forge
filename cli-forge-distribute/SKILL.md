---
name: cli-forge-distribute
description: "Archived reference for the old split-stage npm publication model. New workflows should use Publish instead."
---

# cli-forge Distribute (Archived)

Do not use this stage for new work.

The npm publication contract now belongs to
[`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md). This
directory is retained only as historical reference for the older split-stage
model.

## Purpose

Preserve historical guidance for the earlier split between repo-native release
and npm distribution.

## Canonical References

- [`./instructions/release/npm-publish-runbook.md`](./instructions/release/npm-publish-runbook.md)
- [`./planning-brief.md`](./planning-brief.md)
- [`./shared-planning-brief.md`](./shared-planning-brief.md)
- [`./contracts/validation-report.yml.tpl`](./contracts/validation-report.yml.tpl)

## Entry Gate

| #   | Check                                           | Source        |
| --- | ----------------------------------------------- | ------------- |
| 1   | Caller understands this stage is archived       | User / Router |
| 2   | New release work is being redirected to Publish | Router / User |

## Required Inputs

- None for new workflows. Route new release work to Publish.

## Workflow

1. Tell the caller that npm publication is now owned by Publish.
2. If historical context is needed, use the files in this directory as legacy
   background only; do not treat them as the active workflow contract.
3. Route the user to [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md).

## Outputs

- A redirection to Publish for any new release work

## Exit Gate

| #   | Check                                              |
| --- | -------------------------------------------------- |
| 1   | Caller was redirected to Publish                   |
| 2   | No new workflow claims this stage is authoritative |

## Guardrails

- Do not route new work here.
- Do not present this archived stage as the current source of truth for npm
  publication.

## Next Step

Continue with [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md).
