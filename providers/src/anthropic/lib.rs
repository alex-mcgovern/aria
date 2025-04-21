use crate::{
    models::{BaseProvider, StreamEvent},
    Message,
};
use anyhow::{Context, Result};
use futures_util::stream::{Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest_eventsource::{Error as EventSourceError, EventSource};
use std::pin::Pin;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tools::ToolType;

use super::models::{AnthropicMessage, AnthropicModel, AnthropicRequest, AnthropicStreamEvent};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_MAX_TOKENS: u32 = 4096;

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
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        })
    }

    async fn stream(
        &self,
        messages: &Vec<Message>,
        tools: Option<Vec<ToolType>>,
        max_tokens: Option<u32>,
        temperature: Option<f64>,
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

        // Use the provided max_tokens value, defaulting to a higher value if none is provided
        let request = AnthropicRequest {
            system_prompt: String::new(),
            temperature,
            model: self.model.clone(),
            max_tokens: max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            messages,
            tools,
            stream: Some(true),
        };

        let endpoint = format!("{}/v1/messages", self.base_url);

        let event_source = EventSource::new(
            reqwest::Client::new()
                .post(&endpoint)
                .headers(headers)
                .json(&request),
        )?;

        Ok(self.handle_event_stream(event_source))
    }
}

impl AnthropicProvider {
    fn handle_event_stream(
        &self,
        event_source: EventSource,
    ) -> Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut event_source = event_source;

            while let Some(event_result) = event_source.next().await {
                let send_result = match event_result {
                    Ok(reqwest_eventsource::Event::Open) => tx.send(Ok(StreamEvent::Ping)),
                    Ok(reqwest_eventsource::Event::Message(message)) => {
                        let stream_event =
                            serde_json::from_str::<AnthropicStreamEvent>(&message.data)
                                .context("Failed to parse Anthropic stream event")
                                .and_then(|anthropic_event| anthropic_event.try_into());

                        tx.send(stream_event)
                    }
                    Err(EventSourceError::StreamEnded) => {
                        event_source.close();
                        break;
                    }
                    Err(err) => {
                        let result = tx.send(Err(anyhow::Error::new(err)));
                        event_source.close();
                        result
                    }
                };

                if send_result.is_err() {
                    // Channel closed, receiver dropped
                    event_source.close();
                    break;
                }
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}
