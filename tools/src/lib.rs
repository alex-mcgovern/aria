use serde_json::json;

pub mod models;
pub mod tool_functions;

// Re-exports for backwards compatibility
pub use models::{ToolResult, ToolContent};
pub use tool_functions::run_command::run_command;
pub use tool_functions::read_file::read_file;
pub use tool_functions::write_file::write_file;
pub use tool_functions::list_files::list_files;
pub use tool_functions::tree::tree;

/// Define tool schemas for the LLM
pub fn get_tool_schemas() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "run_command",
            "description": "Run a command in the terminal",
            "input_schema": {
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The command to run"
                    },
                    "args": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Arguments for the command"
                    }
                },
                "required": ["command"]
            }
        }),
        json!({
            "name": "read_file",
            "description": "Read a file and return its contents",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "write_file",
            "description": "Write a file with the given contents",
            "input_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    },
                    "contents": {
                        "type": "string",
                        "description": "Contents to write to the file"
                    }
                },
                "required": ["path", "contents"]
            }
        }),
        json!({
            "name": "list_files",
            "description": "List all files in a directory",
            "input_schema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Path to the directory"
                    }
                },
                "required": ["dir"]
            }
        }),
        json!({
            "name": "tree",
            "description": "List all files in a directory and its subdirectories",
            "input_schema": {
                "type": "object",
                "properties": {
                    "dir": {
                        "type": "string",
                        "description": "Path to the directory"
                    }
                },
                "required": ["dir"]
            }
        }),
    ]
}
