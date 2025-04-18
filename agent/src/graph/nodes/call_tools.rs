use crate::graph::models::{Deps, GraphError, NodeRunner, NodeTransition, State};
use providers::Provider;

/// The tool calling node
#[derive(Debug)]
pub struct CallTools;

impl<P: Provider> NodeRunner<P> for CallTools {
    async fn run(
        &self,
        state: &mut State,
        _deps: &Deps<P>,
    ) -> std::result::Result<NodeTransition, GraphError> {
        // Just a placeholder - in a real implementation we would:
        // 1. Extract tool name and parameters from the last message
        // 2. Call the tool
        // 3. Store the result in tool_outputs
        // 4. Create a new message with the tool result
        Err(GraphError::ToolNotImplemented(
            "Tools not implemented yet".to_string(),
        ))
    }
}
