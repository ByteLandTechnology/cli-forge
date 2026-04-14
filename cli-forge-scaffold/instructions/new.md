# Operation: Scaffold a New CLI Skill

## Purpose

Create a new CLI Skill project directory with all required files, correct
structure, and working boilerplate that compiles and passes linting without any
manual modification. The scaffolded CLI must include the shared runtime
conventions from this Skill package: plain-text `--help`, structured `help`,
stable structured errors, runtime-directory helpers, and Active Context
boilerplate. This operation consumes the description contract approved by the
earlier `description` stage; it does not invent a competing skill summary.
The current scaffold baseline does not implement the planned daemon app-server
capability.

## Inputs

| Input          | Required | Format                                    | Default                   | Description                                                                         |
| -------------- | -------- | ----------------------------------------- | ------------------------- | ----------------------------------------------------------------------------------- |
| `skill_name`   | Yes      | kebab-case: `[a-z][a-z0-9]*(-[a-z0-9]+)*` | —                         | Name of the new Skill. Becomes directory name, Cargo package name, and binary name. |
| `author`       | No       | `Name <email>` or just `Name`             | (omitted from Cargo.toml) | Author for `Cargo.toml` authors field.                                              |
| `version`      | No       | Semver string                             | `0.1.0`                   | Initial version.                                                                    |
| `rust_edition` | No       | Year string                               | `2024`                    | Rust edition for `Cargo.toml`.                                                      |

**Description contract fields** (carry them in from `description`; do NOT ask the user
for these again during scaffold):

| Field         | How to Generate                                                                                                                                                |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `description` | Use the approved one-line purpose summary from the `description` stage. Keep it identical across Cargo, `SKILL.md`, README, and help text. No trailing period. |

## Pre-Checks

Before proceeding, verify:

1. **Name validation**: `skill_name` must match `[a-z][a-z0-9]*(-[a-z0-9]+)*`. If invalid:
   - Reject the name.
   - Suggest a corrected kebab-case version (e.g., `My Tool` → `my-tool`, `search_web` → `search-web`).
   - Wait for user confirmation before proceeding.

2. **Name provided**: If the user did not provide a skill name, ask for one before proceeding.

3. **Directory does not exist**: Check that a directory named `{skill_name}` does not already exist at the target location. If it does, refuse to overwrite and inform the user.

## Token Registry

When expanding templates, replace every `{{TOKEN_NAME}}` with its corresponding value. All tokens use double-curly braces and SCREAMING_SNAKE_CASE.

| Token                   | Derivation                                     | Example (for `search-web`)                     |
| ----------------------- | ---------------------------------------------- | ---------------------------------------------- |
| `{{SKILL_NAME}}`        | Directly from `skill_name` input               | `search-web`                                   |
| `{{SKILL_NAME_SNAKE}}`  | Replace `-` with `_` in skill_name             | `search_web`                                   |
| `{{SKILL_NAME_PASCAL}}` | Capitalize first letter of each segment        | `SearchWeb`                                    |
| `{{SKILL_NAME_UPPER}}`  | Uppercase the entire skill_name (keep hyphens) | `SEARCH-WEB`                                   |
| `{{DESCRIPTION}}`       | Agent-generated one-line summary               | `Search the web and return structured results` |
| `{{VERSION}}`           | From input or default `0.1.0`                  | `0.1.0`                                        |
| `{{AUTHOR}}`            | From input or omit the `authors` line          | `Yu Yang <yu@example.com>`                     |
| `{{CURRENT_DATE}}`      | Today's date in ISO 8601                       | `2026-03-27`                                   |
| `{{RUST_EDITION}}`      | From input or default `2024`                   | `2024`                                         |

## Steps

### Step 1: Create Directory Structure

Create the following directories:

```
{skill_name}/
├── src/
└── tests/
```

This baseline package layout is the minimum generated skill output. Later
capability overlays may add package-local support files or metadata, but
repository-owned CI workflows, release scripts, and release automation are not
copied into generated skill directories by default.

### Step 2: Expand Templates

For each template file, read it from the `templates/` directory in this Skill package, replace all `{{TOKEN_NAME}}` placeholders with actual values, and write the expanded file to the target path.

