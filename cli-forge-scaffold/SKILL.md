---
name: cli-forge-scaffold
description: "Scaffold stage for the cli-forge skill family: create a new Rust CLI Skill project from the authoritative templates and prepare it for validation."
---

# cli-forge Scaffold

Use this stage when the work is to create a brand-new Rust CLI Skill project
from the baseline templates, and the detailed CLI contract (Plan) has already
been explicitly defined and approved.

## Purpose

Generate the baseline codebase for a new skill using the approved
`cli-plan.yml` and the authoritative templates.

This stage does not make design decisions; it strictly follows the blueprint
provided by the Plan stage.

## Canonical References

- [`./instructions/new.md`](./instructions/new.md)
- [`../planning-brief.md`](../planning-brief.md)
- [`../contracts/scaffold-receipt.yml.tpl`](../contracts/scaffold-receipt.yml.tpl)
- [`../templates/scaffold/`](../templates/scaffold/)

## Entry Gate

| #   | Check                                   | Source     |
| --- | --------------------------------------- | ---------- |
| 1   | `cli-plan.yml` exists and is approved   | Plan stage |
| 2   | `skill_name` is known                   | Plan stage |
| 3   | Target project directory DOES NOT exist | Filesystem |

## Required Inputs

- `skill_name`
- Approved `cli-plan.yml`
- Optional `author`, `version`, `rust_edition`

## Workflow

1. Run the pre-checks from
   [`./instructions/new.md`](./instructions/new.md): require the skill
   name, verify `cli-plan.yml`, and refuse to overwrite an existing directory.
2. Create the directory structure.
3. Expand the templates from `../templates/scaffold/` EXACTLY as defined by
   `cli-plan.yml` and the instruction file.
4. Verify that no `{{token}}` placeholders remain unresolved.
5. Run the required verification commands from the generated project root:
   - `cargo build`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
   - `cargo test`
6. Confirm the generated package layout stays within the documented boundary:
   baseline generated files plus feature-local support files. Repository-owned
   CI workflows and release scripts stay outside the generated project until Publish.
7. Generate `.cli-forge/scaffold-receipt.yml` using the template at
   [`../contracts/scaffold-receipt.yml.tpl`](../contracts/scaffold-receipt.yml.tpl).

## Outputs

- A new Rust CLI package directory
- `.cli-forge/scaffold-receipt.yml`

## Exit Gate

| #   | Check                                            |
| --- | ------------------------------------------------ |
| 1   | All templates expanded with no unresolved tokens |
| 2   | `cargo build` passes                             |
| 3   | `cargo clippy -- -D warnings` passes             |
| 4   | `cargo fmt --check` passes                       |
| 5   | `cargo test` passes                              |
| 6   | `scaffold-receipt.yml` generated                 |

## Guardrails

- Use templates EXCLUSIVELY from `../templates/scaffold/`. Do not maintain
  local copies of templates here.
- Do not improvise project structure or dependencies. Everything must be tied
  back to the `cli-plan.yml`.
- If any Cargo verification step fails, block the workflow and fix the
  scaffolded files before handing the work forward.

## Next Step

Continue with [`../cli-forge-validate/SKILL.md`](../cli-forge-validate/SKILL.md)
to run the full compliance rule set.
