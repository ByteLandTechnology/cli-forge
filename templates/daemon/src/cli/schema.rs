//! Generate JSON Schema for daemon protocol

use clap::Parser;
use std::process::ExitCode;

/// Schema command arguments
#[derive(Debug, Clone, Parser)]
pub struct SchemaArgs {
    /// Output format: json (default) or yaml
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Schema command implementation
pub struct SchemaCommand {
    args: SchemaArgs,
}

impl SchemaCommand {
    pub fn new(args: SchemaArgs) -> Self {
        Self { args }
    }

    pub fn execute(&self) -> ExitCode {
        let schema = self.generate_schema();

        match self.args.format.as_str() {
            "yaml" => {
                if let Some(ref path) = self.args.output {
                    if let Err(e) = std::fs::write(path, &schema) {
                        eprintln!("Failed to write schema: {}", e);
                        return ExitCode::from(1);
                    }
                } else {
                    println!("{}", schema);
                }
            }
            "json" | _ => {
                // Pretty-print JSON schema
                match serde_json::from_str::<serde_json::Value>(&schema) {
                    Ok(value) => {
                        let formatted = serde_json::to_string_pretty(&value).unwrap_or(schema);
                        if let Some(ref path) = self.args.output {
                            if let Err(e) = std::fs::write(path, formatted) {
                                eprintln!("Failed to write schema: {}", e);
                                return ExitCode::from(1);
                            }
                        } else {
                            println!("{}", formatted);
                        }
                    }
                    Err(_) => {
                        eprintln!("Invalid schema JSON");
                        return ExitCode::from(1);
                    }
                }
            }
        }

        ExitCode::SUCCESS
    }

    fn generate_schema(&self) -> String {
        // Generate JSON Schema for the daemon JSON-RPC protocol
        let schema = serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "$id": "https://cli-forge.dev/daemon-protocol.json",
            "title": "CLI Forge Daemon Protocol",
            "description": "JSON-RPC 2.0 protocol for CLI Forge daemon communication",
            "type": "object",
            "oneOf": [
                { "$ref": "#/definitions/JsonRpcRequest" },
                { "$ref": "#/definitions/JsonRpcResponse" },
                { "$ref": "#/definitions/JsonRpcNotification" }
            ],
            "definitions": {
                "JsonRpcRequest": {
                    "type": "object",
                    "required": ["jsonrpc", "id", "method"],
                    "properties": {
                        "jsonrpc": {
                            "type": "string",
                            "const": "2.0"
                        },
                        "id": {
                            "oneOf": [
                                { "type": "string" },
                                { "type": "number" }
                            ]
                        },
                        "method": {
                            "type": "string"
                        },
                        "params": {
                            "type": "object",
                            "default": {}
                        }
                    }
                },
                "JsonRpcResponse": {
                    "oneOf": [
                        {
                            "type": "object",
                            "required": ["jsonrpc", "id", "result"],
                            "properties": {
                                "jsonrpc": {
                                    "type": "string",
                                    "const": "2.0"
                                },
                                "id": {
                                    "oneOf": [
                                        { "type": "string" },
                                        { "type": "number" }
                                    ]
                                },
                                "result": {}
                            }
                        },
                        {
                            "type": "object",
                            "required": ["jsonrpc", "id", "error"],
                            "properties": {
                                "jsonrpc": {
                                    "type": "string",
                                    "const": "2.0"
                                },
                                "id": {
                                    "oneOf": [
                                        { "type": "string" },
                                        { "type": "number" }
                                    ]
                                },
                                "error": {
                                    "$ref": "#/definitions/ErrorObject"
                                }
                            }
                        }
                    ]
                },
                "JsonRpcNotification": {
                    "type": "object",
                    "required": ["jsonrpc", "method"],
                    "properties": {
                        "jsonrpc": {
                            "type": "string",
                            "const": "2.0"
                        },
                        "method": {
                            "type": "string"
                        },
                        "params": {
                            "type": "object",
                            "default": {}
                        }
                    }
                },
                "ErrorObject": {
                    "type": "object",
                    "required": ["code", "message"],
                    "properties": {
                        "code": {
                            "type": "integer"
                        },
                        "message": {
                            "type": "string"
                        },
                        "data": {}
                    }
                },
                "DaemonStatus": {
                    "type": "object",
                    "properties": {
                        "lifecycleState": {
                            "type": "string",
                            "enum": ["Stopped", "Starting", "Running", "Stopping", "Failed"]
                        },
                        "health": {
                            "type": "string",
                            "enum": ["Initializing", "Ready", "Degraded", "Unhealthy"]
                        },
                        "instanceId": {
                            "type": "string"
                        },
                        "reason": {
                            "type": ["string", "null"]
                        },
                        "nextAction": {
                            "type": ["string", "null"]
                        }
                    }
                },
                "Methods": {
                    "initialize": {
                        "description": "Initialize daemon session",
                        "params": {
                            "type": "object",
                            "properties": {
                                "version": {
                                    "type": "string"
                                }
                            }
                        },
                        "result": {
                            "type": "object",
                            "properties": {
                                "version": { "type": "string" },
                                "protocol": { "type": "string" },
                                "transport": { "type": "string" }
                            }
                        }
                    },
                    "ping": {
                        "description": "Health check",
                        "params": {},
                        "result": {
                            "type": "object",
                            "properties": {
                                "state": { "type": "string" },
                                "health": { "type": "string" }
                            }
                        }
                    },
                    "start": {
                        "description": "Start daemon",
                        "params": {},
                        "result": {
                            "type": "object",
                            "properties": {
                                "state": { "type": "string" }
                            }
                        }
                    },
                    "stop": {
                        "description": "Stop daemon gracefully",
                        "params": {},
                        "result": {
                            "type": "object",
                            "properties": {
                                "state": { "type": "string" }
                            }
                        }
                    },
                    "restart": {
                        "description": "Restart daemon",
                        "params": {},
                        "result": {
                            "type": "object",
                            "properties": {
                                "state": { "type": "string" }
                            }
                        }
                    },
                    "status": {
                        "description": "Get daemon status",
                        "params": {},
                        "result": {
                            "$ref": "#/definitions/DaemonStatus"
                        }
                    }
                },
                "Notifications": {
                    "stateChanged": {
                        "description": "Lifecycle state changed",
                        "params": {
                            "type": "object",
                            "properties": {
                                "state": { "type": "string" },
                                "health": { "type": "string" }
                            }
                        }
                    },
                    "healthChanged": {
                        "description": "Health status changed",
                        "params": {
                            "type": "object",
                            "properties": {
                                "health": { "type": "string" },
                                "reason": { "type": ["string", "null"] }
                            }
                        }
                    },
                    "error": {
                        "description": "Error occurred",
                        "params": {
                            "type": "object",
                            "properties": {
                                "code": { "type": "integer" },
                                "message": { "type": "string" }
                            }
                        }
                    }
                }
            }
        });

        schema.to_string()
    }
}
