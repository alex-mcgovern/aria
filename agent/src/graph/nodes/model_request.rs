use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use futures_util::StreamExt;
use providers::models::StreamEvent;
use providers::Response;
use providers::{models::StreamProcessor, BaseProvider, StopReason};

/// The model request node
///
/// This node is responsible for making requests to the model with
/// the current message history.
#[derive(Debug)]
pub struct ModelRequest;

impl<P: BaseProvider> NodeRunner<P> for ModelRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        let message_history = state.message_history.clone();
        
        let stream = deps
            .provider
            .stream(&message_history, deps.tools.clone(), Some(deps.max_tokens), deps.temperature)
            .await
            .context("Failed to create stream from provider")?;

        let mut events = Vec::new();
        let mut stream = deps.stream_wrapper.wrap(Box::pin(stream));

        while let Some(event_result) = stream.next().await {
            let event = event_result.context("Error in event stream")?;
            events.push(event);
        }

        let response: Response =
            <StreamEvent as StreamProcessor<StreamEvent>>::process_events(events)
                .context("Failed to process stream events")?;

        let message = response
            .clone()
            .try_into()
            .context("Failed to convert response to message")?;

        state.message_history.push(message);

        match response.stop_reason {
            Some(StopReason::MaxTokens) => Err(GraphError::MaxTokens),
            Some(StopReason::ToolUse) => Ok(NodeTransition::ToCallTools),
            _ => Ok(NodeTransition::ToEnd),
        }
    }
}
