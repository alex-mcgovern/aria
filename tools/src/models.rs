use async_trait::async_trait;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum ToolError {
    InputSchemaSerializationError(serde_json::Error),
    JsonSchemaSerializationError(serde_json::Error),
}

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

/// Trait defining the interface for all tools
#[async_trait]
pub trait Tool<'a, T: ToSchema<'a>> {
    /// Executes the tool with the provided input
    async fn run(&self, input: T) -> ToolResult;

    /// Returns the title/name of the tool
    fn title(&self) -> &'static str;

    /// Returns a description of the tool's usage, best practices, and limitations
    fn description(&self) -> &'static str;

    /// Returns the OpenAPI schema for the input type
    fn input_schema(&self) -> Result<String, ToolError> {
        let openapi = utoipa::openapi::OpenApiBuilder::new().build();
        openapi
            .to_json()
            .map_err(|e| ToolError::InputSchemaSerializationError(e))
    }

    /// Returns a JSON representation of the tool's metadata and schema
    fn to_json_schema(&self) -> Result<String, ToolError> {
        serde_json::to_string(&serde_json::json!({
            "name": self.title(),
            "description": self.description(),
            "input_schema": serde_json::from_str::<serde_json::Value>(&self.input_schema()?).unwrap()
        }))
        .map_err(|e| ToolError::JsonSchemaSerializationError(e))
    }
}
