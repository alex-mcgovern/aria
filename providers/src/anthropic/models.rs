use crate::{
    models::{ContentBlock, MessageContent, Request as GenericRequest, Role, Usage},
    Message, Response, ResponseContentBlock, StopReason,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr, TryFromInto};
use tools::models::ToolName;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AnthropicModel {
    Claude37Sonnet,
}

impl std::fmt::Display for AnthropicModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnthropicModel::Claude37Sonnet => write!(f, "claude-3-7-sonnet-20250219"),
        }
    }
}

impl TryFrom<String> for AnthropicModel {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "claude-3-7-sonnet-20250219" => Ok(AnthropicModel::Claude37Sonnet),
            _ => Err(anyhow::anyhow!("Unknown Anthropic model: {}", value)),
        }
    }
}

/// Represents the role of the message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnthropicRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

impl TryFrom<Role> for AnthropicRole {
    type Error = anyhow::Error;

    fn try_from(role: Role) -> Result<Self, Self::Error> {
        match role {
            Role::User => Ok(AnthropicRole::User),
            Role::Assistant => Ok(AnthropicRole::Assistant),
        }
    }
}

impl TryFrom<AnthropicRole> for Role {
    type Error = anyhow::Error;

    fn try_from(role: AnthropicRole) -> Result<Self, Self::Error> {
        match role {
            AnthropicRole::User => Ok(Role::User),
            AnthropicRole::Assistant => Ok(Role::Assistant),
        }
    }
}

/// Represents different types of content items in a message
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlock {
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

impl TryFrom<ContentBlock> for AnthropicContentBlock {
    type Error = anyhow::Error;

    fn try_from(block: ContentBlock) -> Result<Self, Self::Error> {
        match block {
            ContentBlock::ToolResult {
                tool_use_id,
                content,
            } => Ok(AnthropicContentBlock::ToolResult {
                tool_use_id,
                content,
            }),
            ContentBlock::Text { text } => Ok(AnthropicContentBlock::Text { text }),
            ContentBlock::ToolUse { id, name, input } => {
                Ok(AnthropicContentBlock::ToolUse { id, name, input })
            }
        }
    }
}

/// Represents the content of a message, which can either be plain text or a tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnthropicMessageContent {
    /// Plain text content
    Text(String),
    /// A list of contents (currently only supporting tool results)
    ContentList(Vec<AnthropicContentBlock>),
}

impl TryFrom<MessageContent> for AnthropicMessageContent {
    type Error = anyhow::Error;

    fn try_from(content: MessageContent) -> Result<Self, Self::Error> {
        match content {
            MessageContent::Text(text) => Ok(AnthropicMessageContent::Text(text)),
            MessageContent::ContentList(items) => {
                let converted: Result<Vec<_>, _> = items
                    .into_iter()
                    .map(AnthropicContentBlock::try_from)
                    .collect();
                Ok(AnthropicMessageContent::ContentList(converted?))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: AnthropicRole,
    pub content: Vec<AnthropicContentBlock>,
}

impl TryFrom<Message> for AnthropicMessage {
    type Error = anyhow::Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let content: Result<Vec<_>, _> = message
            .content
            .into_iter()
            .map(AnthropicContentBlock::try_from)
            .collect();
        Ok(AnthropicMessage {
            role: message.role.try_into()?,
            content: content?,
        })
    }
}

impl TryFrom<&Message> for AnthropicMessage {
    type Error = anyhow::Error;

    fn try_from(message: &Message) -> Result<Self> {
        let content: Result<Vec<_>, _> = message
            .content
            .iter()
            .map(|cb| cb.clone().try_into())
            .collect();
        Ok(AnthropicMessage {
            role: message.role.clone().try_into()?,
            content: content?,
        })
    }
}

#[serde_as]
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde_as(as = "DisplayFromStr")]
    pub model: AnthropicModel,
    pub max_tokens: u32,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
}

impl TryFrom<GenericRequest> for AnthropicRequest {
    type Error = anyhow::Error;

    fn try_from(request: GenericRequest) -> Result<Self, Self::Error> {
        let messages: Result<Vec<_>, _> = request
            .messages
            .into_iter()
            .map(AnthropicMessage::try_from)
            .collect();
        // Convert tools to array of JSON schemas
        let tools = request
            .tools
            .map(|tools| {
                tools
                    .into_iter()
                    .map(|tool| {
                        tool.to_json_schema()
                            .map_err(anyhow::Error::from)
                            .and_then(|schema| {
                                serde_json::from_str(&schema).context("Failed to parse JSON schema")
                            })
                    })
                    .collect::<Result<Vec<serde_json::Value>>>()
            })
            .transpose()?;
        Ok(AnthropicRequest {
            system_prompt: request.system_prompt,
            temperature: request.temperature,
            model: request
                .model
                .try_into()
                .context("Failed to convert model string to AnthropicModel")?,
            max_tokens: request.max_tokens,
            messages: messages?,
            tools,
        })
    }
}

