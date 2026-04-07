# Operation: Add Optional Feature Boilerplate

## Purpose

Add either streaming output support or REPL mode to an existing scaffolded CLI
Skill project by expanding the matching template and applying the documented
source patches. Feature additions must preserve the generated runtime
conventions already present in the scaffold, including plain-text `--help`,
structured `help`, user-scoped runtime directories, Active Context support,
and accurate structured-help metadata for newly enabled features. When a
feature request also changes the generated skill's user-facing purpose or
positioning, route through the `description` stage before continuing with this
operation.

## Inputs

| Input          | Required | Format             | Default | Description                                    |
| -------------- | -------- | ------------------ | ------- | ---------------------------------------------- |
| `feature`      | Yes      | `stream` or `repl` | —       | Which optional capability to add.              |
| `project_path` | Yes      | Directory path     | —       | Path to the existing scaffolded Skill project. |

## Prerequisites

- `project_path` must already contain a scaffolded Rust CLI Skill created from this Skill package or an equivalent structure.
- The project must include `Cargo.toml`, `SKILL.md`, `src/main.rs`,
  `src/lib.rs`, and `src/help.rs`.
- The Agent must be able to read templates from this Skill package and write files into the target project.

## Pre-Checks

Before making any edits:

1. Resolve `project_path` to an absolute path.
2. Confirm `Cargo.toml` and `SKILL.md` exist at the project root. If either is missing, stop: this is not a valid Skill project.
3. Confirm `src/main.rs`, `src/lib.rs`, and `src/help.rs` exist. If any are
   missing, stop and explain which scaffolded file is absent.
4. Validate `feature` is exactly `stream` or `repl`. Reject any other value.
5. Detect existing features:
   - If `feature == stream` and `src/stream.rs` already exists, stop and tell the user streaming is already present.
   - If `feature == repl` and `src/repl.rs` already exists, stop and tell the user REPL is already present.

## Common Procedure

1. Read the target project's current `Cargo.toml`, `src/main.rs`, `src/lib.rs`,
   `src/help.rs`, `SKILL.md`, and `tests/cli_test.rs` if it exists.
2. Identify the crate name from `[package].name`; use the snake_case crate path already present in `src/main.rs`.
3. Expand the matching template from this Skill package:
   - `templates/stream.rs.tpl` -> `src/stream.rs`
   - `templates/repl.rs.tpl` -> `src/repl.rs`
4. Apply the code snippets below to the target files.
   - If the project already has the other optional feature, keep exactly one
     output-construction block inside `execute_run` and place feature-specific
     branches immediately before the default one-shot serialization path.
   - Keep `--repl` as an early global branch before the normal command match.
   - Keep `--stream` scoped to the `run` command's output path instead of
     changing help, context, or runtime-directory commands.
   - Preserve the existing `help` subcommand plus any context/runtime-directory
     command surfaces already scaffolded into the project.
   - Keep `src/help.rs` synchronized by updating the global option list and
     `FeatureAvailability` values for the newly enabled feature.
   - Do not replace or bypass the shared daemon contract when the project also
     exposes daemon behavior. Managed background daemon control remains
     `daemon start|stop|restart|status`, with attached foreground execution
     out of scope, single-instance control as the default unless the project
     explicitly documents otherwise, and unsupported runtimes kept out of the
     supported surface.
   - Keep capability-specific support files package-local to the generated
     project. Repository-owned CI workflows, release scripts, and release
     automation stay outside generated skill outputs.
   - If the repository also uses repo-native release automation, keep any
     release/install docs aligned with the same skill description and do not
     make feature docs depend on a parent repository layout.
   - If the repository also documents optional npm publication, keep the same
     approved description contract and version framing across repo-native and
     npm wording, with npm still described as a secondary CLI distribution
     path.
5. Run validation commands after the edits:
   - `cargo build`
   - `cargo test`
   - `cargo clippy -- -D warnings`
   - `cargo fmt --check`
