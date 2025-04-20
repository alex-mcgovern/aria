use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::{models::ContentBlock, Message, ProviderBase, Role};

/// The user request node
#[derive(Debug)]
pub struct UserRequest;

impl<P: ProviderBase> NodeRunner<P> for UserRequest {
    async fn run(
        &self,
        state: &mut State,
        deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Add the user's message to the message history
        state.message_history.push(Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: state.current_user_prompt.clone(),
            }],
        });

        // Transition to the model request node
        Ok(NodeTransition::ToModelRequest)
    }
}
