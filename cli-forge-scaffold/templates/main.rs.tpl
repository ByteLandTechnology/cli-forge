//! Baseline CLI entrypoint for the generated package layout. Optional
//! capabilities may extend the package with package-local support files, but
//! repository-owned CI and release automation stay outside generated outputs.

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use {{SKILL_NAME_SNAKE}}::context::{
    InvocationContextOverrides, PersistedDaemonState, RuntimeOverrides, build_context_state,
    daemon_simulation_flags, inspect_context, load_daemon_state, parse_selectors,
    persist_active_context, persist_daemon_state, resolve_effective_context,
    resolve_runtime_locations,
};
use {{SKILL_NAME_SNAKE}}::help::{plain_text_help, structured_help};
use {{SKILL_NAME_SNAKE}}::{
    DaemonCommandOutput, DaemonLifecycleState, DaemonStatusOutput, Format, StructuredError, run,
    serialize_value, write_structured_error,
};

#[derive(Debug)]
enum AppExit {
    Usage,
    Failure(anyhow::Error),
}

impl From<anyhow::Error> for AppExit {
    fn from(error: anyhow::Error) -> Self {
        Self::Failure(error)
    }
}

/// {{DESCRIPTION}}
#[derive(Parser, Debug)]
#[command(
    name = "{{SKILL_NAME}}",
    version,
    about = "{{DESCRIPTION}}",
    disable_help_flag = true,
    disable_help_subcommand = true
)]
struct Cli {
    /// Output format
    #[arg(long, short, value_enum, global = true, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,

    /// Render plain-text help for the selected command path
    #[arg(long, short = 'h', global = true, action = ArgAction::SetTrue)]
    help: bool,

    /// Override the default configuration directory
    #[arg(long, global = true)]
    config_dir: Option<PathBuf>,

    /// Override the default durable data directory
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    /// Override the runtime state directory
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,

    /// Override the cache directory
    #[arg(long, global = true)]
    cache_dir: Option<PathBuf>,

    /// Override the optional log directory
    #[arg(long, global = true)]
    log_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Yaml,
    Json,
    Toml,
}

impl From<OutputFormat> for Format {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Yaml => Format::Yaml,
            OutputFormat::Json => Format::Json,
            OutputFormat::Toml => Format::Toml,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Return machine-readable help for a command path
    Help(HelpCommand),
    /// Execute the generated leaf command
    Run(RunCommand),
    /// Control the managed background daemon lifecycle
    Daemon(DaemonCommand),
    /// Inspect runtime directory defaults and overrides
    Paths(PathsCommand),
    /// Inspect or persist the Active Context
    Context(ContextCommand),
}

#[derive(Debug, Args)]
struct HelpCommand {
    /// Command path to inspect
    #[arg(value_name = "COMMAND_PATH")]
    path: Vec<String>,
}

#[derive(Debug, Args)]
struct RunCommand {
    /// Required input for the generated leaf command
    input: Option<String>,

    /// Explicit per-invocation context selector override
    #[arg(long = "selector", value_name = "KEY=VALUE")]
    selectors: Vec<String>,

    /// Explicit current-directory ambient cue
    #[arg(long = "cwd")]
    current_directory: Option<PathBuf>,

    /// Include the optional log directory in the resolved runtime paths
    #[arg(long)]
    log_enabled: bool,
}

#[derive(Debug, Args)]
struct DaemonCommand {
    #[command(subcommand)]
    command: Option<DaemonSubcommand>,
}

#[derive(Debug, Subcommand)]
enum DaemonSubcommand {
    /// Start the managed background daemon
    Start,
    /// Stop the managed background daemon
    Stop,
    /// Restart the managed background daemon
    Restart,
    /// Inspect the current daemon lifecycle state
    Status,
}

#[derive(Debug, Args)]
struct PathsCommand {
    /// Include the optional log directory in the resolved runtime paths
    #[arg(long)]
    log_enabled: bool,
}

#[derive(Debug, Args)]
struct ContextCommand {
    #[command(subcommand)]
    command: Option<ContextSubcommand>,
}

#[derive(Debug, Subcommand)]
enum ContextSubcommand {
    /// Display the current persisted and effective context
    Show,
    /// Persist selectors and ambient cues as the Active Context
    Use(ContextUseCommand),
}

#[derive(Debug, Args)]
struct ContextUseCommand {
    /// Optional label for the persisted context
    #[arg(long)]
    name: Option<String>,

    /// Selector to persist in the Active Context
    #[arg(long = "selector", value_name = "KEY=VALUE")]
    selectors: Vec<String>,

