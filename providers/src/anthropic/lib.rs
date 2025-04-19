use crate::{
    anthropic::models::AnthropicResponse,
    models::{Provider, Response},
    Message,
};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tools::ToolType;

use super::models::{AnthropicMessage, AnthropicModel, AnthropicRequest};

#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
    model: AnthropicModel,
}

impl Provider for AnthropicProvider {
    fn new(api_key: String, model: String) -> Result<Self> {
        Ok(AnthropicProvider {
            api_key,
            client: reqwest::Client::new(),
            model: model.try_into()?,
        })
    }

    async fn sync(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let messages: Vec<AnthropicMessage> = messages
            .iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        let tools = tools
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

        let request = AnthropicRequest {
            system_prompt: String::new(),
            temperature: None,
            model: self.model.clone(),
            max_tokens: 1024,
            messages,
            tools,
        };

        let response = self
            .client
            .post("http://127.0.0.1:8080/v1/messages")
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        // Convert the AnthropicResponse to our generic Response type
        let response: Response = anthropic_response.try_into()?;

        Ok(response)
    }
}
