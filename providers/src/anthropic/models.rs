use crate::{
    models::{
        ContentBlock, ContentBlockStartData, ContentDelta, MessageContent, MessageDeltaData,
        MessageStartData, Request as GenericRequest, Role, StreamEvent, StreamProcessor, Usage,
    },
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
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
            stream: None,
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
    #[serde(default)]
    pub input_tokens: u32,
    #[serde(default)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
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
            usage: response.usage.map(|u| u.try_into()).transpose()?,
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

// Anthropic-specific streaming models

/// Anthropic streaming content delta types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
    #[serde(rename = "signature_delta")]
    SignatureDelta { signature: String },
}

impl TryFrom<AnthropicContentDelta> for ContentDelta {
    type Error = anyhow::Error;

    fn try_from(delta: AnthropicContentDelta) -> Result<Self, Self::Error> {
        match delta {
            AnthropicContentDelta::TextDelta { text } => Ok(ContentDelta::TextDelta { text }),
            AnthropicContentDelta::InputJsonDelta { partial_json } => {
                Ok(ContentDelta::InputJsonDelta { partial_json })
            }
            AnthropicContentDelta::ThinkingDelta { thinking } => {
                Ok(ContentDelta::ThinkingDelta { thinking })
            }
            AnthropicContentDelta::SignatureDelta { signature } => {
                Ok(ContentDelta::SignatureDelta { signature })
            }
        }
    }
}

/// Anthropic content block start data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicContentBlockStartData {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
}

impl TryFrom<AnthropicContentBlockStartData> for ContentBlockStartData {
    type Error = anyhow::Error;

    fn try_from(data: AnthropicContentBlockStartData) -> Result<Self, Self::Error> {
        match data {
            AnthropicContentBlockStartData::Text { text } => {
                Ok(ContentBlockStartData::Text { text })
            }
            AnthropicContentBlockStartData::ToolUse { id, name, input } => {
                Ok(ContentBlockStartData::ToolUse { id, name, input })
            }
            AnthropicContentBlockStartData::Thinking { thinking } => {
                Ok(ContentBlockStartData::Thinking { thinking })
            }
        }
    }
}

/// Anthropic message delta data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessageDeltaData {
    pub stop_reason: Option<AnthropicStopReason>,
    pub stop_sequence: Option<String>,
}

impl TryFrom<AnthropicMessageDeltaData> for MessageDeltaData {
    type Error = anyhow::Error;

    fn try_from(data: AnthropicMessageDeltaData) -> Result<Self, Self::Error> {
        Ok(MessageDeltaData {
            stop_reason: match data.stop_reason {
                Some(reason) => Some(reason.try_into()?),
                None => None,
            },
            stop_sequence: data.stop_sequence,
        })
    }
}

/// Anthropic message start data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessageStartData {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    pub role: AnthropicRole,
    pub model: String,
    pub content: Vec<AnthropicResponseContentBlock>,
    pub stop_reason: Option<AnthropicStopReason>,
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicUsage>,
}

impl TryFrom<AnthropicMessageStartData> for MessageStartData {
    type Error = anyhow::Error;

    fn try_from(data: AnthropicMessageStartData) -> Result<Self, Self::Error> {
        let content = data
            .content
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MessageStartData {
            id: data.id,
            r#type: data.r#type,
            role: data.role.try_into()?,
            model: data.model,
            content,
            stop_reason: data.stop_reason.map(|r| r.try_into()).transpose()?,
            stop_sequence: data.stop_sequence,
            usage: data.usage.map(|u| u.try_into()).transpose()?,
        })
    }
}

/// Anthropic error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicStreamErrorData {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Anthropic stream events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnthropicStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: AnthropicMessageStartData },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: AnthropicContentBlockStartData,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: AnthropicContentDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: AnthropicMessageDeltaData,
        #[serde(default)]
        usage: Option<AnthropicUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: AnthropicStreamErrorData },
}

impl TryFrom<AnthropicStreamEvent> for StreamEvent {
    type Error = anyhow::Error;