/// Represents the reason why the LLM stopped generating text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnthropicStopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "max_tokens")]
    MaxTokens,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "tool_use")]
    ToolUse,
}

impl TryFrom<String> for AnthropicStopReason {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "end_turn" => Ok(AnthropicStopReason::EndTurn),
            "max_tokens" => Ok(AnthropicStopReason::MaxTokens),
            "stop_sequence" => Ok(AnthropicStopReason::StopSequence),
            "tool_use" => Ok(AnthropicStopReason::ToolUse),
            _ => Err(anyhow::anyhow!("Unknown stop reason: {}", value)),
        }
    }
}

impl TryFrom<StopReason> for AnthropicStopReason {
    type Error = anyhow::Error;

    fn try_from(reason: StopReason) -> Result<Self, Self::Error> {
        match reason {
            StopReason::EndTurn => Ok(AnthropicStopReason::EndTurn),
            StopReason::MaxTokens => Ok(AnthropicStopReason::MaxTokens),
            StopReason::StopSequence => Ok(AnthropicStopReason::StopSequence),
            StopReason::ToolUse => Ok(AnthropicStopReason::ToolUse),
        }
    }
}

impl TryFrom<AnthropicStopReason> for StopReason {
    type Error = anyhow::Error;

    fn try_from(reason: AnthropicStopReason) -> Result<Self, Self::Error> {
        match reason {
            AnthropicStopReason::EndTurn => Ok(StopReason::EndTurn),
            AnthropicStopReason::MaxTokens => Ok(StopReason::MaxTokens),
            AnthropicStopReason::StopSequence => Ok(StopReason::StopSequence),
            AnthropicStopReason::ToolUse => Ok(StopReason::ToolUse),
        }
    }
}

/// Represents usage statistics for the API request
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicResponseContentBlock {
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

impl TryFrom<ResponseContentBlock> for AnthropicResponseContentBlock {
    type Error = anyhow::Error;

    fn try_from(content: ResponseContentBlock) -> Result<Self, Self::Error> {
        match content {
            ResponseContentBlock::Text { text } => Ok(AnthropicResponseContentBlock::Text { text }),
            ResponseContentBlock::ToolUse { id, name, input } => {
                Ok(AnthropicResponseContentBlock::ToolUse { id, name, input })
            }
        }
    }
}

impl TryFrom<AnthropicResponseContentBlock> for ResponseContentBlock {
    type Error = anyhow::Error;

    fn try_from(content: AnthropicResponseContentBlock) -> Result<Self, Self::Error> {
        match content {
            AnthropicResponseContentBlock::Text { text } => Ok(ResponseContentBlock::Text { text }),
            AnthropicResponseContentBlock::ToolUse { id, name, input } => {
                Ok(ResponseContentBlock::ToolUse { id, name, input })
            }
        }
    }
}

/// Represents usage statistics for the API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

impl TryFrom<Usage> for AnthropicUsage {
    type Error = anyhow::Error;

    fn try_from(usage: Usage) -> Result<Self, Self::Error> {
        Ok(AnthropicUsage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_creation_input_tokens: usage.cache_creation_input_tokens,
            cache_read_input_tokens: usage.cache_read_input_tokens,
        })
    }
}

impl TryFrom<AnthropicUsage> for Usage {
    type Error = anyhow::Error;

    fn try_from(usage: AnthropicUsage) -> Result<Self, Self::Error> {
        Ok(Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_creation_input_tokens: usage.cache_creation_input_tokens,
            cache_read_input_tokens: usage.cache_read_input_tokens,
        })
    }
}

/// A response structure for Anthropic API
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    pub role: AnthropicRole,
    pub model: String,
    pub content: Vec<AnthropicResponseContentBlock>,
    pub stop_reason: Option<AnthropicStopReason>,
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

impl TryFrom<AnthropicResponse> for Response {
    type Error = anyhow::Error;

    fn try_from(response: AnthropicResponse) -> Result<Self, Self::Error> {
        let content: Vec<ResponseContentBlock> = response
            .content
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Response {
            id: response.id,
            r#type: response.r#type,
            role: response.role.try_into()?,
            model: response.model,
            content: content,
            stop_reason: response.stop_reason.map(|r| r.try_into()).transpose()?,
            stop_sequence: response.stop_sequence,
            usage: response.usage.try_into()?,
        })
    }
}

impl std::fmt::Display for AnthropicRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AnthropicRequest {{ model: {}, max_tokens: {} }}",
            self.model.to_string(),
            self.max_tokens
        )
    }
}
