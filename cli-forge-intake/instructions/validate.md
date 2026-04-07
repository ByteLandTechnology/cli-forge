# Operation: Validate an Existing CLI Skill Project

## Purpose

Inspect an existing Rust CLI Skill project for compliance with the
cli-forge planning brief and report the results as a structured markdown
table in the conversation. The validation workflow must cover both the original
structure/build rules and the runtime-conventions surface introduced by feature
`002-cli-runtime-conventions`. It also verifies the generated package boundary:
baseline files must exist, enabled capability overlays may add package-local
support files, and repository-owned CI/release automation must not be treated
as required generated output. If repository docs mention release channels, keep
repo-native release and any optional npm publication wording distinct and
description-aligned.

## Inputs

| Input          | Required | Format         | Default | Description                                 |
| -------------- | -------- | -------------- | ------- | ------------------------------------------- |
| `project_path` | Yes      | Directory path | —       | Path to the Skill project root to validate. |

## Prerequisites

- `project_path` must point to a readable directory on disk.
- The directory must be intended to be a Rust CLI Skill project with `Cargo.toml` at the root.
- A working Rust toolchain must be available if you reach the build checks.

## Pre-Checks

Before running the ruleset:

1. Resolve `project_path` to an absolute path.
2. If the path does not exist, stop and report that the directory was not found.
3. If the path exists but is not a directory, stop and report that `project_path` must be a directory.
4. If `Cargo.toml` is missing, stop and report that the target is not a valid Skill project.
5. If `src/main.rs` is missing, continue only far enough to record the structural failure and skip build commands that require a compilable project.

## Severity Levels

| Severity  | Meaning                                                                | Agent Action                                                                                               |
| --------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `error`   | Required for planning-brief compliance or for a working CLI Skill      | Mark the row `FAIL` and tell the user the project must be fixed.                                           |
| `warning` | Recommended for maintainability or testability but not always blocking | Mark the row `FAIL` with severity `warning`, explain the gap, and continue running the rest of the checks. |

## Validation Rules

Run every rule below in order and record one output row per rule. Some task prose still says "27 checks", but the current `data-model.md` enumerates 28 concrete checks; use the table below as the source of truth and do not omit any of them.

