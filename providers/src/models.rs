use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TryFromInto};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Generic types for streaming events

/// Represents the content delta types in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "input_json_delta")]
    InputJsonDelta { partial_json: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
    #[serde(rename = "signature_delta")]
    SignatureDelta { signature: String },
}

/// Represents the different types of streaming events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStartData },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlockStartData,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: ContentDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaData,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: StreamErrorData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamErrorData {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaData {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlockStartData {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStartData {
    pub id: String,
    #[serde(default)]
    pub r#type: String,
    pub role: Role,
    pub model: String,
    pub content: Vec<ResponseContentBlock>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// A trait for implementing stream processing capability
pub trait StreamProcessor<T>
where
    T: TryInto<StreamEvent>,
{
    /// Process a vector of stream events into a Response
    fn process_events(events: Vec<T>) -> Result<Response>;
}

/// A trait for LLM providers
pub trait BaseProvider {
    /// Initialize the provider with API keys and other configuration
    fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self>
    where
        Self: Sized;

    /// Stream a response from the provider
    fn stream(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> impl std::future::Future<
        Output = Result<impl futures_util::Stream<Item = Result<StreamEvent>> + Send>,
    > + Send;
}

/// A provider factory that creates and manages specific LLM provider implementations
#[derive(Clone)]
pub enum Provider {
    Anthropic(crate::anthropic::AnthropicProvider),
}

impl Provider {
    /// Create a new provider instance for Anthropic
    pub fn new_anthropic(
        api_key: Option<String>,
        model: String,
        base_url: Option<String>,
    ) -> Result<Self> {
        let api_key =
            api_key.ok_or_else(|| anyhow::anyhow!("API key is required for Anthropic provider"))?;
        let provider = crate::anthropic::AnthropicProvider::new(api_key, model, base_url)?;
        Ok(Provider::Anthropic(provider))
    }

    /// Stream a response from the provider
    pub async fn stream<'a>(
        &'a self,
        messages: &'a Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<impl futures_util::Stream<Item = Result<StreamEvent>> + Send + 'a> {
        match self {
            Provider::Anthropic(provider) => provider.stream(messages, tools).await,
        }
    }
}

impl BaseProvider for Provider {
    fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self>
    where
        Self: Sized,
    {
        // Default to Anthropic provider
        Provider::new_anthropic(Some(api_key), model, base_url)
    }

    async fn stream(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<impl futures_util::Stream<Item = Result<StreamEvent>> + Send> {
        // Call the Provider::stream method instead of recursively calling itself
        match self {
            Provider::Anthropic(provider) => provider.stream(messages, tools).await,
        }
    }
}
