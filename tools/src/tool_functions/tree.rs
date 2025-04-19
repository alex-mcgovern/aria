use crate::models::{Tool, ToolContent, ToolResult};
use async_trait::async_trait;
use serde::Deserialize;
use std::{fs, path::Path};
use utoipa::ToSchema;

/// Input parameters for the tree tool
#[derive(Deserialize, ToSchema)]
pub struct TreeInput {
    /// The directory path to list files from recursively
    pub dir: String,
}

/// Tool for recursively listing all files in a directory and its subdirectories
pub struct TreeTool;

#[async_trait]
impl Tool<'_, TreeInput> for TreeTool {
    fn title(&self) -> &'static str {
        "tree"
    }

    fn description(&self) -> &'static str {
        "Recursively lists all files in a directory and its subdirectories. Use absolute paths when possible \
        to avoid ambiguity. Be cautious with deeply nested directories as this can potentially generate large \
        outputs. Consider using list_files instead if you only need the immediate contents of a directory."
    }

    async fn run(&self, input: TreeInput) -> ToolResult {
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
        match visit_dir(Path::new(&input.dir), &mut files) {
            Ok(_) => ToolResult {
                is_error: false,
                content: ToolContent::StringArray(files),
            },
            Err(e) => ToolResult {
                is_error: true,
                content: ToolContent::String(format!(
                    "Failed to traverse directory '{}': {}",
                    input.dir, e
                )),
            },
        }
    }
}