    fn try_from(
        event: AnthropicStreamEvent,
    ) -> std::result::Result<Self, <StreamEvent as TryFrom<AnthropicStreamEvent>>::Error> {
        match event {
            AnthropicStreamEvent::MessageStart { message } => Ok(StreamEvent::MessageStart {
                message: message.try_into()?,
            }),
            AnthropicStreamEvent::ContentBlockStart {
                index,
                content_block,
            } => Ok(StreamEvent::ContentBlockStart {
                index,
                content_block: content_block.try_into()?,
            }),
            AnthropicStreamEvent::ContentBlockDelta { index, delta } => {
                Ok(StreamEvent::ContentBlockDelta {
                    index,
                    delta: delta.try_into()?,
                })
            }
            AnthropicStreamEvent::ContentBlockStop { index } => {
                Ok(StreamEvent::ContentBlockStop { index })
            }
            AnthropicStreamEvent::MessageDelta { delta, usage } => Ok(StreamEvent::MessageDelta {
                delta: delta.try_into()?,
                usage: usage.map(|u| u.try_into()).transpose()?,
            }),
            AnthropicStreamEvent::MessageStop => Ok(StreamEvent::MessageStop),
            AnthropicStreamEvent::Ping => Ok(StreamEvent::Ping),
            AnthropicStreamEvent::Error { error } => Ok(StreamEvent::Error {
                error: crate::models::StreamErrorData {
                    error_type: error.error_type,
                    message: error.message,
                },
            }),
        }
    }
}