6. If validation fails, fix the generated code before reporting success.

## Feature: `stream`

### Step-by-Step

1. Expand `templates/stream.rs.tpl` to `src/stream.rs`.
2. Add the `--stream` flag and dispatch branch to `src/main.rs`.
3. Add `pub mod stream;` to `src/lib.rs`.
4. Update `src/help.rs` so the global option list includes `--stream` and each
   `FeatureAvailability` block marks streaming as enabled.
5. Update `SKILL.md` so the `Output` section documents streaming behavior and
   the unsupported TOML case.

### `main.rs` Patch Snippet

```diff
 #[derive(Parser, Debug)]
 #[command(
     name = "<skill-name>",
     version,
     about = "<description>",
     disable_help_flag = true,
     disable_help_subcommand = true
 )]
 struct Cli {
     ...
+
+    /// Emit run-command records incrementally using the selected streaming protocol
+    #[arg(long, global = true)]
+    stream: bool,
 }

 fn run_cli() -> std::result::Result<(), AppExit> {
     ...
     let runtime_overrides = cli_runtime_overrides(&cli);
+    let stream = cli.stream;

     match cli.command {
         None => render_plain_text_help_for_path(&[]),
         Some(Command::Help(command)) => render_structured_help(&command.path, format),
-        Some(Command::Run(command)) => execute_run(runtime_overrides, command, format),
+        Some(Command::Run(command)) => execute_run(runtime_overrides, command, format, stream),
         Some(Command::Paths(command)) => execute_paths(runtime_overrides, command, format),
         Some(Command::Context(command)) => execute_context(runtime_overrides, command, format),
     }
 }

 fn execute_run(
     overrides: RuntimeOverrides,
     command: RunCommand,
     format: Format,
+    stream: bool,
 ) -> std::result::Result<(), AppExit> {
     ...
     let output = RunResponse { ... };

+    if stream {
+        return <crate_name>::stream::stream_value(&output, format).map_err(AppExit::from);
+    }
+
     let stdout = std::io::stdout();
     let mut stdout = stdout.lock();
     serialize_value(&mut stdout, &output, format).map_err(AppExit::from)?;
     Ok(())
 }
```

### `lib.rs` Patch Snippet

```diff
 pub mod context;
 pub mod help;
+pub mod stream;
```

### `help.rs` Patch Snippet

Update every `FeatureAvailability` block so streaming is marked as enabled:

```diff
 feature_availability: FeatureAvailability {
-    streaming: "optional add-on".to_string(),
+    streaming: "enabled".to_string(),
     repl: "optional add-on".to_string(),
 }
```

Also add `--stream` to the global options rendered for the top-level and leaf
command paths so both plain-text and structured help stay accurate after the
feature is added.

### `SKILL.md` Patch Snippet

Add the following paragraphs inside `## Output` after the one-shot format description:

```md
### Streaming Mode

- Activate streaming with `--stream`.
- `--stream --format yaml` writes YAML multi-document output using `---` before each record and `...` at the end.
- `--stream --format json` writes NDJSON, one compact JSON object per line.
- `--stream --format toml` is unsupported and must fail with a non-zero exit code and an explanation on stderr.
```

## Feature: `repl`

### Step-by-Step

1. Expand `templates/repl.rs.tpl` to `src/repl.rs`.
2. Add the `--repl` flag and dispatch branch to `src/main.rs`.
3. Add `pub mod repl;` to `src/lib.rs`.
4. Update `src/help.rs` so the global option list includes `--repl` and each
   `FeatureAvailability` block marks REPL as enabled.
5. Update `SKILL.md` with a `REPL Mode` section between `Output` and `Errors`.
6. Ensure the generated REPL behavior remains human-oriented:
   - in-session REPL help is plain-text only
   - command history is persisted under the CLI state root
   - tab completion is enabled for command vocabulary and visible context values
   - default REPL output favors readability, while any explicit structured
     result modes continue to honor the selected startup format
