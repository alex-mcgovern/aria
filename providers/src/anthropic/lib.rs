use crate::{
    anthropic::models::AnthropicStreamEvent,
    models::{BaseProvider, StreamEvent},
    Message,
};
use anyhow::{Context, Result};
use futures_util::stream::{Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest_eventsource::{Event, EventSource};
use tools::ToolType;

use super::models::{AnthropicMessage, AnthropicModel, AnthropicRequest};

#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: AnthropicModel,
    base_url: String,
}

impl BaseProvider for AnthropicProvider {
    fn new(api_key: String, model: String, base_url: Option<String>) -> Result<Self> {
        Ok(AnthropicProvider {
            api_key,
            model: model.try_into()?,
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
        })
    }

    async fn stream(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
    ) -> Result<impl Stream<Item = Result<StreamEvent>> + Send> {
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
            stream: Some(true),
        };

        let endpoint = format!("{}/v1/messages", self.base_url);

        let event_source = EventSource::new(
            reqwest::Client::new()
                .post(&endpoint)
                .headers(headers)
                .json(&request), // .build()?,
        )?;

        // Process the SSE event stream
        let event_stream = event_source.map(|event_result| {
            event_result
                .context("Error in event stream")
                .and_then(|event| match event {
                    Event::Open => Ok(StreamEvent::Ping), // Just return a Ping event for Open events
                    Event::Message(message) => {
                        // Parse the event data as an AnthropicStreamEvent
                        let anthropic_event: AnthropicStreamEvent =
                            serde_json::from_str(&message.data)
                                .context("Failed to parse Anthropic stream event")?;

                        // Convert to the generic StreamEvent type
                        let generic_event: StreamEvent = anthropic_event.try_into()?;
                        Ok(generic_event)
                    }
                })
        });

        Ok(event_stream)
    }
}