// Implement TryFrom for collections of events
impl StreamProcessor<AnthropicStreamEvent> for AnthropicStreamEvent {
    fn process_events(events: Vec<AnthropicStreamEvent>) -> Result<Response> {
        let mut id = String::new();
        let mut model = String::new();
        let mut role = AnthropicRole::Assistant;
        let mut stop_reason = None;
        let mut stop_sequence = None;
        let mut usage = AnthropicUsage {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        // Track content blocks by index
        let mut content_blocks: std::collections::HashMap<usize, AnthropicResponseContentBlock> =
            std::collections::HashMap::new();

        // Buffer for accumulating tool use JSON
        let mut json_buffers: std::collections::HashMap<usize, String> =
            std::collections::HashMap::new();

        // Process all events
        for event in events {
            match event {
                AnthropicStreamEvent::MessageStart { message } => {
                    id = message.id;
                    model = message.model;
                    role = message.role;
                    if let Some(event_usage) = message.usage {
                        usage = event_usage;
                    }
                }
                AnthropicStreamEvent::ContentBlockStart {
                    index,
                    content_block,
                } => match content_block {
                    AnthropicContentBlockStartData::Text { text } => {
                        content_blocks.insert(index, AnthropicResponseContentBlock::Text { text });
                    }
                    AnthropicContentBlockStartData::ToolUse { id, name, input } => {
                        content_blocks.insert(
                            index,
                            AnthropicResponseContentBlock::ToolUse {
                                id,
                                name: name.try_into()?,
                                input,
                            },
                        );
                    }
                    _ => {} // Thinking blocks are not added to the final response
                },
                AnthropicStreamEvent::ContentBlockDelta { index, delta } => match delta {
                    AnthropicContentDelta::TextDelta { text } => {
                        if let Some(AnthropicResponseContentBlock::Text {
                            text: existing_text,
                        }) = content_blocks.get_mut(&index)
                        {
                            // Append text to existing text block
                            existing_text.push_str(&text);
                        } else {
                            // Create new text block if it doesn't exist
                            content_blocks
                                .insert(index, AnthropicResponseContentBlock::Text { text });
                        }
                    }
                    AnthropicContentDelta::InputJsonDelta { partial_json } => {
                        // Buffer the partial JSON to be processed at content_block_stop
                        json_buffers
                            .entry(index)
                            .and_modify(|e| e.push_str(&partial_json))
                            .or_insert(partial_json);
                    }
                    _ => {} // Ignore thinking and signature deltas
                },
                AnthropicStreamEvent::ContentBlockStop { index } => {
                    // If we've buffered JSON for a tool use, process it now
                    if let Some(json_string) = json_buffers.remove(&index) {
                        if let Some(AnthropicResponseContentBlock::ToolUse { input, .. }) =
                            content_blocks.get_mut(&index)
                        {
                            // Parse the complete JSON string and update the tool use input
                            match serde_json::from_str::<serde_json::Value>(&json_string) {
                                Ok(json_value) => *input = json_value,
                                Err(e) => {
                                    return Err(anyhow::anyhow!("Failed to parse JSON: {}", e))
                                }
                            }
                        }
                    }
                }
                AnthropicStreamEvent::MessageDelta {
                    delta,
                    usage: delta_usage,
                } => {
                    stop_reason = delta.stop_reason;
                    stop_sequence = delta.stop_sequence;
                    if let Some(u) = delta_usage {
                        usage = u;
                    }
                }
                _ => {} // Ignore other events
            }
        }

        // Convert the collected blocks into a vector sorted by index
        let mut sorted_blocks: Vec<_> = content_blocks.into_iter().collect();
        sorted_blocks.sort_by_key(|(index, _)| *index);
        let content: Vec<AnthropicResponseContentBlock> =
            sorted_blocks.into_iter().map(|(_, block)| block).collect();

        // Build the final response
        Ok(Response {
            id,
            r#type: "message".to_string(),
            role: role.try_into()?,
            model,
            content: content
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
            stop_reason: stop_reason.map(TryInto::try_into).transpose()?,
            stop_sequence,
            usage: Some(usage.try_into()?),
        })
    }
}

// Add trait implementations for the generic StreamProcessor
impl StreamProcessor<StreamEvent> for StreamEvent {
    fn process_events(events: Vec<StreamEvent>) -> Result<Response> {
        // Convert generic events to Anthropic events
        let anthropic_events: Result<Vec<AnthropicStreamEvent>> = events
            .into_iter()
            .map(|event| match event {
                StreamEvent::MessageStart { message } => {
                    // Convert MessageStartData to AnthropicMessageStartData
                    let role: AnthropicRole = message.role.try_into()?;
                    let content: Result<Vec<AnthropicResponseContentBlock>> =
                        message.content.into_iter().map(TryInto::try_into).collect();
                    let stop_reason = message
                        .stop_reason
                        .map(|reason| reason.try_into())
                        .transpose()?;
                    let usage = message.usage.map(|u| u.try_into()).transpose()?;

                    Ok(AnthropicStreamEvent::MessageStart {
                        message: AnthropicMessageStartData {
                            id: message.id,
                            r#type: message.r#type,
                            role,
                            model: message.model,
                            content: content?,
                            stop_reason,
                            stop_sequence: message.stop_sequence,
                            usage,
                        },
                    })
                }
                StreamEvent::ContentBlockStart {
                    index,
                    content_block,
                } => {
                    // Convert ContentBlockStartData to AnthropicContentBlockStartData
                    let anthropic_content_block = match content_block {
                        ContentBlockStartData::Text { text } => {
                            AnthropicContentBlockStartData::Text { text }
                        }
                        ContentBlockStartData::ToolUse { id, name, input } => {
                            AnthropicContentBlockStartData::ToolUse { id, name, input }
                        }
                        ContentBlockStartData::Thinking { thinking } => {
                            AnthropicContentBlockStartData::Thinking { thinking }
                        }
                    };

                    Ok(AnthropicStreamEvent::ContentBlockStart {
                        index,
                        content_block: anthropic_content_block,
                    })
                }
                StreamEvent::ContentBlockDelta { index, delta } => {
                    // Convert ContentDelta to AnthropicContentDelta
                    let anthropic_delta = match delta {
                        ContentDelta::TextDelta { text } => {
                            AnthropicContentDelta::TextDelta { text }
                        }
                        ContentDelta::InputJsonDelta { partial_json } => {
                            AnthropicContentDelta::InputJsonDelta { partial_json }
                        }
                        ContentDelta::ThinkingDelta { thinking } => {
                            AnthropicContentDelta::ThinkingDelta { thinking }
                        }
                        ContentDelta::SignatureDelta { signature } => {
                            AnthropicContentDelta::SignatureDelta { signature }
                        }
                    };

                    Ok(AnthropicStreamEvent::ContentBlockDelta {
                        index,
                        delta: anthropic_delta,
                    })
                }
                StreamEvent::ContentBlockStop { index } => {
                    Ok(AnthropicStreamEvent::ContentBlockStop { index })
                }
                StreamEvent::MessageDelta { delta, usage } => {
                    // Convert MessageDeltaData to AnthropicMessageDeltaData
                    let stop_reason = delta
                        .stop_reason
                        .map(|reason| reason.try_into())
                        .transpose()?;

                    Ok(AnthropicStreamEvent::MessageDelta {
                        delta: AnthropicMessageDeltaData {
                            stop_reason,
                            stop_sequence: delta.stop_sequence,
                        },
                        usage: usage.map(|u| u.try_into()).transpose()?,
                    })
                }
                StreamEvent::MessageStop => Ok(AnthropicStreamEvent::MessageStop),
                StreamEvent::Ping => Ok(AnthropicStreamEvent::Ping),
                StreamEvent::Error { error } => Ok(AnthropicStreamEvent::Error {
                    error: AnthropicStreamErrorData {
                        error_type: error.error_type,
                        message: error.message,
                    },
                }),
            })
            .collect();

        // Process the Anthropic events
        AnthropicStreamEvent::process_events(anthropic_events?)
    }
}