7. Keep existing integration tests passing, and keep the unit tests bundled
   inside `src/repl.rs` when the template provides them.

### `main.rs` Patch Snippet

```diff
 #[derive(Parser, Debug)]
 #[command(
     name = "<skill-name>",
     version,
     about = "<description>",
     disable_help_flag = true,
     disable_help_subcommand = true
 )]
 struct Cli {
     ...

+    /// Start an interactive read-eval-print loop
+    #[arg(long, global = true)]
+    repl: bool,
 }
```

Then, inside the main dispatch path, insert the REPL branch immediately after
`let runtime_overrides = cli_runtime_overrides(&cli);` and before the normal
command match:

```diff
     let runtime_overrides = cli_runtime_overrides(&cli);

+    if cli.repl {
+        let runtime =
+            <crate_name>::context::resolve_runtime_locations(&runtime_overrides, false)?;
+        return <crate_name>::repl::start_repl(format, runtime);
+    }

     match cli.command {
         ...
     }
```

### `help.rs` Patch Snippet

Update every `FeatureAvailability` block so REPL is marked as enabled:

```diff
 feature_availability: FeatureAvailability {
     streaming: "optional add-on".to_string(),
-    repl: "optional add-on".to_string(),
+    repl: "enabled".to_string(),
 }
```

Also add `--repl` to the global options rendered for the top-level and leaf
command paths so both plain-text and structured help stay accurate after the
feature is added.

### `lib.rs` Patch Snippet

```diff
+pub mod repl;
+
 pub mod context;
 pub mod help;
```

### `SKILL.md` Patch Snippet

Insert this section immediately after `## Output` and before `## Errors`:

```md
## REPL Mode

- Start interactive mode with `--repl`.
- The prompt must be `<skill-name>> ` and must be written to stderr.
- `help` inside the REPL is plain text only and explains available commands,
  context inspection controls, and output behavior.
- Each input line is handled as one command and the result is written to stdout.
  Default session output may be more human-readable than one-shot YAML, but any
  explicitly supported structured result mode must stay consistent with the
  startup `--format`.
- Command history must persist under the runtime state directory.
- Tab completion must be available for command names, option names, and visible
  context values.
- `exit`, `quit`, or EOF end the session with exit code `0`.
- Per-command errors are written to stderr and do not terminate the REPL.
```

## Combining Features

If the project already has one optional feature and you are adding the other:

- Keep both flags in the same `Cli` struct.
- Keep `--repl` as the early session-mode branch before the normal command
  match, and keep `--stream` scoped to `execute_run`.
- Keep exactly one `let output = ...;` immediately before the default one-shot
  serialization path inside `execute_run`.
- Keep both module declarations in `src/lib.rs`, and leave them in
  rustfmt-compatible order:

```rust
pub mod context;
pub mod help;
pub mod repl;
pub mod stream;
```

- Keep `src/help.rs` synchronized with the enabled feature set by updating the
  global option list and `FeatureAvailability` values.

- Preserve the `SKILL.md` section order: `Description`, `Prerequisites`, `Invocation`, `Input`, `Output`, optional `REPL Mode`, `Errors`, `Examples`.

## Error Conditions

| Condition                                          | Action                                                |
| -------------------------------------------------- | ----------------------------------------------------- |
| `feature` is not `stream` or `repl`                | Reject the request and ask for a supported feature.   |
| `project_path` does not exist                      | Stop and report the missing path.                     |
| `project_path` is not a scaffolded Skill project   | Stop and explain which required file is missing.      |
| Requested feature file already exists              | Stop and tell the user no changes were made.          |
| Template file is missing from this Skill package   | Stop and report that the Skill package is incomplete. |
| Build/lint/format fails after applying the feature | Fix the project before reporting success.             |

## Final Reporting Behavior

After a successful feature addition, tell the user:

- which feature was added
- which files were created or modified
- that build, test, clippy, and formatting checks pass
