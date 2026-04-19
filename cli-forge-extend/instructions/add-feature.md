# Operation: Add Optional Feature Boilerplate

## Purpose

Add streaming output support, REPL mode, or daemon app-server capability to an
existing scaffolded or scaffold-compatible takeover-adopted CLI Skill project
by expanding the matching template and applying the documented source patches.
Feature additions must preserve the generated runtime
conventions already present in the scaffold, including the four help scenarios
(leaf default structured failure, non-leaf default human-readable help,
`--help` human-readable help, and structured `help`), user-scoped runtime
directories, Active Context support, and accurate structured-help metadata for
newly enabled features. Human-readable help must remain man-like with the
canonical section order `NAME -> SYNOPSIS -> DESCRIPTION -> OPTIONS -> FORMATS
-> EXAMPLES -> EXIT CODES`. When a feature request also changes the generated
skill's user-facing purpose or positioning, route through the `description`
stage before continuing with this operation.

## Inputs

| Input          | Required | Format             | Default | Description                                    |
| -------------- | -------- | ------------------ | ------- | ---------------------------------------------- |
| `feature`      | Yes      | `stream`, `repl`, or `daemon` | —       | Which optional capability to add.              |
| `project_path` | Yes      | Directory path     | —       | Path to the existing scaffolded or takeover-adopted Skill project. |

## Prerequisites

- `project_path` must already contain a scaffolded Rust CLI Skill created from
  this Skill package or a takeover-adopted project that already matches the
  same scaffold-compatible structure.
- The project must include `Cargo.toml`, `SKILL.md`, `src/main.rs`,
  `src/lib.rs`, `src/help.rs`, and `src/context.rs`.
- The Agent must be able to read templates from this Skill package and write files into the target project.

## Pre-Checks

Before making any edits:

1. Resolve `project_path` to an absolute path.
2. Confirm `Cargo.toml` and `SKILL.md` exist at the project root. If either is missing, stop: this is not a valid Skill project.
3. Confirm `src/main.rs`, `src/lib.rs`, `src/help.rs`, and `src/context.rs`
   exist. If any are missing, stop and explain which scaffold-compatible file
   is absent.
4. Confirm at least one baseline receipt exists in `.cli-forge/`:
   `scaffold-receipt.yml` or `takeover-receipt.yml`. If neither is present,
   stop and route the project through Takeover for baseline establishment or
   refresh first, even when `design-contract.yml` and `cli-plan.yml` already
   exist.
5. If the project only has `takeover-receipt.yml`, also confirm the rest of
   the scaffold-compatible patch surface and overlay-required API/dependency
   surface exist:
   - `Cargo.toml`
   - `SKILL.md`
   - `src/context.rs`
   - `tests/cli_test.rs` when the feature workflow is expected to patch the
     generated integration tests directly
   - `src/context.rs` must expose the scaffold-style context/runtime helpers
     the overlays call directly, including `resolve_runtime_locations`
   - for `repl`, `Cargo.toml` must already carry `rustyline`, and
     `src/context.rs` must expose the context API used by `repl.rs.tpl`
   If these files or required surfaces are missing, stop and tell the user
   normalization to the scaffold-compatible layout/surface, or manual feature
   implementation, is required.
6. Validate `feature` is exactly `stream`, `repl`, or `daemon`. Reject any
   other value.
7. Detect existing features:
   - If `feature == stream` and `src/stream.rs` already exists, stop and tell the user streaming is already present.
   - If `feature == repl` and `src/repl.rs` already exists, stop and tell the user REPL is already present.
   - If `feature == daemon` and `src/daemon.rs` already exists, stop and tell the user daemon is already present.

## Common Procedure

1. Read the target project's current `Cargo.toml`, `src/main.rs`, `src/lib.rs`,
   `src/help.rs`, `src/context.rs`, `SKILL.md`, and `tests/cli_test.rs` when
   present. If a takeover-adopted project is missing any file or overlay API/
   dependency surface that the documented patches modify or call directly, stop
   and tell the user this stage cannot safely apply the templates until the
   repository is normalized or the feature is implemented manually.
