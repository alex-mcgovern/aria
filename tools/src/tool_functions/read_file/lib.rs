use std::fs;
use crate::models::{ToolResult, ToolContent};

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