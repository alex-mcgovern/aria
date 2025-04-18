use std::{fs, path::Path};
use crate::models::{ToolResult, ToolContent};

/// List all files in a directory and its subdirectories
pub async fn tree(dir: &str) -> ToolResult {
    fn visit_dir(dir: &Path, files: &mut Vec<String>) -> Result<(), std::io::Error> {
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
    match visit_dir(Path::new(dir), &mut files) {
        Ok(_) => ToolResult {
            is_error: false,
            content: ToolContent::StringArray(files),
        },
        Err(e) => ToolResult {
            is_error: true,
            content: ToolContent::String(format!("Failed to traverse directory '{}': {}", dir, e)),
        },
    }
}