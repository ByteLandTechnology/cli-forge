//! Stdio transport for subprocess mode

use crate::protocol::JsonRpcMessage;
use async_trait::async_trait;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Stdio transport for JSON-Lines communication with parent CLI process
pub struct StdioTransport {
    reader: Arc<Mutex<BufReader<Box<dyn io::Read + Send>>>>,
    writer: Arc<Mutex<Box<dyn io::Write + Send>>>,
    connected: Arc<AtomicBool>,
}

impl StdioTransport {
    /// Create from stdin/stdout
    pub fn new() -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(Box::new(io::stdin())))),
            writer: Arc::new(Mutex::new(Box::new(io::stdout()))),
            connected: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Create from custom input/output streams (for testing)
    pub fn from_streams(
        input: Box<dyn io::Read + Send>,
        output: Box<dyn io::Write + Send>,
    ) -> Self {
        Self {
            reader: Arc::new(Mutex::new(BufReader::new(input))),
            writer: Arc::new(Mutex::new(output)),
            connected: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl super::Transport for StdioTransport {
    async fn send(&self, message: JsonRpcMessage) -> io::Result<()> {
        let line = serde_json::to_string(&message)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut writer = self.writer.lock().await;
        writeln!(writer, "{}", line)
    }

    async fn recv(&self) -> io::Result<JsonRpcMessage> {
        let mut line = String::new();
        let mut reader = self.reader.lock().await;
        reader.read_line(&mut line)?;
        if line.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected EOF on stdin",
            ));
        }
        serde_json::from_str(line.trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    fn peer_info(&self) -> String {
        "stdio:parent".to_string()
    }
}
