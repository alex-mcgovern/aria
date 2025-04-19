use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::de::Error as SerdeError; // Add this import to use the custom() method

#[derive(Debug)]
pub enum ToolError {
    InputSchemaSerializationError(serde_json::Error),
    JsonSchemaSerializationError(serde_json::Error),
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputSchemaSerializationError(e) => {
                write!(f, "Input schema serialization error: {}", e)
            }
            Self::JsonSchemaSerializationError(e) => {
                write!(f, "JSON schema serialization error: {}", e)
            }
        }
    }
}

impl std::error::Error for ToolError {}

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
pub trait Tool<T: JsonSchema> {
    /// Executes the tool with the provided input
    async fn run(&self, input: T) -> ToolResult;

    /// Returns the title/name of the tool
    fn title(&self) -> &'static str;

    /// Returns a description of the tool's usage, best practices, and limitations
    fn description(&self) -> &'static str;

    /// Returns the OpenAPI schema for the input type
    fn input_schema(&self) -> Result<String, ToolError> {
        // Generate the schema using schemars
        let schema = schema_for!(T);
        let schema_json = serde_json::to_value(&schema)
            .map_err(|e| ToolError::InputSchemaSerializationError(e))?;

        // Extract only the required fields from the schema
        let obj = schema_json.as_object().ok_or_else(|| {
            ToolError::InputSchemaSerializationError(SerdeError::custom("Invalid schema structure"))
        })?;

        let filtered = serde_json::json!({
            "type": obj.get("type").ok_or_else(|| ToolError::InputSchemaSerializationError(
            SerdeError::custom("Missing type field")
            ))?,
            "properties": obj.get("properties").ok_or_else(|| ToolError::InputSchemaSerializationError(
            SerdeError::custom("Missing properties field")
            ))?,
            "required": obj.get("required").ok_or_else(|| ToolError::InputSchemaSerializationError(
            SerdeError::custom("Missing required field")
            ))?
        });

        serde_json::to_string(&filtered).map_err(|e| ToolError::InputSchemaSerializationError(e))
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