2. Identify the crate name from `[package].name`; use the snake_case crate path already present in `src/main.rs`.
3. Expand the matching template from this Skill package:
   - `templates/stream.rs.tpl` -> `src/stream.rs`
   - `templates/repl.rs.tpl` -> `src/repl.rs`
   - `templates/daemon.rs.tpl` -> `src/daemon.rs`
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
   - Preserve the four help scenarios exactly: leaf defaults stay structured,
     non-leaf defaults auto-render man-like help, `--help` stays man-like, and
     `help` stays structured.
   - Keep the man-like section sequence intact whenever feature-specific help
     text is added.
   - Do not replace or bypass a project's declared daemon contract when the
     project also exposes daemon behavior. Preserve the daemon command surface,
     routing flags, transport choices, recovery semantics, and documented
     local-only command boundaries already recorded in `cli-plan.yml`.
   - Keep capability-specific support files package-local to the generated
     project. Repository-owned CI workflows, release scripts, and release
     automation stay outside generated skill outputs.
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
     daemon: "optional add-on".to_string(),
 }
```

Also add `--stream` to the global options rendered for the top-level and leaf
command paths so man-like human-readable help and structured help both stay
accurate after the feature is added.

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
     daemon: "optional add-on".to_string(),
 }
```

Also add `--repl` to the global options rendered for the top-level and leaf
command paths so man-like human-readable help and structured help both stay
accurate after the feature is added.

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

## Feature: `daemon`

### Step-by-Step

1. Expand `templates/daemon.rs.tpl` to `src/daemon.rs`.
2. Add the daemon subcommand tree and routing flags to `src/main.rs`.
3. Add `pub mod daemon;` to `src/lib.rs`.
4. Add `uuid = { version = "1", features = ["v4"] }` to `Cargo.toml` under
   `[dependencies]`.
5. Update `src/help.rs` so daemon commands, routing flags, and
   `FeatureAvailability.daemon` are documented.
6. Update `SKILL.md` with a `Daemon Mode` section between `Output` (or
   `REPL Mode` if present) and `Errors`.

### `main.rs` Patch Snippet

Add the daemon subcommand enum:

```diff
 #[derive(Subcommand, Debug)]
 enum Command {
     Help(HelpCommand),
     Run(RunCommand),
     Paths(PathsCommand),
     Context(ContextCommand),
+    #[command(subcommand)]
+    Daemon(DaemonCommand),
 }

+#[derive(Subcommand, Debug)]
+enum DaemonCommand {
+    /// Run the daemon server in the foreground (blocks terminal)
+    Run,
+    /// Start the daemon server in the background
+    Start,
+    /// Stop the running daemon server
+    Stop,
+    /// Restart the daemon server
+    Restart,
+    /// Report daemon health, endpoint, pid, uptime, and next action
+    Status,
+}
```

Add `--via` and `--ensure-daemon` flags to the `Cli` struct:

```diff
 struct Cli {
     ...
+    /// Route execution through the daemon instead of local process
+    #[arg(long, global = true, value_name = "MODE")]
+    r#via: Option<String>,
+
+    /// Auto-start the daemon if not running (only valid with --via daemon)
+    #[arg(long, global = true)]
+    ensure_daemon: bool,
 }
```

Add the daemon command dispatch branch inside `match cli.command`. The
`execute_run` line may or may not already carry a `stream` argument depending
on whether the stream feature has been added first. Match whatever is already
present and do not add or remove arguments to `execute_run` at this point.

```diff
         Some(Command::Context(command)) => execute_context(runtime_overrides, command, format),
+        Some(Command::Daemon(cmd)) => execute_daemon(runtime_overrides, cmd, format),
     }
```

Also add daemon paths to the `render_plain_text_help_for_cli` match block:

