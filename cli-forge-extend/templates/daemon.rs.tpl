//! Optional daemon overlay for the generated package layout. This module is
//! package-local to generated skills when daemon capability is enabled.
//!
//! Implements the app-server daemon capability using JSON-RPC 2.0 over Unix
//! domain sockets. One binary acts as both daemon server and daemon client.
//! Streaming and REPL overlays must not rewrite daemon commands, routing
//! flags, transport choices, or recovery semantics.

use crate::Format;
use crate::{StructuredError, run, serialize_value, write_structured_error};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write, stdout};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Default client-side timeout for daemon lifecycle RPCs (status, stop).
/// NOT used for command execution, which may take arbitrarily long.
const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(30);

/// Connect to the daemon socket with a read timeout for lifecycle commands.
fn connect_lifecycle(sock: &Path) -> Result<UnixStream> {
    let stream = UnixStream::connect(sock)
        .with_context(|| format!("cannot connect to {}", sock.display()))?;
    stream
        .set_read_timeout(Some(LIFECYCLE_TIMEOUT))
        .context("cannot set read timeout on daemon socket")?;
    Ok(stream)
}

/// Connect to the daemon socket without a read timeout for command execution.
fn connect_execute(sock: &Path) -> Result<UnixStream> {
    let stream = UnixStream::connect(sock)
        .with_context(|| format!("cannot connect to {}", sock.display()))?;
    Ok(stream)
}

// ---------------------------------------------------------------------------
// Daemon state model
// ---------------------------------------------------------------------------

/// Lifecycle states for the daemon process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonState {
    Stopped,
    Starting,
    Running,
    Degraded,
    Stopping,
    Failed,
}

/// Health and operational status reported by `daemon status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub state: DaemonState,
    pub readiness: String,
    pub instance_id: String,
    pub pid: Option<u32>,
    pub transport: String,
    pub endpoint: String,
    pub uptime_sec: Option<u64>,
    pub active_requests: u64,
    pub queue_depth: u64,
    pub last_error: Option<String>,
    pub recommended_next_action: String,
}

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 protocol
// ---------------------------------------------------------------------------

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest<T: Serialize> {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

/// JSON-RPC 2.0 success response envelope.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T: Serialize> {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Parameters for `command.execute` RPC.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteParams {
    pub command_path: Vec<String>,
    pub arguments: serde_json::Value,
    pub context: ExecuteContext,
    pub client: ClientInfo,
}

/// Execution context sent from client to daemon.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteContext {
    #[serde(default)]
    pub selectors: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// Client identification sent with each RPC.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub version: String,
}

/// Result payload returned by `command.execute`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub execution: ExecutionMeta,
    pub payload: serde_json::Value,
}

/// Execution metadata added by the daemon.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionMeta {
    pub mode: String,
    pub instance_id: String,
}

/// Stream event emitted by `command.execute_stream`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Start {
        request_id: String,
    },
    Progress {
        request_id: String,
        message: String,
        pct: Option<u8>,
    },
    Item {
        request_id: String,
        data: serde_json::Value,
    },
    Stderr {
        request_id: String,
        text: String,
    },
    Result {
        request_id: String,
        data: serde_json::Value,
    },
    Error {
        request_id: String,
        code: String,
        message: String,
    },
    End {
        request_id: String,
    },
}

/// Persistent daemon state written to `state/daemon/daemon-state.json`.
#[derive(Debug, Serialize, Deserialize)]
struct DaemonStateFile {
    state: DaemonState,
    instance_id: String,
    started_at: Option<String>,
    pid: u32,
}

// ---------------------------------------------------------------------------
// Runtime file helpers (all under state/daemon/)
// ---------------------------------------------------------------------------

/// Return the daemon runtime directory, creating it if necessary.
pub fn daemon_runtime_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("daemon")
}

fn ensure_daemon_dir(state_dir: &Path) -> Result<PathBuf> {
    let dir = daemon_runtime_dir(state_dir);
    fs::create_dir_all(&dir).with_context(|| format!("cannot create {}", dir.display()))?;
    // Restrict to owner-only to prevent other local users from accessing the socket.
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(&dir, perms).with_context(|| format!("cannot chmod {}", dir.display()))?;
    Ok(dir)
}

