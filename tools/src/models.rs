use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{de::Error as SerdeError, Deserialize, Serialize}; // Add this import to use the custom() method

#[derive(Debug)]
pub enum ToolError {
    InputSchemaSerializationError(serde_json::Error),
    JsonSchemaSerializationError(serde_json::Error),
    InvalidToolName(String),
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
            Self::InvalidToolName(name) => {
                write!(f, "Invalid tool name: {}", name)
            }
        }
    }
}

impl std::error::Error for ToolError {}

/// Enum representing all available tool names
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolName {
    ReadFile,
    WriteFile,
    ListFiles,
    Tree,
    RunCommand,
}

impl ToolName {
    /// Convert the enum variant to its string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadFile => "read_file",
            Self::WriteFile => "write_file",
            Self::ListFiles => "list_files",
            Self::Tree => "tree",
            Self::RunCommand => "run_command",
        }
    }
}

impl std::fmt::Display for ToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for ToolName {
    type Error = ToolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "read_file" => Ok(Self::ReadFile),
            "write_file" => Ok(Self::WriteFile),
            "list_files" => Ok(Self::ListFiles),
            "tree" => Ok(Self::Tree),
            "run_command" => Ok(Self::RunCommand),
            _ => Err(ToolError::InvalidToolName(value)),
        }
    }
}

// Add the From implementation for converting ToolName to String
impl From<ToolName> for String {
    fn from(tool_name: ToolName) -> Self {
        tool_name.as_str().to_string()
    }
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
pub trait Tool<T: JsonSchema> {
    /// Executes the tool with the provided input
    async fn run(&self, input: T) -> ToolResult;

    /// Returns the title/name of the tool
    fn title(&self) -> ToolName;

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
            "name": self.title().as_str(),
            "description": self.description(),
            "input_schema": serde_json::from_str::<serde_json::Value>(&self.input_schema()?).unwrap()
        }))
        .map_err(|e| ToolError::JsonSchemaSerializationError(e))
    }
}
