use serde_json::json;
use std::{fs, path::Path, process::Command};

/// A struct to represent the result of tool operations
#[derive(Debug)]
pub struct ToolResult {
    pub is_error: bool,
    pub content: ToolContent,
}

/// Represents either a single string or an array of strings
#[derive(Debug)]
pub enum ToolContent {
    String(String),
    StringArray(Vec<String>),
}

/// Run a command in the terminal
pub async fn run_command(cmd: &str, args: Vec<&str>) -> ToolResult {
    let output = match Command::new(cmd).args(&args).output() {
        Ok(output) => output,
        Err(e) => {
            return ToolResult {
                is_error: true,
                content: ToolContent::String(format!("Failed to execute command: {}", e)),
            };
        }
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(e) => {
            return ToolResult {
                is_error: true,
                content: ToolContent::String(format!("Failed to parse command output: {}", e)),
            };
        }
    };

    let stderr = match String::from_utf8(output.stderr) {
        Ok(stderr) => stderr,
        Err(e) => {
            return ToolResult {
                is_error: true,
                content: ToolContent::String(format!("Failed to parse error output: {}", e)),
            };
        }
    };

    if output.status.success() {
        ToolResult {
            is_error: false,
            content: ToolContent::String(stdout),
        }
    } else {
        ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Command failed: {}", stderr)),
        }
    }
}

/// Read a file and return its contents
pub async fn read_file(path: &str) -> ToolResult {
    match fs::read_to_string(path) {
        Ok(contents) => ToolResult {
            is_error: false,
            content: ToolContent::String(contents),
        },
        Err(e) => ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Failed to read file '{}': {}", path, e)),
        },
    }
}

/// Write a file with the given contents
pub async fn write_file(path: &str, contents: &str) -> ToolResult {
    // Ensure the parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return ToolResult {
                is_error: true,
                content: ToolContent::String(format!(
                    "Failed to create directory '{}': {}",
                    parent.display(),
                    e
                )),
            };
        }
    }

    match fs::write(path, contents) {
        Ok(_) => ToolResult {
            is_error: false,
            content: ToolContent::String(format!("Successfully wrote to file '{}'", path)),
        },
        Err(e) => ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Failed to write to file '{}': {}", path, e)),
        },
    }
}

/// List all files in a directory
pub async fn list_files(dir: &str) -> ToolResult {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut files = Vec::new();
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_owned());
                        }
                    }
                    Err(e) => {
                        return ToolResult {
                            is_error: true,
                            content: ToolContent::String(format!(
                                "Failed to read directory entry: {}",
                                e
                            )),
                        };
                    }
                }
            }
            ToolResult {
                is_error: false,
                content: ToolContent::StringArray(files),
            }
        }
        Err(e) => ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Failed to read directory '{}': {}", dir, e)),
        },
    }
}

/// List all files in a directory and its subdirectories
pub async fn tree(dir: &str) -> ToolResult {
    fn visit_dir(dir: &Path, files: &mut Vec<String>) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(path_str) = path.to_str() {
                files.push(path_str.to_owned());
            }
            if path.is_dir() {
                visit_dir(&path, files)?;
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    match visit_dir(Path::new(dir), &mut files) {
        Ok(_) => ToolResult {
            is_error: false,
            content: ToolContent::StringArray(files),
        },
        Err(e) => ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Failed to traverse directory '{}': {}", dir, e)),
        },
    }
}

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
