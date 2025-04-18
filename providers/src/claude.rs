use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::{Provider, Tool};

pub struct ClaudeProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

impl Provider for ClaudeProvider {
    fn new(api_key: String) -> Self {
        ClaudeProvider {
            api_key,
            client: reqwest::Client::new(),
            model: "claude-3-7-sonnet-20250219".to_string(), // Default model
        }
    }

    async fn send_prompt(&self, prompt: &str, tools: Option<Vec<Tool>>) -> Result<String> {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", HeaderValue::from_str(&self.api_key)?);
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let messages = vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages,
            tools,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Claude API")?;

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Claude API response")?;

        let content = &response_json["content"];
        if let Some(content) = content.as_array() {
            if let Some(first_content) = content.first() {
                if let Some(text) = first_content["text"].as_str() {
                    return Ok(text.to_string());
                }
            }
        }

        Err(anyhow::anyhow!("Failed to get text from Claude response"))
    }
}
