use crate::{
    claude::models::{AnthropicContentBlock, AnthropicRole},
    models::{Provider, Response, ResponseContent, StopReason},
};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tools::ToolType;

use super::models::{AnthropicMessage, AnthropicMessageContent, AnthropicModel, AnthropicRequest};

#[derive(Clone)]
pub struct ClaudeProvider {
    api_key: String,
    client: reqwest::Client,
    model: AnthropicModel,
}

impl Provider for ClaudeProvider {
    fn new(api_key: String, model: String) -> Result<Self> {
        Ok(ClaudeProvider {
            api_key,
            client: reqwest::Client::new(),
            model: model.try_into()?,
        })
    }

    async fn send_prompt(&self, prompt: &str, tools: Option<Vec<ToolType>>) -> Result<Response> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Create a text content block
        let text_block = AnthropicContentBlock::Text {
            text: prompt.to_string(),
        };

        let messages = vec![AnthropicMessage {
            role: AnthropicRole::User,
            content: vec![text_block],
        }];

        // Convert tools to array of JSON schemas
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

        // Create Claude request directly to avoid conversion issues
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
            .context("Failed to send request to Claude API")?;

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        // Extract content based on type
        let content_array = response_json["content"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Content is not an array in Claude response"))?;

        if content_array.is_empty() {
            return Err(anyhow::anyhow!("Empty content array in Claude response"));
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
