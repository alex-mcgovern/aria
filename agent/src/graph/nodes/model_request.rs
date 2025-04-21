use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use futures_util::StreamExt;
use providers::models::{ContentBlockStartData, ContentDelta, StreamEvent};
use providers::{models::StreamProcessor, BaseProvider, StopReason};
use std::sync::mpsc;

/// The model request node
///
/// This node is responsible for making requests to the model with
/// the current message history.
#[derive(Debug)]
pub struct ModelRequest;

// Add a struct to represent streamed text for the ModelRequest node
pub struct StreamedText {
    pub text: String,
}

impl<P: BaseProvider> NodeRunner<P> for ModelRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Setup a channel for yielding text parts
        let (text_sender, text_receiver) = mpsc::channel();
        state.stream_receiver = Some(text_receiver);

        // Run the stream, process the events and collect them into a `Response`
        let response = {
            let stream = deps
                .provider
                .stream(&state.message_history, deps.tools.clone())
                .await
                .context("Failed to create stream from provider")?;

            // Collect all events from the stream
            let mut events = Vec::new();
            let mut stream = Box::pin(stream);

            while let Some(event_result) = stream.next().await {
                let event = event_result.context("Error in event stream")?;

                // Yield text parts when they come in
                match &event {
                    StreamEvent::ContentBlockStart { content_block, .. } => {
                        if let ContentBlockStartData::Text { text } = content_block {
                            if !text.is_empty() {
                                // Send initial text
                                let _ = text_sender.send(StreamedText { text: text.clone() });
                            }
                        }
                    }
                    StreamEvent::ContentBlockDelta { delta, .. } => {
                        if let ContentDelta::TextDelta { text } = delta {
                            if !text.is_empty() {
                                // Send text delta
                                let _ = text_sender.send(StreamedText { text: text.clone() });
                            }
                        }
                    }
                    _ => {}
                }

                events.push(event);
            }

            // Process the collected events into a Response
            <StreamEvent as StreamProcessor<StreamEvent>>::process_events(events)
                .context("Failed to process stream events")?
        };

        state.message_history.push(
            response
                .clone()
                .try_into()
                .context("Failed to convert response to message")?,
        );

        match response.stop_reason {
            Some(StopReason::MaxTokens) => Err(GraphError::MaxTokens),
            Some(StopReason::ToolUse) => Ok(NodeTransition::ToCallTools),
            _ => Ok(NodeTransition::ToEnd),
        }
    }
}
