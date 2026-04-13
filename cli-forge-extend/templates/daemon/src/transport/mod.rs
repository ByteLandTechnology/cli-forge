//! Transport layer module

mod stdio;
mod websocket;

pub use stdio::StdioTransport;
pub use websocket::{WebSocketTransport, TlsConfig, WsServerConfig, WsServer};

use crate::auth::{Authenticator, Credentials};
use crate::protocol::JsonRpcMessage;
use async_trait::async_trait;
use std::io;

/// Transport binding types
#[derive(Debug, Clone)]
pub enum TransportBinding {
    /// TCP socket binding (host:port)
    TcpSocket { host: String, port: u16 },
    /// Unix domain socket path
    UnixSocket { path: String },
    /// Standard I/O (for subprocess mode)
    Stdio,
}

impl TransportBinding {
    /// Get display name for transport
    pub fn name(&self) -> &str {
        match self {
            TransportBinding::TcpSocket { .. } => "tcp",
            TransportBinding::UnixSocket { .. } => "unix",
            TransportBinding::Stdio => "stdio",
        }
    }
}

/// Transport trait for message passing
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message
    async fn send(&self, message: JsonRpcMessage) -> io::Result<()>;

    /// Receive a message (blocking)
    async fn recv(&self) -> io::Result<JsonRpcMessage>;

    /// Check if transport is connected
    fn is_connected(&self) -> bool;

    /// Get peer address/info
    fn peer_info(&self) -> String;
}

/// JSON-Lines codec for stdio transport
pub mod jsonl {
    use crate::protocol::JsonRpcMessage;
    use std::io::{self, BufRead, BufReader, Write};

    /// Encode message to JSON-Lines format
    pub fn encode(msg: &JsonRpcMessage) -> io::Result<String> {
        let json = serde_json::to_string(msg).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("JSON serialization failed: {}", e))
        })?;
        Ok(json)
    }

    /// Decode message from JSON-Lines format
    pub fn decode(line: &str) -> io::Result<JsonRpcMessage> {
        serde_json::from_str(line).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("JSON parse failed: {}", e))
        })
    }

    /// Wrapped writer for JSON-Lines output
    pub struct JsonlWriter<W: Write> {
        writer: W,
    }

    impl<W: Write> JsonlWriter<W> {
        pub fn new(writer: W) -> Self {
            Self { writer }
        }

        pub fn write(&mut self, msg: &JsonRpcMessage) -> io::Result<()> {
            let line = encode(msg)?;
            writeln!(self.writer, "{}", line)
        }
    }

    /// Wrapped reader for JSON-Lines input
    pub struct JsonlReader<R: BufRead> {
        reader: R,
    }

    impl<R: BufRead> JsonlReader<R> {
        pub fn new(reader: R) -> Self {
            Self { reader }
        }

        pub fn read(&mut self) -> io::Result<Option<JsonRpcMessage>> {
            let mut line = String::new();
            match self.reader.read_line(&mut line)? {
                0 => Ok(None),
                _ => Ok(Some(decode(line.trim())?)),
            }
        }
    }
}
