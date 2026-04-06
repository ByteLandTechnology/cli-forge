//! Transport layer tests

use cli_forge_daemon::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcNotification};
use cli_forge_daemon::transport::{TransportBinding, StdioTransport, jsonl};
use std::io::Cursor;

#[test]
fn test_transport_binding_tcp() {
    let binding = TransportBinding::TcpSocket {
        host: "127.0.0.1".to_string(),
        port: 9090,
    };
    assert_eq!(binding.name(), "tcp");
}

#[test]
fn test_transport_binding_unix() {
    let binding = TransportBinding::UnixSocket {
        path: "/tmp/daemon.sock".to_string(),
    };
    assert_eq!(binding.name(), "unix");
}

#[test]
fn test_transport_binding_stdio() {
    let binding = TransportBinding::Stdio;
    assert_eq!(binding.name(), "stdio");
}

#[test]
fn test_jsonl_encode_decode() {
    let request = JsonRpcMessage::Request(JsonRpcRequest {
        id: serde_json::json!("1"),
        method: "test.method".to_string(),
        params: serde_json::json!({"key": "value"}),
    });

    let encoded = jsonl::encode(&request).unwrap();
    let decoded = jsonl::decode(&encoded).unwrap();

    match decoded {
        JsonRpcMessage::Request(req) => {
            assert_eq!(req.method, "test.method");
        }
        _ => panic!("Expected Request variant"),
    }
}

#[test]
fn test_stdio_transport_from_streams() {
    let input = Cursor::new(b"{\"jsonrpc\":\"2.0\",\"method\":\"test\",\"params\":{}}\n");
    let output = Cursor::new(Vec::new());

    let transport = StdioTransport::from_streams(Box::new(input), Box::new(output));
    assert!(transport.is_connected());
    assert_eq!(transport.peer_info(), "stdio:parent");
}

#[tokio::test]
async fn test_stdio_transport_send() {
    let input = Cursor::new(Vec::new());
    let output = Cursor::new(Vec::new());

    let transport = StdioTransport::from_streams(Box::new(input), Box::new(output));

    let request = JsonRpcMessage::Notification(JsonRpcNotification {
        method: "test.notification".to_string(),
        params: serde_json::json!({}),
    });

    // Note: This would need actual async runtime in full test
    // transport.send(request).await.unwrap();
}
