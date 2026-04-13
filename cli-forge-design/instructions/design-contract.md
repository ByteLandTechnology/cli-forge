# Design Contract Instructions

Use this document as the operational source of truth for producing or
refreshing a design contract during the Design stage.

## Pre-Checks

1. Confirm that the Router has classified this work as `design`.
2. Confirm that `.cli-forge/handoff.yml` exists and contains the skill scope.
3. If a `design-contract.yml` already exists, load it as the baseline for
   refresh instead of starting from scratch.

## For New Skills

When creating a brand-new skill contract:

1. Ask the user for the skill's intended purpose in one sentence.
2. Ask how the skill is positioned relative to alternatives or related tools.
3. Record both the purpose summary and the positioning statement.
4. List every surface that must carry the same approved wording:
   - `Cargo.toml` → `package.description`
   - `SKILL.md` → purpose section
   - `README.md` → header description paragraph
   - Help text → summary line in `--help` and structured `help`
   - Release notes → summary paragraph (when publish is in scope)
   - npm package → description field (when distribute is in scope)
5. Ask whether the user wants repo-native release only, or also optional npm
   distribution.
6. Generate `design-contract.yml` from the template at
   `contracts/design-contract.yml.tpl`.
7. Present the contract to the user for approval using a dialog-based chooser
   before proceeding. Do not require an exact reply string, and do not fall
   back to numbered or free-form typed approval input. If dialog tooling is
   unavailable, stop and report that approval is blocked until a dialog-capable
   surface is available.

## For Existing Skills (Refresh)

When refreshing the contract of an existing skill:

1. Load the current `design-contract.yml`.
2. Identify which surfaces have drifted from the approved wording.
3. Propose updated purpose and positioning that resolve the drift.
4. Confirm the sync surfaces list is still complete.
5. Generate an updated `design-contract.yml`.
6. Present the changes to the user for approval using a dialog-based chooser
   before proceeding. Do not require an exact reply string, and do not fall
   back to numbered or free-form typed approval input. If dialog tooling is
   unavailable, stop and report that approval is blocked until a dialog-capable
   surface is available.

## Contract Format

See [`../contracts/design-contract.yml.tpl`](../contracts/design-contract.yml.tpl)
for the canonical format.

## Done Condition

This instruction is complete only when:

- The purpose summary and positioning statement are user-approved.
- The sync surfaces list covers all known downstream consumers.
- The `design-contract.yml` file is written to `.cli-forge/`.
- The next stage (Plan) is ready to consume the contract.
