use std::process::Command;
use crate::models::{ToolResult, ToolContent};

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