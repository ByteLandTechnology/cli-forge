# cli-forge Takeover Planning Brief

Use this brief when adopting a pre-existing Rust CLI Skill project into the
`cli-forge` pipeline.

## Goals

- Reconstruct the missing `.cli-forge/` contracts from observable project
  behavior.
- Keep the recovered contracts traceable to evidence instead of assumptions.
- Preserve user control whenever the existing project is ambiguous or
  internally inconsistent.
- Establish a baseline that downstream `Extend`, `Validate`, and `Publish`
  flows can trust without pretending the project was originally scaffolded by
  `cli-forge`.

## Evidence Sources

Treat the following as admissible evidence for takeover:

- `Cargo.toml`
- `SKILL.md`
- `README.md`
- `src/`
- `tests/`
- observable `--help`, auto-help, and `help --format` output when runnable
- release/publish assets when present

## Recovery Rules

- Prefer observed implementation over inferred intention until the user says
  otherwise.
- If two surfaces disagree, escalate to the user and record the decision in the
  takeover receipt.
- Reconstruct the `design-contract.yml` first, then the `cli-plan.yml`.
- Keep optional capabilities explicit: `stream`, `repl`, and `daemon` must be
  marked `in_scope` or `out_of_scope`.
- Preserve the four help scenarios as observed, even if Validate is expected to
  flag a standards mismatch later.
- Generate `takeover-receipt.yml` instead of `scaffold-receipt.yml` for
  adopted projects.
- Update `.gitignore` to ignore `.cli-forge/` when required by the pipeline.

## Minimum Successful Adoption

Takeover is complete only when all of the following are true:

- `.cli-forge/design-contract.yml` exists and is user-approved
- `.cli-forge/cli-plan.yml` exists and is user-approved
- `.cli-forge/takeover-receipt.yml` exists
- downstream stages can tell the project is adopted, not freshly scaffolded
- unresolved ambiguities are either decided by the user or explicitly block the
  workflow