    /// Ambient current-directory cue to persist
    #[arg(long = "cwd")]
    current_directory: Option<PathBuf>,
}

#[derive(Debug, serde::Serialize)]
struct RunResponse {
    status: String,
    message: String,
    input: String,
    effective_context: std::collections::BTreeMap<String, String>,
}

fn main() {
    let exit_code = match run_cli() {
        Ok(()) => 0,
        Err(AppExit::Usage) => 2,
        Err(AppExit::Failure(error)) => {
            eprintln!("error: {error:#}");
            1
        }
    };

    std::process::exit(exit_code);
}

fn run_cli() -> std::result::Result<(), AppExit> {
    let raw_args: Vec<String> = std::env::args().collect();
    let detected_format = detect_requested_format(&raw_args);

    let cli = match Cli::try_parse_from(&raw_args) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error, detected_format),
    };

    let format: Format = cli.format.into();

    if cli.help {
        return render_plain_text_help_for_cli(&cli);
    }

    let runtime_overrides = cli_runtime_overrides(&cli);

    match cli.command {
        None => render_plain_text_help_for_path(&[]),
        Some(Command::Help(command)) => render_structured_help(&command.path, format),
        Some(Command::Run(command)) => execute_run(runtime_overrides, command, format),
        Some(Command::Daemon(command)) => execute_daemon(runtime_overrides, command, format),
        Some(Command::Paths(command)) => execute_paths(runtime_overrides, command, format),
        Some(Command::Context(command)) => execute_context(runtime_overrides, command, format),
    }
}

fn handle_parse_error(error: clap::Error, format: Format) -> std::result::Result<(), AppExit> {
    if error.kind() == clap::error::ErrorKind::DisplayVersion {
        error.print().map_err(|err| AppExit::Failure(err.into()))?;
        return Ok(());
    }

    let structured_error =
        StructuredError::new("usage.parse_error", error.to_string(), "help_usage", format);
    let mut stderr = std::io::stderr().lock();
    write_structured_error(&mut stderr, &structured_error, format).map_err(AppExit::from)?;
    Err(AppExit::Usage)
}

fn cli_runtime_overrides(cli: &Cli) -> RuntimeOverrides {
    RuntimeOverrides {
        config_dir: cli.config_dir.clone(),
        data_dir: cli.data_dir.clone(),
        state_dir: cli.state_dir.clone(),
        cache_dir: cli.cache_dir.clone(),
        log_dir: cli.log_dir.clone(),
    }
}

fn detect_requested_format(args: &[String]) -> Format {
    let mut args = args.iter().peekable();
    while let Some(arg) = args.next() {
        if let Some(value) = arg.strip_prefix("--format=") {
            return parse_format_token(value).unwrap_or(Format::Yaml);
        }
        if (arg == "--format" || arg == "-f")
            && let Some(value) = args.peek()
        {
            return parse_format_token(value).unwrap_or(Format::Yaml);
        }
    }
    Format::Yaml
}

fn parse_format_token(token: &str) -> Option<Format> {
    match token {
        "yaml" => Some(Format::Yaml),
        "json" => Some(Format::Json),
        "toml" => Some(Format::Toml),
        _ => None,
    }
}

fn render_plain_text_help_for_cli(cli: &Cli) -> std::result::Result<(), AppExit> {
    let path = match &cli.command {
        None => Vec::new(),
        Some(Command::Help(_)) => vec!["help".to_string()],
        Some(Command::Run(_)) => vec!["run".to_string()],
        Some(Command::Daemon(DaemonCommand { command: None })) => vec!["daemon".to_string()],
        Some(Command::Daemon(DaemonCommand {
            command: Some(DaemonSubcommand::Start),
        })) => vec!["daemon".to_string(), "start".to_string()],
        Some(Command::Daemon(DaemonCommand {
            command: Some(DaemonSubcommand::Stop),
        })) => vec!["daemon".to_string(), "stop".to_string()],
        Some(Command::Daemon(DaemonCommand {
            command: Some(DaemonSubcommand::Restart),
        })) => vec!["daemon".to_string(), "restart".to_string()],
        Some(Command::Daemon(DaemonCommand {
            command: Some(DaemonSubcommand::Status),
        })) => vec!["daemon".to_string(), "status".to_string()],
        Some(Command::Paths(_)) => vec!["paths".to_string()],
        Some(Command::Context(ContextCommand { command: None })) => vec!["context".to_string()],
        Some(Command::Context(ContextCommand {
            command: Some(ContextSubcommand::Show),
        })) => vec!["context".to_string(), "show".to_string()],
        Some(Command::Context(ContextCommand {
            command: Some(ContextSubcommand::Use(_)),
        })) => vec!["context".to_string(), "use".to_string()],
    };

    render_plain_text_help_for_path(&path)
}

