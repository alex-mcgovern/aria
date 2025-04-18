use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod claude;

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

/// A generic response structure for LLM providers
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub stop_reason: Option<StopReason>,
}

/// A trait for LLM providers
pub trait Provider {
    /// Initialize the provider with API keys and other configuration
    fn new(api_key: String) -> Self
    where
        Self: Sized;

    /// Send a prompt to the provider and get a response
    fn send_prompt(
        &self,
        prompt: &str,
        tools: Option<Vec<Tool>>,
    ) -> impl std::future::Future<Output = Result<ProviderResponse>> + Send;
}

/// Represents a tool that can be used by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "input_schema")]
    pub input_schema: serde_json::Value,
}
