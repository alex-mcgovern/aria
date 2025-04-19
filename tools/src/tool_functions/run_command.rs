use crate::models::{Tool, ToolContent, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Command;
use utoipa::ToSchema;

/// Input parameters for the run_command tool
#[derive(Deserialize, ToSchema)]
pub struct RunCommandInput {
    /// The command to run
    pub cmd: String,
    /// The arguments to pass to the command
    pub args: Vec<String>,
}

/// Tool for executing shell commands
pub struct RunCommandTool;

#[async_trait]
impl Tool<'_, RunCommandInput> for RunCommandTool {
    fn title(&self) -> &'static str {
        "run_command"
    }

    fn description(&self) -> &'static str {
        "Executes a shell command with the specified arguments. Use with caution as shell commands can be \
        potentially dangerous. Always validate and sanitize inputs before passing them to this tool. \
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
