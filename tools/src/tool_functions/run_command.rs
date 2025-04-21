use crate::models::{Tool, ToolContent, ToolName, ToolResult};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Input parameters for the run_command tool
#[derive(Deserialize, JsonSchema, Debug)]
pub struct RunCommandInput {
    /// The command to run
    pub cmd: String,
    /// The arguments to pass to the command
    pub args: Vec<String>,
}

/// Tool for executing shell commands
#[derive(Debug, Serialize, Clone)]
pub struct RunCommandTool;

#[async_trait]
impl Tool<RunCommandInput> for RunCommandTool {
    fn title(&self) -> ToolName {
        ToolName::RunCommand
    }

    fn description(&self) -> &'static str {
        "Executes a shell command with the specified arguments. The 'cmd' parameter is a string (like 'ls' or 'git'), \
        and 'args' is a list of strings for the command arguments (like ['-l', '/tmp']). Use with caution as shell \
        commands can be potentially dangerous. Always validate and sanitize inputs before passing them to this tool. \
        Avoid commands that require interactive input as this tool doesn't handle stdin interactions."
    }

    async fn run(&self, input: RunCommandInput) -> ToolResult {
        let output = match Command::new(&input.cmd).args(&input.args).output() {
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
}
