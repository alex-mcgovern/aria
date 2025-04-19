use crate::models::{Tool, ToolContent, ToolName, ToolResult};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

/// Input parameters for the write_file tool
#[derive(Deserialize, JsonSchema, Debug)]
pub struct WriteFileInput {
    /// The path of the file to write
    pub path: String,
    /// The contents to write to the file
    pub contents: String,
}

/// Tool for writing content to files
#[derive(Debug, Serialize, Clone)]
pub struct WriteFileTool;

#[async_trait]
impl Tool<WriteFileInput> for WriteFileTool {
    fn title(&self) -> ToolName {
        ToolName::WriteFile
    }

    fn description(&self) -> &'static str {
        "Writes content to a file at the specified path, creating the file and any parent directories \
        if they don't exist. Use absolute paths when possible to avoid ambiguity. Be careful when using \
        this tool as it will overwrite existing files without warning. Always verify the path is correct."
    }

    async fn run(&self, input: WriteFileInput) -> ToolResult {
        // Ensure the parent directory exists
        if let Some(parent) = Path::new(&input.path).parent() {
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

        match fs::write(&input.path, &input.contents) {
            Ok(_) => ToolResult {
                is_error: false,
                content: ToolContent::String(format!(
                    "Successfully wrote to file '{}'",
                    input.path
                )),
            },
            Err(e) => ToolResult {
                is_error: true,
                content: ToolContent::String(format!(
                    "Failed to write to file '{}': {}",
                    input.path, e
                )),
            },
        }
    }
}
