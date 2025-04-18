/// A struct to represent the result of tool operations
#[derive(Debug)]
pub struct ToolResult {
    pub is_error: bool,
    pub content: ToolContent,
}

/// Represents either a single string or an array of strings
#[derive(Debug)]
pub enum ToolContent {
    String(String),
    StringArray(Vec<String>),
}
