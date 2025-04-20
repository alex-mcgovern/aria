use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::{models::ContentBlock, BaseProvider, Role};

/// The starting node
#[derive(Debug)]
pub struct Start;

impl<P: BaseProvider> NodeRunner<P> for Start {
    async fn run(
        &self,
        state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Setup initial state with user input
        state.message_history.push(providers::Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: state.current_user_prompt.clone(),
            }],
        });
        Ok(NodeTransition::ToUserRequest)
    }
}
