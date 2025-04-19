use crate::models::{Provider, ProviderResponse, StopReason};
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tools::ToolType;

use super::models::{ClaudeModel, ClaudeRequest, Message};

#[derive(Clone)]
pub struct ClaudeProvider {
    api_key: String,
    client: reqwest::Client,
    model: ClaudeModel,
}

impl Provider for ClaudeProvider {
    fn new(api_key: String, model: String) -> Result<Self> {
        Ok(ClaudeProvider {
            api_key,
            client: reqwest::Client::new(),
            model: model.try_into()?,
        })
    }

    async fn send_prompt(
        &self,
        prompt: &str,
        tools: Option<Vec<ToolType>>,
    ) -> Result<ProviderResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
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
        let model = ClaudeModel::try_from(self.model.clone())?;
        let request = ClaudeRequest {
            system_prompt: String::new(),
            temperature: None,
            model,
            max_tokens: 1024,
            messages,
            tools,
        };

        println!("Request: {}", request);

        let response = self
            .client
            .post("http://127.0.0.1:8080/v1/messages")
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        // Print status code and headers
        println!("Response Status: {}", response.status());
        println!("Response Headers: {:#?}", response.headers());

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        println!("Response json: {:#?}", response_json);

        // Extract content text
        let content = &response_json["content"];
        let mut content_text = String::new();
        if let Some(content) = content.as_array() {
            if let Some(first_content) = content.first() {
                if let Some(text) = first_content["text"].as_str() {
                    content_text = text.to_string();
                }
            }
        }

        if content_text.is_empty() {
            return Err(anyhow::anyhow!("Failed to get text from Claude response"));
        }

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

        Ok(ProviderResponse {
            content: content_text,
            stop_reason,
        })
    }
}
