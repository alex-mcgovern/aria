use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod claude;

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
    ) -> impl std::future::Future<Output = Result<String>> + Send;
}

/// Represents a tool that can be used by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "input_schema")]
    pub input_schema: serde_json::Value,
}
