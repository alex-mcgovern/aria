use anyhow::Result;
use providers::Provider;
use providers::ProviderType;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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

impl TryFrom<&Config> for Provider {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        Provider::new(
            config.provider.clone(),
            config.api_key.clone().unwrap_or_default(),
            config.model.clone(),
            config.provider_base_url.clone(),
        )
    }
}
