use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use providers::{models::ContentBlock, Message, Provider, ResponseContent, Role, StopReason};

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

        // Create content block based on the response type
        let content_block = match &response.content {
            ResponseContent::Text { text } => {
                // For text responses, use text content block
                ContentBlock::Text { text: text.clone() }
            }
            ResponseContent::ToolUse { id, name, input } => {
                // For tool use, create a formatted string representation as text
                ContentBlock::Text {
                    text: format!(
                        "Tool Use: {} (id: {}), Input: {}",
                        name.as_str(),
                        id,
                        serde_json::to_string_pretty(input).unwrap_or_default()
                    ),
                }
            }
        };

        // Push message with content array containing the block
        state.message_history.push(Message {
            role: Role::Assistant,
            content: vec![content_block],
        });

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
