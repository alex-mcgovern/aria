use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TryFromInto};
use std::fmt;
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
#[serde_as]
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
    /// A request to use a tool
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        #[serde_as(as = "TryFromInto<String>")]
        name: ToolName,
        input: serde_json::Value,
    },
}

impl TryFrom<ResponseContentBlock> for ContentBlock {
    type Error = anyhow::Error;
    fn try_from(value: ResponseContentBlock) -> Result<Self, Self::Error> {
        match value {
            ResponseContentBlock::Text { text } => Ok(ContentBlock::Text { text }),
            ResponseContentBlock::ToolUse { id, name, input } => {
                Ok(ContentBlock::ToolUse { id, name, input })
            }
        }
    }
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

impl TryFrom<Response> for Message {
    type Error = anyhow::Error;
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        let content = response
            .content
            .into_iter()
            .map(ContentBlock::try_from)
            .collect::<Result<Vec<ContentBlock>>>()?;
        Ok(Message {
            role: Role::Assistant,
            content,
        })
    }
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        #[serde_as(as = "TryFromInto<String>")]
        name: ToolName,
        input: serde_json::Value,
    },
}

/// Represents usage statistics for the API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

/// A generic response structure for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    pub role: Role,
    pub model: String,
    pub content: Vec<ResponseContentBlock>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

/// A trait for LLM providers
pub trait BaseProvider {
    /// Initialize the provider with API keys and other configuration
    fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self>
    where
        Self: Sized;
    /// Send a prompt to the provider and get a response
    fn sync(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> impl std::future::Future<Output = Result<Response>> + Send;
}

/// A provider factory that creates and manages specific LLM provider implementations
#[derive(Clone)]
pub enum Provider {
    Anthropic(crate::anthropic::AnthropicProvider),
}

impl Provider {
    /// Create a new provider instance for Anthropic
    pub fn new_anthropic(api_key: String, model: String, base_url: Option<String>) -> Result<Self> {
        let provider = crate::anthropic::AnthropicProvider::new(api_key, model, base_url)?;
        Ok(Provider::Anthropic(provider))
    }

    /// Send a prompt to the provider and get a response
    pub async fn sync(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<Response> {
        match self {
            Provider::Anthropic(provider) => provider.sync(messages, tools).await,
        }
    }
}

impl BaseProvider for Provider {
    fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self>
    where
        Self: Sized,
    {
        // Default to Anthropic provider
        Provider::new_anthropic(api_key, model, base_url)
    }

    async fn sync(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<Response> {
        match self {
            Provider::Anthropic(provider) => provider.sync(messages, tools).await,
        }
    }
}