```diff
         Some(Command::Context(ContextCommand {
             command: Some(ContextSubcommand::Use(_)),
         })) => vec!["context".to_string(), "use".to_string()],
+        Some(Command::Daemon(DaemonCommand::Run)) => vec!["daemon".to_string(), "run".to_string()],
+        Some(Command::Daemon(DaemonCommand::Start)) => vec!["daemon".to_string(), "start".to_string()],
+        Some(Command::Daemon(DaemonCommand::Stop)) => vec!["daemon".to_string(), "stop".to_string()],
+        Some(Command::Daemon(DaemonCommand::Restart)) => vec!["daemon".to_string(), "restart".to_string()],
+        Some(Command::Daemon(DaemonCommand::Status)) => vec!["daemon".to_string(), "status".to_string()],
     };
```

Add the `execute_daemon` function:

```rust
fn execute_daemon(
    overrides: RuntimeOverrides,
    command: DaemonCommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    let runtime = <crate_name>::context::resolve_runtime_locations(&overrides, false)
        .map_err(AppExit::from)?;
    let state_dir = &runtime.state_dir;
    let skill_name = "<skill-name>";
    let version = env!("CARGO_PKG_VERSION");

    match command {
        DaemonCommand::Run => {
            <crate_name>::daemon::run_daemon(state_dir, skill_name, version)
                .map_err(AppExit::from)
        }
        DaemonCommand::Start => {
            let status = <crate_name>::daemon::start_daemon(
                state_dir, skill_name, std::time::Duration::from_secs(30),
            ).map_err(AppExit::from)?;
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            serialize_value(&mut out, &status, format).map_err(AppExit::from)
        }
        DaemonCommand::Stop => {
            <crate_name>::daemon::stop_daemon(
                state_dir, std::time::Duration::from_secs(10),
            ).map_err(AppExit::from)
        }
        DaemonCommand::Restart => {
            let status = <crate_name>::daemon::restart_daemon(
                state_dir, skill_name, std::time::Duration::from_secs(30),
            ).map_err(AppExit::from)?;
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            serialize_value(&mut out, &status, format).map_err(AppExit::from)
        }
        DaemonCommand::Status => {
            let status = <crate_name>::daemon::query_status(state_dir)
                .map_err(AppExit::from)?;
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            serialize_value(&mut out, &status, format).map_err(AppExit::from)
        }
    }
}
```

Add daemon routing inside `execute_run`, after input validation succeeds and
before the default one-shot serialization. The `via` and `ensure_daemon`
values come from the `Cli` struct — extract them in `run_cli` and pass them
through to `execute_run` as new parameters:

In `run_cli`, after `let runtime_overrides = cli_runtime_overrides(&cli);`
(and after the `--repl` early branch if present), extract:

```rust
    let via = cli.r#via.clone();
    let ensure_daemon = cli.ensure_daemon;
```

Update the `execute_run` call in the match arm to forward these:

```rust
    Some(Command::Run(command)) => execute_run(
        runtime_overrides, command, format,
        /* existing args like `stream` if present, then: */
        via, ensure_daemon,
    ),
```

Then extend `execute_run`'s signature with the two new parameters:

```rust
fn execute_run(
    overrides: RuntimeOverrides,
    command: RunCommand,
    format: Format,
    // ... any existing params from stream/repl ...
    via: Option<String>,
    ensure_daemon: bool,
) -> std::result::Result<(), AppExit> {
```

Inside `execute_run`, after constructing `let output = RunResponse { ... };`
and **before** any existing `--stream` branch, add the daemon routing:

```rust
    if via.as_deref() == Some("daemon") {
        if ensure_daemon && !<crate_name>::daemon::is_daemon_running(&runtime.state_dir) {
            <crate_name>::daemon::start_daemon(
                &runtime.state_dir, "<skill-name>",
                std::time::Duration::from_secs(30),
            ).map_err(AppExit::from)?;
        }
        return <crate_name>::daemon::execute_via_daemon(
            &["run".to_string()],
            &input,
            effective_context.effective_values.clone(),
            None,
            format,
            &runtime.state_dir,
            env!("CARGO_PKG_VERSION"),
        ).map_err(AppExit::from);
    }
```

### `lib.rs` Patch Snippet

```diff
+pub mod daemon;
+
 pub mod context;
 pub mod help;
```

