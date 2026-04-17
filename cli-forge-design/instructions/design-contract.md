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
   - Help text → summary line reused across non-leaf auto-help, `--help`
     man-like help, and structured `help`
   - Release notes → summary paragraph (when publish is in scope)
   - npm package → description field (when publish is in scope)
5. Record the downstream help-contract requirement so later stages keep the
   same behavior: leaf commands fail with structured errors instead of
   auto-help, non-leaf command paths auto-render man-like help, `--help`
   renders man-like help, and `help` returns structured output. Store these
   values in the `help_contract` block of `design-contract.yml`.
6. Record that human-readable help is man-like and must preserve the canonical
   section order `NAME -> SYNOPSIS -> DESCRIPTION -> OPTIONS -> FORMATS ->
   EXAMPLES -> EXIT CODES`. Store this under
   `help_contract.human_readable_help`.
7. Record that the Publish stage owns both the repo-native GitHub Release and
   npm publication when release work is later adopted.
8. Generate `design-contract.yml` from the template at
   `contracts/design-contract.yml.tpl`.
9. Present the contract to the user for approval using a dialog-based chooser
   when dialog tooling is available. If dialogs are unavailable, use a
   numbered text menu with `1. approve and continue`, `2. request changes`,
   `3. stop for now`, and `Other: <custom response>`. Accept exact replies
   `1`, `2`, or `3`, or `Other: ...`. If a numeric reply includes additional
   text such as `1 - <note>`, ask for clarification before proceeding. Do not
   require the user to type the full option label before proceeding.

## For Existing Skills (Refresh)

When refreshing the contract of an existing skill:

1. Load the current `design-contract.yml`.
2. Identify which surfaces have drifted from the approved wording.
3. Propose updated purpose and positioning that resolve the drift.
4. Confirm the sync surfaces list is still complete.
5. Confirm the stored `help_contract` block still matches the downstream
   requirement for the four help scenarios and the canonical man-like section
   order.
6. Generate an updated `design-contract.yml`.
7. Present the changes to the user for approval using a dialog-based chooser
   when dialog tooling is available. If dialogs are unavailable, use a
   numbered text menu with `1. approve and continue`, `2. request changes`,
   `3. stop for now`, and `Other: <custom response>`. Accept exact replies
   `1`, `2`, or `3`, or `Other: ...`. If a numeric reply includes additional
   text such as `1 - <note>`, ask for clarification before proceeding. Do not
   require the user to type the full option label before proceeding.

## Contract Format

See [`./contracts/design-contract.yml.tpl`](./contracts/design-contract.yml.tpl)
for the canonical format.

## Done Condition

This instruction is complete only when:

- The purpose summary and positioning statement are user-approved.
- The sync surfaces list covers all known downstream consumers.
- The `help_contract` block is written and matches the downstream requirement.
- The `design-contract.yml` file is written to `.cli-forge/`.
- The next stage (Plan) is ready to consume the contract.