fn render_plain_text_help_for_path(path: &[String]) -> std::result::Result<(), AppExit> {
    let Some(help_text) = plain_text_help(path) else {
        let mut stderr = std::io::stderr().lock();
        let error = StructuredError::new(
            "help.unknown_path",
            format!("unknown help path '{}'", path.join(" ")),
            "help_usage",
            Format::Yaml,
        );
        write_structured_error(&mut stderr, &error, Format::Yaml).map_err(AppExit::from)?;
        return Err(AppExit::Usage);
    };

    println!("{help_text}");
    Ok(())
}

fn render_structured_help(path: &[String], format: Format) -> std::result::Result<(), AppExit> {
    let Some(help_document) = structured_help(path) else {
        let mut stderr = std::io::stderr().lock();
        let error = StructuredError::new(
            "help.unknown_path",
            format!("unknown help path '{}'", path.join(" ")),
            "help_usage",
            format,
        );
        write_structured_error(&mut stderr, &error, format).map_err(AppExit::from)?;
        return Err(AppExit::Usage);
    };

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serialize_value(&mut stdout, &help_document, format).map_err(AppExit::from)?;
    Ok(())
}

fn execute_paths(
    overrides: RuntimeOverrides,
    command: PathsCommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    let runtime =
        resolve_runtime_locations(&overrides, command.log_enabled).map_err(AppExit::from)?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serialize_value(&mut stdout, &runtime.summary(), format).map_err(AppExit::from)?;
    Ok(())
}

fn execute_context(
    overrides: RuntimeOverrides,
    command: ContextCommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    match command.command {
        None => render_plain_text_help_for_path(&["context".to_string()]),
        Some(ContextSubcommand::Show) => {
            let runtime = resolve_runtime_locations(&overrides, false).map_err(AppExit::from)?;
            let inspection = inspect_context(&runtime, &InvocationContextOverrides::default())
                .map_err(AppExit::from)?;
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            serialize_value(&mut stdout, &inspection, format).map_err(AppExit::from)?;
            Ok(())
        }
        Some(ContextSubcommand::Use(command)) => {
            let selectors = parse_selectors(&command.selectors).map_err(AppExit::from)?;
            let current_directory = command.current_directory;
            if selectors.is_empty() && current_directory.is_none() && command.name.is_none() {
                let error = StructuredError::new(
                    "context.missing_values",
                    "provide at least one --selector, --cwd, or --name when persisting an Active Context",
                    "runtime_state",
                    format,
                );
                let mut stderr = std::io::stderr().lock();
                write_structured_error(&mut stderr, &error, format).map_err(AppExit::from)?;
                return Err(AppExit::Usage);
            }

            let runtime = resolve_runtime_locations(&overrides, false).map_err(AppExit::from)?;
            let state = build_context_state(command.name, selectors, current_directory);
            let persisted = persist_active_context(&runtime, &state).map_err(AppExit::from)?;
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            serialize_value(&mut stdout, &persisted, format).map_err(AppExit::from)?;
            Ok(())
        }
    }
}

fn execute_daemon(
    overrides: RuntimeOverrides,
    command: DaemonCommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    match command.command {
        None => render_plain_text_help_for_path(&["daemon".to_string()]),
        Some(DaemonSubcommand::Status) => {
            let runtime = resolve_runtime_locations(&overrides, false).map_err(AppExit::from)?;
            let mut state = load_daemon_state(&runtime).map_err(AppExit::from)?;
            if daemon_simulation_flags().unexpected_exit
                && matches!(state.state, DaemonLifecycleState::Running)
            {
                state.state = DaemonLifecycleState::Failed;
                state.readiness = "not_ready".to_string();
                state.reason = Some("the managed daemon exited unexpectedly".to_string());
                state.recommended_next_action = "restart".to_string();
                state.last_action = "status".to_string();
                persist_daemon_state(&runtime, &state).map_err(AppExit::from)?;
            }

            let status = DaemonStatusOutput {
                state: state.state,
                readiness: state.readiness,
                reason: state.reason,
                recommended_next_action: state.recommended_next_action,
                instance_model: state.instance_model,
                instance_id: state.instance_id,
            };

            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            serialize_value(&mut stdout, &status, format).map_err(AppExit::from)?;
            Ok(())
        }
        Some(subcommand) => execute_daemon_lifecycle(overrides, subcommand, format),
    }
}

