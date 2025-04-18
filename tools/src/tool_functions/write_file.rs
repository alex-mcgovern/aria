use std::{fs, path::Path};
use crate::models::{ToolResult, ToolContent};

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