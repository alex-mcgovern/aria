use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::Provider;

/// The end node
#[derive(Debug)]
pub struct End;

impl<P: Provider> NodeRunner<P> for End {
    async fn run(
        &self,
        _state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // End node doesn't transition to any other node
        Ok(NodeTransition::Terminal)
    }
}