fn execute_daemon_lifecycle(
    overrides: RuntimeOverrides,
    subcommand: DaemonSubcommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    let runtime = resolve_runtime_locations(&overrides, false).map_err(AppExit::from)?;
    let flags = daemon_simulation_flags();
    let mut state = load_daemon_state(&runtime).map_err(AppExit::from)?;
    let action = match subcommand {
        DaemonSubcommand::Start => "start",
        DaemonSubcommand::Stop => "stop",
        DaemonSubcommand::Restart => "restart",
        DaemonSubcommand::Status => unreachable!("status handled separately"),
    };

    if flags.block_control {
        return render_daemon_command_output(
            DaemonCommandOutput {
                action: action.to_string(),
                result: "blocked".to_string(),
                state: state.state,
                message: "another daemon control action is already in progress".to_string(),
                recommended_next_action: "status".to_string(),
                instance_model: state.instance_model,
                instance_id: state.instance_id,
            },
            format,
        );
    }

    let output = match subcommand {
        DaemonSubcommand::Start => {
            daemon_start_output(&runtime, &mut state, flags.fail_start, flags.timeout_start)
        }
        DaemonSubcommand::Stop => {
            daemon_stop_output(&runtime, &mut state, flags.fail_stop, flags.timeout_stop)
        }
        DaemonSubcommand::Restart => daemon_restart_output(
            &runtime,
            &mut state,
            flags.fail_restart,
            flags.timeout_restart,
        ),
        DaemonSubcommand::Status => unreachable!("status handled separately"),
    }
    .map_err(AppExit::from)?;

    render_daemon_command_output(output, format)
}

fn daemon_start_output(
    runtime: &{{SKILL_NAME_SNAKE}}::context::RuntimeLocations,
    state: &mut PersistedDaemonState,
    fail: bool,
    timeout: bool,
) -> anyhow::Result<DaemonCommandOutput> {
    if matches!(state.state, DaemonLifecycleState::Running) {
        return Ok(daemon_command_output(
            "start",
            "no_op",
            state.clone(),
            "the managed daemon is already running",
            "status",
        ));
    }

    if timeout {
        state.state = DaemonLifecycleState::Starting;
        state.readiness = "pending".to_string();
        state.reason = Some(
            "the daemon did not report a terminal outcome before the timeout expired".to_string(),
        );
        state.recommended_next_action = "status".to_string();
        state.last_action = "start".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "start",
            "timed_out",
            state.clone(),
            "the managed daemon is still transitioning; inspect status for the current observable state",
            "status",
        ));
    }

    if fail {
        state.state = DaemonLifecycleState::Failed;
        state.readiness = "not_ready".to_string();
        state.reason = Some("the managed daemon failed to start".to_string());
        state.recommended_next_action = "restart".to_string();
        state.last_action = "start".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "start",
            "failed",
            state.clone(),
            "the managed daemon failed to start",
            "restart",
        ));
    }

    state.state = DaemonLifecycleState::Running;
    state.readiness = "ready".to_string();
    state.reason = None;
    state.recommended_next_action = "status".to_string();
    state.last_action = "start".to_string();
    persist_daemon_state(runtime, state)?;
    Ok(daemon_command_output(
        "start",
        "running",
        state.clone(),
        "the managed daemon is now running",
        "status",
    ))
}

fn daemon_stop_output(
    runtime: &{{SKILL_NAME_SNAKE}}::context::RuntimeLocations,
    state: &mut PersistedDaemonState,
    fail: bool,
    timeout: bool,
) -> anyhow::Result<DaemonCommandOutput> {
    if matches!(state.state, DaemonLifecycleState::Stopped) {
        return Ok(daemon_command_output(
            "stop",
            "no_op",
            state.clone(),
            "the managed daemon is already stopped",
            "start",
        ));
    }

    if timeout {
        state.state = DaemonLifecycleState::Stopping;
        state.readiness = "pending".to_string();
        state.reason = Some("the daemon did not stop before the timeout expired".to_string());
        state.recommended_next_action = "status".to_string();
        state.last_action = "stop".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "stop",
            "timed_out",
            state.clone(),
            "the managed daemon is still stopping; inspect status for the current observable state",
            "status",
        ));
    }

    if fail {
        state.state = DaemonLifecycleState::Failed;
        state.readiness = "not_ready".to_string();
        state.reason = Some("the managed daemon failed to stop cleanly".to_string());
        state.recommended_next_action = "status".to_string();
        state.last_action = "stop".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "stop",
            "failed",
            state.clone(),
            "the managed daemon failed to stop cleanly",
            "status",
        ));
    }

    state.state = DaemonLifecycleState::Stopped;
    state.readiness = "inactive".to_string();
    state.reason = None;
    state.recommended_next_action = "start".to_string();
    state.last_action = "stop".to_string();
    persist_daemon_state(runtime, state)?;
    Ok(daemon_command_output(
        "stop",
        "stopped",
        state.clone(),
        "the managed daemon is now stopped",
        "start",
    ))
}

