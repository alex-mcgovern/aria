use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProviderType {
    Anthropic,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Anthropic => write!(f, "Anthropic"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub provider: ProviderType,
    pub provider_base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    #[serde(default = "default_response_max_tokens")]
    pub response_max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_response_max_tokens() -> u32 {
    8192
}