| Check ID     | Category       | Principle | Severity | What to Verify                                                                                                                 |
| ------------ | -------------- | --------- | -------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `STRUCT-001` | structure      | Standards | error    | `SKILL.md` exists at the project root.                                                                                         |
| `STRUCT-002` | structure      | Standards | error    | `Cargo.toml` exists at the project root.                                                                                       |
| `STRUCT-003` | structure      | Standards | error    | `src/main.rs` exists.                                                                                                          |
| `STRUCT-004` | structure      | Standards | warning  | `src/lib.rs` exists.                                                                                                           |
| `STRUCT-005` | structure      | Standards | error    | `tests/cli_test.rs` exists.                                                                                                    |
| `NAME-001`   | naming         | Standards | error    | Directory name matches lowercase kebab-case: `[a-z][a-z0-9]*(-[a-z0-9]+)*`.                                                    |
| `NAME-002`   | naming         | Standards | error    | `[package].name` in `Cargo.toml` matches the directory name exactly.                                                           |
| `DEPS-001`   | dependencies   | VI        | error    | `clap` dependency exists and enables the `derive` feature.                                                                     |
| `DEPS-002`   | dependencies   | VI        | error    | `serde` dependency exists and enables the `derive` feature.                                                                    |
| `DEPS-003`   | dependencies   | VI        | error    | `serde_yaml` or `serde_yml` dependency exists.                                                                                 |
| `DEPS-004`   | dependencies   | VI        | error    | `serde_json` dependency exists.                                                                                                |
| `DEPS-005`   | dependencies   | VI        | error    | `toml` dependency exists.                                                                                                      |
| `DEPS-006`   | dependencies   | VI        | error    | `anyhow` or `thiserror` dependency exists.                                                                                     |
| `META-001`   | dependencies   | VI        | error    | `[package].name` is populated with a non-empty value.                                                                          |
| `META-002`   | dependencies   | VI        | error    | `[package].version` is populated with a non-empty value.                                                                       |
| `META-003`   | dependencies   | VI        | error    | `[package].edition` is populated with a non-empty value.                                                                       |
| `META-004`   | dependencies   | VI        | error    | `[package].description` is populated with a non-empty value.                                                                   |
| `SKILL-001`  | skill_md       | II        | error    | `## Description` section is present.                                                                                           |
| `SKILL-002`  | skill_md       | II        | error    | `## Prerequisites` section is present.                                                                                         |
| `SKILL-003`  | skill_md       | II        | error    | `## Invocation` section is present.                                                                                            |
| `SKILL-004`  | skill_md       | II        | error    | `## Input` section is present.                                                                                                 |
| `SKILL-005`  | skill_md       | II        | error    | `## Output` section is present.                                                                                                |
| `SKILL-006`  | skill_md       | II        | error    | `## Errors` section is present.                                                                                                |
| `SKILL-007`  | skill_md       | II        | error    | `## Examples` section is present.                                                                                              |
| `SKILL-008`  | skill_md       | II        | error    | Required sections appear in the canonical order. If `## REPL Mode` exists, it must appear between `## Output` and `## Errors`. |
| `BUILD-001`  | build          | VI        | error    | `cargo build` succeeds.                                                                                                        |
| `BUILD-002`  | build          | VI        | error    | `cargo clippy -- -D warnings` succeeds.                                                                                        |
| `BUILD-003`  | build          | VI        | error    | `cargo fmt --check` succeeds.                                                                                                  |
| `HELP-001`   | help           | III-D     | error    | `--help` remains plain-text only and exits `0`.                                                                                |
| `HELP-002`   | help           | III-D     | error    | Structured help is available through `help` only.                                                                              |
| `HELP-003`   | help           | III-D     | error    | Top-level and non-leaf auto-help exits `0` and lists subcommands.                                                              |
| `HELP-004`   | help           | III-D     | error    | Structured `help` output documents command path, options, defaults, output formats, runtime directories, and Active Context.   |
| `DIR-001`    | runtime_dirs   | III       | error    | Config/data/state/cache are documented separately.                                                                             |
| `DIR-002`    | runtime_dirs   | III       | error    | Runtime directory defaults are user-scoped unless explicitly overridden.                                                       |
| `DIR-003`    | runtime_dirs   | III       | warning  | Optional log location is documented separately when logging is enabled.                                                        |
| `CTX-001`    | active_context | III       | error    | Active Context can be inspected.                                                                                               |
| `CTX-002`    | active_context | III       | error    | Active Context can be switched or persisted.                                                                                   |
| `CTX-003`    | active_context | III       | error    | Explicit per-invocation overrides take precedence without mutating persisted defaults.                                         |
| `CTX-004`    | active_context | III       | error    | Effective context remains visible to the user.                                                                                 |
| `ERR-001`    | errors         | III       | error    | Leaf-command validation failures preserve the selected output format.                                                          |
| `ERR-002`    | errors         | III       | error    | Structured errors include at least `code` and `message`.                                                                       |
| `REPL-001`   | repl           | III-C     | error    | REPL help is plain-text only.                                                                                                  |
| `REPL-002`   | repl           | III-C     | error    | REPL supports command history.                                                                                                 |
| `REPL-003`   | repl           | III-C     | error    | REPL supports tab completion.                                                                                                  |
| `REPL-004`   | repl           | III-C     | warning  | REPL default output behavior is human-readable.                                                                                |

## Steps

### Step 1: Load Project Metadata

1. Read `Cargo.toml`.
2. Parse `[package]`, `[dependencies]`, and `[dev-dependencies]`.
3. Record parsing errors as failures in the relevant metadata or dependency rows.

### Step 2: Run Structure Checks

Inspect the root layout and create rows for `STRUCT-001` through `STRUCT-005`.

- For each required file, mark:
  - `PASS` when the file exists where expected.
  - `FAIL` when missing.
- In the `Details` column, state the exact path that was found or missing.

Also classify package-boundary expectations while reviewing structure:

- baseline generated outputs are `SKILL.md`, `Cargo.toml`, `src/`, `tests/`,
  and optional human-facing docs such as `README.md`
- package-local packaging-ready metadata or support fixtures are allowed only
  when the enabled capability requires them
- repository-owned CI workflows, release scripts, and release automation are
  not required generated outputs and should not be treated as missing files

### Step 3: Run Naming Checks

1. Compare the directory basename of `project_path` against the kebab-case regex for `NAME-001`.
2. Compare the directory basename to `[package].name` for `NAME-002`.
3. If `Cargo.toml` could not be parsed, mark `NAME-002` as `FAIL` and explain that package metadata could not be read.