fn daemon_restart_output(
    runtime: &{{SKILL_NAME_SNAKE}}::context::RuntimeLocations,
    state: &mut PersistedDaemonState,
    fail: bool,
    timeout: bool,
) -> anyhow::Result<DaemonCommandOutput> {
    if matches!(state.state, DaemonLifecycleState::Stopped) {
        return Ok(daemon_command_output(
            "restart",
            "blocked",
            state.clone(),
            "restart is unavailable while the managed daemon is stopped; use start instead",
            "start",
        ));
    }

    if timeout {
        state.state = DaemonLifecycleState::Starting;
        state.readiness = "pending".to_string();
        state.reason = Some(
            "the daemon restart did not reach a terminal outcome before the timeout expired"
                .to_string(),
        );
        state.recommended_next_action = "status".to_string();
        state.last_action = "restart".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "restart",
            "timed_out",
            state.clone(),
            "the managed daemon restart is still in progress; inspect status for the current observable state",
            "status",
        ));
    }

    if fail {
        state.state = DaemonLifecycleState::Failed;
        state.readiness = "not_ready".to_string();
        state.reason = Some("the managed daemon failed to restart".to_string());
        state.recommended_next_action = "restart".to_string();
        state.last_action = "restart".to_string();
        persist_daemon_state(runtime, state)?;
        return Ok(daemon_command_output(
            "restart",
            "failed",
            state.clone(),
            "the managed daemon failed to restart",
            "restart",
        ));
    }

    state.state = DaemonLifecycleState::Running;
    state.readiness = "ready".to_string();
    state.reason = None;
    state.recommended_next_action = "status".to_string();
    state.last_action = "restart".to_string();
    persist_daemon_state(runtime, state)?;
    Ok(daemon_command_output(
        "restart",
        "running",
        state.clone(),
        "the managed daemon completed a controlled restart",
        "status",
    ))
}

fn daemon_command_output(
    action: &str,
    result: &str,
    state: PersistedDaemonState,
    message: &str,
    recommended_next_action: &str,
) -> DaemonCommandOutput {
    DaemonCommandOutput {
        action: action.to_string(),
        result: result.to_string(),
        state: state.state,
        message: message.to_string(),
        recommended_next_action: recommended_next_action.to_string(),
        instance_model: state.instance_model,
        instance_id: state.instance_id,
    }
}

fn render_daemon_command_output(
    output: DaemonCommandOutput,
    format: Format,
) -> std::result::Result<(), AppExit> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serialize_value(&mut stdout, &output, format).map_err(AppExit::from)?;
    Ok(())
}

fn execute_run(
    overrides: RuntimeOverrides,
    command: RunCommand,
    format: Format,
) -> std::result::Result<(), AppExit> {
    let runtime =
        resolve_runtime_locations(&overrides, command.log_enabled).map_err(AppExit::from)?;
    let selectors = parse_selectors(&command.selectors).map_err(AppExit::from)?;
    let invocation_overrides = InvocationContextOverrides {
        selectors,
        current_directory: command.current_directory,
    };
    let persisted_context =
        {{SKILL_NAME_SNAKE}}::context::load_active_context(&runtime).map_err(AppExit::from)?;
    let effective_context =
        resolve_effective_context(persisted_context.as_ref(), &invocation_overrides);

    let Some(input) = command.input else {
        let error = StructuredError::new(
            "run.missing_input",
            "the run command requires <INPUT>; use --help for plain-text help",
            "leaf_validation",
            format,
        )
        .with_detail("command", "run");
        let mut stderr = std::io::stderr().lock();
        write_structured_error(&mut stderr, &error, format).map_err(AppExit::from)?;
        return Err(AppExit::Usage);
    };

    let response = run(&input, effective_context.effective_values.clone());
    let output = RunResponse {
        status: response.status,
        message: response.message,
        input: response.input,
        effective_context: response.effective_context,
    };

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    serialize_value(&mut stdout, &output, format).map_err(AppExit::from)?;
    Ok(())
}