### `help.rs` Patch Snippet

Update every `FeatureAvailability` block so daemon is marked as enabled:

```diff
 feature_availability: FeatureAvailability {
     streaming: "optional add-on".to_string(),
     repl: "optional add-on".to_string(),
+    daemon: "enabled".to_string(),
 }
```

Add daemon subcommands and routing flags to the global options rendered for
the top-level and leaf command paths so man-like human-readable help and
structured help both stay accurate after the feature is added:

- `daemon run`, `daemon start`, `daemon stop`, `daemon restart`, `daemon status`
- `--via local|daemon` (global, routing flag)
- `--ensure-daemon` (global, auto-start flag)

### `Cargo.toml` Patch Snippet

Append `uuid` and `libc` to the `[dependencies]` section. Insert after the
last existing dependency line (the exact anchor varies by which features are
already present):

```diff
 [dependencies]
 clap = { version = "4", features = ["derive"] }
 ...
+uuid = { version = "1", features = ["v4"] }
+libc = "0.2"
```

### `SKILL.md` Patch Snippet

Insert this section immediately after `## Output` (or after `## REPL Mode` if
present) and before `## Errors`:

```md
## Daemon Mode

- The daemon is a long-lived background app-server that accepts leaf-command
  execution requests over a Unix domain socket using JSON-RPC 2.0.
- Lifecycle commands: `daemon run` (foreground), `daemon start` (background),
  `daemon stop`, `daemon restart`, `daemon status`.
- Route leaf commands through the daemon with `--via daemon`.
- Auto-start the daemon with `--ensure-daemon` (only valid with `--via daemon`).
- Commands not daemonizable must reject `--via daemon` with a structured error.
- Daemon transport stays JSON-RPC internally; the client converts to the
  user's requested `--format` (YAML, JSON, or TOML) at the CLI boundary.
- Daemon runtime files live under `state/daemon/` (`daemon.pid`, `daemon.sock`,
  `daemon-state.json`, `daemon.log`).
```

## Combining Features

If the project already has one or more optional features and you are adding
another:

- Keep all flags in the same `Cli` struct.
- Keep `--repl` as the early session-mode branch before the normal command
  match, and keep `--stream` scoped to `execute_run`.
- Keep `--via daemon` routing inside `execute_run`, immediately before the
  `--stream` branch.
- Keep exactly one `let output = ...;` immediately before the default one-shot
  serialization path inside `execute_run`.
- Keep all module declarations in `src/lib.rs`, and leave them in
  rustfmt-compatible alphabetical order:

```rust
pub mod context;
pub mod daemon;
pub mod help;
pub mod repl;
pub mod stream;
```

- Keep `src/help.rs` synchronized with the enabled feature set by updating the
  global option list and `FeatureAvailability` values.

- Preserve the `SKILL.md` section order: `Description`, `Prerequisites`,
  `Invocation`, `Input`, `Output`, optional `REPL Mode`, optional
  `Daemon Mode`, `Errors`, `Examples`.

## Error Conditions

| Condition                                          | Action                                                |
| -------------------------------------------------- | ----------------------------------------------------- |
| `feature` is not `stream`, `repl`, or `daemon`     | Reject the request and ask for a supported feature.   |
| `project_path` does not exist                      | Stop and report the missing path.                     |
| `project_path` lacks any baseline receipt | Stop and route the project through Takeover first. |
| Takeover baseline exists but required scaffold-compatible files are missing | Stop and tell the user normalization or manual feature implementation is required. |
| Requested feature file already exists              | Stop and tell the user no changes were made.          |
| Template file is missing from this Skill package   | Stop and report that the Skill package is incomplete. |
| Build/lint/format fails after applying the feature | Fix the project before reporting success.             |
| Daemon `uuid` dependency missing from Cargo.toml   | Add `uuid = { version = "1", features = ["v4"] }` before proceeding. |

## Final Reporting Behavior

After a successful feature addition, tell the user:

- which feature was added
- which files were created or modified
- that build, test, clippy, and formatting checks pass
