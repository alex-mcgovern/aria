use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use providers::{Provider, StopReason};

/// The model request node
///
/// This node is responsible for making requests to the model with
/// the current message history.
#[derive(Debug)]
pub struct ModelRequest;

impl<P: Provider> NodeRunner<P> for ModelRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Send the current message history to the LLM provider
        let response = deps
            .provider
            .sync(&state.message_history, deps.tools.clone())
            .await
            .context("Failed to send prompt to provider")?;

        // Push message with content array containing the block
        state.message_history.push(
            response
                .clone()
                .try_into()
                .context("Failed to convert response to message")?,
        );

        // Route based on stop reason
        match response.stop_reason {
            Some(StopReason::MaxTokens) => {
                return Err(GraphError::MaxTokens);
            }
            Some(StopReason::ToolUse) => Ok(NodeTransition::ToCallTools),
            _ => {
                // EndTurn, StopSequence, or None
                Ok(NodeTransition::ToEnd)
            }
        }
    }
}
