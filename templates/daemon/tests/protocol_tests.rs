//! JSON-RPC protocol tests

use cli_forge_daemon::protocol::{
    JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcNotification,
    JsonRpcNotification, ErrorObject, success_response, error_response, notification,
    error_codes,
};
use serde_json::json;

#[test]
fn test_parse_request() {
    let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"test","params":{"key":"value"}}"#;
    let msg: JsonRpcMessage = serde_json::from_str(json_str).unwrap();

    match msg {
        JsonRpcMessage::Request(req) => {
            assert_eq!(req.method, "test");
            assert_eq!(req.params["key"], "value");
        }
        _ => panic!("Expected Request variant"),
    }
}

#[test]
fn test_parse_response_success() {
    let json_str = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
    let msg: JsonRpcMessage = serde_json::from_str(json_str).unwrap();

    match msg {
        JsonRpcMessage::Response(JsonRpcResponse::Success { id, result }) => {
            assert_eq!(id, json!(1));
            assert_eq!(result["status"], "ok");
        }
        _ => panic!("Expected Success response variant"),
    }
}

#[test]
fn test_parse_response_error() {
    let json_str = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
    let msg: JsonRpcMessage = serde_json::from_str(json_str).unwrap();

    match msg {
        JsonRpcMessage::Response(JsonRpcResponse::Error { id, error }) => {
            assert_eq!(error.code, -32601);
            assert_eq!(error.message, "Method not found");
        }
        _ => panic!("Expected Error response variant"),
    }
}

#[test]
fn test_parse_notification() {
    let json_str = r#"{"jsonrpc":"2.0","method":"event","params":{"data":123}}"#;
    let msg: JsonRpcMessage = serde_json::from_str(json_str).unwrap();

    match msg {
        JsonRpcMessage::Notification(notif) => {
            assert_eq!(notif.method, "event");
            assert_eq!(notif.params["data"], 123);
        }
        _ => panic!("Expected Notification variant"),
    }
}

#[test]
fn test_success_response() {
    let response = success_response(json!("1"), json!({"value": 42}));
    match response {
        JsonRpcResponse::Success { id, result } => {
            assert_eq!(id, json!("1"));
            assert_eq!(result["value"], 42);
        }
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_error_response() {
    let error = ErrorObject::new(error_codes::METHOD_NOT_FOUND, "Method not found");
    let response = error_response(json!("1"), error);

    match response {
        JsonRpcResponse::Error { id, error } => {
            assert_eq!(id, json!("1"));
            assert_eq!(error.code, error_codes::METHOD_NOT_FOUND);
        }
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_notification() {
    let notif = notification("test.event", json!({"key": "value"}));
    assert_eq!(notif.method, "test.event");
    assert_eq!(notif.params["key"], "value");
}

#[test]
fn test_error_object_with_data() {
    let error = ErrorObject::with_data(
        error_codes::INTERNAL_ERROR,
        "Internal error",
        json!({"details": "something went wrong"}),
    );
    assert_eq!(error.code, error_codes::INTERNAL_ERROR);
    assert!(error.data.is_some());
}

#[test]
fn test_error_codes() {
    assert_eq!(error_codes::SERVER_OVERLOADED, -32001);
    assert_eq!(error_codes::INVALID_REQUEST, -32600);
    assert_eq!(error_codes::METHOD_NOT_FOUND, -32601);
    assert_eq!(error_codes::INTERNAL_ERROR, -32603);
}
