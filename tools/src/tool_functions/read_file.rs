use crate::models::{Tool, ToolContent, ToolName, ToolResult};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

/// Input parameters for the read_file tool
#[derive(Deserialize, JsonSchema, Debug)]
pub struct ReadFileInput {
    /// The path of the file to read
    pub path: String,
}

/// Tool for reading file contents
#[derive(Debug, Serialize, Clone)]
pub struct ReadFileTool;

#[async_trait]
impl Tool<ReadFileInput> for ReadFileTool {
    fn title(&self) -> ToolName {
        ToolName::ReadFile
    }

    fn description(&self) -> &'static str {
        "Reads the content of a file at the specified path. Use absolute paths when possible to avoid \
        ambiguity. Always verify that the file exists before trying to read it. This tool is best used \
        for text files - binary files may not render correctly."
    }

    async fn run(&self, input: ReadFileInput) -> ToolResult {
        match fs::read_to_string(&input.path) {
            Ok(contents) => ToolResult {
                is_error: false,
                content: ToolContent::String(contents),
            },
            Err(e) => ToolResult {
                is_error: true,
                content: ToolContent::String(format!(
                    "Failed to read file '{}': {}",
                    input.path, e
                )),
            },
        }
    }
}