### Step 4: Run Dependency and Metadata Checks

Inspect `Cargo.toml` for `DEPS-001` through `DEPS-006` and `META-001` through `META-004`.

- Treat a dependency as present only if it is declared in `[dependencies]`.
- For `clap` and `serde`, confirm the `derive` feature is enabled.
- For `serde_yaml`, accept either `serde_yaml` or `serde_yml`.
- For `anyhow` / `thiserror`, either crate satisfies `DEPS-006`.
- For metadata rows, non-empty string values are required.

### Step 5: Run `SKILL.md` Contract Checks

1. Read `SKILL.md` only if it exists.
2. Search for the required section headings:
   - `## Description`
   - `## Prerequisites`
   - `## Invocation`
   - `## Input`
   - `## Output`
   - optional `## REPL Mode`
   - `## Errors`
   - `## Examples`
3. Mark `SKILL-001` through `SKILL-007` based on the presence of each required section.
4. For `SKILL-008`, verify that the sections appear in canonical order. If any required section is missing, still evaluate order based on the headings that do exist and explain which requirement prevented a full pass.
5. If `SKILL.md` is missing entirely, mark `SKILL-001` through `SKILL-008` as `FAIL` with `error:` details that explain the file is absent; this is how the report surfaces the Principle II contract violation in addition to `STRUCT-001`.

### Step 6: Run Build Checks

Run build commands only when `Cargo.toml` and `src/main.rs` exist. Prefer running them from the project root in this order:

1. `cargo build`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`

For each command:

- `PASS` if the command exits `0`.
- `FAIL` if the command exits non-zero.
- Capture the key stderr/stdout reason in the `Details` column.
- If an earlier structural failure makes the command impossible or misleading, mark the row `FAIL` and explain that the build checks were skipped because the project is incomplete.

### Step 7: Run Runtime Convention Checks

When the project exposes the generated runtime-conventions surface, inspect and
record the following:

1. **Help channels**
   - Confirm `--help` stays plain-text.
   - Confirm `help --format yaml|json|toml` returns structured help.
   - Confirm top-level or non-leaf invocation without a leaf command returns
     plain-text help with subcommands and exit `0`.
2. **Runtime directories**
   - Confirm config/data/state/cache are documented separately.
   - Confirm user-scoped defaults are documented.
   - If logging is supported, confirm the log path is documented separately.
3. **Active Context**
   - Confirm there is a visible way to inspect the current Active Context.
   - Confirm there is a visible way to switch or persist it.
   - Confirm explicit per-invocation overrides take precedence and do not
     silently mutate the persisted context.
4. **Errors**
   - Confirm missing leaf-command inputs return a structured error in the
     selected output format rather than raw help text.
   - Confirm structured errors include at least `code` and `message`.
5. **REPL**
   - If REPL mode is present, confirm REPL help is plain text only.
   - Confirm REPL history and tab completion are available.
   - Confirm default REPL output favors readability while any explicit
     structured result modes stay documented and consistent.

## Output Format

Return the report directly in the conversation as a markdown table with exactly these columns:

| Check ID | Category | Principle | Status | Details |
| -------- | -------- | --------- | ------ | ------- |

Formatting rules:

- `Status` must be `PASS` or `FAIL`.
- Prefix `Details` with the severity, for example `error:` or `warning:`.
- Include a short summary after the table:
  - total passed checks
  - total failed checks
  - whether the project is planning-brief-compliant overall

## Error Conditions

| Condition                         | Action                                                                                                  |
| --------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `project_path` does not exist     | Stop immediately and report the missing path.                                                           |
| `project_path` is not a directory | Stop immediately and report the invalid input type.                                                     |
| `Cargo.toml` is missing           | Stop immediately and report that the target is not a valid Skill project.                               |
| `Cargo.toml` cannot be parsed     | Continue with file presence checks, but mark dependency and metadata rows as failed with parse details. |
| Required files are missing        | Continue with the remaining non-build checks so the user receives a full report.                        |
| Build command fails               | Record the failure details; do not hide earlier passing rows.                                           |

## Final Reporting Behavior

- If every `error` row passes and there are no failed `warning` rows, report the project as compliant.
- If any `error` row fails, report the project as non-compliant.
- If only `warning` rows fail, report the project as usable but needing follow-up improvements.
