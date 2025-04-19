use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use providers::{models::ContentBlock, Message, Provider, ResponseContent, Role, StopReason};

/// The user request node
#[derive(Debug)]
pub struct UserRequest;

impl<P: Provider> NodeRunner<P> for UserRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Send the current messages to the LLM provider
        let response = deps
            .provider
            .send_prompt(&state.current_user_prompt, deps.tools.clone())
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
                // In a more advanced implementation, this could be structured differently
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
        state.messages.push(Message {
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
