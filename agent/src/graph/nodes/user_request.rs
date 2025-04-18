use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use anyhow::Context;
use providers::{Message, Provider, Role, StopReason};

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

        // Add the response to messages
        state.messages.push(Message {
            role: Role::Assistant,
            content: response.content.clone(),
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
