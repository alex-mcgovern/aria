use crate::models::{Tool, ToolContent, ToolResult};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;

/// Input parameters for the list_files tool
#[derive(Deserialize, JsonSchema)]
pub struct ListFilesInput {
    /// The directory path to list files from
    pub dir: String,
}

/// Tool for listing all files in a directory
#[derive(Debug, Serialize, Clone)]
pub struct ListFilesTool;

#[async_trait]
impl Tool<ListFilesInput> for ListFilesTool {
    fn title(&self) -> &'static str {
        "list_files"
    }

    fn description(&self) -> &'static str {
        "Lists all files in the specified directory. Best practice is to provide an absolute path to avoid \
        ambiguity. This tool does not recursively list subdirectories - use the tree tool for that purpose. \
        Verify the directory exists before calling this tool."
    }

    async fn run(&self, input: ListFilesInput) -> ToolResult {
        match fs::read_dir(&input.dir) {
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
                content: ToolContent::String(format!(
                    "Failed to read directory '{}': {}",
                    input.dir, e
                )),
            },
        }
    }
}
