pub mod models;
pub mod tool_functions;

use models::ToolError;
// Re-exports for backwards compatibility
pub use models::{Tool, ToolContent, ToolResult};

use serde::Serialize;
// Tool struct re-exports
pub use tool_functions::list_files::{ListFilesInput, ListFilesTool};
pub use tool_functions::read_file::{ReadFileInput, ReadFileTool};
pub use tool_functions::run_command::{RunCommandInput, RunCommandTool};
pub use tool_functions::tree::{TreeInput, TreeTool};
pub use tool_functions::write_file::{WriteFileInput, WriteFileTool};

#[derive(Debug, Serialize, Clone)]
pub enum ToolType {
    ListFiles(ListFilesTool),
    ReadFile(ReadFileTool),
    RunCommand(RunCommandTool),
    Tree(TreeTool),
    WriteFile(WriteFileTool),
}

impl ToolType {
    pub fn to_json_schema(&self) -> Result<std::string::String, ToolError> {
        match self {
            ToolType::ListFiles(tool) => tool.to_json_schema(),
            ToolType::ReadFile(tool) => tool.to_json_schema(),
            ToolType::RunCommand(tool) => tool.to_json_schema(),
            ToolType::Tree(tool) => tool.to_json_schema(),
            ToolType::WriteFile(tool) => tool.to_json_schema(),
        }
    }
}
