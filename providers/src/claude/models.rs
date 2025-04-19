use crate::models::{Request as GenericRequest, Role};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tools::ToolType;

#[derive(Debug, Clone)]
pub enum ClaudeModel {
    Claude3Sonnet,
}

impl ToString for ClaudeModel {
    fn to_string(&self) -> String {
        match self {
            ClaudeModel::Claude3Sonnet => "claude-3-7-sonnet-20250219".to_string(),
        }
    }
}

impl TryFrom<String> for ClaudeModel {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "claude-3-7-sonnet-20250219" => Ok(ClaudeModel::Claude3Sonnet),
            _ => Err(anyhow::anyhow!("Unknown Claude model: {}", value)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClaudeRequest {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(serialize_with = "serialize_claude_model")]
    pub model: ClaudeModel,
    pub max_tokens: u32,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub text: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

impl TryFrom<GenericRequest> for ClaudeRequest {
    type Error = anyhow::Error;

    fn try_from(req: GenericRequest) -> Result<Self, Self::Error> {
        let messages = req
            .messages
            .into_iter()
            .map(|m| Message {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content,
            })
            .collect();

        let tools = req
            .tools
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

        Ok(ClaudeRequest {
            system_prompt: req.system_prompt,
            temperature: req.temperature,
            model: req.model.try_into()?,
            max_tokens: req.max_tokens,
            messages,
            tools,
        })
    }
}

fn serialize_claude_model<S>(model: &ClaudeModel, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&model.to_string())
}

impl std::fmt::Display for ClaudeRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClaudeRequest {{ model: {}, max_tokens: {} }}",
            self.model.to_string(),
            self.max_tokens
        )
    }
}
