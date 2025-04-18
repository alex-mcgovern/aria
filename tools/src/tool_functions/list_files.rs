use std::fs;
use crate::models::{ToolResult, ToolContent};

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