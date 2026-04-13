---
name: cli-forge
description: "Router for the cli-forge skill family: classify the request, check filesystem state, gather inputs, and route to the earliest incomplete stage."
---

# cli-forge Router

Use this parent skill as the stable entry point for the full `cli-forge`
workflow.

This skill does not do any design, planning, or implementation work itself. Its
job is to identify the current stage, recover the earliest incomplete phase from
the repository state, assemble required inputs, and route work to the correct
child skill.

## Purpose

Act as the intake layer and traffic controller.

- Classify requests into one of seven stages: Design, Plan, Scaffold, Extend,
  Validate, Publish, or Distribute.
- Check the current filesystem state (presence of directories, base files,
  validation reports).
- Assemble inputs and generate the `handoff.yml` contract.
- Resume interrupted workflows when the user provides partial context.

## Canonical References

- [`../contracts/handoff.yml.tpl`](../contracts/handoff.yml.tpl)
- [`../planning-brief.md`](../planning-brief.md)

## Entry Gate

| #   | Check               | Source |
| --- | ------------------- | ------ |
| 1   | User request exists | User   |

## Required Inputs

- The user request (explicit or ambiguous)
- Target directory path. **CRITICAL**: If the user does not provide the project path, you MUST actively search the workspace for `.cli-forge/` directories to locate the active project, or explicitly ask the user for the path. Do not assume the root directory is the target project.

## Workflow

1. Read the user request and classify the expected outcome.
2. Inspect the filesystem to disambiguate the stage:
   - Missing target directory + new request -> Design
   - `design-contract.yml` exists but no `cli-plan.yml` -> Plan
   - `cli-plan.yml` exists but no source files -> Scaffold
   - Existing project + feature request (stream/repl/daemon) -> Extend
   - Existing project + audit request (or post-edit verification) -> Validate
   - Passed validation + repo-native release request -> Publish
   - Passed validation + npm distribution request -> Distribute
3. Confirm the required inputs for the chosen path:
   - Scaffold: `skill_name`
   - Extend: `project_path` and `feature` (stream/repl/daemon)
   - Validate: `project_path`
   - Publish/Distribute: `publish_mode`, `publish_channel`
4. Use the template at `contracts/handoff.yml.tpl` to generate
   `.cli-forge/handoff.yml` in the target project directory. This explicitly
   records the classification and inputs for downstream consumption.
5. Provide a clear handoff response specifying which child skill should be
   invoked next. Use a dialog-based chooser for the next-step handoff whenever
   the platform supports it (for example, `request_user_input`). Do not require
   the user to type an exact phrase or skill name. If dialog tooling is
   unavailable, accept any clear natural-language confirmation of the desired
   next step.

## Outputs

- `.cli-forge/handoff.yml` — The explicit routing contract

## Exit Gate

| #   | Check                                              |
| --- | -------------------------------------------------- |
| 1   | Request intent classified successfully             |
| 2   | Required inputs for the downstream stage assembled |
| 3   | `handoff.yml` generated                            |
| 4   | Explicit handoff made to the correct child stage   |

## Guardrails

- **CRITICAL DIRECTIVE TO THE ASSISTANT**: You MUST NOT bypass the staged pipeline. Do not write, generate, or scaffold code yourself during this stage.
- **CRITICAL DIRECTIVE TO THE ASSISTANT**: You MUST STOP and yield to the user after generating `handoff.yml` and explaining the next steps. Do not invoke the next stage autonomously. Use a dialog-based selection for the handoff whenever supported, and never require the user to explicitly type the next skill name.
- The Router must stay thin. Do not execute templates, run compilation steps, or define CLI contracts here.
- Never force a workflow forward if an earlier stage is incomplete. For
  example, if the user asks to "validate" but the project is missing the
  scaffold baseline, route back to Scaffold.
- If the user asks for a release but validation is missing or stale, route to
  Validate first.
- If the target project directory is unknown, you MUST ask the user or search for `.cli-forge/` folders. Do not assume the current working directory applies.

## Next Step

Route to one of:

- [`../cli-forge-design/SKILL.md`](../cli-forge-design/SKILL.md)
- [`../cli-forge-plan/SKILL.md`](../cli-forge-plan/SKILL.md)
- [`../cli-forge-scaffold/SKILL.md`](../cli-forge-scaffold/SKILL.md)
- [`../cli-forge-extend/SKILL.md`](../cli-forge-extend/SKILL.md)
- [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
- [`../cli-forge-publish/SKILL.md`](../cli-forge-publish/SKILL.md)
- [`../cli-forge-distribute/SKILL.md`](../cli-forge-distribute/SKILL.md)
