use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::Provider;
use providers::Role;

/// The starting node
#[derive(Debug)]
pub struct Start;

impl<P: Provider> NodeRunner<P> for Start {
    async fn run(
        &self,
        state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Setup initial state with user input
        state.messages.push(providers::Message {
            role: Role::User,
            content: state.current_user_prompt.clone(),
        });
        Ok(NodeTransition::ToUserRequest)
    }
}
