# Takeover Instructions

Use this document as the operational source of truth for adopting an existing
project into the `cli-forge` pipeline.

## Pre-Checks

1. Confirm the Router classified the work as `takeover`, or that the user
   explicitly asked to adopt/backfill an existing project.
2. Resolve `project_path` to an absolute path.
3. Confirm `project_path` exists and is a directory.
4. Confirm `Cargo.toml` exists at the project root.
5. If both `.cli-forge/design-contract.yml` and `.cli-forge/cli-plan.yml`
   already exist and are approved:
   - continue when the recorded `takeover_mode` is `contract_refresh`
   - continue when the recorded `takeover_mode` is `baseline_establishment`
     because a downstream stage needs a takeover receipt for an existing
     adopted baseline
   - otherwise stop and route to Validate, Plan refresh, or another
     downstream stage instead of performing redundant takeover work
6. Record the user's post-adoption objective as one of: `validate`, `extend`,
   `publish`, or general pipeline adoption.
7. Record the takeover mode as one of: `first_adoption`,
   `baseline_establishment`, or `contract_refresh`.

## Step 1: Build the Evidence Inventory

Inspect the repository and record the current observable contract surface:

1. Read `Cargo.toml` for package name, version, edition, and dependencies.
2. Read `SKILL.md` if present.
3. Read `README.md` if present.
4. Inspect `src/` and `tests/` to recover the command tree, flags, output
   formats, runtime directory behavior, and optional capabilities.
5. Inspect help behavior from code, tests, or command execution:
   - top-level auto-help
   - non-leaf auto-help
   - `--help`
   - structured `help`
6. Inspect any daemon or publish surfaces when present.

### Discovery Gate

The discovery gate passes only when all of the following are true:

- the skill purpose can be described without guessing
- the primary invocation name is known
- the command tree is known
- every known flag has at least a provisional type, default, and description
- optional capabilities are either evidenced or reduced to a specific user
  question

If any of the above are still ambiguous, stop and ask the user concise,
targeted questions before generating contracts.

## Step 1.5: Prepare the Pipeline Workspace

1. Ensure `.cli-forge/` exists in the project root before writing any recovered
   contracts.
2. If takeover mode is `first_adoption`, treat this as the first write barrier:
   no step should attempt to write `.cli-forge/design-contract.yml`,
   `.cli-forge/cli-plan.yml`, or `.cli-forge/takeover-receipt.yml` until the
   directory exists.

## Step 2: Reconstruct the Design Contract

1. Derive the one-line purpose summary from the strongest user-facing evidence.
2. Derive the positioning statement from the current docs and observable
   behavior.
3. List every sync surface that should reuse the approved wording.
4. Preserve the downstream help contract expectations that the project actually
   exposes.
5. If the wording differs across `Cargo.toml`, `SKILL.md`, `README.md`, help
   text, or release assets, present the mismatch and ask the user which wording
   must become authoritative.
6. Generate `.cli-forge/design-contract.yml` from
   `contracts/design-contract.yml.tpl`.

### Design Gate

This gate passes only when:

- purpose and positioning are backed by evidence or explicit user direction
- sync surfaces are complete
- the user approves the reconstructed wording

Do not continue to plan reconstruction until the design contract is approved.

## Step 3: Reconstruct the CLI Plan

1. Derive the command tree from code, help output, and tests.
2. Derive flags, types, defaults, descriptions, and output formats per command.
3. Recover the four help scenarios exactly as implemented today.
4. Recover runtime directory behavior and Active Context behavior from code,
   docs, and tests.
5. Mark `stream`, `repl`, and `daemon` as `in_scope` or `out_of_scope`.
6. If daemon behavior is present, compare it against the app-server planning
   contract and record the actual project behavior plus any gaps.
7. If a required plan detail is unclear, ask the user before locking the plan.
8. Generate `.cli-forge/cli-plan.yml` from `contracts/cli-plan.yml.tpl`.
   When refreshing or reconstructing the plan, preserve the help behavior that
   is actually observed today, even when it differs from cli-forge defaults.
   The same rule applies to output formats, error shape, runtime overrides,
   and Active Context support: record what the adopted project really does
   today instead of backfilling aspirational cli-forge-standard behavior.

### Plan Gate

This gate passes only when:

- every command and flag is either evidenced or user-confirmed
- capability scope is explicit
- runtime behavior is described without hidden assumptions
- the user approves the reconstructed plan

Do not silently "fix" the project by writing an aspirational plan. The plan
must describe the current adopted contract, even when Validate is expected to
flag non-compliance later.

## Step 4: Establish the Takeover Baseline

1. Confirm `.cli-forge/` still exists in the project root.
2. Ensure `.gitignore` exists and includes `.cli-forge/`.
3. Generate `.cli-forge/takeover-receipt.yml` from
   `contracts/takeover-receipt.yml.tpl`.
