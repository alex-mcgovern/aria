use crate::{
    anthropic::models::{AnthropicContentBlock, AnthropicRole},
    models::{Provider, Response, ResponseContent, StopReason},
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

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        // Extract content based on type
        let content_array = response_json["content"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Content is not an array in Anthropic response"))?;

        if content_array.is_empty() {
            return Err(anyhow::anyhow!("Empty content array in Anthropic response"));
        }

        let first_content = &content_array[0];
        let content_type = first_content["type"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing type in content"))?;

        let response_content: ResponseContent = match content_type {
            "text" => {
                let text = first_content["text"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing text in text content"))?
                    .to_string();
                ResponseContent::Text { text }
            }
            "tool_use" => {
                let id = first_content["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing id in tool_use content"))?
                    .to_string();
                let name = first_content["name"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing name in tool_use content"))?
                    .to_string();
                let input = first_content["input"].clone();
                ResponseContent::ToolUse {
                    id,
                    name: name.try_into()?,
                    input,
                }
            }
            other => return Err(anyhow::anyhow!("Unknown content type: {}", other)),
        };

        // Extract stop reason
        let stop_reason = match response_json["stop_reason"].as_str() {
            Some("end_turn") => Some(StopReason::EndTurn),
            Some("max_tokens") => Some(StopReason::MaxTokens),
            Some("stop_sequence") => Some(StopReason::StopSequence),
            Some("tool_use") => Some(StopReason::ToolUse),
            Some(other) => return Err(anyhow::anyhow!("Unknown stop reason: {}", other)),
            None => return Err(anyhow::anyhow!("Missing stop reason in response")),
        };

        println!("Stop reason: {:?}", stop_reason);

        Ok(Response {
            content: response_content,
            stop_reason,
        })
    }
}
