use serde_json::json;
pub mod models;
pub mod tool_functions;

// Re-exports for backwards compatibility
pub use models::{Tool, ToolContent, ToolResult};

// Tool struct re-exports
pub use tool_functions::list_files::{ListFilesInput, ListFilesTool};
pub use tool_functions::read_file::{ReadFileInput, ReadFileTool};
pub use tool_functions::run_command::{RunCommandInput, RunCommandTool};
pub use tool_functions::tree::{TreeInput, TreeTool};
pub use tool_functions::write_file::{WriteFileInput, WriteFileTool};

/// Define tool schemas for the LLM
pub fn get_tool_schemas() -> Vec<serde_json::Value> {
    let run_command_tool = RunCommandTool;
    let read_file_tool = ReadFileTool;
    let write_file_tool = WriteFileTool;
    let list_files_tool = ListFilesTool;
    let tree_tool = TreeTool;

    vec![
        json!({
            "name": run_command_tool.title(),
            "description": run_command_tool.description(),
            "input_schema": serde_json::from_str(&run_command_tool.input_schema()).unwrap()
        }),
        json!({
            "name": read_file_tool.title(),
            "description": read_file_tool.description(),
            "input_schema": serde_json::from_str(&read_file_tool.input_schema()).unwrap()
        }),
        json!({
            "name": write_file_tool.title(),
            "description": write_file_tool.description(),
            "input_schema": serde_json::from_str(&write_file_tool.input_schema()).unwrap()
        }),
        json!({
            "name": list_files_tool.title(),
            "description": list_files_tool.description(),
            "input_schema": serde_json::from_str(&list_files_tool.input_schema()).unwrap()
        }),
        json!({
            "name": tree_tool.title(),
            "description": tree_tool.description(),
            "input_schema": serde_json::from_str(&tree_tool.input_schema()).unwrap()
        }),
    ]
}