4. Record:
   - the project path
   - which contracts were generated
   - which evidence sources were used
   - which user decisions resolved ambiguity
   - any follow-up work still required
5. Do not generate `scaffold-receipt.yml`. A recovered baseline must stay
   distinguishable from a true scaffolded baseline.
6. Determine whether the adopted project is already scaffold-compatible for
   Extend by checking both file presence and the overlay templates' required
   API/dependency surface:
   - `Cargo.toml`
   - `SKILL.md`
   - `src/main.rs`
   - `src/lib.rs`
   - `src/help.rs`
   - `src/context.rs`
   - `tests/cli_test.rs` should be present when the feature workflow is
     expected to patch the generated integration tests directly
   - `src/context.rs` must expose the scaffold-style context/runtime helpers
     the overlays call directly, including `resolve_runtime_locations`
   - when claiming REPL compatibility, the adopted dependency surface must
     already include `rustyline` and the scaffold-style context API consumed by
     `repl.rs.tpl` instead of only matching filenames
7. Record the recorded post-adoption objective, the next-stage decision, and
   any layout blockers in `takeover-receipt.yml`.
8. Also record the takeover mode so later stages can distinguish first
   adoption from baseline establishment or explicit contract refresh.
9. Also record whether `design-contract.yml` or `cli-plan.yml` were rewritten
   during takeover, because any rewrite makes an existing validation report
   stale for Publish gating.
10. Also record whether `.gitignore` or any other validation-covered baseline
    surface changed during takeover. Any such change makes a pre-existing
    validation report stale for Publish gating, even when the approved design
    and plan contracts were preserved.

### Baseline Gate

This gate passes only when:

- the required `.cli-forge/` contracts exist
- `.gitignore` protects the transient pipeline directory
- the project is explicitly marked as `takeover`-adopted
- the next step is resolved from the recorded objective instead of assumed

## Step 5: Resolve the Next Stage

Once the baseline gate passes, choose the next step from the recorded
post-adoption objective:

- `validate` or general adoption: hand off to Validate and tell the user
  whether the expected outcome is straightforward validation or validation with
  known standards gaps.
- `publish`: inspect `.cli-forge/validation-report.yml`. If no fresh report is
  present, hand off to Validate first. Also hand off to Validate first when
  Takeover rewrote or refreshed `design-contract.yml` or `cli-plan.yml`,
  because the existing validation report is then stale against the current
  adopted contract. Hand off to Validate first for the same reason when
  takeover added or modified `.gitignore` or any other baseline surface that
  Validate checks structurally. Only hand off directly to Publish when all of
  the following are true: takeover ran as `baseline_establishment`, it
  preserved the approved contracts without rewriting them, it preserved
  validation-covered baseline surfaces without changing them, and the
  validation report is fresh with aggregate result `compliant` or `warning`.
- `extend`: hand off to Extend only if the project already satisfies the
  scaffold-compatible layout and overlay API/dependency assumptions listed
  above. If those files or required surfaces are not present, stop and tell the
  user template-based extension is blocked until the repository is normalized
  to that layout/surface or the feature is implemented manually.

In every case, preserve any unresolved ambiguity as a blocking condition rather
than guessing.

## User Communication Rules

- Ask only focused questions that unblock a concrete contract decision.
- When code, docs, tests, or help output disagree, never pick a winner
  silently.
- When behavior exists but violates `cli-forge` standards, preserve the actual
  current behavior in the backfilled contract and warn the user that Validate
  is expected to surface the mismatch.
- When takeover rewrites `cli-plan.yml`, treat any pre-existing
  `validation-report.yml` as stale for Publish gating until Validate runs
  again against the refreshed plan.
- When takeover changes `.gitignore` or any other validation-covered baseline
  surface, treat any pre-existing `validation-report.yml` as stale for Publish
  gating until Validate runs again against the updated baseline.
- When the user asked for contract refresh, or when a downstream stage needs a
  takeover baseline receipt for an already-contracted project, do not reject
  the repository just because both contracts already exist. Use the recorded
  takeover mode to decide whether to preserve the existing contracts or
  reconstruct them from current evidence.
- When the user wants `extend`, never imply Takeover itself normalized the
  repository into scaffold layout. Confirm the required files, scaffold-style
  context/runtime API, and overlay dependency surface exist before routing to
  Extend.

## Done Condition

This instruction is complete only when:

- the evidence inventory is complete
- `design-contract.yml` is written and approved
- `cli-plan.yml` is written and approved
- `takeover-receipt.yml` is written
- the next stage is chosen from the recorded objective, or the workflow is
  explicitly blocked on a user decision or layout mismatch