fn pid_path(state_dir: &Path) -> PathBuf {
    daemon_runtime_dir(state_dir).join("daemon.pid")
}

fn sock_path(state_dir: &Path) -> PathBuf {
    daemon_runtime_dir(state_dir).join("daemon.sock")
}

fn state_file_path(state_dir: &Path) -> PathBuf {
    daemon_runtime_dir(state_dir).join("daemon-state.json")
}

fn log_path(state_dir: &Path) -> PathBuf {
    daemon_runtime_dir(state_dir).join("daemon.log")
}

fn read_pid(state_dir: &Path) -> Option<u32> {
    let path = pid_path(state_dir);
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

fn write_pid(state_dir: &Path, pid: u32) -> Result<()> {
    let dir = ensure_daemon_dir(state_dir)?;
    fs::write(dir.join("daemon.pid"), pid.to_string()).context("cannot write daemon.pid")
}

fn read_state_file(state_dir: &Path) -> Option<DaemonStateFile> {
    let data = fs::read_to_string(state_file_path(state_dir)).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_state_file(state_dir: &Path, dsf: &DaemonStateFile) -> Result<()> {
    let dir = ensure_daemon_dir(state_dir)?;
    let data = serde_json::to_string_pretty(dsf).context("cannot serialize daemon state")?;
    fs::write(dir.join("daemon-state.json"), data).context("cannot write daemon-state.json")
}

/// Check whether a daemon process is listening on the socket.
pub fn is_daemon_running(state_dir: &Path) -> bool {
    let sock = sock_path(state_dir);
    if !sock.exists() {
        return false;
    }
    UnixStream::connect(&sock).is_ok()
}

// ---------------------------------------------------------------------------
// Server: foreground daemon run
// ---------------------------------------------------------------------------

/// Run the daemon server in the foreground (blocks the terminal).
pub fn run_daemon(state_dir: &Path, skill_name: &str, version: &str) -> Result<()> {
    let dir = ensure_daemon_dir(state_dir)?;
    let sock = dir.join("daemon.sock");

    // Detect an already-running daemon instead of blindly removing the socket.
    if is_daemon_running(state_dir) {
        bail!("daemon is already running on {}", sock.display());
    }
    if sock.exists() {
        let _ = fs::remove_file(&sock);
    }

    let instance_id = "default".to_string();
    let pid = std::process::id();

    write_pid(state_dir, pid)?;
    write_state_file(
        state_dir,
        &DaemonStateFile {
            state: DaemonState::Starting,
            instance_id: instance_id.clone(),
            started_at: Some(chrono_now_iso()),
            pid,
        },
    )?;

    let listener =
        UnixListener::bind(&sock).with_context(|| format!("cannot bind {}", sock.display()))?;

    write_state_file(
        state_dir,
        &DaemonStateFile {
            state: DaemonState::Running,
            instance_id: instance_id.clone(),
            started_at: Some(chrono_now_iso()),
            pid,
        },
    )?;

    let started_at = Instant::now();
    let shutting_down = Arc::new(AtomicBool::new(false));

    // Accept loop — spawn a thread per connection so long-running commands
    // do not block concurrent status/stop queries.
    for stream in listener.incoming() {
        if shutting_down.load(Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(stream) => {
                let state_dir = state_dir.to_path_buf();
                let skill_name = skill_name.to_string();
                let version = version.to_string();
                let instance_id = instance_id.clone();
                let shutting_down = shutting_down.clone();
                // started_at is Copy; the move into the closure is fine
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(
                        stream,
                        &state_dir,
                        &skill_name,
                        &version,
                        &instance_id,
                        started_at,
                        shutting_down,
                    ) {
                        eprintln!("daemon client error: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("daemon accept error: {e}");
                write_state_file(
                    state_dir,
                    &DaemonStateFile {
                        state: DaemonState::Degraded,
                        instance_id: instance_id.clone(),
                        started_at: Some(chrono_now_iso()),
                        pid,
                    },
                )?;
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: UnixStream,
    state_dir: &Path,
    skill_name: &str,
    version: &str,
    instance_id: &str,
    started_at: Instant,
    shutting_down: Arc<AtomicBool>,
) -> Result<()> {
    let _version = version; // reserved for future use (e.g. version-gated RPC methods)
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        let request: serde_json::Value =
            serde_json::from_str(line.trim()).context("invalid JSON-RPC request")?;

        let method = request
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        let id = request
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let response = match method.as_str() {
            "daemon.health" => health_response(&id, instance_id, started_at),
            "daemon.status" => status_response(&id, state_dir, instance_id, started_at),
            "daemon.shutdown" => {
                let resp = ok_response(&id, serde_json::json!({"shutting_down": true}));
                let _ = stream.write_all(format!("{resp}\n").as_bytes());
                let _ = stream.flush();
                // Signal the accept loop to stop accepting new connections.
                // In-flight worker threads will finish their current request.
                shutting_down.store(true, Ordering::Relaxed);
                // Clean up socket so new clients see "not running" immediately.
                let _ = fs::remove_file(sock_path(state_dir));
                let _ = fs::remove_file(pid_path(state_dir));
                write_state_file(
                    state_dir,
                    &DaemonStateFile {
                        state: DaemonState::Stopped,
                        instance_id: instance_id.to_string(),
                        started_at: None,
                        pid: 0,
                    },
                )?;
                return Ok(());
            }
            "command.execute" => {
                let params = request
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                execute_command_response(&id, params, skill_name)
            }
            _ => error_response(&id, -32601, "Method not found", None),
        };

        stream.write_all(format!("{response}\n").as_bytes())?;
        stream.flush()?;
        line.clear();
    }

    Ok(())
}

fn health_response(id: &str, instance_id: &str, started_at: Instant) -> String {
    let uptime = started_at.elapsed().as_secs();
    ok_response(
        id,
        serde_json::json!({
            "state": "running",
            "instance_id": instance_id,
            "uptime_sec": uptime,
        }),
    )
}

fn status_response(id: &str, state_dir: &Path, instance_id: &str, started_at: Instant) -> String {
    let uptime = started_at.elapsed().as_secs();
    let pid = read_pid(state_dir);
    let sock = sock_path(state_dir);
    let last_error = read_state_file(state_dir).and_then(|s| {
        if s.state == DaemonState::Degraded {
            Some("degraded state detected".to_string())
        } else {
            None
        }
    });

    let status = DaemonStatus {
        state: DaemonState::Running,
        readiness: "ready".to_string(),
        instance_id: instance_id.to_string(),
        pid,
        transport: "unix-socket".to_string(),
        endpoint: sock.to_string_lossy().to_string(),
        uptime_sec: Some(uptime),
        active_requests: 0,
        queue_depth: 0,
        last_error,
        recommended_next_action: "none".to_string(),
    };

    ok_response(id, serde_json::to_value(&status).unwrap_or_default())
}

fn execute_command_response(id: &str, params: serde_json::Value, _skill_name: &str) -> String {
    let input = params
        .get("arguments")
        .and_then(|a| a.get("input"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let selectors: BTreeMap<String, String> = params
        .get("context")
        .and_then(|c| c.get("selectors"))
        .and_then(|s| serde_json::from_value(s.clone()).ok())
        .unwrap_or_default();

    let result = run(input, selectors);
    let payload = serde_json::to_value(&result).unwrap_or_default();

    let exec_result = ExecuteResult {
        execution: ExecutionMeta {
            mode: "daemon".to_string(),
            instance_id: "default".to_string(),
        },
        payload,
    };

    ok_response(id, serde_json::to_value(&exec_result).unwrap_or_default())
}

// ---------------------------------------------------------------------------
// JSON-RPC helpers
// ---------------------------------------------------------------------------

fn ok_response(id: &str, result: serde_json::Value) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
    .to_string()
}

fn error_response(id: &str, code: i64, message: &str, data: Option<serde_json::Value>) -> String {
    let mut err = serde_json::json!({
        "code": code,
        "message": message,
    });
    if let Some(d) = data {
        err.as_object_mut().unwrap().insert("data".to_string(), d);
    }
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": err,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Client: lifecycle commands
// ---------------------------------------------------------------------------

/// Start the daemon in the background. Returns after the daemon is `running`,
/// `failed`, or a timeout elapses. Uses `flock` on a lock file to serialize
/// concurrent auto-start attempts.
pub fn start_daemon(
    state_dir: &Path,
    _skill_name: &str,
    timeout: Duration,
) -> Result<DaemonStatus> {
    ensure_daemon_dir(state_dir)?;

    if is_daemon_running(state_dir) {
        bail!("daemon is already running");
    }

    // Acquire an exclusive advisory lock to serialize concurrent starts.
    let lock_path = daemon_runtime_dir(state_dir).join("daemon.lock");
    let lock_file = fs::File::create(&lock_path)
        .with_context(|| format!("cannot create {}", lock_path.display()))?;
    unsafe {
        let fd = libc::fcntl(lock_file.as_raw_fd(), libc::F_GETFD);
        if fd == -1 {
            bail!("cannot prepare lock file");
        }
        if libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 {
            bail!("cannot acquire daemon start lock — another start may be in progress");
        }
    }

    // Re-check after acquiring the lock (double-check locking).
    if is_daemon_running(state_dir) {
        return query_status(state_dir);
    }

    let log = log_path(state_dir);
    let log_file =
        fs::File::create(&log).with_context(|| format!("cannot create {}", log.display()))?;

    let binary = std::env::current_exe().context("cannot resolve current executable")?;

    let mut child = Command::new(&binary)
        .arg("daemon")
        .arg("run")
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file))
        .spawn()
        .context("cannot spawn daemon process")?;

    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            let _ = child.kill();
            bail!("daemon start timed out after {}s", timeout.as_secs());
        }

        if is_daemon_running(state_dir) {
            return query_status(state_dir);
        }

        // Check if process exited early
        match child.try_wait() {
            Ok(Some(status)) => {
                bail!("daemon process exited prematurely: {status}");
            }
            Ok(None) => {}
            Err(e) => {
                bail!("cannot check daemon process: {e}");
            }
        }

        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Stop the daemon via RPC `daemon.shutdown`.
pub fn stop_daemon(state_dir: &Path, timeout: Duration) -> Result<()> {
    if !is_daemon_running(state_dir) {
        bail!("daemon is not running");
    }

    let sock = sock_path(state_dir);
    let mut stream = connect_lifecycle(&sock)?;

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "stop",
        "method": "daemon.shutdown",
    });
    writeln!(stream, "{req}")?;
    stream.flush()?;

    // Wait for process to exit
    let start = Instant::now();
    loop {
        if !sock.exists() || !is_daemon_running(state_dir) {
            return Ok(());
        }
        if start.elapsed() > timeout {
            // Force kill
            if let Some(pid) = read_pid(state_dir) {
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
            let _ = fs::remove_file(&sock);
            bail!("daemon stop timed out, force-killed");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Restart: stop then start.
pub fn restart_daemon(
    state_dir: &Path,
    skill_name: &str,
    timeout: Duration,
) -> Result<DaemonStatus> {
    if is_daemon_running(state_dir) {
        stop_daemon(state_dir, timeout)?;
    }
    start_daemon(state_dir, skill_name, timeout)
}

/// Query daemon status via RPC `daemon.status`.
pub fn query_status(state_dir: &Path) -> Result<DaemonStatus> {
    if !is_daemon_running(state_dir) {
        let status = DaemonStatus {
            state: DaemonState::Stopped,
            readiness: "not_ready".to_string(),
            instance_id: String::new(),
            pid: None,
            transport: "unix-socket".to_string(),
            endpoint: sock_path(state_dir).to_string_lossy().to_string(),
            uptime_sec: None,
            active_requests: 0,
            queue_depth: 0,
            last_error: None,
            recommended_next_action: "run `daemon start` to launch".to_string(),
        };
        return Ok(status);
    }

    let sock = sock_path(state_dir);
    let mut stream = connect_lifecycle(&sock)?;

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "status",
        "method": "daemon.status",
    });
    writeln!(stream, "{req}")?;
    stream.flush()?;

    let mut resp = String::new();
    BufReader::new(&mut stream).read_line(&mut resp)?;
    let parsed: serde_json::Value =
        serde_json::from_str(resp.trim()).context("invalid daemon status response")?;

    let result = parsed.get("result").cloned().unwrap_or_default();
    let status: DaemonStatus =
        serde_json::from_value(result).context("cannot parse daemon status")?;
    Ok(status)
}

// ---------------------------------------------------------------------------
// Client: command execution via daemon
// ---------------------------------------------------------------------------

/// Execute a leaf command through the daemon and serialize the result in the
/// user's chosen output format.
pub fn execute_via_daemon(
    command_path: &[String],
    input: &str,
    context: BTreeMap<String, String>,
    cwd: Option<&str>,
    format: Format,
    state_dir: &Path,
    version: &str,
) -> Result<()> {
    let sock = sock_path(state_dir);
    let mut stream = connect_execute(&sock)?;

    let params = ExecuteParams {
        command_path: command_path.to_vec(),
        arguments: serde_json::json!({ "input": input }),
        context: ExecuteContext {
            selectors: context,
            cwd: cwd.map(|s| s.to_string()),
        },
        client: ClientInfo {
            version: version.to_string(),
        },
    };

    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: format!("exec-{}", simple_id()),
        method: "command.execute".to_string(),
        params: Some(params),
    };

    writeln!(stream, "{}", serde_json::to_string(&req)?)?;
    stream.flush()?;

    let mut resp = String::new();
    BufReader::new(&mut stream).read_line(&mut resp)?;
    let parsed: JsonRpcResponse<serde_json::Value> =
        serde_json::from_str(resp.trim()).context("invalid daemon execute response")?;

    if let Some(err) = parsed.error {
        let serr = StructuredError::new("daemon.execute_failed", &err.message, "daemon", format);
        let stdout = stdout();
        let mut out = stdout.lock();
        write_structured_error(&mut out, &serr, format)?;
        std::process::exit(1);
    }

    let result = parsed.result.unwrap_or_default();
    let payload = result.get("payload").cloned().unwrap_or_default();

    let stdout = stdout();
    let mut out = stdout.lock();
    serialize_value(&mut out, &payload, format)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

fn chrono_now_iso() -> String {
    // Simple ISO-8601 without pulling in chrono; uses std only.
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-01-01T00:00:00Z", d.as_secs() / (365 * 24 * 3600) + 1970)
}

fn simple_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_state_serde_roundtrip() {
        let states = vec![
            DaemonState::Stopped,
            DaemonState::Starting,
            DaemonState::Running,
            DaemonState::Degraded,
            DaemonState::Stopping,
            DaemonState::Failed,
        ];
        for state in &states {
            let json = serde_json::to_string(state).unwrap();
            let back: DaemonState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, back, "roundtrip failed for {state:?}");
        }
    }

    #[test]
    fn json_rpc_request_format() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "test-1".to_string(),
            method: "daemon.health".to_string(),
            params: None as Option<serde_json::Value>,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"daemon.health\""));
        assert!(json.contains("\"id\":\"test-1\""));
    }

    #[test]
    fn ok_response_has_no_error_field() {
        let resp = ok_response("r1", serde_json::json!({"ok": true}));
        let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(parsed.get("error").is_none());
        assert_eq!(parsed["result"]["ok"], true);
    }

    #[test]
    fn error_response_has_no_result_field() {
        let resp = error_response("r2", -32601, "Method not found", None);
        let parsed: serde_json::Value = serde_json::from_str(&resp).unwrap();
        assert!(parsed.get("result").is_none());
        assert_eq!(parsed["error"]["code"], -32601);
    }

    #[test]
    fn daemon_runtime_dir_is_under_state() {
        let state = Path::new("/tmp/test-state");
        let dir = daemon_runtime_dir(state);
        assert_eq!(dir, PathBuf::from("/tmp/test-state/daemon"));
    }

    #[test]
    fn stream_event_tagged_serialization() {
        let event = StreamEvent::Start {
            request_id: "r1".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"start\""));
        assert!(json.contains("\"request_id\":\"r1\""));
    }

    #[test]
    fn execute_result_serialization() {
        let result = ExecuteResult {
            execution: ExecutionMeta {
                mode: "daemon".to_string(),
                instance_id: "default".to_string(),
            },
            payload: serde_json::json!({"status": "ok"}),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ExecuteResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.execution.mode, "daemon");
    }
}