| Template                    | Write To                         |
| --------------------------- | -------------------------------- |
| `templates/.gitignore.tpl`  | `{skill_name}/.gitignore`        |
| `templates/Cargo.toml.tpl`  | `{skill_name}/Cargo.toml`        |
| `templates/main.rs.tpl`     | `{skill_name}/src/main.rs`       |
| `templates/lib.rs.tpl`      | `{skill_name}/src/lib.rs`        |
| `templates/help.rs.tpl`     | `{skill_name}/src/help.rs`       |
| `templates/context.rs.tpl`  | `{skill_name}/src/context.rs`    |
| `templates/SKILL.md.tpl`    | `{skill_name}/SKILL.md`          |
| `templates/cli_test.rs.tpl` | `{skill_name}/tests/cli_test.rs` |
| `templates/README.md.tpl`   | `{skill_name}/README.md`         |

**Important**: After expansion, verify no `{{` or `}}` token markers remain in any generated file. If any remain, you missed a token — go back and fix it.

The expanded files must reuse one approved description contract:

- `Cargo.toml` package description
- `SKILL.md` frontmatter and `## Description`
- `README.md` overview
- structured and plain-text help summaries

The expanded files must also preserve the invocation contract:

- `SKILL.md` must use the bare command name as the final agent-facing contract.
- `README.md` may additionally show `cargo run -- ...` for local development.
- `README.md` may additionally show `./target/release/{skill_name} ...` for a
  built release binary.
- Tests should exercise the compiled CLI binary rather than relying on
  `cargo run -- ...` as the primary verification path.
- Do not present `cargo run -- ...` as the canonical installed skill
  interface.
- If the target repo later adopts repo-native release automation, its clone
  `->` checkout `->` install helper flow must remain a repository-owned concern
  that reuses this same description contract.
- If the target repo later adopts Publish-stage npm publication, the npm
  package wording must keep the same approved description contract and the same
  released version as the repo-native GitHub Release.

If `cli-plan.yml` marks `capabilities.daemon: in_scope`, stop before
generation and report that scaffold support for the daemon app-server contract
is not implemented yet. Do not silently emit the old managed-daemon placeholder
and do not drop daemon from the generated package without telling the user.

If `author` was omitted:

- Delete the `authors = [""]` line from `Cargo.toml`.
- Delete the entire `## Author` section from `README.md`.

### Step 3: Verify Output

After writing all files, verify:

1. **All 9 files exist** in the generated directory:
   - `.gitignore`
   - `Cargo.toml`
   - `src/main.rs`
   - `src/lib.rs`
   - `src/help.rs`
   - `src/context.rs`
   - `SKILL.md`
   - `tests/cli_test.rs`
   - `README.md`

2. **Project compiles**: Run `cargo build` in the generated directory. Must succeed with zero errors and zero warnings.

3. **Linting passes**: Run `cargo clippy -- -D warnings`. Must pass with zero issues.

4. **Formatting passes**: Run `cargo fmt --check`. Must pass with zero issues.

5. **Tests pass**: Run `cargo test`. Must pass.

6. **Runtime contract is present**:
   - Top-level invocation should auto-display plain-text help and exit `0`.
   - `help run --format yaml` should return structured help.
   - `paths` should document user-scoped runtime directories.
   - `context show` should expose the generated Active Context surface.
   - `SKILL.md` should document the bare command contract, while `README.md`
     distinguishes canonical invocation from local development and
     release-binary invocation.
   - CLI tests should exercise the compiled binary path rather than depending
     on `cargo run -- ...`.
7. **Package boundary is respected**:
   - The generated project contains only the documented baseline files plus
     any package-local support files required by enabled capabilities.
   - Repository-owned CI workflows, release scripts, and release automation are
     not copied into the generated project by default.
   - Repository-owned install helpers such as `scripts/install-current-release.sh`
     are likewise not copied into the generated project by default.
   - Optional npm publication references, if later added in repository docs,
     remain repository-owned release guidance rather than generated package
     runtime files.

If any check fails, fix the generated files and re-verify.

### Step 4: Report

Tell the user:

- The project was created at `{skill_name}/`.
- List the generated files.
- Confirm that build, lint, format, tests, and basic runtime-contract checks all pass.

## Error Conditions

| Condition                               | Action                                                                   |
| --------------------------------------- | ------------------------------------------------------------------------ |
| Skill name contains invalid characters  | Reject. Suggest corrected kebab-case name.                               |
| Skill name is empty / not provided      | Ask user for a name.                                                     |
| Target directory already exists         | Refuse. Inform user the directory exists.                                |
| Template file not found                 | Report which template is missing. This indicates a broken Skill package. |
| Build/lint/format fails after expansion | Fix the generated files. Do not leave a broken project.                  |
