use crate::{
    models::{ContentBlock, MessageContent, Request as GenericRequest, Role},
    Message, Response, ResponseContent, StopReason,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};
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

/// Represents different types of content items in a message
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

/// Represents the different types of content that can be returned by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicResponseContent {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: ToolName,
        input: serde_json::Value,
    },
}

impl TryFrom<ResponseContent> for AnthropicResponseContent {
    type Error = anyhow::Error;

    fn try_from(content: ResponseContent) -> Result<Self, Self::Error> {
        match content {
            ResponseContent::Text { text } => Ok(AnthropicResponseContent::Text { text }),
            ResponseContent::ToolUse { id, name, input } => {
                Ok(AnthropicResponseContent::ToolUse { id, name, input })
            }
        }
    }
}

/// A generic response structure for LLM providers
#[derive(Debug, Clone)]
pub struct AnthropicResponse {
    pub content: AnthropicResponseContent,
    pub stop_reason: Option<AnthropicStopReason>,
}

impl TryFrom<Response> for AnthropicResponse {
    type Error = anyhow::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        Ok(AnthropicResponse {
            content: response.content.try_into()?,
            stop_reason: response.stop_reason.map(|r| r.try_into()).transpose()?,
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
