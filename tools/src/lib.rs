use anyhow::Result;
use serde_json::json;
use std::{fs, path::Path, process::Command};

/// Run a command in the terminal
pub async fn run_command(cmd: &str, args: Vec<&str>) -> Result<String> {
    let output = Command::new(cmd).args(&args).output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(anyhow::anyhow!("Command failed: {}", stderr))
    }
}

/// Read a file and return its contents
pub async fn read_file(path: &str) -> Result<String> {
    let contents = fs::read_to_string(path)?;
    Ok(contents)
}

/// Write a file with the given contents
pub async fn write_file(path: &str, contents: &str) -> Result<()> {
    // Ensure the parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, contents)?;
    Ok(())
}

/// List all files in a directory
pub async fn list_files(dir: &str) -> Result<Vec<String>> {
    let entries = fs::read_dir(dir)?;
    let mut files = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if let Some(path_str) = path.to_str() {
            files.push(path_str.to_owned());
        }
    }

    Ok(files)
}

/// List all files in a directory and its subdirectories
pub async fn tree(dir: &str) -> Result<Vec<String>> {
    fn visit_dir(dir: &Path, files: &mut Vec<String>) -> Result<()> {
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
    visit_dir(Path::new(dir), &mut files)?;

    Ok(files)
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
