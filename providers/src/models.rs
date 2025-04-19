use anyhow::Result;
use serde::{Deserialize, Serialize};
use tools::{models::ToolName, ToolType};

/// Represents the role of the message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

/// Represents different types of content items in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// The result of a tool execution
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
    },
    /// Plain text content
    #[serde(rename = "text")]
    Text { text: String },
}

/// Represents the content of a message, which can either be plain text or a tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// A list of contents (currently only supporting tool results)
    ContentList(Vec<ContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Serialize)]
pub struct Request {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolType>>,
}

/// Represents the reason why the LLM stopped generating text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "max_tokens")]
    MaxTokens,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "tool_use")]
    ToolUse,
}

/// Represents the different types of content that can be returned by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: ToolName,
        input: serde_json::Value,
    },
}

/// A generic response structure for LLM providers
#[derive(Debug, Clone)]
pub struct Response {
    pub content: ResponseContent,
    pub stop_reason: Option<StopReason>,
}

/// A trait for LLM providers
pub trait Provider {
    /// Initialize the provider with API keys and other configuration
    fn new(api_key: String, model: String) -> Result<Self>
    where
        Self: Sized;

    /// Send a prompt to the provider and get a response
    fn send_prompt(
        &self,
        prompt: &str,
        tools: Option<Vec<ToolType>>,
    ) -> impl std::future::Future<Output = Result<Response>> + Send;
}
